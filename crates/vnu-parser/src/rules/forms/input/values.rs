use vnu_core::VNuMessage;
use crate::context::ElementContext;
use crate::rules::datatypes::temporal::*;
use crate::rules::datatypes::tokens::*;
use super::allowed::is_attr_allowed_for_type;

fn check_url_attr(val: &str, attr: &str, line: usize, col: usize, msgs: &mut Vec<VNuMessage>) {
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
                format!("Bad value “{}” for attribute “{}” on element “input”.", val, attr),
                line,
                col,
            ));
        }
        UrlValidationResult::Valid => {}
    }
}

pub fn validate_input_values(tag: &ElementContext, effective_type: &str, msgs: &mut Vec<VNuMessage>) {
    if let Some(name) = tag.get_attr("name") {
        if name.is_empty() {
            msgs.push(VNuMessage::error(
                "Bad value “” for attribute “name” on element “input”.",
                tag.line,
                tag.col,
            ));
        }
    }

    if let Some(size) = tag.get_attr("size") {
        if !is_valid_positive_integer(size) {
            msgs.push(VNuMessage::error(
                format!("Bad value “{}” for attribute “size” on element “input”.", size),
                tag.line,
                tag.col,
            ));
        }
    }

    if let Some(ph) = tag.get_attr("placeholder") {
        if ph.contains('\n') || ph.contains('\r') {
            msgs.push(VNuMessage::error(
                format!("Bad value “{}” for attribute “placeholder” on element “input”.", ph),
                tag.line,
                tag.col,
            ));
        }
    }

    if effective_type == "color" {
        if let Some(val) = tag.get_attr("value") {
            if !is_valid_simple_color(val) {
                msgs.push(VNuMessage::error(
                    format!("Bad value “{}” for attribute “value” on element “input”.", val),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }

    if let Some(ac) = tag.get_attr("autocomplete") {
        if is_attr_allowed_for_type("autocomplete", effective_type) && !is_valid_autocomplete(ac) {
            msgs.push(VNuMessage::error(
                format!("Bad value “{}” for attribute “autocomplete” on element “input”.", ac),
                tag.line,
                tag.col,
            ));
        }
    }

    if effective_type == "url" {
        if let Some(val) = tag.get_attr("value") {
            let has_scheme = val.split_once(':').map(|(scheme, _)| {
                !scheme.is_empty() && scheme.chars().next().unwrap().is_ascii_alphabetic() && scheme.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
            }).unwrap_or(false);
            if !has_scheme || val.starts_with('#') || val.starts_with('/') {
                msgs.push(VNuMessage::error(
                    format!("Bad value “{}” for attribute “value” on element “input”.", val),
                    tag.line,
                    tag.col,
                ));
            } else {
                check_url_attr(val, "value", tag.line, tag.col, msgs);
            }
        }
    }

    if let Some(src) = tag.get_attr("src") {
        if src.trim().is_empty() {
            msgs.push(VNuMessage::error(
                format!("Bad value “{}” for attribute “src” on element “input”.", src),
                tag.line,
                tag.col,
            ));
        } else {
            check_url_attr(src, "src", tag.line, tag.col, msgs);
        }
    }

    if let Some(fa) = tag.get_attr("formaction") {
        if fa.trim().is_empty() {
            msgs.push(VNuMessage::error(
                format!("Bad value “{}” for attribute “formaction” on element “input”.", fa),
                tag.line,
                tag.col,
            ));
        } else {
            check_url_attr(fa, "formaction", tag.line, tag.col, msgs);
        }
    }

    for attr in &["min", "max", "value"] {
        if let Some(val) = tag.get_attr(attr) {
            let valid = match effective_type {
                "date" => is_valid_date(val),
                "month" => is_valid_month(val),
                "week" => is_valid_week(val),
                "time" => is_valid_time(val),
                "datetime-local" => is_valid_datetime_local(val),
                "number" | "range" => is_valid_float(val),
                _ => true,
            };
            if !valid {
                msgs.push(VNuMessage::error(
                    format!("Bad value “{}” for attribute “{}” on element “input”.", val, attr),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }

    if let Some(step) = tag.get_attr("step") {
        if !step.eq_ignore_ascii_case("any") && !is_valid_positive_float(step) {
            msgs.push(VNuMessage::error(
                format!("Bad value “{}” for attribute “step” on element “input”.", step),
                tag.line,
                tag.col,
            ));
        }
    }
}
