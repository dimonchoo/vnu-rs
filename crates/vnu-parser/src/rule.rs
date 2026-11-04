use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;

    fn reset(&mut self) {}

    fn on_doctype(
        &mut self,
        _ctx: &ValidationContext,
        _name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        vec![]
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, _tag: &ElementContext) -> Vec<VNuMessage> {
        vec![]
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        _name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        vec![]
    }

    fn on_stray_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        _name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        vec![]
    }

    fn handles_stray_end_tags(&self) -> bool {
        false
    }

    fn on_characters(
        &mut self,
        _ctx: &ValidationContext,
        _text: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        vec![]
    }

    fn handles_character_tokens(&self) -> bool {
        false
    }

    fn interested_in_characters(&self, _ctx: &ValidationContext) -> bool {
        false
    }

    fn on_finish(&mut self, _ctx: &ValidationContext) -> Vec<VNuMessage> {
        vec![]
    }
}
