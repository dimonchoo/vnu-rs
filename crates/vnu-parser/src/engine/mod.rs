pub mod attribute_normalization;
pub mod dispatch;
pub mod highlight;
pub mod html_stack;
pub mod scan_policy;
pub mod sink_adapter;
pub mod stack_policy;
pub mod tag_column;
pub mod tree_adapter;

use self::dispatch::StreamingValidator;
use self::highlight::apply_highlight;
use self::sink_adapter::StreamingSink;
use crate::context::LineIndex;
use crate::rule::Rule;
use crate::rules::style::css_validator::CssValidator;
use crate::rules::{default_rules, rules_with_options};
use crate::schema::relaxng::SchemaValidator;
use html5ever::tokenizer::{BufferQueue, Tokenizer, TokenizerOpts, TokenizerResult};
use std::cell::RefCell;
use vnu_core::{VNuMessage, VNuReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckMode {
    Html,
    Css,
    Svg,
}

pub struct ValidatorEngine {
    rules: Vec<Box<dyn Rule>>,
    char_rule_indices: Vec<usize>,
    stray_rule_indices: Vec<usize>,
    line_index: LineIndex,
    /// Kept between documents: its derivative memos are a cache.
    schema: Option<SchemaValidator>,
}

impl Default for ValidatorEngine {
    fn default() -> Self {
        Self::new(default_rules())
    }
}

impl ValidatorEngine {
    pub fn with_options(enable_langdetect: bool) -> Self {
        Self::new(rules_with_options(enable_langdetect))
    }

    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        let char_rule_indices = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.handles_character_tokens())
            .map(|(i, _)| i)
            .collect();
        let stray_rule_indices = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.handles_stray_end_tags())
            .map(|(i, _)| i)
            .collect();
        Self {
            rules,
            char_rule_indices,
            stray_rule_indices,
            line_index: LineIndex::default(),
            schema: None,
        }
    }

    pub fn validate_as(&mut self, text: &str, mode: CheckMode) -> VNuReport {
        match mode {
            CheckMode::Html => self.validate(text),
            CheckMode::Css => {
                self.line_index.reset(text);
                let mut report = VNuReport::new();
                report.messages = CssValidator::validate_stylesheet(text, &self.line_index);
                self.attach_extracts(text, &mut report.messages);
                report
            }
            CheckMode::Svg => self.validate_svg(text),
        }
    }

    pub fn validate(&mut self, html: &str) -> VNuReport {
        let mut report = VNuReport::new();
        self.line_index.reset(html);

        for r in &mut self.rules {
            r.reset();
        }

        // Take rules and cached indices temporarily for streaming validation
        let active_rules = std::mem::take(&mut self.rules);
        let char_indices = std::mem::take(&mut self.char_rule_indices);
        let stray_indices = std::mem::take(&mut self.stray_rule_indices);
        let sink = StreamingSink {
            inner: RefCell::new(StreamingValidator::with_indices(
                html,
                active_rules,
                &self.line_index,
                char_indices,
                stray_indices,
                self.schema.take().unwrap_or_default(),
            )),
        };

        let tok = Tokenizer::new(sink, TokenizerOpts::default());
        let input = BufferQueue::default();
        input.push_back(html.into());

        let is_script_step = matches!(tok.feed(&input), TokenizerResult::Script(_));
        tok.end();

        // Reclaim rules, cached indices, and messages
        let mut inner = tok.sink.inner.into_inner();
        let char_indices = std::mem::take(&mut inner.char_rule_indices);
        let stray_indices = std::mem::take(&mut inner.stray_rule_indices);
        let halted_at = inner.halted_at();
        let (_ctx, rules, messages, mut scan_policy, schema) = inner.finish();
        self.schema = Some(schema);
        scan_policy
            .tracker
            .check_raw_script_step(is_script_step, html);

        self.rules = rules;
        self.char_rule_indices = char_indices;
        self.stray_rule_indices = stray_indices;
        report.messages.extend(messages);

        // Java stops at "Cannot recover after last error": nothing after that token
        // is reported, tokenizer errors included.
        scan_policy.maybe_scan(
            &html[..halted_at.unwrap_or(html.len())],
            &mut report.messages,
        );
        self.attach_extracts(html, &mut report.messages);
        report
    }

    fn validate_svg(&mut self, text: &str) -> VNuReport {
        match root_element_name(text) {
            Some(name) if name.eq_ignore_ascii_case("svg") => self.validate(text),
            Some(name) => {
                self.line_index.reset(text);
                let (line, col) = root_element_location(text, name);
                let mut report = VNuReport::new();
                report.messages.push(VNuMessage::error(
                    format!("Element “{name}” not allowed as the root of an SVG document."),
                    line,
                    col,
                ));
                self.attach_extracts(text, &mut report.messages);
                report
            }
            None => {
                let mut report = VNuReport::new();
                report
                    .messages
                    .push(VNuMessage::error("The document has no root element.", 1, 1));
                report
            }
        }
    }

    fn attach_extracts(&self, source: &str, messages: &mut [VNuMessage]) {
        for msg in messages {
            if let Some(l) = msg.last_line {
                let snippet = self.line_index.line_snippet(source, l);
                if !snippet.is_empty() {
                    if snippet.len() <= 512 {
                        let truncated = truncate_snippet(snippet, 120);
                        apply_highlight(snippet, truncated, msg);
                        msg.extract = Some(truncated.to_string());
                    } else {
                        // Highlighting searches for the first match on the line, which is
                        // the wrong tag when a minified line has many copies. Keep the
                        // column from the tag cursor and show the bytes around it.
                        let column = msg.first_column.unwrap_or(1);
                        msg.extract = Some(window_around(snippet, column));
                    }
                }
            }
        }
    }
}

fn truncate_snippet(snippet: &str, max_bytes: usize) -> &str {
    if snippet.len() <= max_bytes {
        return snippet;
    }
    let mut end = max_bytes;
    while end > 0 && !snippet.is_char_boundary(end) {
        end -= 1;
    }
    &snippet[..end]
}

fn window_around(snippet: &str, column: usize) -> String {
    let center = column.saturating_sub(1).min(snippet.len());
    let mut start = center.saturating_sub(40);
    while start > 0 && !snippet.is_char_boundary(start) {
        start -= 1;
    }
    let mut end = (center + 80).min(snippet.len());
    while end < snippet.len() && !snippet.is_char_boundary(end) {
        end += 1;
    }
    snippet[start..end].to_string()
}

fn root_element_name(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i] != b'<' {
            return None;
        }
        if text[i..].starts_with("<!--") {
            let Some(end) = text[i + 4..].find("-->") else {
                return None;
            };
            i += 4 + end + 3;
            continue;
        }
        if text[i..].starts_with("<?") {
            let Some(end) = text[i + 2..].find("?>") else {
                return None;
            };
            i += 2 + end + 2;
            continue;
        }
        if text[i..].starts_with("<!") {
            let Some(end) = text[i + 2..].find('>') else {
                return None;
            };
            i += 2 + end + 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < bytes.len()
            && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-' || bytes[end] == b':')
        {
            end += 1;
        }
        if end == start {
            return None;
        }
        return Some(&text[start..end]);
    }
    None
}

fn root_element_location(text: &str, name: &str) -> (usize, usize) {
    let bytes = text.as_bytes();
    let name_len = name.len();
    let mut i = 0;
    while i + 1 + name_len <= bytes.len() {
        if bytes[i] == b'<' && text[i + 1..i + 1 + name_len].eq_ignore_ascii_case(name) {
            let after = i + 1 + name_len;
            let boundary = after >= bytes.len()
                || !(bytes[after].is_ascii_alphanumeric()
                    || bytes[after] == b'-'
                    || bytes[after] == b':');
            if boundary {
                let line = text[..i].bytes().filter(|byte| *byte == b'\n').count() + 1;
                let col = text[..i].rfind('\n').map(|at| i - at).unwrap_or(i + 1);
                return (line, col);
            }
        }
        i += 1;
    }
    (1, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_mode_does_not_parse_the_file_as_html() {
        let mut engine = ValidatorEngine::default();
        let report = engine.validate_as("p { colr: red; }\n", CheckMode::Css);
        assert!(
            report
                .messages
                .iter()
                .any(|msg| msg.message.contains("colr"))
        );
        assert!(
            report
                .messages
                .iter()
                .all(|msg| !msg.message.contains("doctype"))
        );
    }

    #[test]
    fn svg_mode_rejects_an_html_root() {
        let mut engine = ValidatorEngine::default();
        let report = engine.validate_as(
            "<!DOCTYPE html><html lang=\"en\"><head><title>T</title></head><body></body></html>",
            CheckMode::Svg,
        );
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|msg| {
            msg.message
                .contains("not allowed as the root of an SVG document")
        }));
        assert!(
            report
                .messages
                .iter()
                .all(|msg| !msg.message.contains("doctype"))
        );
    }

    #[test]
    fn svg_mode_accepts_an_svg_root() {
        let mut engine = ValidatorEngine::default();
        let report = engine.validate_as(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 10 10\"></svg>",
            CheckMode::Svg,
        );
        assert!(report.is_valid(), "{:?}", report.messages);
        assert!(
            report
                .messages
                .iter()
                .any(|msg| msg.message.contains("preset for SVG"))
        );
        assert!(
            report
                .messages
                .iter()
                .all(|msg| !msg.message.contains("“html” start tag"))
        );
    }
}
