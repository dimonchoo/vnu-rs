use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct HeadTitleRule {
    emitted: bool,
    current_title: Option<(usize, usize, bool)>,
}

impl Rule for HeadTitleRule {
    fn name(&self) -> &'static str {
        "head-title"
    }

    fn reset(&mut self) {
        self.emitted = false;
        self.current_title = None;
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        if ctx.mode == crate::context::DocumentMode::Svg {
            return vec![];
        }

        if tag.name.eq_ignore_ascii_case("title") && !ctx.is_descendant_of("svg") {
            self.current_title = Some((tag.line, tag.col, false));
        }

        if tag.name.eq_ignore_ascii_case("body") && !ctx.title_seen && !self.emitted {
            self.emitted = true;
            return vec![VNuMessage::error(
                "Element “head” is missing a required instance of child element “title”.",
                tag.line,
                tag.col,
            )];
        }
        vec![]
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if name.eq_ignore_ascii_case("title") {
            if let Some((line, col, has_content)) = self.current_title.take() {
                if !has_content {
                    msgs.push(VNuMessage::error(
                        "Element “title” must not be empty.",
                        line,
                        col,
                    ));
                }
            }
        }
        msgs
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, _ctx: &ValidationContext) -> bool {
        self.current_title.is_some()
    }

    fn on_characters(
        &mut self,
        _ctx: &ValidationContext,
        text: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        if let Some((_, _, ref mut has_content)) = self.current_title {
            if text.chars().any(|c| !c.is_ascii_whitespace()) {
                *has_content = true;
            }
        }
        vec![]
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        if ctx.mode == crate::context::DocumentMode::Svg {
            return vec![];
        }
        if !ctx.title_seen && !self.emitted {
            self.emitted = true;
            let (line, col) = ctx
                .open_elements
                .last()
                .map(|e| (e.line, e.col))
                .unwrap_or((1, 1));
            return vec![VNuMessage::error(
                "Element “head” is missing a required instance of child element “title”.",
                line,
                col,
            )];
        }
        vec![]
    }
}
