//! Tree construction like nu htmlparser's `TreeBuilder` driven through `SAXStreamer`
//! (`htmlparser/src/nu/validator/htmlparser/impl/TreeBuilder.java`).
//!
//! Java v.Nu parses in streaming mode: nothing is ever moved once it has been emitted.
//! Whenever the HTML algorithm would move nodes (foster parenting, the adoption agency
//! reparenting, attributes added to `html` or `body`), `SAXStreamer.fatal()` reports
//! "Cannot recover after last error. Any further errors will be ignored." and parsing
//! stops. The builder keeps that behaviour: after [`TreeBuilder::halted`] turns true it
//! ignores all further input.
//!
//! The builder keeps only the stack of open elements and the list of active formatting
//! elements. It reports parse errors with Java's wording and, when asked to, records
//! element pushes and pops as [`TreeEvent`]s.
//!
//! Layout: `stack` (stack of open elements, scopes, implied end tags), `insert`
//! (element insertion), `formatting` (active formatting elements, adoption agency),
//! `text` (doctype, characters, end of file), `start_tag`, `start_other` and
//! `end_tag` (per-mode handlers), `in_body` (the "in body" insertion mode).

mod end_tag;
mod formatting;
mod in_body;
mod insert;
mod names;
mod stack;
mod start_other;
mod start_tag;
mod text;

use self::names::groups::*;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use vnu_core::{MessageSubType, VNuMessage};

const ADDRESS_ETC: u32 = ADDRESS_OR_ARTICLE_OR_ASIDE_OR_DETAILS_OR_DIALOG_OR_DIR_OR_FIGCAPTION_OR_FIGURE_OR_FOOTER_OR_HEADER_OR_HGROUP_OR_MAIN_OR_NAV_OR_SEARCH_OR_SECTION_OR_SUMMARY;
const B_ETC: u32 = B_OR_BIG_OR_CODE_OR_EM_OR_I_OR_S_OR_SMALL_OR_STRIKE_OR_STRONG_OR_TT_OR_U;
const H1_H6: u32 = H1_OR_H2_OR_H3_OR_H4_OR_H5_OR_H6;
const DIV_ETC: u32 = DIV_OR_BLOCKQUOTE_OR_CENTER_OR_MENU;
const RUBY_ETC: u32 = RUBY_OR_SPAN_OR_SUB_OR_SUP_OR_VAR;
const TBODY_ETC: u32 = TBODY_OR_THEAD_OR_TFOOT;

const CANNOT_RECOVER: &str = "Cannot recover after last error. Any further errors will be ignored.";

type Attrs = Rc<[(String, String)]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ns {
    Html,
    Svg,
    MathMl,
}

/// Tokenizer state the tree builder asks for after a start tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerState {
    Rcdata,
    Rawtext,
    ScriptData,
    Plaintext,
}

/// An element entering or leaving the stack, like SAX `startElement`/`endElement`.
#[derive(Debug, Clone)]
pub enum TreeEvent {
    Push {
        ns: Ns,
        name: Cow<'static, str>,
        attrs: Attrs,
        line: usize,
        col: usize,
    },
    Pop {
        ns: Ns,
        name: Cow<'static, str>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    InRow,
    InTableBody,
    InTable,
    InCaption,
    InCell,
    FramesetOk,
    InBody,
    InHead,
    InHeadNoscript,
    InColumnGroup,
    AfterBody,
    InFrameset,
    AfterFrameset,
    Initial,
    BeforeHtml,
    BeforeHead,
    AfterHead,
    AfterAfterBody,
    AfterAfterFrameset,
    Text,
    InTemplate,
}

/// What a mode handler did with a token: Java's `break` out of the token loop,
/// `continue` (reprocess in the new mode) or a `switch` fall-through to another mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Done,
    Reprocess,
    FallTo(Mode),
}

#[derive(Debug, Clone)]
struct ElemName {
    name: Cow<'static, str>,
    camel: Cow<'static, str>,
    flags: u32,
}

impl ElemName {
    fn lookup(name: &str) -> Self {
        match names::lookup(name) {
            Some((n, c, f)) => Self {
                name: Cow::Borrowed(n),
                camel: Cow::Borrowed(c),
                flags: f,
            },
            None => Self {
                name: Cow::Owned(name.to_string()),
                camel: Cow::Owned(name.to_string()),
                flags: OTHER | NOT_INTERNED,
            },
        }
    }

    fn group(&self) -> u32 {
        self.flags & GROUP_MASK
    }
}

/// Where an element started; `tainted` once "Unclosed element" was reported for it.
#[derive(Debug)]
struct Loc {
    line: usize,
    col: usize,
    tainted: Cell<bool>,
}

#[derive(Debug)]
struct StackNode {
    flags: u32,
    name: Cow<'static, str>,
    pop_name: Cow<'static, str>,
    ns: Ns,
    attrs: Attrs,
    loc: Rc<Loc>,
}

type Node = Rc<StackNode>;

impl StackNode {
    fn group(&self) -> u32 {
        self.flags & GROUP_MASK
    }
    fn is_scoping(&self) -> bool {
        self.flags & SCOPING != 0
    }
    fn is_special(&self) -> bool {
        self.flags & SPECIAL != 0
    }
    fn is_foster_parenting(&self) -> bool {
        self.flags & FOSTER_PARENTING != 0
    }
    fn is_html_integration_point(&self) -> bool {
        self.flags & HTML_INTEGRATION_POINT != 0
    }
    fn is_optional_end_tag(&self) -> bool {
        self.flags & OPTIONAL_END_TAG != 0
    }
    fn is_html(&self, name: &str) -> bool {
        self.ns == Ns::Html && self.name == name
    }
    /// A copy for the stack or the formatting list that keeps the start location.
    fn reopen(&self) -> Node {
        Rc::new(StackNode {
            flags: self.flags,
            name: self.name.clone(),
            pop_name: self.pop_name.clone(),
            ns: self.ns,
            attrs: self.attrs.clone(),
            loc: self.loc.clone(),
        })
    }
}

fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{c}')
}

fn starts_with_ci(literal: &str, s: Option<&str>) -> bool {
    s.is_some_and(|s| {
        s.len() >= literal.len()
            && s.as_bytes()[..literal.len()].eq_ignore_ascii_case(literal.as_bytes())
    })
}

fn equals_ci(literal: &str, s: Option<&str>) -> bool {
    s.is_some_and(|s| s.eq_ignore_ascii_case(literal))
}

fn no_attrs() -> Attrs {
    Rc::from(Vec::new())
}

#[derive(Debug)]
pub struct TreeBuilder {
    mode: Mode,
    original_mode: Mode,
    frameset_ok: bool,
    need_to_drop_lf: bool,
    quirks: bool,
    template_modes: Vec<Mode>,
    stack: Vec<Node>,
    list: Vec<Option<Node>>,
    form_pointer: bool,
    head_pointer: bool,
    /// Characters seen in table modes, flushed (and checked) before the next token.
    table_chars: bool,
    table_non_space: bool,
    table_chars_loc: (usize, usize),
    id_locations: HashMap<String, (usize, usize)>,
    line: usize,
    col: usize,
    halted: bool,
    record_events: bool,
    messages: Vec<VNuMessage>,
    events: Vec<TreeEvent>,
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self {
            mode: Mode::Initial,
            original_mode: Mode::Initial,
            frameset_ok: true,
            need_to_drop_lf: false,
            quirks: false,
            template_modes: Vec::new(),
            stack: Vec::with_capacity(64),
            list: Vec::with_capacity(16),
            form_pointer: false,
            head_pointer: false,
            table_chars: false,
            table_non_space: false,
            table_chars_loc: (1, 1),
            id_locations: HashMap::new(),
            line: 1,
            col: 1,
            halted: false,
            record_events: false,
            messages: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl TreeBuilder {
    /// True once a streamability violation stopped the parse, like Java's `fatal()`.
    pub fn halted(&self) -> bool {
        self.halted
    }

    /// True when the current node is not an HTML element (CDATA sections allowed).
    pub fn in_foreign(&self) -> bool {
        self.stack.last().is_some_and(|n| n.ns != Ns::Html)
    }

    /// Moves the parse errors reported so far into `out`.
    pub fn take_messages(&mut self, out: &mut Vec<VNuMessage>) {
        if !self.messages.is_empty() {
            out.append(&mut self.messages);
        }
    }

    /// Moves the recorded element pushes and pops into `out`.
    pub fn take_events(&mut self, out: &mut Vec<TreeEvent>) {
        if !self.events.is_empty() {
            out.append(&mut self.events);
        }
    }

    /// Records element pushes and pops for [`TreeBuilder::take_events`]. Off by default.
    pub fn set_record_events(&mut self, on: bool) {
        self.record_events = on;
    }

    fn set_position(&mut self, line: usize, col: usize) {
        self.line = line;
        self.col = col;
    }

    // ---------------------------------------------------------------- reporting

    fn err(&mut self, message: impl Into<String>) {
        if self.halted {
            return;
        }
        self.messages
            .push(VNuMessage::error(message, self.line, self.col));
    }

    fn fatal(&mut self) {
        if self.halted {
            return;
        }
        let mut msg = VNuMessage::error(CANNOT_RECOVER, self.line, self.col);
        msg.sub_type = Some(MessageSubType::Fatal);
        self.messages.push(msg);
        self.halted = true;
    }

    fn report_unclosed(&mut self, pos: usize) {
        let node = &self.stack[pos];
        if node.is_optional_end_tag() || node.loc.tainted.get() {
            return;
        }
        node.loc.tainted.set(true);
        let msg = VNuMessage::error(
            format!("Unclosed element “{}”.", node.pop_name),
            node.loc.line,
            node.loc.col,
        );
        if !self.halted {
            self.messages.push(msg);
        }
    }

    /// Reports the elements above `elt_pos`, innermost first.
    fn err_list_unclosed_start_tags(&mut self, elt_pos: usize) {
        for i in (elt_pos + 1..self.stack.len()).rev() {
            self.report_unclosed(i);
        }
    }

    /// `errListUnclosedStartTags(0)` reports everything above the root.
    fn err_list_all_unclosed(&mut self) {
        self.err_list_unclosed_start_tags(0);
    }

    fn err_unclosed_elements(&mut self, elt_pos: usize, name: &str) {
        self.err(format!(
            "End tag “{name}” seen, but there were open elements."
        ));
        self.err_list_unclosed_start_tags(elt_pos);
    }

    fn err_unclosed_elements_implied(&mut self, elt_pos: usize, name: &str) {
        self.err(format!(
            "End tag “{name}” implied, but there were open elements."
        ));
        self.err_list_unclosed_start_tags(elt_pos);
    }

    fn err_unclosed_elements_cell(&mut self, elt_pos: usize) {
        self.err("A table cell was implicitly closed, but there were open elements.");
        self.err_list_unclosed_start_tags(elt_pos);
    }

    fn err_stray_start_tag(&mut self, name: &str) {
        self.err(format!("Stray start tag “{name}”."));
    }

    fn err_stray_end_tag(&mut self, name: &str) {
        self.err(format!("Stray end tag “{name}”."));
    }

    fn err_foo_seen_when_foo_open(&mut self, name: &str) {
        self.err(format!(
            "Start tag “{name}” seen but an element of the same type was already open."
        ));
    }

    fn err_no_element_to_close(&mut self, name: &str) {
        self.err(format!(
            "No “{name}” element in scope but a “{name}” end tag seen."
        ));
    }

    fn err_html_start_tag_in_foreign_context(&mut self, name: &str) {
        self.err(format!(
            "HTML start tag “{name}” in a foreign namespace context."
        ));
    }

    fn err_eof_with_unclosed_elements(&mut self) {
        self.err("End of file seen and there were open elements.");
        self.err_list_all_unclosed();
    }

    fn err_end_with_unclosed_elements(&mut self, name: &str) {
        self.err(format!(
            "End tag for  “{name}” seen, but there were unclosed elements."
        ));
        self.err_list_all_unclosed();
    }

    fn err_form_when_form_open(&mut self) {
        self.err("Saw a “form” start tag, but there was already an active “form” element. Nested forms are not allowed. Ignoring the tag.");
    }
}

#[cfg(test)]
mod tests;
