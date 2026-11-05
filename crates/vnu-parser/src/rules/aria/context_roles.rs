use super::landmarks::AriaMiscState;
use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

pub fn validate_context_roles(
    ctx: &ValidationContext,
    tag: &ElementContext,
    elem_name: &str,
    roles: &[&str],
    state: &AriaMiscState,
    msgs: &mut Vec<VNuMessage>,
) {
    let role = tag.get_attr("role").unwrap_or("");

    // li inside composite widgets
    if elem_name.eq_ignore_ascii_case("li") && !role.is_empty() {
        if (ctx.has_ancestor_role("listbox") || ctx.has_ancestor_role("list"))
            && !roles
                .iter()
                .any(|&r| r.eq_ignore_ascii_case("group") || r.eq_ignore_ascii_case("option"))
        {
            msgs.push(VNuMessage::error(
                "An “li” element that is a descendant of a “role=listbox” element or “role=list” element must not have any “role” value other than “group” or “option”.",
                tag.line,
                tag.col,
            ));
        }
        const MENU_DESCENDANT_ROLES: &[&str] = &[
            "group",
            "menuitem",
            "menuitemcheckbox",
            "menuitemradio",
            "separator",
            "none",
            "presentation",
        ];
        if (ctx.has_ancestor_role("menu") || ctx.has_ancestor_role("menubar"))
            && !roles.iter().any(|&r| {
                MENU_DESCENDANT_ROLES
                    .iter()
                    .any(|&mr| mr.eq_ignore_ascii_case(r))
            })
        {
            msgs.push(VNuMessage::error(
                "An “li” element that is a descendant of a “role=menu” element or “role=menubar” element must not have any “role” value other than “group”, “menuitem”, “menuitemcheckbox”, “menuitemradio”, or “separator”.",
                tag.line,
                tag.col,
            ));
        }
        if ctx.has_ancestor_role("tablist") && !roles.iter().any(|&r| r.eq_ignore_ascii_case("tab"))
        {
            msgs.push(VNuMessage::error(
                "An “li” element that is a descendant of a “role=tablist” element must not have any “role” value other than “tab”.",
                tag.line,
                tag.col,
            ));
        }
        if ctx.has_ancestor_role("tree")
            && !roles.iter().any(|&r| r.eq_ignore_ascii_case("treeitem"))
        {
            msgs.push(VNuMessage::error(
                "An “li” element that is a descendant of a “role=tree” element must not have any “role” value other than “treeitem”.",
                tag.line,
                tag.col,
            ));
        }
    }

    // Context roles requiring parent roles
    for &r in roles {
        if r.eq_ignore_ascii_case("cell") && !ctx.has_ancestor_role("row") {
            msgs.push(VNuMessage::error(
                "An element with “role=cell” must be contained in, or owned by, an element with the “role” value “row”.",
                tag.line,
                tag.col,
            ));
        }
        if r.eq_ignore_ascii_case("option")
            && !ctx.has_ancestor_role("listbox")
            && !elem_name.eq_ignore_ascii_case("option")
            && !tag
                .get_attr("id")
                .is_some_and(|id| state.listbox_owned_ids.contains(id))
        {
            msgs.push(VNuMessage::error(
                "An element with “role=option” must be contained in, or owned by, an element with the “role” value “listbox”.",
                tag.line,
                tag.col,
            ));
        }
        if r.eq_ignore_ascii_case("row")
            && !ctx.has_ancestor_role("table")
            && !ctx.has_ancestor_role("grid")
            && !ctx.has_ancestor_role("treegrid")
            && !ctx.has_ancestor_role("rowgroup")
            && !elem_name.eq_ignore_ascii_case("tr")
        {
            msgs.push(VNuMessage::error(
                "An element with “role=row” must be contained in, or owned by, an element with the “role” value “treegrid”, “grid”, “rowgroup”, or “table”.",
                tag.line,
                tag.col,
            ));
        }
    }

    // select element ARIA restrictions
    if elem_name.eq_ignore_ascii_case("select") {
        if tag.has_attr("aria-multiselectable") {
            msgs.push(VNuMessage::warning(
                "The “aria-multiselectable” attribute should not be used with the “select ” element.",
                tag.line,
                tag.col,
            ));
        }
        if roles.iter().any(|&r| r.eq_ignore_ascii_case("button")) {
            msgs.push(VNuMessage::error(
                "Bad value “button” for attribute “role” on element “select”.",
                tag.line,
                tag.col,
            ));
        }
        if roles.iter().any(|&r| r.eq_ignore_ascii_case("combobox"))
            && !tag.has_attr("aria-expanded")
        {
            msgs.push(VNuMessage::error(
                "Element “select” is missing required attribute “aria-expanded”.",
                tag.line,
                tag.col,
            ));
        }
    }

    // dl child div role restriction
    if elem_name.eq_ignore_ascii_case("div")
        && ctx
            .parent()
            .map(|p| p.name.eq_ignore_ascii_case("dl"))
            .unwrap_or(false)
        && !role.is_empty()
        && !roles
            .iter()
            .any(|&r| r.eq_ignore_ascii_case("presentation") || r.eq_ignore_ascii_case("none"))
    {
        msgs.push(VNuMessage::error(
            "A “div” child of a “dl” element must not have any “role” value other than “presentation” or “none”.",
            tag.line,
            tag.col,
        ));
    }

    // Miscellaneous single-case constraints
    if tag.has_attr("placeholder") && tag.has_attr("aria-placeholder") {
        msgs.push(VNuMessage::error(
            "The “aria-placeholder” attribute must not be specified on elements that have a “placeholder” attribute.",
            tag.line,
            tag.col,
        ));
    }

    if roles.iter().any(|&r| r.eq_ignore_ascii_case("listitem")) && tag.has_attr("aria-level") {
        msgs.push(VNuMessage::warning(
            "The “aria-level” attribute should not be used on any element which has “role=listitem”.",
            tag.line,
            tag.col,
        ));
    }
}
