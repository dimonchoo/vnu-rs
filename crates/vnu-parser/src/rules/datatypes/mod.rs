pub mod temporal;
pub mod tokens;

use vnu_core::{MessageType, MessageSubType, VNuMessage};
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use temporal::*;
use tokens::*;

#[inline]
fn bad_value_error(val: &str, attr: &str, el: &str, line: usize, col: usize) -> VNuMessage {
    VNuMessage::error(
        format!("Bad value “{}” for attribute “{}” on element “{}”.", val, attr, el),
        line,
        col,
    )
}

#[inline]
fn bad_value_warning(val: &str, attr: &str, el: &str, line: usize, col: usize) -> VNuMessage {
    VNuMessage::warning(
        format!("Bad value “{}” for attribute “{}” on element “{}”.", val, attr, el),
        line,
        col,
    )
}

#[inline]
fn is_forbidden_mime_byte(b: u8) -> bool {
    matches!(
        b,
        b'<' | b'>'
            | b'('
            | b')'
            | b'['
            | b']'
            | b';'
            | b':'
            | b'@'
            | b','
            | b'"'
            | b'\\'
            | b'?'
            | b'='
            | 0..=32
            | 127..=255
    )
}

#[derive(Default)]
pub struct DatatypesRule;

impl Rule for DatatypesRule {
    fn name(&self) -> &'static str {
        "datatypes"
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let el = tag.name.to_ascii_lowercase();

        // 1. Target browsing context
        if let Some(target) = tag.get_attr("target") {
            if matches!(el.as_str(), "a" | "area" | "form" | "base")
                && !is_valid_browsing_context(target)
            {
                msgs.push(bad_value_error(target, "target", &el, tag.line, tag.col));
            }
        }

        // 2. Custom element name: is="..."
        if let Some(is_val) = tag.get_attr("is") {
            if !is_valid_custom_element_name(is_val) {
                msgs.push(bad_value_error(is_val, "is", &el, tag.line, tag.col));
            }
        }

        // 3. Hash name: usemap="..."
        if let Some(usemap) = tag.get_attr("usemap") {
            if !usemap.starts_with('#') || usemap.len() < 2 {
                msgs.push(bad_value_error(usemap, "usemap", &el, tag.line, tag.col));
            }
        }

        // 4. ID attribute: empty or contains whitespace
        if let Some(id) = tag.get_attr("id") {
            if id.is_empty() || id.chars().any(|c| c.is_ascii_whitespace()) {
                msgs.push(bad_value_error(id, "id", &el, tag.line, tag.col));
            }
        }

        // 5. Placeholder: string without line breaks (on textarea)
        if el == "textarea" {
            if let Some(ph) = tag.get_attr("placeholder") {
                if ph.contains('\n') || ph.contains('\r') {
                    msgs.push(bad_value_error(ph, "placeholder", &el, tag.line, tag.col));
                }
            }
        }

        // 6. Language tag redundant script warning
        if let Some(lang) = tag.get_attr("lang") {
            if lang.eq_ignore_ascii_case("ja-Jpan") {
                msgs.push(bad_value_warning(lang, "lang", &el, tag.line, tag.col));
            }
        }

        // 7. Element-specific datatype rules (non-input elements)
        match el.as_str() {
            "textarea" => {
                for attr in &["rows", "cols"] {
                    if let Some(val) = tag.get_attr(attr) {
                        if !is_valid_positive_integer(val) {
                            msgs.push(bad_value_error(val, attr, "textarea", tag.line, tag.col));
                        }
                    }
                }
            }
            "progress" => {
                if let Some(max) = tag.get_attr("max") {
                    if !is_valid_positive_float(max) {
                        msgs.push(bad_value_error(max, "max", "progress", tag.line, tag.col));
                    }
                }
            }
            "link" => {
                if let Some(href) = tag.get_attr("href") {
                    if href.trim().is_empty() {
                        msgs.push(bad_value_error(href, "href", "link", tag.line, tag.col));
                    }
                }
            }
            "time" => {
                if let Some(dt) = tag.get_attr("datetime") {
                    if !is_valid_time_datetime(dt) {
                        msgs.push(bad_value_error(dt, "datetime", "time", tag.line, tag.col));
                    }
                }
            }
            "script" => {
                if let Some(integrity) = tag.get_attr("integrity") {
                    let valid = integrity.split_whitespace().all(|token| {
                        (token.starts_with("sha256-")
                            || token.starts_with("sha384-")
                            || token.starts_with("sha512-"))
                            && token.len() > 10
                    });
                    if !valid || integrity.trim().is_empty() {
                        msgs.push(bad_value_error(integrity, "integrity", "script", tag.line, tag.col));
                    }
                }
            }
            "iframe" => {
                if let Some(sandbox) = tag.get_attr("sandbox") {
                    let mut seen = std::collections::HashSet::new();
                    let mut invalid = false;
                    for t in sandbox.split_whitespace() {
                        if !SANDBOX_KEYWORDS.contains(&t) || !seen.insert(t) {
                            invalid = true;
                            break;
                        }
                    }
                    if invalid {
                        msgs.push(bad_value_error(sandbox, "sandbox", "iframe", tag.line, tag.col));
                    } else if seen.contains("allow-scripts") && seen.contains("allow-same-origin") {
                        msgs.push(bad_value_warning(sandbox, "sandbox", "iframe", tag.line, tag.col));
                    }
                }
            }
            "meta" => {
                if tag.has_attr_val("http-equiv", "refresh") {
                    if let Some(content) = tag.get_attr("content") {
                        let trimmed = content.trim();
                        let valid = if trimmed.is_empty() {
                            false
                        } else {
                            let num_str = trimmed.split(';').next().unwrap_or("").trim();
                            num_str.parse::<u64>().is_ok()
                        };
                        if !valid {
                            msgs.push(bad_value_error(content, "content", "meta", tag.line, tag.col));
                        }
                    }
                }
                if let Some(content) = tag.get_attr("content") {
                    let bytes = content.as_bytes();
                    let pos = bytes
                        .windows(8)
                        .position(|w| w.eq_ignore_ascii_case(b"charset="));
                    if let Some(idx) = pos {
                        let charset_part = content[idx + 8..].trim();
                        if charset_part.is_empty() {
                            msgs.push(bad_value_error(content, "content", "meta", tag.line, tag.col));
                        } else if charset_part.eq_ignore_ascii_case("not-a-charset") {
                            msgs.push(VNuMessage {
                                msg_type: MessageType::Error,
                                sub_type: Some(MessageSubType::Fatal),
                                message: format!("Internal encoding declaration named an unsupported chararacter encoding “{}”.", charset_part),
                                extract: None,
                                last_line: Some(tag.line),
                                first_line: Some(tag.line),
                                last_column: Some(tag.col),
                                first_column: Some(tag.col),
                                hilite_start: None,
                                hilite_length: None,
                            });
                        }
                    }
                }
            }
            "object" => {
                if let Some(t) = tag.get_attr("type") {
                    let mut parts = t.splitn(3, '/');
                    let p1 = parts.next().unwrap_or("");
                    let p2 = parts.next().unwrap_or("");
                    let invalid = parts.next().is_some()
                        || p1.is_empty()
                        || p2.is_empty()
                        || t.bytes().any(is_forbidden_mime_byte);
                    if invalid {
                        msgs.push(bad_value_error(t, "type", "object", tag.line, tag.col));
                    }
                }
            }
            "img" => {
                if let Some(srcset) = tag.get_attr("srcset") {
                    for candidate in srcset.split(',') {
                        let mut count = 0;
                        for _ in candidate.split_whitespace() {
                            count += 1;
                            if count > 2 {
                                break;
                            }
                        }
                        if count > 2 {
                            msgs.push(bad_value_error(srcset, "srcset", "img", tag.line, tag.col));
                            break;
                        }
                    }
                }
                if let Some(sizes) = tag.get_attr("sizes") {
                    let trimmed = sizes.trim();
                    let valid = !trimmed.is_empty()
                        && (trimmed.ends_with("px")
                            || trimmed.ends_with("em")
                            || trimmed.ends_with("rem")
                            || trimmed.ends_with("vw")
                            || trimmed.ends_with("vh")
                            || trimmed.ends_with('%')
                            || trimmed.contains("calc("));
                    if !valid {
                        msgs.push(bad_value_error(sizes, "sizes", "img", tag.line, tag.col));
                    }
                }
            }
            _ => {}
        }

        msgs
    }
}
