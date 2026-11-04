pub mod naming;
pub mod roles;

use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use naming::is_prohibited_from_being_named;
use roles::{PROHIBIT_INTERACTIVE_ROLES, VALID_ARIA_ATTRIBUTES, VALID_ARIA_ROLES};
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct AriaRule {
    pending_activedescendants: Vec<(String, usize, usize)>,
}

impl Rule for AriaRule {
    fn name(&self) -> &'static str {
        "aria"
    }

    fn reset(&mut self) {
        self.pending_activedescendants.clear();
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let elem_name = &tag.name;

        if let Some(target_id) = tag.get_attr("aria-activedescendant") {
            self.pending_activedescendants
                .push((target_id.to_string(), tag.line, tag.col));
        }

        validate_roles(tag, elem_name, &mut msgs);
        validate_unnecessary_roles(tag, elem_name, &mut msgs);
        validate_descendant_constraints(ctx, tag, elem_name, &mut msgs);
        validate_naming_rules(tag, elem_name, &mut msgs);
        validate_attributes(tag, elem_name, &mut msgs);

        msgs
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        for (id_ref, line, col) in &self.pending_activedescendants {
            if !ctx.id_to_tag.contains_key(id_ref) {
                msgs.push(VNuMessage::error(
                    format!(
                        "The “aria-activedescendant” attribute references “{}”, which is not the ID of any element in this document.",
                        id_ref
                    ),
                    *line,
                    *col,
                ));
            }
        }
        msgs
    }
}

fn validate_roles(tag: &ElementContext, elem_name: &str, msgs: &mut Vec<VNuMessage>) {
    if let Some(role_attr) = tag.get_attr("role") {
        if role_attr == "presentation" || role_attr == "none" {
            let has_global_aria = tag
                .attrs
                .iter()
                .any(|(k, _)| k.starts_with("aria-") && !matches!(k.as_str(), "aria-hidden"));
            if has_global_aria {
                msgs.push(VNuMessage::warning(
                    "The “presentation” role does not affect elements that have global ARIA attributes.",
                    tag.line,
                    tag.col,
                ));
            }
        }
        for token in role_attr.split_whitespace() {
            if !VALID_ARIA_ROLES
                .iter()
                .any(|&r| r.eq_ignore_ascii_case(token))
            {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for attribute “role” on element “{}”.",
                        token, elem_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }
}

fn validate_unnecessary_roles(tag: &ElementContext, elem_name: &str, msgs: &mut Vec<VNuMessage>) {
    let role_attr = match tag.get_attr("role") {
        Some(r) => r,
        None => return,
    };

    for token in role_attr.split_whitespace() {
        if elem_name.eq_ignore_ascii_case("input") {
            let input_type = tag.get_attr("type").unwrap_or("text");
            let has_list = tag.has_attr("list");
            if !has_list {
                if token.eq_ignore_ascii_case("textbox") && input_type.eq_ignore_ascii_case("text")
                {
                    msgs.push(VNuMessage::warning(
                        format!(
                            "The “textbox” role is unnecessary for an “input” element that has no “list” attribute and whose type is “{}”.",
                            input_type
                        ),
                        tag.line,
                        tag.col,
                    ));
                } else if token.eq_ignore_ascii_case("searchbox")
                    && input_type.eq_ignore_ascii_case("search")
                {
                    msgs.push(VNuMessage::warning(
                        format!(
                            "The “searchbox” role is unnecessary for an “input” element that has no “list” attribute and whose type is “{}”.",
                            input_type
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            } else if token.eq_ignore_ascii_case("combobox") {
                msgs.push(VNuMessage::warning(
                    format!(
                        "The “combobox” role is unnecessary for an “input” element that has a “list” attribute and whose type is “{}”.",
                        input_type
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        } else if (elem_name.eq_ignore_ascii_case("a")
            || elem_name.eq_ignore_ascii_case("area")
            || elem_name.eq_ignore_ascii_case("link"))
            && tag.has_attr("href")
            && token.eq_ignore_ascii_case("link")
        {
            msgs.push(VNuMessage::warning(
                format!(
                    "The “link” role is unnecessary for element “{}” with attribute “href”.",
                    elem_name
                ),
                tag.line,
                tag.col,
            ));
        } else if (elem_name.eq_ignore_ascii_case("tbody")
            || elem_name.eq_ignore_ascii_case("tfoot")
            || elem_name.eq_ignore_ascii_case("thead"))
            && token.eq_ignore_ascii_case("rowgroup")
        {
            msgs.push(VNuMessage::warning(
                format!(
                    "The “rowgroup” role is unnecessary for element “{}”.",
                    elem_name
                ),
                tag.line,
                tag.col,
            ));
        } else if elem_name.eq_ignore_ascii_case("th")
            && (token.eq_ignore_ascii_case("columnheader")
                || token.eq_ignore_ascii_case("rowheader"))
        {
            msgs.push(VNuMessage::warning(
                format!("The “{}” role is unnecessary for element “th”.", token),
                tag.line,
                tag.col,
            ));
        } else if elem_name.eq_ignore_ascii_case("button") && token.eq_ignore_ascii_case("button") {
            msgs.push(VNuMessage::warning(
                "The “button” role is unnecessary for element “button”.",
                tag.line,
                tag.col,
            ));
        }
    }
}

fn validate_descendant_constraints(
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

fn validate_naming_rules(tag: &ElementContext, elem_name: &str, msgs: &mut Vec<VNuMessage>) {
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

fn validate_attributes(tag: &ElementContext, elem_name: &str, msgs: &mut Vec<VNuMessage>) {
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
    for (attr_name, _) in &tag.attrs {
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
