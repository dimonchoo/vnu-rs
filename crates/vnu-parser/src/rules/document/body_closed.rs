use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct BodyClosedRule {
    body_closed: bool,
}

impl Rule for BodyClosedRule {
    fn name(&self) -> &'static str {
        "body_closed"
    }

    fn reset(&mut self) {
        self.body_closed = false;
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, _ctx: &ValidationContext) -> bool {
        self.body_closed
    }

    fn handles_stray_end_tags(&self) -> bool {
        true
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if self.body_closed {
            msgs.push(VNuMessage::error(
                format!("Stray start tag “{}”.", tag.name),
                tag.line,
                tag.col,
            ));
        }
        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        if name.eq_ignore_ascii_case("body") {
            self.body_closed = true;
        }
        vec![]
    }

    fn on_stray_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        name: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        if self.body_closed && !name.eq_ignore_ascii_case("html") {
            vec![VNuMessage::error(
                "Saw an end tag after “body” had been closed.",
                line,
                col,
            )]
        } else {
            vec![VNuMessage::error(
                format!("Stray end tag “{}”.", name),
                line,
                col,
            )]
        }
    }

    fn on_characters(
        &mut self,
        _ctx: &ValidationContext,
        text: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if self.body_closed && !text.trim().is_empty() {
            msgs.push(VNuMessage::error(
                "Non-space character after body.".to_string(),
                line,
                col,
            ));
        }
        msgs
    }
}
