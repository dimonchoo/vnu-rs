use super::dispatch::StreamingValidator;
use crate::context::DocumentMode;
use crate::treebuilder::TokenizerState;
use html5ever::tokenizer::states::RawKind;
use html5ever::tokenizer::{TagKind, Token, TokenSink, TokenSinkResult};
use std::cell::RefCell;

pub struct StreamingSink<'a> {
    pub inner: RefCell<StreamingValidator<'a>>,
}

impl<'a> TokenSink for StreamingSink<'a> {
    type Handle = ();

    fn adjusted_current_node_present_but_not_in_html_namespace(&self) -> bool {
        self.inner.borrow().in_foreign_content()
    }

    fn process_token(&self, token: Token, line_number: u64) -> TokenSinkResult<()> {
        let mut raw_kind = match &token {
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
                } else if n.eq_ignore_ascii_case("textarea")
                    || (n.eq_ignore_ascii_case("title")
                        && !{
                            let v = self.inner.borrow();
                            v.ctx.mode == DocumentMode::Svg || v.ctx.is_descendant_of("svg")
                        })
                {
                    Some(TokenSinkResult::RawData(RawKind::Rcdata))
                } else if n.eq_ignore_ascii_case("plaintext") {
                    Some(TokenSinkResult::Plaintext)
                } else {
                    None
                }
            }
            _ => None,
        };

        let line = line_number as usize;
        match token {
            Token::DoctypeToken(d) => {
                let mut inner = self.inner.borrow_mut();
                let col = inner.column_for_marker(line, "<!");
                inner.on_doctype(d, line, col);
            }
            Token::TagToken(tag) => match tag.kind {
                TagKind::StartTag => {
                    let mut attrs: Vec<(String, String)> = tag
                        .attrs
                        .into_iter()
                        .map(|a| (a.name.local.to_string(), a.value.to_string()))
                        .collect();

                    let mut inner = self.inner.borrow_mut();
                    let col = inner.column_for_tag(line, tag.name.as_ref(), false);
                    inner.normalizer.normalize_attributes(line, &mut attrs);
                    let state = inner.on_start_tag(
                        tag.name.to_string(),
                        attrs,
                        tag.self_closing,
                        line,
                        col,
                    );
                    // The tree builder knows the insertion mode and namespace, so its
                    // choice replaces the name-based guess above.
                    if inner.tree_active() {
                        raw_kind = state.map(|s| match s {
                            TokenizerState::Rcdata => TokenSinkResult::RawData(RawKind::Rcdata),
                            TokenizerState::Rawtext => TokenSinkResult::RawData(RawKind::Rawtext),
                            TokenizerState::ScriptData => {
                                TokenSinkResult::RawData(RawKind::ScriptData)
                            }
                            TokenizerState::Plaintext => TokenSinkResult::Plaintext,
                        });
                    }
                }
                TagKind::EndTag => {
                    let mut inner = self.inner.borrow_mut();
                    let col = inner.column_for_tag(line, tag.name.as_ref(), true);
                    inner.on_end_tag(tag.name.as_ref(), line, col);
                }
            },
            Token::CharacterTokens(text) => {
                self.inner.borrow_mut().on_characters(&text, line, 1);
            }
            Token::CommentToken(c) => {
                self.inner.borrow_mut().on_comment(&c, line, 1);
            }
            Token::NullCharacterToken => {
                self.inner.borrow_mut().on_null_character(line, 1);
            }
            Token::ParseError(err) => {
                self.inner.borrow_mut().on_parse_error(err, line, 1);
            }
            Token::EOFToken => {}
        }

        raw_kind.unwrap_or(TokenSinkResult::Continue)
    }
}
