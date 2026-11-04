use std::collections::HashSet;
use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

#[derive(Default)]
pub struct DuplicateIdRule {
    seen: HashSet<String>,
}

impl Rule for DuplicateIdRule {
    fn name(&self) -> &'static str {
        "duplicate-id"
    }

    fn reset(&mut self) {
        self.seen.clear();
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        if let Some(id) = tag.get_attr("id") {
            if !id.trim().is_empty() && !self.seen.insert(id.to_string()) {
                return vec![VNuMessage::error(
                    format!("Duplicate ID “{}”.", id),
                    tag.line,
                    tag.col,
                )];
            }
        }
        vec![]
    }
}
