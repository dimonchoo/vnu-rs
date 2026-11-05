use super::temporal::*;
use super::tokens::*;
use super::urls::check_url_attr;
use super::{bad_value_error, bad_value_warning};
use crate::context::ElementContext;
use vnu_core::{MessageSubType, MessageType, VNuMessage};

const SANDBOX_KEYWORDS: &[&str] = &[
    "allow-forms",
    "allow-modals",
    "allow-orientation-lock",
    "allow-pointer-lock",
    "allow-popups",
    "allow-popups-to-escape-sandbox",
    "allow-presentation",
    "allow-same-origin",
    "allow-scripts",
    "allow-top-navigation",
    "allow-top-navigation-by-user-activation",
    "allow-top-navigation-to-custom-protocols",
    "allow-downloads",
];

pub fn validate_element_datatypes(tag: &ElementContext, el: &str, msgs: &mut Vec<VNuMessage>) {
    match el {
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
            if let Some(val) = tag.get_attr("value") {
                if !is_valid_float(val) || val.trim().parse::<f64>().is_ok_and(|v| v < 0.0) {
                    msgs.push(bad_value_error(val, "value", "progress", tag.line, tag.col));
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
        "del" | "ins" => {
            if let Some(dt) = tag.get_attr("datetime") {
                if !is_valid_del_ins_datetime(dt) {
                    msgs.push(bad_value_error(dt, "datetime", el, tag.line, tag.col));
                }
            }
        }
        "button" => {
            if let Some(fa) = tag.get_attr("formaction") {
                if fa.trim().is_empty() {
                    msgs.push(bad_value_error(
                        fa,
                        "formaction",
                        "button",
                        tag.line,
                        tag.col,
                    ));
                } else {
                    check_url_attr(fa, "formaction", "button", tag.line, tag.col, msgs);
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
                    msgs.push(bad_value_error(
                        integrity,
                        "integrity",
                        "script",
                        tag.line,
                        tag.col,
                    ));
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
                    msgs.push(bad_value_error(
                        sandbox, "sandbox", "iframe", tag.line, tag.col,
                    ));
                } else if seen.contains("allow-scripts") && seen.contains("allow-same-origin") {
                    msgs.push(bad_value_warning(
                        sandbox, "sandbox", "iframe", tag.line, tag.col,
                    ));
                }
            }
        }
        "meta" => {
            if let Some(content) = tag.get_attr("content") {
                let bytes = content.as_bytes();
                let pos = bytes
                    .windows(8)
                    .position(|w| w.eq_ignore_ascii_case(b"charset="));
                if let Some(idx) = pos {
                    let charset_part = content[idx + 8..].trim();
                    if charset_part.is_empty() {
                        msgs.push(bad_value_error(
                            content, "content", "meta", tag.line, tag.col,
                        ));
                    } else if charset_part.eq_ignore_ascii_case("not-a-charset") {
                        msgs.push(VNuMessage {
                            msg_type: MessageType::Error,
                            url: None,
                            sub_type: Some(MessageSubType::Fatal),
                            message: format!(
                                "Internal encoding declaration named an unsupported chararacter encoding “{}”.",
                                charset_part
                            ),
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
        "img" => {
            if let Some(controls) = tag.get_attr("controls") {
                if !controls.is_empty() && !controls.eq_ignore_ascii_case("controls") {
                    msgs.push(bad_value_error(
                        controls, "controls", "img", tag.line, tag.col,
                    ));
                }
            }
            for dim in &["width", "height"] {
                if let Some(val) = tag.get_attr(dim) {
                    if let Ok(num) = val.trim().parse::<i64>() {
                        if num < 0 {
                            msgs.push(bad_value_error(val, dim, "img", tag.line, tag.col));
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
