use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use crate::rules::datatypes::tokens::RESERVED_CUSTOM_ELEMENT_NAMES;
use crate::schema::is_standard_html_element;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct CustomElementsRule;

impl Rule for CustomElementsRule {
    fn name(&self) -> &'static str {
        "custom-elements"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();

        if tag.name.contains('-') {
            let name_lower = tag.name.to_ascii_lowercase();
            if RESERVED_CUSTOM_ELEMENT_NAMES.contains(&name_lower.as_str()) {
                msgs.push(VNuMessage::error(
                    format!("Element “{}” not allowed.", name_lower),
                    tag.line,
                    tag.col,
                ));
            } else if tag.has_attr("is") {
                msgs.push(VNuMessage::error(
                    "Autonomous custom elements must not specify the “is” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
        } else if !is_standard_html_element(&tag.name)
            && !ctx.is_descendant_of("svg")
            && !ctx.is_descendant_of("math")
        {
            let name_lower = tag.name.to_ascii_lowercase();
            let parent_name = ctx.parent().map(|p| p.name.as_str()).unwrap_or("body");
            msgs.push(VNuMessage::error(
                format!(
                    "Element “{}” not allowed as child of “{}” in this context.",
                    name_lower, parent_name
                ),
                tag.line,
                tag.col,
            ));
        }

        msgs
    }
}
