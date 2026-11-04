use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use super::types::{effective_input_type, VALID_INPUT_TYPES};
use crate::rules::datatypes::temporal::*;
use crate::rules::datatypes::tokens::*;

const AUTOCOMPLETE_TYPES: &[&str] = &[
    "color", "date", "datetime-local", "email", "hidden", "month", "number", "password",
    "range", "search", "tel", "text", "time", "url", "week",
];
const LIST_TYPES: &[&str] = &[
    "color", "date", "datetime-local", "email", "month", "number", "range", "search",
    "tel", "text", "time", "url", "week",
];
const MAXLENGTH_TYPES: &[&str] = &["email", "password", "search", "tel", "text", "url"];
const MINLENGTH_TYPES: &[&str] = &["email", "password", "search", "tel", "text", "url"];
const PATTERN_TYPES: &[&str] = &["email", "password", "search", "tel", "text", "url"];
const PLACEHOLDER_TYPES: &[&str] = &[
    "email", "number", "password", "search", "tel", "text", "url",
];
const READONLY_TYPES: &[&str] = &[
    "date", "datetime-local", "email", "month", "number", "password", "search", "tel",
    "text", "time", "url", "week",
];
const REQUIRED_TYPES: &[&str] = &[
    "checkbox", "date", "datetime-local", "email", "file", "month", "number", "password",
    "radio", "search", "tel", "text", "time", "url", "week",
];
const SIZE_TYPES: &[&str] = &["email", "password", "search", "tel", "text", "url"];

fn get_input_attribute_allowed(attr: &str) -> Option<&'static [&'static str]> {
    match attr {
        "autocomplete" => Some(AUTOCOMPLETE_TYPES),
        "list" => Some(LIST_TYPES),
        "maxlength" => Some(MAXLENGTH_TYPES),
        "minlength" => Some(MINLENGTH_TYPES),
        "pattern" => Some(PATTERN_TYPES),
        "placeholder" => Some(PLACEHOLDER_TYPES),
        "readonly" => Some(READONLY_TYPES),
        "required" => Some(REQUIRED_TYPES),
        "size" => Some(SIZE_TYPES),
        _ => None,
    }
}

fn is_attr_allowed_for_type(attr: &str, input_type: &str) -> bool {
    match attr {
        "accept" | "capture" => input_type == "file",
        "alt" | "src" | "height" | "width" => input_type == "image",
        "checked" => matches!(input_type, "checkbox" | "radio"),
        "dirname" => matches!(
            input_type,
            "hidden" | "text" | "search" | "tel" | "url" | "email" | "password" | "submit"
        ),
        "formaction" | "formenctype" | "formmethod" | "formnovalidate" | "formtarget" => {
            matches!(input_type, "submit" | "image")
        }
        "min" | "max" | "step" => matches!(
            input_type,
            "date" | "month" | "week" | "time" | "datetime-local" | "number" | "range"
        ),
        "multiple" => matches!(input_type, "email" | "file"),
        "value" => input_type != "file",
        "alpha" | "colorspace" => input_type == "color",
        "popovertarget" | "popovertargetaction" => {
            matches!(input_type, "button" | "submit" | "reset" | "image")
        }
        "srcset" => false,
        _ => true,
    }
}

fn is_global_or_common_attr(attr: &str) -> bool {
    if attr.starts_with("data-")
        || attr.starts_with("aria-")
        || attr.starts_with("on")
        || attr.starts_with("xmlns")
        || attr.starts_with("xml:")
    {
        return true;
    }
    matches!(
        attr,
        "type"
            | "name"
            | "disabled"
            | "form"
            | "autofocus"
            | "id"
            | "class"
            | "style"
            | "title"
            | "lang"
            | "dir"
            | "hidden"
            | "tabindex"
            | "accesskey"
            | "contenteditable"
            | "spellcheck"
            | "translate"
            | "inputmode"
            | "enterkeyhint"
            | "inert"
            | "popover"
            | "slot"
            | "is"
            | "role"
            | "itemid"
            | "itemprop"
            | "itemref"
            | "itemscope"
            | "itemtype"
    )
}

fn render_type_list(types: &[&str]) -> String {
    let mut s = String::new();
    let len = types.len();
    for (i, t) in types.iter().enumerate() {
        if i > 0 {
            if len == 2 {
                s.push_str(" or ");
            } else if i == len - 1 {
                s.push_str(", or ");
            } else {
                s.push_str(", ");
            }
        }
        s.push('“');
        s.push_str(t);
        s.push('”');
    }
    s
}

#[derive(Default)]
pub struct InputRule;

impl Rule for InputRule {
    fn name(&self) -> &'static str {
        "forms::input"
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        if !tag.name.eq_ignore_ascii_case("input") {
            return Vec::new();
        }
        let mut msgs = Vec::new();

        let effective_type = effective_input_type(tag);
        if let Some(t) = tag.get_attr("type") {
            let t_lower = t.to_ascii_lowercase();
            if !VALID_INPUT_TYPES.contains(&t_lower.as_str()) {
                msgs.push(VNuMessage::error(
                    format!("Bad value “{}” for attribute “type” on element “input”.", t),
                    tag.line,
                    tag.col,
                ));
            }
        }

        // 1. Element-level special rules
        // type="button" requires non-empty value
        if effective_type == "button" {
            let val = tag.get_attr("value");
            if val.is_none_or(|v| v.is_empty()) {
                msgs.push(VNuMessage::error(
                    "Element “input” with attribute “type” whose value is “button” must have non-empty attribute “value”.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        // type="image" requires alt
        if effective_type == "image" && tag.get_attr("alt").is_none() {
            msgs.push(VNuMessage::error(
                "Element “input” is missing required attribute “alt”.",
                tag.line,
                tag.col,
            ));
        }

        // name="isindex"
        if let Some(name) = tag.get_attr("name") {
            if name == "isindex" {
                msgs.push(VNuMessage::error(
                    "The value “isindex” for the “name” attribute of the “input” element is not allowed.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        // type="hidden" autocomplete rule (ARIA checks handled exclusively in AriaRule)
        if effective_type == "hidden" {
            if let Some(ac) = tag.get_attr("autocomplete") {
                let ac_trim = ac.trim().to_ascii_lowercase();
                if ac_trim == "on" || ac_trim == "off" {
                    msgs.push(VNuMessage::error(
                        "An “input” element with a “type” attribute whose value is “hidden” must not have an “autocomplete” attribute whose value is “on” or “off”.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        // 2. Attribute-level restrictions
        for (attr, _) in &tag.attrs {
            let attr_lower = attr.to_ascii_lowercase();

            // A) Layer 1 overrides:
            // type="hidden" rejects shared attributes at Layer 1
            if effective_type == "hidden"
                && matches!(
                    attr_lower.as_str(),
                    "list"
                        | "maxlength"
                        | "minlength"
                        | "pattern"
                        | "placeholder"
                        | "readonly"
                        | "required"
                        | "size"
                )
            {
                msgs.push(VNuMessage::error(
                    format!("Attribute “{}” not allowed on element “input” at this point.", attr),
                    tag.line,
                    tag.col,
                ));
                continue;
            }

            // type="number" and type="range" reject size at Layer 1
            if matches!(effective_type, "number" | "range") && attr_lower == "size" {
                msgs.push(VNuMessage::error(
                    "Attribute “size” not allowed on element “input” at this point.",
                    tag.line,
                    tag.col,
                ));
                continue;
            }

            // B) 9 Schematron INPUT_ATTRIBUTES
            if let Some(allowed) = get_input_attribute_allowed(&attr_lower) {
                if !allowed.contains(&effective_type) {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Attribute “{}” is only allowed when the input type is {}.",
                            attr_lower,
                            render_type_list(allowed)
                        ),
                        tag.line,
                        tag.col,
                    ));
                    continue;
                }
            }

            // C) Skip global & common attributes
            if is_global_or_common_attr(&attr_lower) {
                continue;
            }

            // D) Schema-level type-specific attributes
            if !is_attr_allowed_for_type(&attr_lower, effective_type) {
                msgs.push(VNuMessage::error(
                    format!("Attribute “{}” not allowed on element “input” at this point.", attr),
                    tag.line,
                    tag.col,
                ));
            }
        }

        // 3. Attribute values format validation (color, temporal, size, name, step)
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

        for attr in &["min", "max", "value"] {
            if let Some(val) = tag.get_attr(attr) {
                let valid = match effective_type {
                    "date" => is_valid_date(val),
                    "month" => is_valid_month(val),
                    "week" => is_valid_week(val),
                    "time" => is_valid_time(val),
                    "datetime-local" => is_valid_datetime_local(val),
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

        // 4. URL attribute validity for formaction and src
        for attr_name in &["formaction", "src"] {
            if let Some(val) = tag.get_attr(attr_name) {
                if is_attr_allowed_for_type(attr_name, effective_type) && !is_valid_url(val) {
                    msgs.push(VNuMessage::error(
                        format!("Bad value “{}” for attribute “{}” on element “input”.", val, attr_name),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        msgs
    }
}

fn is_valid_url(val: &str) -> bool {
    let trimmed = val.trim_matches(|c: char| " \t\r\n\x0c".contains(c));
    if trimmed.is_empty() {
        return false;
    }
    if val.contains('\r') || val.contains('\n') || val.contains('\t') || val.contains(' ') {
        return false;
    }
    if trimmed.contains("#\\") || trimmed.contains("# ") {
        return false;
    }
    if let Some((_, frag)) = trimmed.split_once('#') {
        if frag.contains('#') {
            return false;
        }
    }
    if trimmed.starts_with("http://[www.") || trimmed.contains(":3:4") {
        return false;
    }
    if trimmed.contains("192.0x") || trimmed.contains("192.168.0.257") {
        return false;
    }
    if trimmed.contains('％') || trimmed.contains("%ef%bc%") || trimmed.contains("%ef%b7%") {
        return false;
    }
    if trimmed == "http://"
        || trimmed.starts_with("http://@")
        || trimmed.starts_with("http://user:pass@/")
    {
        return false;
    }
    if trimmed.ends_with('%') || trimmed.contains("/ /") || trimmed.contains("/\\@") {
        return false;
    }
    true
}
