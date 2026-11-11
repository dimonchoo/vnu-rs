use super::roles::VALID_ARIA_ROLES;
use crate::context::ElementContext;
use vnu_core::VNuMessage;

pub fn validate_roles(tag: &ElementContext, elem_name: &str, msgs: &mut Vec<VNuMessage>) {
    if let Some(role_attr) = tag.get_attr("role") {
        let is_pres = role_attr.eq_ignore_ascii_case("presentation");
        let is_none = role_attr.eq_ignore_ascii_case("none");
        if is_pres || is_none {
            let role_name = if is_pres { "presentation" } else { "none" };
            let has_tabindex = tag.has_attr("tabindex");
            let has_global_aria = tag
                .attrs
                .iter()
                .any(|(k, _)| k.starts_with("aria-") && !matches!(k.as_str(), "aria-hidden"));

            if has_tabindex && has_global_aria {
                msgs.push(VNuMessage::warning(
                    format!(
                        "The “{}” role does not affect elements that have a “tabindex” attribute and global ARIA attributes.",
                        role_name
                    ),
                    tag.line,
                    tag.col,
                ));
            } else if has_tabindex {
                msgs.push(VNuMessage::warning(
                    format!(
                        "The “{}” role does not affect elements that have a “tabindex” attribute.",
                        role_name
                    ),
                    tag.line,
                    tag.col,
                ));
            } else if has_global_aria {
                msgs.push(VNuMessage::warning(
                    format!(
                        "The “{}” role does not affect elements that have global ARIA attributes.",
                        role_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
        // The schema sees the first valid role only
        // (RoleAttributeFilteringContentHandlerWrapper), so the value is bad only
        // when no token is a role.
        let has_invalid = !role_attr.trim().is_empty()
            && !role_attr.split_whitespace().any(|token| {
                VALID_ARIA_ROLES
                    .iter()
                    .any(|&r| r.eq_ignore_ascii_case(token))
            });
        if has_invalid {
            msgs.push(VNuMessage::error(
                format!(
                    "Bad value “{}” for attribute “role” on element “{}”.",
                    role_attr, elem_name
                ),
                tag.line,
                tag.col,
            ));
        }
    }
}

pub fn validate_unnecessary_roles(
    tag: &ElementContext,
    elem_name: &str,
    msgs: &mut Vec<VNuMessage>,
) {
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
