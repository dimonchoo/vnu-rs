use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

pub fn check_button_in_select(
    ctx: &ValidationContext,
    tag: &ElementContext,
    msgs: &mut Vec<VNuMessage>,
) {
    if !tag.name.eq_ignore_ascii_case("button") {
        return;
    }
    let Some(parent) = ctx.parent() else {
        return;
    };
    if !parent.name.eq_ignore_ascii_case("select") {
        return;
    }

    let mut select_is_dropdown = true;
    let select_has_multiple = parent.has_attr("multiple");
    if let Some(size_str) = parent.get_attr("size") {
        if let Ok(size_val) = size_str.trim().parse::<usize>() {
            if size_val > 1 {
                select_is_dropdown = false;
            }
        }
    } else if select_has_multiple {
        select_is_dropdown = false;
    }

    if !select_is_dropdown {
        msgs.push(VNuMessage::error(
            "A “button” element is only allowed as a child of a “select” element that is a drop-down box (one without a “size” attribute greater than 1 and without a “multiple” attribute).",
            tag.line,
            tag.col,
        ));
    } else {
        for (attr_name, _) in &tag.attrs {
            let lower = attr_name.to_ascii_lowercase();
            if lower == "role" || lower.starts_with("aria-") {
                msgs.push(VNuMessage::error(
                    format!(
                        "The “{}” attribute must not be used on a “button” element that is a child of a “select” element.",
                        lower
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }
}

pub fn check_selectedcontent(
    ctx: &ValidationContext,
    tag: &ElementContext,
    msgs: &mut Vec<VNuMessage>,
) {
    if !tag.name.eq_ignore_ascii_case("selectedcontent") {
        return;
    }

    let mut inside_button_in_select = false;
    for (i, elem) in ctx.open_elements.iter().enumerate().rev() {
        if elem.name.eq_ignore_ascii_case("button") {
            if i > 0 && ctx.open_elements[i - 1].name.eq_ignore_ascii_case("select") {
                inside_button_in_select = true;
            }
            break;
        }
    }

    if inside_button_in_select {
        for (attr_name, _) in &tag.attrs {
            let lower = attr_name.to_ascii_lowercase();
            if lower == "role" || lower.starts_with("aria-") {
                msgs.push(VNuMessage::error(
                    format!(
                        "The “{}” attribute must not be used on a “selectedcontent” element inside the “button” part of a customizable “select” element.",
                        lower
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }
}
