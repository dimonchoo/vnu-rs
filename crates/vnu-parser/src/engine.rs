use std::cell::RefCell;
use html5ever::tokenizer::states::RawKind;
use html5ever::tokenizer::{
    BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
    TokenizerResult,
};
use vnu_core::{MessageSubType, MessageType, VNuMessage, VNuReport};
use crate::context::{DocumentMode, ElementContext, LineIndex, ValidationContext};
use crate::rule::Rule;
use crate::rules::default_rules;

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn is_void_element(name: &str) -> bool {
    VOID_ELEMENTS.iter().any(|&v| v.eq_ignore_ascii_case(name))
}

fn apply_implicit_closures(open_elements: &mut Vec<ElementContext>, tag_name: &str, is_start_tag: bool) {
    if is_start_tag {
        if tag_name.eq_ignore_ascii_case("li") {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("li") {
                    open_elements.pop();
                }
            }
        } else if tag_name.eq_ignore_ascii_case("option") {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("option") {
                    open_elements.pop();
                }
            }
        } else if tag_name.eq_ignore_ascii_case("dt") || tag_name.eq_ignore_ascii_case("dd") {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("dt") || top.name.eq_ignore_ascii_case("dd") {
                    open_elements.pop();
                }
            }
        } else if tag_name.eq_ignore_ascii_case("td") || tag_name.eq_ignore_ascii_case("th") {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("td") || top.name.eq_ignore_ascii_case("th") {
                    open_elements.pop();
                }
            }
        } else if tag_name.eq_ignore_ascii_case("tr") {
            while let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("td")
                    || top.name.eq_ignore_ascii_case("th")
                    || top.name.eq_ignore_ascii_case("tr")
                {
                    open_elements.pop();
                } else {
                    break;
                }
            }
        } else if tag_name.eq_ignore_ascii_case("p")
            || ["div", "ul", "ol", "table", "h1", "h2", "h3", "h4", "h5", "h6",
                "pre", "blockquote", "form", "hr", "header", "footer", "nav", "section", "article"]
                .iter()
                .any(|&e| e.eq_ignore_ascii_case(tag_name))
        {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("p") {
                    open_elements.pop();
                }
            }
        }
    } else if tag_name.eq_ignore_ascii_case("ul") || tag_name.eq_ignore_ascii_case("ol") {
        if let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("li") {
                open_elements.pop();
            }
        }
    } else if tag_name.eq_ignore_ascii_case("dl") {
        if let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("dt") || top.name.eq_ignore_ascii_case("dd") {
                open_elements.pop();
            }
        }
    } else if tag_name.eq_ignore_ascii_case("tr") {
        if let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("td") || top.name.eq_ignore_ascii_case("th") {
                open_elements.pop();
            }
        }
    } else if tag_name.eq_ignore_ascii_case("datalist")
        || tag_name.eq_ignore_ascii_case("select")
        || tag_name.eq_ignore_ascii_case("optgroup")
    {
        if let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("option") {
                open_elements.pop();
            }
        }
    } else if ["table", "tbody", "thead", "tfoot"].iter().any(|&e| e.eq_ignore_ascii_case(tag_name)) {
        while let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("td")
                || top.name.eq_ignore_ascii_case("th")
                || top.name.eq_ignore_ascii_case("tr")
            {
                open_elements.pop();
            } else {
                break;
            }
        }
    }
}

struct StreamingValidator<'a> {
    ctx: ValidationContext<'a>,
    rules: Vec<Box<dyn Rule>>,
    messages: Vec<VNuMessage>,
    first_tag_seen: bool,
}

impl<'a> StreamingValidator<'a> {
    fn new(source: &'a str, rules: Vec<Box<dyn Rule>>) -> Self {
        Self {
            ctx: ValidationContext::new(source),
            rules,
            messages: Vec::new(),
            first_tag_seen: false,
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

                        if !self.first_tag_seen {
                            self.first_tag_seen = true;
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
                        } else if name_str.eq_ignore_ascii_case("head") {
                            self.ctx.head_seen = true;
                        } else if name_str.eq_ignore_ascii_case("body") {
                            self.ctx.body_seen = true;
                        } else if name_str.eq_ignore_ascii_case("title") {
                            self.ctx.title_seen = true;
                        }

                        if let Some(id) = elem.get_attr("id") {
                            self.ctx.seen_ids.insert(id.to_string());
                        }

                        // Implicit end tags for optional HTML5 elements
                        apply_implicit_closures(&mut self.ctx.open_elements, &name_str, true);

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

                        // Implicit closure before closing parent containers
                        apply_implicit_closures(&mut self.ctx.open_elements, &name_str, false);

                        if let Some(pos) = self.ctx.open_elements.iter().rposition(|e| e.name.eq_ignore_ascii_case(&name_str)) {
                            if pos + 1 < self.ctx.open_elements.len() {
                                let current_open = self.ctx.open_elements.last().map(|e| e.name.as_str()).unwrap_or("");
                                self.messages.push(VNuMessage::error(
                                    format!(
                                        "End tag “{}” did not match the name of the current open element (“{}”).",
                                        name_str, current_open
                                    ),
                                    line,
                                    col,
                                ));
                            }

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
        let raw_kind = match &token {
            Token::TagToken(tag) if tag.kind == TagKind::StartTag => {
                let n = tag.name.as_ref();
                if n.eq_ignore_ascii_case("script") {
                    Some(TokenSinkResult::RawData(RawKind::ScriptData))
                } else if n.eq_ignore_ascii_case("style")
                    || n.eq_ignore_ascii_case("iframe")
                    || n.eq_ignore_ascii_case("xmp")
                    || n.eq_ignore_ascii_case("noembed")
                    || n.eq_ignore_ascii_case("noframes")
                {
                    Some(TokenSinkResult::RawData(RawKind::Rawtext))
                } else if n.eq_ignore_ascii_case("title") || n.eq_ignore_ascii_case("textarea") {
                    Some(TokenSinkResult::RawData(RawKind::Rcdata))
                } else if n.eq_ignore_ascii_case("plaintext") {
                    Some(TokenSinkResult::Plaintext)
                } else {
                    None
                }
            }
            _ => None,
        };
        self.inner.borrow_mut().handle_token(token, line_number);
        raw_kind.unwrap_or(TokenSinkResult::Continue)
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

        for r in &mut self.rules {
            r.reset();
        }

        // Take rules temporarily for streaming validation
        let active_rules = std::mem::take(&mut self.rules);
        let sink = StreamingSink {
            inner: RefCell::new(StreamingValidator::new(html, active_rules)),
        };

        let tok = Tokenizer::new(sink, TokenizerOpts::default());
        let input = BufferQueue::default();
        input.push_back(html.into());

        match tok.feed(&input) {
            TokenizerResult::Done | TokenizerResult::Script(_) => {}
        }
        tok.end();

        // Reclaim rules and messages
        let inner = tok.sink.inner.into_inner();
        let (_ctx, mut rules, messages) = inner.finish();
        for r in &mut rules {
            r.reset();
        }
        self.rules = rules;
        report.messages = messages;

        // Populate line extracts where possible (truncated to 120 chars for performance)
        for msg in &mut report.messages {
            if let Some(l) = msg.last_line {
                let snippet = line_index.line_snippet(html, l);
                if !snippet.is_empty() {
                    let truncated = if snippet.len() > 120 {
                        &snippet[..120]
                    } else {
                        snippet
                    };
                    msg.extract = Some(truncated.to_string());
                }
            }
        }

        report
    }
}
