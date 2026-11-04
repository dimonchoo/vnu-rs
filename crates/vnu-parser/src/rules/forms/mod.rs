pub mod input;
pub mod select;
pub mod types;

pub use input::InputRule;
pub use select::SelectRule;
pub use types::{effective_input_type, VALID_BUTTON_TYPES, VALID_INPUT_TYPES};

use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct FormsRule {
    pending_commandfor: Vec<(String, usize, usize)>,
}

impl Rule for FormsRule {
    fn name(&self) -> &'static str {
        "forms"
    }

    fn reset(&mut self) {
        self.pending_commandfor.clear();
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
            if let Some(cf) = tag.get_attr("commandfor") {
                self.pending_commandfor.push((cf.to_string(), tag.line, tag.col));
            }
        }

        msgs
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        for (cf, line, col) in &self.pending_commandfor {
            if !ctx.id_to_tag.contains_key(cf) {
                msgs.push(VNuMessage::error(
                    "The value of the “commandfor” attribute of the “button” element must be the ID of an element in the same tree as the “button” with the “commandfor” attribute.".to_string(),
                    *line,
                    *col,
                ));
            }
        }
        msgs
    }
}

