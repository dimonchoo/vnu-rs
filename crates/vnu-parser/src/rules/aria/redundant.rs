use crate::context::ElementContext;
use vnu_core::VNuMessage;

pub fn validate_redundant_roles(
    tag: &ElementContext,
    elem_name: &str,
    roles: &[&str],
    msgs: &mut Vec<VNuMessage>,
) {
    for &r in roles {
        match elem_name {
            "article" if r.eq_ignore_ascii_case("article") => {
                msgs.push(VNuMessage::warning(
                    "The “article” role is unnecessary for element “article”.",
                    tag.line,
                    tag.col,
                ));
            }
            "aside" if r.eq_ignore_ascii_case("complementary") => {
                msgs.push(VNuMessage::warning(
                    "The “complementary” role is unnecessary for element “aside”.",
                    tag.line,
                    tag.col,
                ));
            }
            "dd" if r.eq_ignore_ascii_case("definition") => {
                msgs.push(VNuMessage::warning(
                    "The “definition” role is unnecessary for element “dd”.",
                    tag.line,
                    tag.col,
                ));
            }
            "details" if r.eq_ignore_ascii_case("group") => {
                msgs.push(VNuMessage::warning(
                    "The “group” role is unnecessary for element “details”.",
                    tag.line,
                    tag.col,
                ));
            }
            "dialog" if r.eq_ignore_ascii_case("dialog") => {
                msgs.push(VNuMessage::warning(
                    "The “dialog” role is unnecessary for element “dialog”.",
                    tag.line,
                    tag.col,
                ));
            }
            "dt" if r.eq_ignore_ascii_case("term") => {
                msgs.push(VNuMessage::warning(
                    "The “term” role is unnecessary for element “dt”.",
                    tag.line,
                    tag.col,
                ));
            }
            "figure" if r.eq_ignore_ascii_case("figure") => {
                msgs.push(VNuMessage::warning(
                    "The “figure” role is unnecessary for element “figure”.",
                    tag.line,
                    tag.col,
                ));
            }
            "footer" if r.eq_ignore_ascii_case("contentinfo") => {
                msgs.push(VNuMessage::warning(
                    "The “contentinfo” role is unnecessary for element “footer”.",
                    tag.line,
                    tag.col,
                ));
            }
            "form" if r.eq_ignore_ascii_case("form") => {
                msgs.push(VNuMessage::warning(
                    "The “form” role is unnecessary for element “form”.",
                    tag.line,
                    tag.col,
                ));
            }
            "header" if r.eq_ignore_ascii_case("banner") => {
                msgs.push(VNuMessage::warning(
                    "The “banner” role is unnecessary for element “header”.",
                    tag.line,
                    tag.col,
                ));
            }
            "hr" if r.eq_ignore_ascii_case("separator") => {
                msgs.push(VNuMessage::warning(
                    "The “separator” role is unnecessary for element “hr”.",
                    tag.line,
                    tag.col,
                ));
            }
            "img" if r.eq_ignore_ascii_case("img") => {
                msgs.push(VNuMessage::warning(
                    "The “img” role is unnecessary for element “img”.",
                    tag.line,
                    tag.col,
                ));
            }
            "input"
                if r.eq_ignore_ascii_case("spinbutton") && tag.has_attr_val("type", "number") =>
            {
                msgs.push(VNuMessage::warning(
                    "The “spinbutton” role is unnecessary for element “input” whose type is “number”.",
                    tag.line,
                    tag.col,
                ));
            }
            "li" if r.eq_ignore_ascii_case("listitem") => {
                msgs.push(VNuMessage::warning(
                    "The “listitem” role is unnecessary for element “li”.",
                    tag.line,
                    tag.col,
                ));
            }
            "main" if r.eq_ignore_ascii_case("main") => {
                msgs.push(VNuMessage::warning(
                    "The “main” role is unnecessary for element “main”.",
                    tag.line,
                    tag.col,
                ));
            }
            "output" if r.eq_ignore_ascii_case("status") => {
                msgs.push(VNuMessage::warning(
                    "The “status” role is unnecessary for element “output”.",
                    tag.line,
                    tag.col,
                ));
            }
            "progress" if r.eq_ignore_ascii_case("progressbar") => {
                msgs.push(VNuMessage::warning(
                    "The “progressbar” role is unnecessary for element “progress”.",
                    tag.line,
                    tag.col,
                ));
            }
            "section" if r.eq_ignore_ascii_case("region") => {
                msgs.push(VNuMessage::warning(
                    "The “region” role is unnecessary for element “section”.",
                    tag.line,
                    tag.col,
                ));
            }
            "table" if r.eq_ignore_ascii_case("table") => {
                msgs.push(VNuMessage::warning(
                    "The “table” role is unnecessary for element “table”.",
                    tag.line,
                    tag.col,
                ));
            }
            "ul" if r.eq_ignore_ascii_case("list") => {
                msgs.push(VNuMessage::warning(
                    "The “list” role is unnecessary for element “ul”.",
                    tag.line,
                    tag.col,
                ));
            }
            "math" => {
                msgs.push(VNuMessage::warning(
                    "Element “math” does not need a “role” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
            "nav" if r.eq_ignore_ascii_case("navigation") => {
                msgs.push(VNuMessage::warning(
                    "The “navigation” role is unnecessary for element “nav”.",
                    tag.line,
                    tag.col,
                ));
            }
            "s" if r.eq_ignore_ascii_case("deletion") => {
                msgs.push(VNuMessage::warning(
                    "The “deletion” role is unnecessary for element “s”.",
                    tag.line,
                    tag.col,
                ));
            }
            "select" if r.eq_ignore_ascii_case("listbox") => {
                let size = tag
                    .get_attr("size")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                if tag.has_attr("multiple") || size > 1 {
                    msgs.push(VNuMessage::warning(
                        "The “listbox” role is unnecessary for element “select” with a “multiple” attribute or with a “size” attribute whose value is greater than 1.",
                        tag.line,
                        tag.col,
                    ));
                } else {
                    msgs.push(VNuMessage::error(
                        "The “listbox” role is not allowed for element “select” without a “multiple” attribute and without a “size” attribute whose value is greater than 1.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
            _ => {}
        }
    }
}
