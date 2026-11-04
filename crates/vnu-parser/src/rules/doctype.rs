use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct DoctypeRule {
    emitted: bool,
}

impl Rule for DoctypeRule {
    fn name(&self) -> &'static str {
        "doctype"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        if !ctx.doctype_seen && !self.emitted {
            self.emitted = true;
            return vec![VNuMessage::error(
                "Start tag seen without seeing a doctype first. Expected “<!DOCTYPE html>”.",
                tag.line,
                tag.col,
            )];
        }
        vec![]
    }
}
