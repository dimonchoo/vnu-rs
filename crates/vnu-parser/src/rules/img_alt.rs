use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct ImgAltRule;

impl Rule for ImgAltRule {
    fn name(&self) -> &'static str {
        "img-alt"
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        if tag.name.eq_ignore_ascii_case("img") && !tag.has_attr("alt") {
            return vec![VNuMessage::error(
                "An “img” element must have an “alt” attribute, except under certain conditions. For details, consult guidance on providing text alternatives for images.",
                tag.line,
                tag.col,
            )];
        }
        vec![]
    }
}
