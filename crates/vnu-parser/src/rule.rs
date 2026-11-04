use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};

pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;

    fn on_doctype(&mut self, _ctx: &ValidationContext, _name: &str, _line: usize, _col: usize) -> Vec<VNuMessage> {
        vec![]
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, _tag: &ElementContext) -> Vec<VNuMessage> {
        vec![]
    }

    fn on_end_tag(&mut self, _ctx: &ValidationContext, _name: &str, _line: usize, _col: usize) -> Vec<VNuMessage> {
        vec![]
    }

    fn on_finish(&mut self, _ctx: &ValidationContext) -> Vec<VNuMessage> {
        vec![]
    }
}
