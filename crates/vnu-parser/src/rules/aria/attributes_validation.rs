use super::naming::is_prohibited_from_being_named;
use super::roles::{PROHIBIT_INTERACTIVE_ROLES, VALID_ARIA_ATTRIBUTES};
use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

pub fn validate_descendant_constraints(
    ctx: &ValidationContext,
    tag: &ElementContext,
    elem_name: &str,
    msgs: &mut Vec<VNuMessage>,
) {
    if let Some(parent) = ctx.parent() {
        if parent.has_attr_val("role", "list") && tag.has_attr_val("role", "group") {
            msgs.push(VNuMessage::error(
                "An element with “role=group” must not be a child of an element with “role=list”.",
                tag.line,
                tag.col,
            ));
        }

        if parent.has_attr_val("role", "rowgroup")
            && !tag.has_attr_val("role", "row")
            && !elem_name.eq_ignore_ascii_case("tr")
        {
            msgs.push(VNuMessage::error(
                "An element that is a child of an element with “role=rowgroup” must have “role=row”.",
                tag.line,
                tag.col,
            ));
        }

        if parent.has_attr_val("role", "group") {
            if ctx.has_ancestor_role("menu") || ctx.has_ancestor_role("menubar") {
                let role = tag.get_attr("role").unwrap_or("");
                let is_menuitem = role.split_whitespace().any(|r| {
                    r.eq_ignore_ascii_case("menuitem")
                        || r.eq_ignore_ascii_case("menuitemcheckbox")
                        || r.eq_ignore_ascii_case("menuitemradio")
                });
                if !is_menuitem {
                    msgs.push(VNuMessage::error(
                        "An element with “role=group” that is a descendant of an element with “role=menu” or “role=menubar” must contain only elements with “role=menuitem”, “role=menuitemcheckbox”, or “role=menuitemradio”.",
                        tag.line,
                        tag.col,
                    ));
                }
            }

            if ctx.has_ancestor_role("tree") {
                let role = tag.get_attr("role").unwrap_or("");
                let is_treeitem = role
                    .split_whitespace()
                    .any(|r| r.eq_ignore_ascii_case("treeitem"));
                if !is_treeitem {
                    msgs.push(VNuMessage::error(
                        "An element with “role=group” that is a descendant of an element with “role=tree” must contain only elements with “role=treeitem”.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }
    }

    // Interactive descendants inside prohibited roles - walk upward only to first interactive role
    for parent in ctx.open_elements.iter().rev() {
        if let Some(r) = parent.get_attr("role") {
            for role_token in r.split_whitespace() {
                if PROHIBIT_INTERACTIVE_ROLES
                    .iter()
                    .any(|&p| p.eq_ignore_ascii_case(role_token))
                {
                    if elem_name.eq_ignore_ascii_case("a") && tag.has_attr("href") {
                        msgs.push(VNuMessage::warning(
                            format!(
                                "The element “a” with the attribute “href” should not appear as a descendant of an element with the attribute “role={}”.",
                                role_token
                            ),
                            tag.line,
                            tag.col,
                        ));
                    }
                    if tag.has_attr("tabindex") {
                        msgs.push(VNuMessage::warning(
                            format!(
                                "An element with the attribute “tabindex” should not appear as a descendant of an element with the attribute “role={}”.",
                                role_token
                            ),
                            tag.line,
                            tag.col,
                        ));
                    }
                }
            }
        }
    }
}

pub fn validate_naming_rules(tag: &ElementContext, elem_name: &str, msgs: &mut Vec<VNuMessage>) {
    for label_attr in &[
        "aria-label",
        "aria-labelledby",
        "aria-braillelabel",
        "aria-brailleroledescription",
    ] {
        if tag.has_attr(label_attr) && is_prohibited_from_being_named(elem_name, tag) {
            msgs.push(VNuMessage::error(
                format!(
                    "The “{}” attribute must not be specified on any “{}” element unless the element has a “role” value other than “caption”, “code”, “deletion”, “emphasis”, “generic”, “insertion”, “paragraph”, “presentation”, “strong”, “subscript”, or “superscript”.",
                    label_attr, elem_name
                ),
                tag.line,
                tag.col,
            ));
        }
    }
}

pub fn validate_attributes(tag: &ElementContext, elem_name: &str, msgs: &mut Vec<VNuMessage>) {
    if elem_name.eq_ignore_ascii_case("input") {
        let input_type = tag.get_attr("type").unwrap_or("text");
        if (input_type.eq_ignore_ascii_case("checkbox") || input_type.eq_ignore_ascii_case("radio"))
            && tag.has_attr("aria-checked")
        {
            msgs.push(VNuMessage::error(
                format!(
                    "The “aria-checked” attribute must not be used on an “input” element which has a “type” attribute whose value is “{}”.",
                    input_type
                ),
                tag.line,
                tag.col,
            ));
        }
        if input_type.eq_ignore_ascii_case("hidden") && tag.has_attr("aria-hidden") {
            msgs.push(VNuMessage::error(
                "The “aria-hidden” attribute must not be specified on an “input” element whose “type” attribute has the value “hidden”.",
                tag.line,
                tag.col,
            ));
        }
        if tag.has_attr("list") && tag.has_attr("aria-haspopup") {
            msgs.push(VNuMessage::warning(
                format!(
                    "The “aria-haspopup” attribute should not be used on an “input” element that has a “list” attribute and whose type is “{}”.",
                    input_type
                ),
                tag.line,
                tag.col,
            ));
        }
    }

    if tag.has_attr("aria-disabled") && tag.has_attr("disabled") {
        msgs.push(VNuMessage::warning(
            "Attribute “aria-disabled” is unnecessary for elements that have attribute “disabled”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("aria-hidden") && tag.has_attr("hidden") {
        msgs.push(VNuMessage::warning(
            "Attribute “aria-hidden” is unnecessary for elements that have attribute “hidden”.",
            tag.line,
            tag.col,
        ));
    }

    if elem_name.eq_ignore_ascii_case("a")
        && tag.has_attr("href")
        && tag.has_attr_val("aria-disabled", "true")
    {
        msgs.push(VNuMessage::warning(
            "An “aria-disabled” attribute whose value is “true” should not be specified on an “a” element that has an “href” attribute.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("aria-grabbed") {
        msgs.push(VNuMessage::warning(
            "The “aria-grabbed” attribute is deprecated and should not be used. Support for it is poor and is unlikely to improve.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("aria-dropeffect") {
        msgs.push(VNuMessage::warning(
            "The “aria-dropeffect” attribute is deprecated and should not be used. Support for it is poor and is unlikely to improve.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("aria-expanded") {
        if tag.has_attr("popovertarget") {
            msgs.push(VNuMessage::error(
                "The “aria-expanded” attribute must not be used on any element which has a “popovertarget” attribute.",
                tag.line,
                tag.col,
            ));
        }
        if tag.has_attr("command") {
            msgs.push(VNuMessage::error(
                "The “aria-expanded” attribute must not be used on any element which has a “command” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }

    if tag.has_attr_val("aria-hidden", "true") && tag.has_attr_val("hidden", "until-found") {
        msgs.push(VNuMessage::error(
            "Attribute “aria-hidden” with value “true” must not be specified on elements with “hidden” attribute value “until-found”. This combination prevents content from being accessible to assistive technology when revealed through search.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("aria-checked") {
        const ALLOWED_ARIA_CHECKED_ROLES: &[&str] = &[
            "checkbox",
            "menuitemcheckbox",
            "menuitemradio",
            "option",
            "radio",
            "switch",
            "treeitem",
        ];
        let role = tag.get_attr("role").unwrap_or("");
        let is_allowed_role = role.split_whitespace().any(|r| {
            ALLOWED_ARIA_CHECKED_ROLES
                .iter()
                .any(|&a| a.eq_ignore_ascii_case(r))
        });
        let is_native_checkable = elem_name.eq_ignore_ascii_case("input")
            && (tag.has_attr_val("type", "checkbox") || tag.has_attr_val("type", "radio"));
        if !is_allowed_role && !is_native_checkable {
            msgs.push(VNuMessage::error(
                format!(
                    "Attribute “aria-checked” not allowed on element “{}” at this point.",
                    elem_name
                ),
                tag.line,
                tag.col,
            ));
        }
    }

    const NO_ARIA_ELEMENTS: &[&str] = &["meta", "base", "head", "html"];
    for (attr_name, _) in tag.attrs.iter() {
        if attr_name.starts_with("aria-")
            && (NO_ARIA_ELEMENTS
                .iter()
                .any(|&e| e.eq_ignore_ascii_case(elem_name))
                || !VALID_ARIA_ATTRIBUTES
                    .iter()
                    .any(|&a| a.eq_ignore_ascii_case(attr_name)))
        {
            msgs.push(VNuMessage::error(
                format!(
                    "Attribute “{}” not allowed on element “{}” at this point.",
                    attr_name, elem_name
                ),
                tag.line,
                tag.col,
            ));
        }
    }
}
