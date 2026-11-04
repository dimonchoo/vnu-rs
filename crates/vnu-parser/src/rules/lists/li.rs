use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct LiRule;

impl Rule for LiRule {
    fn name(&self) -> &'static str {
        "lists::li"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if tag.name.eq_ignore_ascii_case("li") {
            if let Some(val) = tag.get_attr("value") {
                let parent = ctx.parent();
                let parent_is_ol = parent.is_some_and(|p| p.name.eq_ignore_ascii_case("ol"));
                let parent_has_presentation = parent.is_some_and(|p| {
                    p.get_attr("role")
                        .is_some_and(|r| r == "presentation" || r == "none")
                });

                if !parent_is_ol || parent_has_presentation {
                    msgs.push(VNuMessage::error(
                        "Attribute “value” not allowed on element “li” at this point.",
                        tag.line,
                        tag.col,
                    ));
                } else if val.trim().parse::<i64>().is_err() {
                    msgs.push(VNuMessage::error(
                        format!("Bad value “{}” for attribute “value” on element “li”.", val),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }
        msgs
    }
}
