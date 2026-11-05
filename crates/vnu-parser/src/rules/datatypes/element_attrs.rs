use super::bad_value_error;
use crate::context::ElementContext;
use vnu_core::VNuMessage;

pub fn validate_element_attributes(tag: &ElementContext, el: &str, msgs: &mut Vec<VNuMessage>) {
    // accesskey
    if let Some(ak) = tag.get_attr("accesskey") {
        let tokens: Vec<&str> = ak.split_whitespace().collect();
        let mut seen = std::collections::HashSet::new();
        let mut bad = false;
        for tok in &tokens {
            if tok.chars().count() != 1 || !seen.insert(tok) {
                bad = true;
                break;
            }
        }
        if bad {
            msgs.push(bad_value_error(ak, "accesskey", el, tag.line, tag.col));
        }
    }

    // enterkeyhint
    if let Some(ekh) = tag.get_attr("enterkeyhint") {
        if !matches!(
            ekh,
            "enter" | "done" | "go" | "next" | "previous" | "search" | "send"
        ) {
            msgs.push(bad_value_error(ekh, "enterkeyhint", el, tag.line, tag.col));
        }
    }

    // spellcheck
    if let Some(sc) = tag.get_attr("spellcheck") {
        if !matches!(sc, "" | "true" | "false") {
            msgs.push(bad_value_error(sc, "spellcheck", el, tag.line, tag.col));
        }
    }

    // headingoffset
    if let Some(ho) = tag.get_attr("headingoffset") {
        let n = ho.trim().parse::<i64>();
        if !matches!(n, Ok(0..=8)) {
            msgs.push(VNuMessage::error(
                "The value of the “headingoffset” attribute must be a number between “0” and “8”.",
                tag.line,
                tag.col,
            ));
        }
    }

    // template shadowrootslotassignment
    if el == "template" {
        if let Some(srsa) = tag.get_attr("shadowrootslotassignment") {
            if !matches!(srsa, "named" | "manual") {
                msgs.push(bad_value_error(
                    srsa,
                    "shadowrootslotassignment",
                    "template",
                    tag.line,
                    tag.col,
                ));
            }
        }
    }

    // Popover global attribute: "" | "auto" | "manual" | "hint"
    if let Some(popover) = tag.get_attr("popover") {
        if !matches!(
            popover.to_ascii_lowercase().as_str(),
            "" | "auto" | "manual" | "hint"
        ) {
            msgs.push(bad_value_error(popover, "popover", el, tag.line, tag.col));
        }
    }

    // iframe obsolete attributes
    if el == "iframe" {
        if tag.has_attr("allowpaymentrequest") {
            msgs.push(VNuMessage::error(
                "Attribute “allowpaymentrequest” not allowed on element “iframe” at this point.",
                tag.line,
                tag.col,
            ));
        }
        if tag.has_attr("seamless") {
            msgs.push(VNuMessage::error(
                "Attribute “seamless” not allowed on element “iframe” at this point.",
                tag.line,
                tag.col,
            ));
        }
    }

    // embed percentage width/height
    if el == "embed" {
        for dim in &["width", "height"] {
            if let Some(val) = tag.get_attr(dim) {
                if val.contains('%') {
                    msgs.push(bad_value_error(val, dim, "embed", tag.line, tag.col));
                }
            }
        }
    }

    // track non-empty label
    if el == "track" {
        if let Some(label) = tag.get_attr("label") {
            if label.trim().is_empty() {
                msgs.push(VNuMessage::error(
                    "Attribute “label” for element “track” must have non-empty value.",
                    tag.line,
                    tag.col,
                ));
            }
        }
    }

    // ol start attribute integer check
    if el == "ol" {
        if let Some(s) = tag.get_attr("start") {
            if s.trim().parse::<i64>().is_err() {
                msgs.push(bad_value_error(s, "start", "ol", tag.line, tag.col));
            }
        }
    }

    // td colspan="0"
    if el == "td" {
        if let Some(cs) = tag.get_attr("colspan") {
            if cs.trim() == "0" {
                msgs.push(bad_value_error(cs, "colspan", "td", tag.line, tag.col));
            }
        }
    }

    // audio/video loading attribute
    if (el == "audio" || el == "video") && tag.has_attr("loading") {
        if let Some(l) = tag.get_attr("loading") {
            if !matches!(l.to_ascii_lowercase().as_str(), "lazy" | "eager") {
                msgs.push(bad_value_error(l, "loading", el, tag.line, tag.col));
            }
        }
    }

    // Placeholder: string without line breaks (on textarea)
    if el == "textarea" {
        if let Some(ph) = tag.get_attr("placeholder") {
            if ph.contains('\n') || ph.contains('\r') {
                msgs.push(bad_value_error(ph, "placeholder", el, tag.line, tag.col));
            }
        }
    }
}
