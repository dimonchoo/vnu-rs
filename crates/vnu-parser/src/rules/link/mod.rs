pub mod body_context;
pub mod relations;

use self::body_context::check_link_body_context;
use self::relations::check_link_relations;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct LinkRule;

impl Rule for LinkRule {
    fn name(&self) -> &'static str {
        "link"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if !tag.name.eq_ignore_ascii_case("link") {
            return msgs;
        }

        let has_rel = tag.has_attr("rel");
        let rel_val = tag.get_attr("rel").unwrap_or_default().to_ascii_lowercase();
        let rel_tokens: Vec<&str> = rel_val.split_whitespace().collect();

        check_link_relations(tag, has_rel, &rel_tokens, &mut msgs);
        check_link_body_context(ctx, tag, &rel_tokens, &mut msgs);

        msgs
    }
}
