//! `SchemaValidator`: the tree builder's events through the schema wrappers into
//! Jing's `PatternValidator`.

use super::attributes::{filter_attributes, permute_attributes, sax_attribute};
use super::{CUSTOM_ELEMENTS_NS, XHTML_NS, datatypes, html5_schema, messages, ns_uri};
use crate::treebuilder::Ns;
use vnu_core::VNuMessage;
use vnu_relaxng::{Attribute, PatternValidator, ValidationError};

pub struct SchemaValidator {
    /// `None` when the embedded schema cannot be loaded.
    validator: Option<PatternValidator>,
    /// `TemplateElementDroppingContentHandlerWrapper.numberOfTemplatesDeep`.
    templates_deep: usize,
    errors: Vec<ValidationError>,
    /// Text not yet given to the validator, and where it starts. nu htmlparser's
    /// tree builder buffers text the same way and flushes it before the next tag,
    /// comment or the end of the document.
    pending_text: String,
    pending_at: (usize, usize),
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self {
            validator: html5_schema().map(|schema| {
                PatternValidator::new(schema, &datatypes::Html5DatatypeLibraryFactory)
            }),
            templates_deep: 0,
            errors: Vec::new(),
            pending_text: String::new(),
            pending_at: (1, 1),
        }
    }
}

impl SchemaValidator {
    /// `startDocument`.
    pub fn reset(&mut self) {
        if let Some(v) = &mut self.validator {
            v.reset();
            v.start_document();
        }
        self.templates_deep = 0;
        self.pending_text.clear();
    }

    /// `TreeBuilder.flushCharacters`: hands the buffered text to the validator.
    pub fn flush_characters(&mut self, out: &mut Vec<VNuMessage>) {
        if self.pending_text.is_empty() {
            return;
        }
        if let Some(v) = &mut self.validator {
            v.characters(&self.pending_text);
        }
        self.pending_text.clear();
        let (line, col) = self.pending_at;
        self.report(line, col, out);
    }

    pub fn start_element(
        &mut self,
        ns: Ns,
        name: &str,
        attrs: &[(String, String)],
        line: usize,
        col: usize,
        out: &mut Vec<VNuMessage>,
    ) {
        self.flush_characters(out);
        if self.validator.is_none() {
            return;
        }
        let mut uri = ns_uri(ns);
        // NamespaceChangingContentHandlerWrapper
        if ns == Ns::Html && name.contains('-') {
            uri = CUSTOM_ELEMENTS_NS;
        }
        // TemplateElementDroppingContentHandlerWrapper
        if uri == XHTML_NS && name == "template" {
            self.templates_deep += 1;
            if self.templates_deep != 1 {
                return;
            }
        } else if self.templates_deep > 0 {
            return;
        }
        let mut atts: Vec<Attribute<'_>> = Vec::with_capacity(attrs.len());
        for (attr_name, value) in attrs {
            if let Some(att) = sax_attribute(ns, attr_name, value) {
                atts.push(att);
            }
        }
        permute_attributes(ns, name, &mut atts);
        filter_attributes(uri, &mut atts, line, col, out);
        if let Some(v) = &mut self.validator {
            v.start_element(uri, name, &atts);
        }
        self.report(line, col, out);
    }

    pub fn end_element(
        &mut self,
        ns: Ns,
        name: &str,
        line: usize,
        col: usize,
        out: &mut Vec<VNuMessage>,
    ) {
        self.flush_characters(out);
        if ns == Ns::Html && name == "template" {
            self.templates_deep = self.templates_deep.saturating_sub(1);
            if self.templates_deep != 0 {
                return;
            }
        } else if self.templates_deep > 0 {
            return;
        }
        if let Some(v) = &mut self.validator {
            v.end_element();
        }
        self.report(line, col, out);
    }

    /// Text of the current node. Like Java, text inside a dropped `template`
    /// subtree still goes to the validator: `TemplateElementDroppingContentHandlerWrapper`
    /// drops elements only.
    pub fn characters(&mut self, text: &str, line: usize, col: usize) {
        if self.pending_text.is_empty() {
            self.pending_at = (line, col);
        }
        self.pending_text.push_str(text);
    }

    fn report(&mut self, line: usize, col: usize, out: &mut Vec<VNuMessage>) {
        let Some(v) = &mut self.validator else {
            return;
        };
        if !v.has_errors() {
            return;
        }
        v.take_errors(&mut self.errors);
        for e in self.errors.drain(..) {
            let text = messages::message_text(v, &e);
            let warning = messages::is_role_directory_warning(v, &e) || messages::is_warning(&e);
            out.push(if warning {
                VNuMessage::warning(text, line, col)
            } else {
                VNuMessage::error(text, line, col)
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(events: &[(&str, &str)]) -> Vec<String> {
        let mut v = SchemaValidator::default();
        v.reset();
        let mut out = Vec::new();
        for (kind, arg) in events {
            match *kind {
                "start" => v.start_element(Ns::Html, arg, &[], 1, 1, &mut out),
                "end" => v.end_element(Ns::Html, arg, 1, 1, &mut out),
                _ => v.characters(arg, 1, 1),
            }
        }
        out.into_iter().map(|m| m.message).collect()
    }

    fn page(body: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
        let mut e = vec![
            ("start", "html"),
            ("start", "head"),
            ("start", "title"),
            ("text", "t"),
            ("end", "title"),
            ("end", "head"),
            ("start", "body"),
        ];
        e.extend_from_slice(body);
        e.push(("end", "body"));
        e.push(("end", "html"));
        e
    }

    #[test]
    fn valid_page_has_no_errors() {
        assert_eq!(
            run(&page(&[("start", "p"), ("text", "x"), ("end", "p")])),
            Vec::<String>::new()
        );
    }

    #[test]
    fn div_in_ul() {
        let msgs = run(&page(&[
            ("start", "ul"),
            ("start", "div"),
            ("end", "div"),
            ("end", "ul"),
        ]));
        assert_eq!(
            msgs,
            vec![
                "Element “div” not allowed as child of element “ul” in this context. (Suppressing further errors from this subtree.)"
            ]
        );
    }

    #[test]
    fn text_in_ul() {
        let msgs = run(&page(&[
            ("start", "ul"),
            ("text", "x"),
            ("start", "li"),
            ("end", "li"),
            ("end", "ul"),
        ]));
        assert_eq!(
            msgs,
            vec!["Text not allowed in element “ul” in this context."]
        );
    }
}
