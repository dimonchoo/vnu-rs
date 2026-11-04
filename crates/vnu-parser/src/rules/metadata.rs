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
        } else if name_lower == "meta" {
            if let Some(charset) = tag.get_attr("charset") {
                if !charset.eq_ignore_ascii_case("utf-8") {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Internal encoding declaration “{}” disagrees with the actual encoding of the document (“utf-8”).",
                            charset
                        ),
                        tag.line,
                        tag.col,
                    ));
                    msgs.push(VNuMessage::error(
                        "The only allowed value for the “charset” attribute for the “meta” element is “utf-8”.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        } else if name_lower == "link" {
            if tag.has_attr("rel") && !tag.has_attr("href") && !tag.has_attr("imagesrcset") {
                msgs.push(VNuMessage::error(
                    "A “link” element must have an “href” or “imagesrcset” attribute, or both.",
                    tag.line,
                    tag.col,
                ));
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
