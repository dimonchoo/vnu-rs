use crate::treebuilder::{TokenizerState, TreeBuilder, TreeEvent};
use html5ever::tokenizer::Doctype;
use std::rc::Rc;
use vnu_core::VNuMessage;

/// Owns the tree builder for one document: whether it applies (HTML only; Java parses
/// XHTML and SVG with an XML parser), its messages, and where a fatal error stopped it.
///
/// The first start tag decides whether the document is HTML. Messages from before it
/// (doctype, leading text) wait in the builder until then.
#[derive(Debug)]
pub struct TreeAdapter {
    tree: TreeBuilder,
    active: bool,
    decided: bool,
    halted_at: Option<(usize, usize)>,
}

impl Default for TreeAdapter {
    fn default() -> Self {
        let mut tree = TreeBuilder::default();
        tree.set_record_events(true);
        Self {
            tree,
            active: true,
            decided: false,
            halted_at: None,
        }
    }
}

impl TreeAdapter {
    /// True while tree construction (and its parse errors) applies to the document.
    pub fn active(&self) -> bool {
        self.active
    }

    /// Line and column of the token that stopped the parse ("Cannot recover").
    pub fn halted_at(&self) -> Option<(usize, usize)> {
        self.halted_at
    }

    pub fn halted(&self) -> bool {
        self.halted_at.is_some()
    }

    /// Moves the recorded element pushes and pops into `out`.
    pub fn take_events(&mut self, out: &mut Vec<TreeEvent>) {
        self.tree.take_events(out);
    }

    /// True when the current node is foreign, so `<![CDATA[` opens a CDATA section.
    pub fn in_foreign(&self) -> bool {
        self.active && self.tree.in_foreign()
    }

    /// Called with the first start tag: XML documents get no HTML tree construction.
    pub fn decide(&mut self, xml_document: bool, out: &mut Vec<VNuMessage>) {
        if self.decided {
            return;
        }
        self.decided = true;
        self.active = !xml_document;
        if self.active {
            self.tree.take_messages(out);
        } else {
            self.tree.take_messages(&mut Vec::new());
        }
    }

    fn live(&self) -> bool {
        self.active && self.halted_at.is_none()
    }

    fn drain(&mut self, line: usize, col: usize, out: &mut Vec<VNuMessage>) {
        if !self.decided || !self.active {
            return;
        }
        self.tree.take_messages(out);
        if self.tree.halted() && self.halted_at.is_none() {
            self.halted_at = Some((line, col));
        }
    }

    pub fn doctype(&mut self, d: &Doctype, line: usize, col: usize, out: &mut Vec<VNuMessage>) {
        if !self.live() {
            return;
        }
        self.tree.doctype(
            d.name.as_deref(),
            d.public_id.as_deref(),
            d.system_id.as_deref(),
            d.force_quirks,
            line,
            col,
        );
        self.drain(line, col, out);
    }

    /// `name` is lowercase (html5ever lowercases tag names).
    pub fn start_tag(
        &mut self,
        name: &str,
        attrs: Rc<[(String, String)]>,
        self_closing: bool,
        line: usize,
        col: usize,
        out: &mut Vec<VNuMessage>,
    ) -> Option<TokenizerState> {
        if !self.live() {
            return None;
        }
        let state = self.tree.start_tag(name, attrs, self_closing, line, col);
        self.drain(line, col, out);
        state
    }

    pub fn end_tag(&mut self, name: &str, line: usize, col: usize, out: &mut Vec<VNuMessage>) {
        if !self.live() {
            return;
        }
        self.tree.end_tag(name, line, col);
        self.drain(line, col, out);
    }

    pub fn characters(&mut self, text: &str, line: usize, out: &mut Vec<VNuMessage>) {
        if !self.live() {
            return;
        }
        self.tree.characters(text, line, 1);
        self.drain(line, 1, out);
    }

    pub fn comment(&mut self) {
        if self.live() {
            self.tree.comment();
        }
    }

    /// End of input. A document without any start tag is HTML.
    pub fn eof(&mut self, line: usize, col: usize, out: &mut Vec<VNuMessage>) {
        self.decide(false, out);
        if !self.live() {
            return;
        }
        self.tree.eof(line, col);
        self.drain(line, col, out);
    }
}
