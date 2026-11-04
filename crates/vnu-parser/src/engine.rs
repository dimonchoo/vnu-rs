use std::cell::RefCell;
use html5ever::tokenizer::{
    BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
    TokenizerResult,
};
use vnu_core::{VNuMessage, VNuReport};
use crate::context::{ElementContext, LineIndex, ValidationContext};
use crate::rule::Rule;
use crate::rules::default_rules;

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn is_void_element(name: &str) -> bool {
    VOID_ELEMENTS.iter().any(|&v| v.eq_ignore_ascii_case(name))
}

struct StreamingValidator<'a> {
    ctx: ValidationContext<'a>,
    rules: Vec<Box<dyn Rule>>,
    messages: Vec<VNuMessage>,
}

impl<'a> StreamingValidator<'a> {
    fn new(source: &'a str, rules: Vec<Box<dyn Rule>>) -> Self {
        Self {
            ctx: ValidationContext::new(source),
            rules,
            messages: Vec::new(),
        }
    }

    fn handle_token(&mut self, token: Token, line_u64: u64) {
        let line = line_u64 as usize;
        match token {
            Token::DoctypeToken(d) => {
                self.ctx.doctype_seen = true;
                let name = d.name.as_deref().unwrap_or("");
                for rule in &mut self.rules {
                    self.messages.extend(rule.on_doctype(&self.ctx, name, line, 1));
                }
            }
            Token::TagToken(tag) => {
                let name_str = tag.name.to_string();
                let col = 1;
                match tag.kind {
                    TagKind::StartTag => {
                        let attrs: Vec<(String, String)> = tag
                            .attrs
                            .into_iter()
                            .map(|a| (a.name.local.to_string(), a.value.to_string()))
                            .collect();

                        let elem = ElementContext {
                            name: name_str.clone(),
                            attrs,
                            line,
                            col,
                            byte_offset: 0,
                        };

                        if name_str.eq_ignore_ascii_case("html") {
                            self.ctx.html_seen = true;
                        } else if name_str.eq_ignore_ascii_case("head") {
                            self.ctx.head_seen = true;
                        } else if name_str.eq_ignore_ascii_case("body") {
                            self.ctx.body_seen = true;
                        } else if name_str.eq_ignore_ascii_case("title") {
                            self.ctx.title_seen = true;
                        }

                        for rule in &mut self.rules {
                            self.messages.extend(rule.on_start_tag(&self.ctx, &elem));
                        }

                        if !is_void_element(&name_str) && !tag.self_closing {
                            self.ctx.open_elements.push(elem);
                        }
                    }
                    TagKind::EndTag => {
                        if is_void_element(&name_str) {
                            self.messages.push(VNuMessage::error(
                                format!("End tag “{}”.", name_str),
                                line,
                                col,
                            ));
                            return;
                        }

                        if let Some(pos) = self.ctx.open_elements.iter().rposition(|e| e.name.eq_ignore_ascii_case(&name_str)) {
                            for rule in &mut self.rules {
                                self.messages.extend(rule.on_end_tag(&self.ctx, &name_str, line, col));
                            }
                            self.ctx.open_elements.truncate(pos);
                        } else {
                            self.messages.push(VNuMessage::error(
                                format!("Stray end tag “{}”.", name_str),
                                line,
                                col,
                            ));
                        }
                    }
                }
            }
            Token::ParseError(err) => {
                self.messages.push(VNuMessage::error(err.to_string(), line, 1));
            }
            _ => {}
        }
    }

    fn finish(mut self) -> (ValidationContext<'a>, Vec<Box<dyn Rule>>, Vec<VNuMessage>) {
        for rule in &mut self.rules {
            self.messages.extend(rule.on_finish(&self.ctx));
        }

        for unclosed in &self.ctx.open_elements {
            if !["html", "body", "head", "p", "li", "dt", "dd", "tr", "td", "th"]
                .iter()
                .any(|&t| t.eq_ignore_ascii_case(&unclosed.name))
            {
                self.messages.push(VNuMessage::error(
                    format!("End of file seen and there were open elements: “{}”.", unclosed.name),
                    unclosed.line,
                    unclosed.col,
                ));
            }
        }

        (self.ctx, self.rules, self.messages)
    }
}

struct StreamingSink<'a> {
    inner: RefCell<StreamingValidator<'a>>,
}

impl<'a> TokenSink for StreamingSink<'a> {
    type Handle = ();
    fn process_token(&self, token: Token, line_number: u64) -> TokenSinkResult<()> {
        self.inner.borrow_mut().handle_token(token, line_number);
        TokenSinkResult::Continue
    }
}

pub struct ValidatorEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl Default for ValidatorEngine {
    fn default() -> Self {
        Self {
            rules: default_rules(),
        }
    }
}

impl ValidatorEngine {
    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }

    pub fn validate(&mut self, html: &str) -> VNuReport {
        let mut report = VNuReport::new();
        let line_index = LineIndex::new(html);

        // Take rules temporarily for streaming validation
        let active_rules = std::mem::take(&mut self.rules);
        let sink = StreamingSink {
            inner: RefCell::new(StreamingValidator::new(html, active_rules)),
        };

        let tok = Tokenizer::new(sink, TokenizerOpts::default());
        let mut input = BufferQueue::default();
        input.push_back(html.into());

        match tok.feed(&mut input) {
            TokenizerResult::Done | TokenizerResult::Script(_) => {}
        }
        tok.end();

        // Reclaim rules and messages
        let inner = tok.sink.inner.into_inner();
        let (_ctx, rules, messages) = inner.finish();
        self.rules = rules;
        report.messages = messages;

        // Populate line extracts where possible
        for msg in &mut report.messages {
            if let Some(l) = msg.last_line {
                let snippet = line_index.line_snippet(html, l);
                if !snippet.is_empty() {
                    msg.extract = Some(snippet.to_string());
                }
            }
        }

        report
    }
}
