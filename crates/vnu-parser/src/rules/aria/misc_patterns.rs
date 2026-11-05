pub use super::landmarks::AriaMiscState;

use super::landmarks::{validate_landmarks_and_counts, validate_role_landmarks_and_headings};
use super::prohibited::validate_prohibited_and_context_roles;
use super::redundant::validate_redundant_roles;
use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

pub fn validate_misc_aria_patterns(
    ctx: &ValidationContext,
    tag: &ElementContext,
    elem_name: &str,
    state: &mut AriaMiscState,
    msgs: &mut Vec<VNuMessage>,
) {
    let role = tag.get_attr("role").unwrap_or("");
    let roles: Vec<&str> = role.split_whitespace().collect();

    validate_landmarks_and_counts(tag, elem_name, &roles, state, msgs);
    validate_role_landmarks_and_headings(ctx, tag, elem_name, &roles, msgs);
    validate_redundant_roles(tag, elem_name, &roles, msgs);
    validate_prohibited_and_context_roles(ctx, tag, elem_name, &roles, state, msgs);
}
