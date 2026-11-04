use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct MenuRule;

impl Rule for MenuRule {
    fn name(&self) -> &'static str {
        "lists::menu"
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, ctx: &ValidationContext) -> bool {
        ctx.open_elements
            .last()
            .is_some_and(|top| top.name.eq_ignore_ascii_case("menu"))
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if let Some(parent) = ctx.parent() {
            if parent.name.eq_ignore_ascii_case("menu")
                && !matches!(name_lower.as_str(), "li" | "script" | "template")
            {
                msgs.push(VNuMessage::error(
                    format!(
                        "Element “{}” not allowed as child of “menu” in this context.",
                        name_lower
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }

        msgs
    }

    fn on_characters(
        &mut self,
        ctx: &ValidationContext,
        text: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if text.trim().is_empty() {
            return msgs;
        }

        if ctx
            .open_elements
            .last()
            .is_some_and(|p| p.name.eq_ignore_ascii_case("menu"))
        {
            msgs.push(VNuMessage::error(
                "Text not allowed in “menu” in this context.",
                line,
                col,
            ));
        }

        msgs
    }
}
