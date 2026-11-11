use crate::context::{DocumentMode, ElementContext, LineIndex, ValidationContext};
use crate::engine::html_stack::{apply_implicit_closures, ensure_in_body_for_tag, is_void_element};
use crate::engine::scan_policy::ScanPolicy;
use crate::engine::tree_adapter::TreeAdapter;
use crate::rule::Rule;
use crate::schema::relaxng::SchemaValidator;
use crate::schema::relaxng::messages::display_wording;
use crate::treebuilder::{TokenizerState, TreeEvent};
use html5ever::tokenizer::Doctype;
use std::borrow::Cow;
use std::rc::Rc;
use vnu_core::{MessageSubType, MessageType, VNuMessage};

pub struct StreamingValidator<'a> {
    pub ctx: ValidationContext<'a>,
    pub rules: Vec<Box<dyn Rule>>,
    pub char_rule_indices: Vec<usize>,
    pub stray_rule_indices: Vec<usize>,
    pub messages: Vec<VNuMessage>,
    pub scan_policy: ScanPolicy,
    pub in_script_or_style: bool,
    pub first_tag_seen: bool,
    pub line_index: &'a LineIndex,
    pub normalizer: super::attribute_normalization::XhtmlAttributeNormalizer,
    tag_cursor_line: usize,
    tag_cursor_byte: usize,
    /// Tree construction and its parse errors, like nu htmlparser.
    tree: TreeAdapter,
    tree_events: Vec<TreeEvent>,
    /// RELAX NG validation of the tree the builder emits, like Jing behind v.Nu.
    pub schema: SchemaValidator,
}

impl<'a> StreamingValidator<'a> {
    pub fn with_indices(
        source: &'a str,
        rules: Vec<Box<dyn Rule>>,
        line_index: &'a LineIndex,
        char_rule_indices: Vec<usize>,
        stray_rule_indices: Vec<usize>,
        mut schema: SchemaValidator,
    ) -> Self {
        schema.reset();
        let mut ctx = ValidationContext::new(source);
        ctx.line_index = Some(line_index);
        let normalizer = super::attribute_normalization::XhtmlAttributeNormalizer::build(source);
        Self {
            ctx,
            rules,
            char_rule_indices,
            stray_rule_indices,
            messages: Vec::new(),
            scan_policy: ScanPolicy::default(),
            in_script_or_style: false,
            first_tag_seen: false,
            line_index,
            normalizer,
            tag_cursor_line: 0,
            tag_cursor_byte: 0,
            tree: TreeAdapter::default(),
            tree_events: Vec::new(),
            schema,
        }
    }

    /// True while the tree builder reports tree-construction errors.
    pub fn tree_active(&self) -> bool {
        self.tree.active()
    }

    /// True when the current node is foreign, so `<![CDATA[` opens a CDATA section.
    pub fn in_foreign_content(&self) -> bool {
        self.tree.in_foreign()
    }

    fn halted(&self) -> bool {
        self.tree.halted()
    }

    /// Byte offset just past the token that stopped the parse ("Cannot recover").
    pub fn halted_at(&self) -> Option<usize> {
        let (line, col) = self.tree.halted_at()?;
        let source = self.ctx.source;
        let start = (self.line_index.line_offset(line) + col.saturating_sub(1)).min(source.len());
        Some(
            source[start..]
                .find('>')
                .map(|p| start + p + 1)
                .unwrap_or(source.len()),
        )
    }

    /// Runs the rules on the elements the tree builder pushed and popped, so they see
    /// the same tree as Java's checkers: implied `html`/`head`/`body`/`tbody`, elements
    /// closed by optional end tags, nothing for ignored tags. `line`/`col` locate the
    /// token that closed popped elements.
    fn replay_tree_events(&mut self, line: usize, col: usize) {
        let mut events = std::mem::take(&mut self.tree_events);
        self.tree.take_events(&mut events);
        for event in events.drain(..) {
            match event {
                TreeEvent::Push {
                    ns,
                    name,
                    attrs,
                    line: start_line,
                    col: start_col,
                } => {
                    // Jing runs before the checkers on every event.
                    let schema_from = self.messages.len();
                    self.schema.start_element(
                        ns,
                        &name,
                        &attrs,
                        start_line,
                        start_col,
                        &mut self.messages,
                    );
                    let rules_from = self.messages.len();
                    let elem = ElementContext {
                        name: name.into_owned(),
                        attrs,
                        line: start_line,
                        col: start_col,
                        byte_offset: self.line_index.line_offset(start_line),
                    };
                    match elem.name.as_str() {
                        "head" => self.ctx.head_seen = true,
                        "body" => self.ctx.body_seen = true,
                        _ => {}
                    }
                    for rule in &mut self.rules {
                        if rule.interested_in_start_tag(&elem) {
                            self.messages.extend(rule.on_start_tag(&self.ctx, &elem));
                        }
                    }
                    drop_duplicates(&mut self.messages, schema_from, rules_from);
                    self.ctx.open_elements.push(elem);
                }
                TreeEvent::Pop { ns, name } => {
                    let schema_from = self.messages.len();
                    self.schema
                        .end_element(ns, &name, line, col, &mut self.messages);
                    let rules_from = self.messages.len();
                    for rule in &mut self.rules {
                        self.messages
                            .extend(rule.on_end_tag(&self.ctx, &name, line, col));
                    }
                    drop_duplicates(&mut self.messages, schema_from, rules_from);
                    self.ctx.open_elements.pop();
                }
            }
        }
        self.tree_events = events;
    }

    pub fn column_for_tag(&mut self, line: usize, name: &str, end_tag: bool) -> usize {
        self.column_from_line(line, |snippet, from| {
            super::tag_column::find_tag(snippet, from, name, end_tag)
        })
    }

    pub fn column_for_marker(&mut self, line: usize, marker: &str) -> usize {
        self.column_from_line(line, |snippet, from| {
            super::tag_column::find_marker(snippet, from, marker)
        })
    }

    fn column_from_line(
        &mut self,
        line: usize,
        find: impl FnOnce(&str, usize) -> Option<(usize, usize)>,
    ) -> usize {
        if self.tag_cursor_line != line {
            self.tag_cursor_line = line;
            self.tag_cursor_byte = 0;
        }
        let snippet = self.line_index.line_snippet(self.ctx.source, line);
        match find(snippet, self.tag_cursor_byte) {
            Some((col, next)) => {
                self.tag_cursor_byte = next;
                col
            }
            None => 1,
        }
    }

    pub fn on_doctype(&mut self, d: Doctype, line: usize, col: usize) {
        if self.halted() {
            return;
        }
        self.tree.doctype(&d, line, col, &mut self.messages);
        self.scan_policy
            .tracker
            .on_doctype(&d, self.ctx.doctype_seen);
        self.ctx.doctype_seen = true;
        let name = d.name.as_deref().unwrap_or("");
        for rule in &mut self.rules {
            self.messages
                .extend(rule.on_doctype(&self.ctx, name, line, col));
        }
    }

    pub fn on_start_tag(
        &mut self,
        name_str: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
        line: usize,
        col: usize,
    ) -> Option<TokenizerState> {
        if self.halted() {
            return None;
        }
        let byte_offset = self.line_index.line_offset(line);
        self.scan_policy
            .tracker
            .on_start_tag(&name_str, &attrs, byte_offset);
        if name_str.eq_ignore_ascii_case("script") || name_str.eq_ignore_ascii_case("style") {
            self.in_script_or_style = true;
        }

        let elem = ElementContext {
            name: name_str.clone(),
            attrs: Rc::from(attrs),
            line,
            col,
            byte_offset,
        };

        if !self.first_tag_seen {
            self.first_tag_seen = true;
            // Java parses XHTML and SVG with an XML parser: no HTML tree construction.
            let xml = name_str.eq_ignore_ascii_case("svg")
                || (name_str.eq_ignore_ascii_case("html")
                    && elem.has_attr_val("xmlns", "http://www.w3.org/1999/xhtml"));
            self.tree.decide(xml, &mut self.messages);
            self.ctx.html_parser = self.tree.active();
            if elem.has_attr_val("xmlns", "http://www.w3.org/1999/xhtml")
                && !name_str.eq_ignore_ascii_case("html")
            {
                self.messages.push(VNuMessage::error(
                    format!("Element “{}” not allowed in this context.", name_str),
                    line,
                    col,
                ));
            }
            if name_str.eq_ignore_ascii_case("svg") {
                self.ctx.mode = DocumentMode::Svg;
                if !elem.has_attr("xmlns") {
                    self.messages.push(VNuMessage {
                        msg_type: MessageType::NonDocumentError,
                        url: None,
                        sub_type: Some(MessageSubType::Schema),
                        message: "XML document with no namespace; cannot determine any schema to use for validation.".into(),
                        extract: None,
                        last_line: None,
                        first_line: None,
                        last_column: None,
                        first_column: None,
                        hilite_start: None,
                        hilite_length: None,
                    });
                } else {
                    self.messages.push(VNuMessage {
                        msg_type: MessageType::Info,
                        url: None,
                        sub_type: None,
                        message: "Using the preset for SVG + URL + HTML + MathML based on the root namespace.".into(),
                        extract: None,
                        last_line: None,
                        first_line: None,
                        last_column: None,
                        first_column: None,
                        hilite_start: None,
                        hilite_length: None,
                    });
                }
            }
        }

        if name_str.eq_ignore_ascii_case("html") {
            self.ctx.html_seen = true;
            if elem.has_attr_val("xmlns", "http://www.w3.org/1999/xhtml") {
                self.ctx.is_xhtml = true;
            }
        } else if name_str.eq_ignore_ascii_case("head") {
            self.ctx.head_seen = true;
        } else if name_str.eq_ignore_ascii_case("body") {
            self.ctx.body_seen = true;
        } else if name_str.eq_ignore_ascii_case("title") {
            self.ctx.title_seen = true;
        }

        let state = self.tree.start_tag(
            &name_str,
            elem.attrs.clone(),
            self_closing,
            line,
            col,
            &mut self.messages,
        );
        if self.tree.active() {
            if let Some(id) = elem.get_attr("id") {
                self.ctx.id_to_tag.insert(id.to_string(), name_str);
            }
            self.replay_tree_events(line, col);
            return if self.halted() { None } else { state };
        }
        // XHTML and SVG: the hand-made stack below stands in for the XML parser.

        if self.ctx.mode == DocumentMode::Html {
            if let Some(m) = ensure_in_body_for_tag(&mut self.ctx, &name_str, line, col) {
                self.messages.push(m);
            }
        }

        if let Some(id) = elem.get_attr("id") {
            self.ctx
                .id_to_tag
                .insert(id.to_string(), elem.name.to_ascii_lowercase());
        }

        // Implicit end tags for optional HTML5 elements
        if let Some(m) =
            apply_implicit_closures(&mut self.ctx.open_elements, &name_str, true, line, col)
        {
            self.messages.push(m);
        }

        for rule in &mut self.rules {
            if rule.interested_in_start_tag(&elem) {
                self.messages.extend(rule.on_start_tag(&self.ctx, &elem));
            }
        }

        super::stack_policy::handle_self_closing_and_stack(
            &mut self.ctx,
            &mut self.rules,
            elem,
            self_closing,
            line,
            col,
            &mut self.messages,
        );
        state
    }

    pub fn on_end_tag(&mut self, name_str: &str, line: usize, col: usize) {
        if self.halted() {
            return;
        }
        self.tree.end_tag(name_str, line, col, &mut self.messages);
        if self.tree.active() {
            self.replay_tree_events(line, col);
        }
        if self.halted() {
            return;
        }
        if name_str.eq_ignore_ascii_case("script") || name_str.eq_ignore_ascii_case("style") {
            self.in_script_or_style = false;
        }
        self.scan_policy
            .tracker
            .on_end_tag(name_str, is_void_element(name_str));
        if self.tree.active() {
            return;
        }
        super::stack_policy::handle_end_tag(
            &mut self.ctx,
            &mut self.rules,
            &self.stray_rule_indices,
            name_str,
            line,
            col,
            &mut self.messages,
        );
    }

    pub fn on_characters(&mut self, text: &str, line: usize, _col: usize) {
        if self.halted() {
            return;
        }
        self.tree.characters(text, line, &mut self.messages);
        if self.tree.active() {
            self.replay_tree_events(line, 1);
        }
        if self.halted() {
            return;
        }
        if self.tree.active() {
            self.schema.characters(text, line, 1);
        }
        self.scan_policy
            .tracker
            .on_characters(text, self.in_script_or_style);

        for &idx in &self.char_rule_indices {
            if self.rules[idx].interested_in_characters(&self.ctx) {
                self.messages
                    .extend(self.rules[idx].on_characters(&self.ctx, text, line, 1));
            }
        }
    }

    pub fn on_comment(&mut self, _comment: &str, _line: usize, _col: usize) {
        self.tree.comment();
        if !self.halted() {
            self.schema.flush_characters(&mut self.messages);
        }
    }

    pub fn on_null_character(&mut self, _line: usize, _col: usize) {}

    pub fn on_parse_error(&mut self, err: Cow<'static, str>, line: usize, _col: usize) {
        if self.halted() {
            return;
        }
        let err_str = err.to_string();
        if self.ctx.is_xhtml
            && (err_str.contains("XML processing instruction in HTML")
                || (err_str == "Bad character"
                    && self
                        .line_index
                        .line_snippet(self.ctx.source, line)
                        .trim_start()
                        .starts_with("<?xml")))
        {
            return;
        }
        self.scan_policy.tracker.on_parse_error();
        self.messages.push(VNuMessage::error(err_str, line, 1));
    }

    pub fn finish(
        mut self,
    ) -> (
        ValidationContext<'a>,
        Vec<Box<dyn Rule>>,
        Vec<VNuMessage>,
        ScanPolicy,
        SchemaValidator,
    ) {
        let end = self.line_index.location(self.ctx.source.len());
        self.tree.eof(end.line, end.col, &mut self.messages);
        if self.tree.active() {
            self.replay_tree_events(end.line, end.col);
        }
        if !self.halted() {
            for rule in &mut self.rules {
                self.messages.extend(rule.on_finish(&self.ctx));
            }
        }

        if let Some(unclosed) = self.ctx.open_elements.iter().find(|e| {
            ![
                "html", "body", "head", "p", "li", "dt", "dd", "tr", "td", "th",
            ]
            .iter()
            .any(|&t| t.eq_ignore_ascii_case(&e.name))
        }) {
            self.scan_policy.tracker.needs_scan = true;
            if !self.tree.active() {
                self.messages.push(VNuMessage::error(
                    "End of file seen and there were open elements.".to_string(),
                    unclosed.line,
                    unclosed.col,
                ));
            }
        }

        (
            self.ctx,
            self.rules,
            self.messages,
            self.scan_policy,
            self.schema,
        )
    }
}

/// Drops the rule messages of one event (`messages[rules_from..]`) that repeat a
/// schema message of the same event (`messages[schema_from..rules_from]`), in either
/// of its wordings: the hand-written content-model rules still cover some of what
/// the schema reports.
fn drop_duplicates(messages: &mut Vec<VNuMessage>, schema_from: usize, rules_from: usize) {
    if schema_from == rules_from || rules_from == messages.len() {
        return;
    }
    let mut keep = rules_from;
    for i in rules_from..messages.len() {
        let text = messages[i].message.as_str();
        let dup = messages[schema_from..rules_from]
            .iter()
            .any(|m| m.message == text || display_wording(&m.message) == text);
        if !dup {
            messages.swap(keep, i);
            keep += 1;
        }
    }
    messages.truncate(keep);
}
