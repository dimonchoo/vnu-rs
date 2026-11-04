use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct HtmlLangRule;

impl Rule for HtmlLangRule {
    fn name(&self) -> &'static str {
        "html-lang"
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        if tag.name.eq_ignore_ascii_case("html") && !tag.has_attr("lang") {
            return vec![VNuMessage::warning(
                "Consider adding a “lang” attribute to the “html” start tag to declare the language of this document.",
                tag.line,
                tag.col,
            )];
        }
        vec![]
    }
}
