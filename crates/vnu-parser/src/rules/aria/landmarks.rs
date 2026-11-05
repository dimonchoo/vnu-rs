use crate::context::{ElementContext, ValidationContext};
use std::collections::HashSet;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct AriaMiscState {
    pub visible_main_count: usize,
    pub visible_role_main_count: usize,
    pub tab_count: usize,
    pub tabpanel_count: usize,
    pub listbox_owned_ids: HashSet<String>,
}

pub fn validate_landmarks_and_counts(
    tag: &ElementContext,
    elem_name: &str,
    roles: &[&str],
    state: &mut AriaMiscState,
    msgs: &mut Vec<VNuMessage>,
) {
    if roles.len() > 1 {
        for r in roles {
            if *r == "input" {
                msgs.push(VNuMessage::error(
                    "Discarding unrecognized token “input” from value of attribute “role”. Browsers ignore any token that is not a defined ARIA non-abstract role.",
                    tag.line,
                    tag.col,
                ));
            }
        }
        if roles == ["button", "link"] {
            msgs.push(VNuMessage::info(
                "Discarding superfluous token “link” from value of attribute “role”. Browsers only process the first token found that is a defined ARIA non-abstract role.",
                tag.line,
                tag.col,
            ));
        }
    }

    // Main element tracking
    let is_hidden = tag.has_attr("hidden");
    if elem_name.eq_ignore_ascii_case("main") && !is_hidden {
        state.visible_main_count += 1;
        if state.visible_main_count > 1 {
            msgs.push(VNuMessage::error(
                "A document must not include more than one visible “main” element.",
                tag.line,
                tag.col,
            ));
        }
    }

    if roles.iter().any(|&r| r.eq_ignore_ascii_case("main")) && !is_hidden {
        state.visible_role_main_count += 1;
        if state.visible_role_main_count > 1 || state.visible_main_count >= 1 {
            msgs.push(VNuMessage::warning(
                "A document should not include more than one visible element with “role=main”.",
                tag.line,
                tag.col,
            ));
        }
    }

    // Tab and tabpanel tracking
    if roles.iter().any(|&r| r.eq_ignore_ascii_case("tab"))
        && tag.has_attr_val("aria-selected", "true")
    {
        state.tab_count += 1;
    }
    if roles.iter().any(|&r| r.eq_ignore_ascii_case("tabpanel")) {
        state.tabpanel_count += 1;
    }
    if roles.iter().any(|&r| r.eq_ignore_ascii_case("listbox")) {
        if let Some(owns) = tag.get_attr("aria-owns") {
            for id in owns.split_whitespace() {
                state.listbox_owned_ids.insert(id.to_string());
            }
        }
    }
}

pub fn validate_role_landmarks_and_headings(
    ctx: &ValidationContext,
    tag: &ElementContext,
    elem_name: &str,
    roles: &[&str],
    msgs: &mut Vec<VNuMessage>,
) {
    // Prohibited Global ARIA on role=main
    if roles.iter().any(|&r| r.eq_ignore_ascii_case("main")) {
        for attr in &["aria-disabled", "aria-haspopup", "aria-invalid"] {
            if tag.has_attr(attr) {
                msgs.push(VNuMessage::warning(
                    format!(
                        "The “{}” attribute should not be used on any element which has “role=main”.",
                        attr
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }

    // aria-hidden=true on body
    if elem_name.eq_ignore_ascii_case("body") && tag.has_attr_val("aria-hidden", "true") {
        msgs.push(VNuMessage::error(
            "“aria-hidden=true” must not be used on the “body” element.",
            tag.line,
            tag.col,
        ));
    }

    if elem_name == "div" && tag.has_attr("aria-readonly") && !tag.has_attr("role") {
        msgs.push(VNuMessage::error(
            "Element “div” is missing one or more of the following attributes: “aria-checked”, “aria-expanded”, “aria-valuenow”, “role”.",
            tag.line,
            tag.col,
        ));
    }
    if elem_name == "output" && tag.has_attr("aria-pressed") {
        msgs.push(VNuMessage::error(
            "Element “output” is missing required attribute “role”.",
            tag.line,
            tag.col,
        ));
    }

    // Prohibited roles inside headings h1-h6
    const PROHIBITED_HEADING_ROLES: &[&str] = &[
        "alert",
        "alertdialog",
        "application",
        "dialog",
        "document",
        "feed",
        "listbox",
        "log",
        "marquee",
        "math",
        "note",
        "status",
        "tabpanel",
        "timer",
        "toolbar",
    ];

    let is_inside_heading = ctx.open_elements.iter().any(|e| {
        let n = e.name.as_str();
        n.len() == 2
            && (n.as_bytes()[0] == b'h' || n.as_bytes()[0] == b'H')
            && (b'1'..=b'6').contains(&n.as_bytes()[1])
    });

    if is_inside_heading {
        for &r in roles {
            if PROHIBITED_HEADING_ROLES
                .iter()
                .any(|&p| p.eq_ignore_ascii_case(r))
            {
                msgs.push(VNuMessage::error(
                    format!(
                        "An element with the attribute “role={}” must not appear as a descendant of an “h1”, “h2”, “h3”, “h4”, “h5”, or “h6” element.",
                        r
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }
}
