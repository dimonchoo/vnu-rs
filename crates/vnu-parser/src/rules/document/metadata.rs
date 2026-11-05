use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct MetadataRule {
    base_seen: bool,
    link_or_script_seen: bool,
}

impl Rule for MetadataRule {
    fn name(&self) -> &'static str {
        "metadata"
    }

    fn reset(&mut self) {
        self.base_seen = false;
        self.link_or_script_seen = false;
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "base" {
            if ctx.is_descendant_of("body") {
                msgs.push(VNuMessage::error(
                    "Element “base” not allowed as child of “body” in this context.",
                    tag.line,
                    tag.col,
                ));
            } else if self.base_seen {
                msgs.push(VNuMessage::error(
                    "Element “base” not allowed as child of element “head” in this context. (Suppressing further errors from this subtree.)",
                    tag.line,
                    tag.col,
                ));
            } else {
                self.base_seen = true;
                if self.link_or_script_seen {
                    msgs.push(VNuMessage::error(
                        "The “base” element must come before any “link” or “script” elements in the document.",
                        tag.line,
                        tag.col,
                    ));
                }
            }

            if !tag.has_attr("href") && !tag.has_attr("target") {
                msgs.push(VNuMessage::error(
                    "Element “base” is missing one or more of the following attributes: “href”, “target”.",
                    tag.line,
                    tag.col,
                ));
            }
        } else if name_lower == "basefont" {
            msgs.push(VNuMessage::error(
                "Element “basefont” not allowed as child of “head” in this context.",
                tag.line,
                tag.col,
            ));
        } else if name_lower == "link" || name_lower == "script" {
            self.link_or_script_seen = true;
            if name_lower == "script" && tag.has_attr("language") {
                msgs.push(VNuMessage::warning(
                    "The “language” attribute on the “script” element is obsolete. Use the “type” attribute instead.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        msgs
    }
}
