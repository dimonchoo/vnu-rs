use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

const OBSOLETE_ELEMENTS_CSS: &[&str] = &[
    "center", "font", "marquee", "blink", "big", "strike", "tt", "frame", "frameset", "noframes",
];

const OBSOLETE_ATTRIBUTES_CSS: &[&str] = &[
    "align",
    "bgcolor",
    "valign",
    "hspace",
    "vspace",
    "marginheight",
    "marginwidth",
    "frameborder",
    "scrolling",
];

#[derive(Default)]
pub struct ObsoleteRule;

impl Rule for ObsoleteRule {
    fn name(&self) -> &'static str {
        "obsolete"
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if OBSOLETE_ELEMENTS_CSS.contains(&name_lower.as_str()) {
            msgs.push(VNuMessage::error(
                format!("The “{}” element is obsolete. Use CSS instead.", name_lower),
                tag.line,
                tag.col,
            ));
        } else if name_lower == "acronym" {
            msgs.push(VNuMessage::error(
                "The “acronym” element is obsolete. Use the “abbr” element instead.",
                tag.line,
                tag.col,
            ));
        } else if name_lower == "keygen" {
            msgs.push(VNuMessage::error(
                "The “keygen” element is obsolete.",
                tag.line,
                tag.col,
            ));
        } else if name_lower == "applet" {
            msgs.push(VNuMessage::error(
                "The “applet” element is obsolete. Use “embed” or “object” element instead.",
                tag.line,
                tag.col,
            ));
        }

        for (attr_name, _) in &tag.attrs {
            let attr_lower = attr_name.to_ascii_lowercase();
            if attr_lower == "contextmenu" {
                msgs.push(VNuMessage::error(
                    "The “contextmenu” attribute is obsolete. Use script to handle “contextmenu” event instead.",
                    tag.line,
                    tag.col,
                ));
            }
            if name_lower == "menu" && attr_lower == "type" {
                msgs.push(VNuMessage::warning(
                    "The “type” attribute on the “menu” element is obsolete. Use script to handle “contextmenu” event instead.",
                    tag.line,
                    tag.col,
                ));
            }
            if name_lower == "a" && attr_lower == "name" {
                msgs.push(VNuMessage::warning(
                    "The “name” attribute on the “a” element is obsolete. Consider putting an “id” attribute on the nearest container instead.",
                    tag.line,
                    tag.col,
                ));
            }
            if OBSOLETE_ATTRIBUTES_CSS.contains(&attr_lower.as_str()) {
                msgs.push(VNuMessage::warning(
                    format!(
                        "The “{}” attribute on the “{}” element is obsolete. Use CSS instead.",
                        attr_lower, name_lower
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }

        msgs
    }
}
