use crate::context::{DocumentMode, ElementContext, LineIndex, ValidationContext};
use crate::engine::html_stack::{apply_implicit_closures, ensure_in_body_for_tag, is_void_element};
use crate::engine::scan_policy::ScanPolicy;
use crate::rule::Rule;
use html5ever::tokenizer::Doctype;
use std::borrow::Cow;
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
}

impl<'a> StreamingValidator<'a> {
    pub fn with_indices(
        source: &'a str,
        rules: Vec<Box<dyn Rule>>,
        line_index: &'a LineIndex,
        char_rule_indices: Vec<usize>,
        stray_rule_indices: Vec<usize>,
    ) -> Self {
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
        }
    }

    pub fn on_doctype(&mut self, d: Doctype, line: usize) {
        self.scan_policy
            .tracker
            .on_doctype(&d, self.ctx.doctype_seen);
        self.ctx.doctype_seen = true;
        let name = d.name.as_deref().unwrap_or("");
        for rule in &mut self.rules {
            self.messages
                .extend(rule.on_doctype(&self.ctx, name, line, 1));
        }
    }

    pub fn on_start_tag(
        &mut self,
        name_str: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
        line: usize,
        col: usize,
    ) {
        let byte_offset = self.line_index.line_offset(line);
        self.scan_policy
            .tracker
            .on_start_tag(&name_str, &attrs, byte_offset);
        if name_str.eq_ignore_ascii_case("script") || name_str.eq_ignore_ascii_case("style") {
            self.in_script_or_style = true;
        }

        let elem = ElementContext {
            name: name_str.clone(),
            attrs,
            line,
            col,
            byte_offset,
        };

        if !self.first_tag_seen {
            self.first_tag_seen = true;
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
    }

    pub fn on_end_tag(&mut self, name_str: &str, line: usize, col: usize) {
        if name_str.eq_ignore_ascii_case("script") || name_str.eq_ignore_ascii_case("style") {
            self.in_script_or_style = false;
        }
        self.scan_policy
            .tracker
            .on_end_tag(name_str, is_void_element(name_str));
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

    pub fn on_comment(&mut self, _comment: &str, _line: usize, _col: usize) {}

    pub fn on_null_character(&mut self, _line: usize, _col: usize) {}

    pub fn on_parse_error(&mut self, err: Cow<'static, str>, line: usize, _col: usize) {
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
    ) {
        for rule in &mut self.rules {
            self.messages.extend(rule.on_finish(&self.ctx));
        }

        if let Some(unclosed) = self.ctx.open_elements.iter().find(|e| {
            ![
                "html", "body", "head", "p", "li", "dt", "dd", "tr", "td", "th",
            ]
            .iter()
            .any(|&t| t.eq_ignore_ascii_case(&e.name))
        }) {
            self.scan_policy.tracker.needs_scan = true;
            self.messages.push(VNuMessage::error(
                "End of file seen and there were open elements.".to_string(),
                unclosed.line,
                unclosed.col,
            ));
        }

        (self.ctx, self.rules, self.messages, self.scan_policy)
    }
}
