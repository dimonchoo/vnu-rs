use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct NestingRule;

impl Rule for NestingRule {
    fn name(&self) -> &'static str {
        "nesting"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "a" && ctx.is_descendant_of("a") {
            msgs.push(VNuMessage::error(
                "Start tag “a” seen but an element of the same type was already open.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "button" && ctx.is_descendant_of("button") {
            msgs.push(VNuMessage::error(
                "The element “button” must not appear as a descendant of the “button” element.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "button" && ctx.is_descendant_of("a") {
            msgs.push(VNuMessage::error(
                "The element “button” must not appear as a descendant of the “a” element.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "a" && ctx.is_descendant_of("button") {
            msgs.push(VNuMessage::error(
                "The element “a” must not appear as a descendant of the “button” element.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "div" {
            if let Some(parent) = ctx.parent() {
                if parent.name.eq_ignore_ascii_case("p") {
                    msgs.push(VNuMessage::error(
                        "An element “div” not allowed as child of element “p” in this context.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        msgs
    }
}
