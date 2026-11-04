use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct MetadataRule {
    base_seen: bool,
}

impl Rule for MetadataRule {
    fn name(&self) -> &'static str {
        "metadata"
    }

    fn reset(&mut self) {
        self.base_seen = false;
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "base" {
            if self.base_seen {
                msgs.push(VNuMessage::error(
                    "Element “base” not allowed as child of element “head” in this context. (Suppressing further errors from this subtree.)",
                    tag.line,
                    tag.col,
                ));
            } else {
                self.base_seen = true;
            }
        } else if name_lower == "script" && tag.has_attr("language") {
            msgs.push(VNuMessage::warning(
                "The “language” attribute on the “script” element is obsolete. Use the “type” attribute instead.",
                tag.line,
                tag.col,
            ));
        }

        msgs
    }
}
