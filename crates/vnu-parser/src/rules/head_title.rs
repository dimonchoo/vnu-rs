use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct HeadTitleRule {
    emitted: bool,
}

impl Rule for HeadTitleRule {
    fn name(&self) -> &'static str {
        "head-title"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        if ctx.mode == crate::context::DocumentMode::Svg {
            return vec![];
        }
        if (tag.name.eq_ignore_ascii_case("body") || tag.name.eq_ignore_ascii_case("main"))
            && !ctx.title_seen
            && !self.emitted
        {
            self.emitted = true;
            return vec![VNuMessage::error(
                "Element “head” is missing a required instance of child element “title”.",
                tag.line,
                tag.col,
            )];
        }
        vec![]
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        if ctx.mode == crate::context::DocumentMode::Svg {
            return vec![];
        }
        if !ctx.title_seen && !self.emitted {
            self.emitted = true;
            let (line, col) = ctx.open_elements.last().map(|e| (e.line, e.col)).unwrap_or((1, 1));
            return vec![VNuMessage::error(
                "Element “head” is missing a required instance of child element “title”.",
                line,
                col,
            )];
        }
        vec![]
    }
}
