use std::cell::RefCell;
use html5ever::tokenizer::states::RawKind;
use html5ever::tokenizer::{
    TagKind, Token, TokenSink, TokenSinkResult,
};
use vnu_core::{MessageSubType, MessageType, VNuMessage};
use crate::context::{DocumentMode, ElementContext, LineIndex, ValidationContext};
use crate::engine::scan_policy::ScanPolicy;
use crate::rule::Rule;

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

pub fn is_void_element(name: &str) -> bool {
    VOID_ELEMENTS.iter().any(|&v| v.eq_ignore_ascii_case(name))
}

pub fn apply_implicit_closures(open_elements: &mut Vec<ElementContext>, tag_name: &str, is_start_tag: bool) {
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
            || ["dl", "div", "ul", "ol", "table", "h1", "h2", "h3", "h4", "h5", "h6",
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
    } else if tag_name.eq_ignore_ascii_case("dl") || tag_name.eq_ignore_ascii_case("div") {
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
}

impl<'a> StreamingValidator<'a> {
    pub fn new(source: &'a str, rules: Vec<Box<dyn Rule>>, line_index: &'a LineIndex) -> Self {
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
            ctx: ValidationContext::new(source),
            rules,
            char_rule_indices,
            stray_rule_indices,
            messages: Vec::new(),
            scan_policy: ScanPolicy::default(),
            in_script_or_style: false,
            first_tag_seen: false,
            line_index,
        }
    }

    pub fn handle_token(&mut self, token: Token, line_u64: u64) {
        let line = line_u64 as usize;
        match token {
            Token::DoctypeToken(d) => {
                self.scan_policy.tracker.on_doctype(&d, self.ctx.doctype_seen);
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

                        let byte_offset = self.line_index.line_offset(line);
                        self.scan_policy.tracker.on_start_tag(&name_str, &attrs, byte_offset);
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
                            self.ctx.id_to_tag.insert(id.to_string(), elem.name.to_ascii_lowercase());
                        }

                        // Implicit end tags for optional HTML5 elements
                        apply_implicit_closures(&mut self.ctx.open_elements, &name_str, true);

                        for rule in &mut self.rules {
                            self.messages.extend(rule.on_start_tag(&self.ctx, &elem));
                        }

                        if self.ctx.mode == DocumentMode::Html && tag.self_closing && !is_void_element(&name_str) {
                            self.messages.push(VNuMessage::error(
                                "Self-closing syntax (“/>”) used on a non-void HTML element. Ignoring the slash and treating as a start tag.",
                                line,
                                col,
                            ));
                            self.ctx.open_elements.push(elem);
                        } else if !is_void_element(&name_str) && !tag.self_closing {
                            self.ctx.open_elements.push(elem);
                        }
                    }
                    TagKind::EndTag => {
                        if name_str.eq_ignore_ascii_case("script") || name_str.eq_ignore_ascii_case("style") {
                            self.in_script_or_style = false;
                        }
                        self.scan_policy.tracker.on_end_tag(&name_str, is_void_element(&name_str));
                        if is_void_element(&name_str) {
                            let mut handled = false;
                            for &idx in &self.stray_rule_indices {
                                let stray = self.rules[idx].on_stray_end_tag(&self.ctx, &name_str, line, col);
                                if !stray.is_empty() {
                                    self.messages.extend(stray);
                                    handled = true;
                                    break;
                                }
                            }
                            if !handled {
                                self.messages.push(VNuMessage::error(
                                    format!("Stray end tag “{}”.", name_str),
                                    line,
                                    col,
                                ));
                            }
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
                        } else if name_str.eq_ignore_ascii_case("p") {
                            self.messages.push(VNuMessage::error(
                                "No “p” element in scope but a “p” end tag seen.".to_string(),
                                line,
                                col,
                            ));
                        } else {
                            let mut handled = false;
                            for &idx in &self.stray_rule_indices {
                                let stray = self.rules[idx].on_stray_end_tag(&self.ctx, &name_str, line, col);
                                if !stray.is_empty() {
                                    self.messages.extend(stray);
                                    handled = true;
                                    break;
                                }
                            }
                            if !handled {
                                self.messages.push(VNuMessage::error(
                                    format!("Stray end tag “{}”.", name_str),
                                    line,
                                    col,
                                ));
                            }
                        }
                    }
                }
            }
            Token::CharacterTokens(text) => {
                let s: &str = &text;
                self.scan_policy.tracker.on_characters(s, self.in_script_or_style);

                for &idx in &self.char_rule_indices {
                    if self.rules[idx].interested_in_characters(&self.ctx) {
                        self.messages.extend(self.rules[idx].on_characters(&self.ctx, s, line, 1));
                    }
                }
            }
            Token::ParseError(err) => {
                self.scan_policy.tracker.on_parse_error();
                self.messages.push(VNuMessage::error(err.to_string(), line, 1));
            }
            _ => {}
        }
    }

    pub fn finish(mut self) -> (ValidationContext<'a>, Vec<Box<dyn Rule>>, Vec<VNuMessage>, ScanPolicy) {
        for rule in &mut self.rules {
            self.messages.extend(rule.on_finish(&self.ctx));
        }

        if let Some(unclosed) = self.ctx.open_elements.iter().find(|e| {
            !["html", "body", "head", "p", "li", "dt", "dd", "tr", "td", "th"]
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

pub struct StreamingSink<'a> {
    pub inner: RefCell<StreamingValidator<'a>>,
}

impl<'a> TokenSink for StreamingSink<'a> {
    type Handle = ();
    fn process_token(&self, token: Token, line_number: u64) -> TokenSinkResult<()> {
        let raw_kind = match &token {
            Token::TagToken(tag) if tag.kind == TagKind::StartTag && !tag.self_closing => {
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
