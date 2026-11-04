pub mod input;
pub mod types;

pub use input::InputRule;
pub use types::{effective_input_type, VALID_BUTTON_TYPES, VALID_INPUT_TYPES};

use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct FormsRule;

impl Rule for FormsRule {
    fn name(&self) -> &'static str {
        "forms"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "form" && ctx.is_descendant_of("form") {
            msgs.push(VNuMessage::error(
                "Saw a “form” start tag, but there was already an active “form” element. Nested forms are not allowed. Ignoring the tag.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "button" {
            if let Some(t) = tag.get_attr("type") {
                let t_lower = t.to_ascii_lowercase();
                if !VALID_BUTTON_TYPES.contains(&t_lower.as_str()) {
                    msgs.push(VNuMessage::error(
                        format!("Bad value “{}” for attribute “type” on element “button”.", t),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        msgs
    }
}
