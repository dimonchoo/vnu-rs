use super::tokens::{UrlValidationResult, validate_url_syntax};
use crate::context::ElementContext;
use vnu_core::VNuMessage;

#[inline]
pub fn check_url_attr(
    val: &str,
    attr: &str,
    el: &str,
    line: usize,
    col: usize,
    msgs: &mut Vec<VNuMessage>,
) {
    match validate_url_syntax(val) {
        UrlValidationResult::ForbiddenCodePoint(cp) => {
            msgs.push(VNuMessage::error(
                format!("Forbidden code point U+{:04x}.", cp),
                line,
                col,
            ));
        }
        UrlValidationResult::Invalid => {
            msgs.push(VNuMessage::error(
                format!(
                    "Bad value “{}” for attribute “{}” on element “{}”.",
                    val, attr, el
                ),
                line,
                col,
            ));
        }
        UrlValidationResult::Valid => {}
    }
}

pub fn validate_url_attributes(tag: &ElementContext, el: &str, msgs: &mut Vec<VNuMessage>) {
    if let Some(href) = tag.get_attr("href") {
        if el == "link"
            || (matches!(el, "a" | "area" | "base")
                && (!href.trim().is_empty() || href.contains(['\r', '\n', '\t'])))
        {
            check_url_attr(href, "href", el, tag.line, tag.col, msgs);
        }
    }
    if let Some(src) = tag.get_attr("src") {
        if matches!(
            el,
            "img" | "audio" | "video" | "source" | "track" | "iframe" | "embed" | "script"
        ) {
            check_url_attr(src, "src", el, tag.line, tag.col, msgs);
        }
    }
    if let Some(cite) = tag.get_attr("cite") {
        if matches!(el, "del" | "ins" | "blockquote" | "q")
            && (!cite.trim().is_empty() || cite.contains(['\r', '\n', '\t']))
        {
            check_url_attr(cite, "cite", el, tag.line, tag.col, msgs);
        }
    }
    if let Some(data) = tag.get_attr("data") {
        if el == "object" {
            check_url_attr(data, "data", el, tag.line, tag.col, msgs);
        }
    }
    if let Some(action) = tag.get_attr("action") {
        if el == "form" {
            check_url_attr(action, "action", el, tag.line, tag.col, msgs);
        }
    }
    if let Some(poster) = tag.get_attr("poster") {
        if el == "video" {
            check_url_attr(poster, "poster", "video", tag.line, tag.col, msgs);
        }
    }
    if let Some(itemtype) = tag.get_attr("itemtype") {
        if !tag.has_attr("itemscope") {
            msgs.push(VNuMessage::error(
                "The “itemtype” attribute must not be specified on elements that do not have an “itemscope” attribute specified.",
                tag.line,
                tag.col,
            ));
        }
        let mut bad = false;
        for token in itemtype.split_whitespace() {
            let has_scheme = if let Some(pos) = token.find(':') {
                let prefix = &token[..pos];
                !prefix.is_empty()
                    && prefix
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
                    && !token[..pos].contains(['/', '?', '#'])
            } else {
                false
            };
            if !has_scheme {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for attribute “itemtype” on element “{}”.",
                        itemtype, el
                    ),
                    tag.line,
                    tag.col,
                ));
                bad = true;
                break;
            }
        }
        if !bad {
            check_url_attr(itemtype, "itemtype", el, tag.line, tag.col, msgs);
        }
    }
    if let Some(itemid) = tag.get_attr("itemid") {
        if !itemid.trim().is_empty() || itemid.contains(['\r', '\n', '\t']) {
            check_url_attr(itemid, "itemid", el, tag.line, tag.col, msgs);
        }
    }
}
