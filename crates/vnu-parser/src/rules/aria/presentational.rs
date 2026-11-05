use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

pub fn validate_presentational_descendants(
    ctx: &ValidationContext,
    tag: &ElementContext,
    elem_name: &str,
    msgs: &mut Vec<VNuMessage>,
) {
    if !matches!(elem_name, "h1" | "button" | "label") {
        return;
    }

    for parent in ctx.open_elements.iter().rev() {
        if let Some(parent_role) = parent.get_attr("role") {
            for pr in parent_role.split_whitespace() {
                if pr.eq_ignore_ascii_case("button") && elem_name.eq_ignore_ascii_case("h1") {
                    msgs.push(VNuMessage::error(
                        "The element “h1” must not appear as a descendant of an element with the attribute “role=button”.",
                        tag.line,
                        tag.col,
                    ));
                }
                if pr.eq_ignore_ascii_case("img") && elem_name.eq_ignore_ascii_case("button") {
                    msgs.push(VNuMessage::warning(
                        "The element “button” should not appear as a descendant of an element with the attribute “role=img”.",
                        tag.line,
                        tag.col,
                    ));
                }
                const LABEL_DISALLOWED_ROLES: &[&str] = &[
                    "separator",
                    "progressbar",
                    "img",
                    "slider",
                    "scrollbar",
                    "math",
                ];
                if elem_name.eq_ignore_ascii_case("label")
                    && LABEL_DISALLOWED_ROLES
                        .iter()
                        .any(|&dr| dr.eq_ignore_ascii_case(pr))
                {
                    msgs.push(VNuMessage::warning(
                        format!(
                            "The element “label” should not appear as a descendant of an element with the attribute “role={}”.",
                            pr
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }
    }
}
