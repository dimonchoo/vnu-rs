use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

const OBSOLETE_ELEMENTS_CSS: &[&str] = &[
    "center", "font", "marquee", "big", "tt", "frame", "frameset", "nobr", "basefont",
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

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if ctx.is_descendant_of("svg") || ctx.is_descendant_of("math") {
            return msgs;
        }
        let name_lower = tag.name.to_ascii_lowercase();

        if OBSOLETE_ELEMENTS_CSS.contains(&name_lower.as_str()) {
            msgs.push(VNuMessage::error(
                format!("The “{}” element is obsolete. Use CSS instead.", name_lower),
                tag.line,
                tag.col,
            ));
        } else if name_lower == "strike" {
            msgs.push(VNuMessage::error(
                "The “strike” element is obsolete. Use “del” or “s” element instead.",
                tag.line,
                tag.col,
            ));
        } else if name_lower == "dir" {
            msgs.push(VNuMessage::error(
                "The “dir” element is obsolete. Use the “ul” element instead.",
                tag.line,
                tag.col,
            ));
        } else if name_lower == "noframes" {
            msgs.push(VNuMessage::error(
                "The “noframes” element is obsolete. Use the “iframe” element and CSS instead, or use server-side includes.",
                tag.line,
                tag.col,
            ));
        } else if !name_lower.contains('-')
            && !ctx.is_descendant_of("svg")
            && !ctx.is_descendant_of("math")
            && !crate::schema::elements::is_standard_html_element(&name_lower)
        {
            msgs.push(VNuMessage::error(
                format!(
                    "The “{}” element is a completely-unknown element that is not allowed anywhere in any HTML content.",
                    tag.name
                ),
                tag.line,
                tag.col,
            ));
        } else if name_lower == "param" {
            msgs.push(VNuMessage::error(
                "The “param” element is obsolete. Use the “data” attribute of the “object” element to set the URL of the external resource.",
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

        for (attr_name, _) in tag.attrs.iter() {
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
            if name_lower == "head" && attr_lower == "profile" {
                msgs.push(VNuMessage::warning(
                    "The “profile” attribute on the “head” element is obsolete. To declare which “meta” terms are used in the document, instead register the names as meta extensions. To trigger specific UA behaviors, use a “link” element instead.",
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
            if name_lower == "img" && attr_lower == "border" {
                msgs.push(VNuMessage::warning(
                    "The “border” attribute on the “img” element is obsolete. Consider specifying “img { border: 0; }” in CSS instead.",
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
