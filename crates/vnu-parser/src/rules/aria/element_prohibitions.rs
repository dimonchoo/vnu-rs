use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

pub fn validate_element_prohibitions(
    ctx: &ValidationContext,
    tag: &ElementContext,
    elem_name: &str,
    roles: &[&str],
    msgs: &mut Vec<VNuMessage>,
) {
    // Prohibited ARIA on br and wbr
    if matches!(elem_name, "br" | "wbr") {
        if tag.has_attr("aria-atomic") {
            msgs.push(VNuMessage::error(
                format!(
                    "Attribute “aria-atomic” not allowed on element “{}” at this point.",
                    elem_name
                ),
                tag.line,
                tag.col,
            ));
        }
        if let Some(r) = tag.get_attr("role") {
            if !r.eq_ignore_ascii_case("none") && !r.eq_ignore_ascii_case("presentation") {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for attribute “role” on element “{}”.",
                        r, elem_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }

    // Prohibited ARIA attributes on div without appropriate role
    if roles.iter().any(|&r| r.eq_ignore_ascii_case("listbox"))
        && tag.has_attr("aria-expanded")
        && elem_name.eq_ignore_ascii_case("div")
    {
        msgs.push(VNuMessage::error(
            "Attribute “aria-expanded” not allowed on element “div” at this point.",
            tag.line,
            tag.col,
        ));
    }

    const MULTISELECTABLE_ROLES: &[&str] = &["grid", "listbox", "tablist", "tree", "treegrid"];
    if tag.has_attr("aria-multiselectable") {
        let ok = roles.iter().any(|r| {
            MULTISELECTABLE_ROLES
                .iter()
                .any(|&m| m.eq_ignore_ascii_case(r))
        });
        if !ok && elem_name.eq_ignore_ascii_case("div") {
            msgs.push(VNuMessage::error(
                "Attribute “aria-multiselectable” not allowed on element “div” at this point.",
                tag.line,
                tag.col,
            ));
        }
    }

    if tag.has_attr("aria-placeholder") {
        let ok = roles
            .iter()
            .any(|r| r.eq_ignore_ascii_case("textbox") || r.eq_ignore_ascii_case("searchbox"));
        if !ok && elem_name.eq_ignore_ascii_case("div") {
            msgs.push(VNuMessage::error(
                "Attribute “aria-placeholder” not allowed on element “div” at this point.",
                tag.line,
                tag.col,
            ));
        }
    }

    const READONLY_ROLES: &[&str] = &[
        "checkbox",
        "combobox",
        "grid",
        "gridcell",
        "listbox",
        "radiogroup",
        "slider",
        "spinbutton",
        "textbox",
        "treegrid",
        "columnheader",
        "rowheader",
        "searchbox",
        "switch",
    ];
    if tag.has_attr("aria-readonly") {
        let ok = roles
            .iter()
            .any(|r| READONLY_ROLES.iter().any(|&ro| ro.eq_ignore_ascii_case(r)));
        if !ok && elem_name.eq_ignore_ascii_case("div") {
            if tag.has_attr("contenteditable") {
                msgs.push(VNuMessage::error(
                    "Element “div” is missing one or more of the following attributes: “aria-checked”, “aria-expanded”, “aria-valuenow”, “role”.",
                    tag.line,
                    tag.col,
                ));
            } else {
                msgs.push(VNuMessage::error(
                    "Attribute “aria-readonly” not allowed on element “div” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
        }
    }

    const SELECTED_ROLES: &[&str] = &[
        "gridcell",
        "option",
        "row",
        "tab",
        "columnheader",
        "rowheader",
        "treeitem",
    ];
    if tag.has_attr("aria-selected") {
        let ok = roles
            .iter()
            .any(|r| SELECTED_ROLES.iter().any(|&ro| ro.eq_ignore_ascii_case(r)));
        if !ok && elem_name.eq_ignore_ascii_case("div") {
            msgs.push(VNuMessage::error(
                "Attribute “aria-selected” not allowed on element “div” at this point.",
                tag.line,
                tag.col,
            ));
        }
    }

    const RANGE_ROLES: &[&str] = &[
        "meter",
        "progressbar",
        "scrollbar",
        "separator",
        "slider",
        "spinbutton",
    ];
    if tag.has_attr("aria-valuemax") {
        let ok = roles
            .iter()
            .any(|r| RANGE_ROLES.iter().any(|&m| m.eq_ignore_ascii_case(r)));
        if !ok && elem_name.eq_ignore_ascii_case("div") {
            msgs.push(VNuMessage::error(
                "Attribute “aria-valuemax” not allowed on element “div” at this point.",
                tag.line,
                tag.col,
            ));
        }
    }

    if tag.has_attr("aria-valuemin") {
        let ok = roles
            .iter()
            .any(|&r| RANGE_ROLES.iter().any(|&m| m.eq_ignore_ascii_case(r)));
        if !ok && elem_name.eq_ignore_ascii_case("div") {
            msgs.push(VNuMessage::error(
                "Attribute “aria-valuemin” not allowed on element “div” at this point.",
                tag.line,
                tag.col,
            ));
        }
    }

    // aria-valuemax / aria-valuemin conflicts
    if tag.has_attr("aria-valuemax") && tag.has_attr("max") {
        msgs.push(VNuMessage::error(
            "The “aria-valuemax” attribute must not be used on an element which has a “max” attribute.",
            tag.line,
            tag.col,
        ));
    }
    if tag.has_attr("aria-valuemin") && tag.has_attr("min") {
        msgs.push(VNuMessage::error(
            "The “aria-valuemin” attribute must not be used on an element which has a “min” attribute.",
            tag.line,
            tag.col,
        ));
    }
    if elem_name.eq_ignore_ascii_case("input") && tag.has_attr_val("type", "number") {
        if tag.has_attr("aria-valuemax") {
            msgs.push(VNuMessage::warning(
                "The “aria-valuemax” attribute should not be used on an “input” element which has a “type” attribute whose value is “number”.",
                tag.line,
                tag.col,
            ));
        }
        if tag.has_attr("aria-valuemin") {
            msgs.push(VNuMessage::warning(
                "The “aria-valuemin” attribute should not be used on an “input” element which has a “type” attribute whose value is “number”.",
                tag.line,
                tag.col,
            ));
        }
    }

    // Associated label restrictions
    if elem_name.eq_ignore_ascii_case("label") && tag.has_attr("for") {
        if tag.has_attr("aria-label") {
            msgs.push(VNuMessage::error(
                "The “aria-label” attribute must not be used on any “label” element that is associated with a labelable element.",
                tag.line,
                tag.col,
            ));
        }
        if tag.has_attr("role") {
            msgs.push(VNuMessage::error(
                "The “role” attribute must not be used on any “label” element that is associated with a labelable element.",
                tag.line,
                tag.col,
            ));
        }
    }

    // Summary for parent details
    if elem_name.eq_ignore_ascii_case("summary") {
        let role = tag.get_attr("role").unwrap_or("");
        if tag.has_attr("role")
            && ctx
                .parent()
                .map(|p| p.name.eq_ignore_ascii_case("details"))
                .unwrap_or(false)
        {
            msgs.push(VNuMessage::error(
                "The “role” attribute must not be used on any “summary” element that is a summary for its parent “details” element.",
                tag.line,
                tag.col,
            ));
        }
        if role.is_empty() {
            if tag.has_attr("aria-expanded") {
                msgs.push(VNuMessage::error(
                    "Element “summary” is missing one or more of the following attributes: “aria-checked”, “aria-level”, “role”.",
                    tag.line,
                    tag.col,
                ));
            }
            if tag.has_attr("aria-pressed") {
                msgs.push(VNuMessage::error(
                    "Element “summary” is missing required attribute “role”.",
                    tag.line,
                    tag.col,
                ));
            }
            if tag.has_attr("aria-selected") {
                msgs.push(VNuMessage::error(
                    "Element “summary” is missing one or more of the following attributes: “aria-checked”, “role”.",
                    tag.line,
                    tag.col,
                ));
            }
        }
    }
}
