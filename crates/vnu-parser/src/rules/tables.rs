use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct TablesRule;

impl Rule for TablesRule {
    fn name(&self) -> &'static str {
        "tables"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "tr" {
            let parent_is_table_section = ctx.parent().map(|p| {
                let p_name = p.name.to_ascii_lowercase();
                p_name == "table" || p_name == "thead" || p_name == "tbody" || p_name == "tfoot"
            }).unwrap_or(false);

            if !parent_is_table_section {
                msgs.push(VNuMessage::error(
                    "Stray start tag “tr”.",
                    tag.line,
                    tag.col,
                ));
            }
        } else if name_lower == "td" || name_lower == "th" {
            let parent_is_tr = ctx.parent().map(|p| p.name.eq_ignore_ascii_case("tr")).unwrap_or(false);
            if !parent_is_tr {
                if ctx.is_descendant_of("table") {
                    msgs.push(VNuMessage::error(
                        format!("“{}” start tag in table body.", name_lower),
                        tag.line,
                        tag.col,
                    ));
                } else {
                    msgs.push(VNuMessage::error(
                        format!("Stray start tag “{}”.", name_lower),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        } else if matches!(name_lower.as_str(), "thead" | "tbody" | "tfoot" | "caption" | "colgroup") {
            let parent_is_table = ctx.parent().map(|p| p.name.eq_ignore_ascii_case("table")).unwrap_or(false);
            if !parent_is_table {
                msgs.push(VNuMessage::error(
                    format!("Stray start tag “{}”.", name_lower),
                    tag.line,
                    tag.col,
                ));
            }
        }

        msgs
    }
}
