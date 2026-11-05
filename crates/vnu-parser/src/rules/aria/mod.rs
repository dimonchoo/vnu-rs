pub mod attributes_validation;
pub mod context_roles;
pub mod element_prohibitions;
pub mod landmarks;
pub mod misc_patterns;
pub mod naming;
pub mod presentational;
pub mod prohibited;
pub mod redundant;
pub mod roles;
pub mod roles_validation;

use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use attributes_validation::{
    validate_attributes, validate_descendant_constraints, validate_naming_rules,
};
use misc_patterns::{AriaMiscState, validate_misc_aria_patterns};
use roles_validation::{validate_roles, validate_unnecessary_roles};
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct AriaRule {
    pending_activedescendants: Vec<(String, usize, usize)>,
    misc_state: AriaMiscState,
}

impl Rule for AriaRule {
    fn name(&self) -> &'static str {
        "aria"
    }

    fn reset(&mut self) {
        self.pending_activedescendants.clear();
        self.misc_state = AriaMiscState::default();
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let elem_name = &tag.name;

        if let Some(target_id) = tag.get_attr("aria-activedescendant") {
            self.pending_activedescendants
                .push((target_id.to_string(), tag.line, tag.col));
        }

        validate_roles(tag, elem_name, &mut msgs);
        validate_unnecessary_roles(tag, elem_name, &mut msgs);
        validate_descendant_constraints(ctx, tag, elem_name, &mut msgs);
        validate_naming_rules(tag, elem_name, &mut msgs);
        validate_attributes(tag, elem_name, &mut msgs);
        validate_misc_aria_patterns(ctx, tag, elem_name, &mut self.misc_state, &mut msgs);

        msgs
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        for (id_ref, line, col) in &self.pending_activedescendants {
            if !ctx.id_to_tag.contains_key(id_ref) {
                msgs.push(VNuMessage::error(
                    format!(
                        "The “aria-activedescendant” attribute references “{}”, which is not the ID of any element in this document.",
                        id_ref
                    ),
                    *line,
                    *col,
                ));
            }
        }
        if self.misc_state.tab_count > 0 && self.misc_state.tabpanel_count == 0 {
            msgs.push(VNuMessage::error(
                "Every active “role=tab” element must have a corresponding “role=tabpanel” element.",
                1,
                1,
            ));
        }
        msgs
    }
}
