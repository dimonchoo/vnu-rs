use super::context_roles::validate_context_roles;
use super::element_prohibitions::validate_element_prohibitions;
use super::landmarks::AriaMiscState;
use super::presentational::validate_presentational_descendants;
use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

pub fn validate_prohibited_and_context_roles(
    ctx: &ValidationContext,
    tag: &ElementContext,
    elem_name: &str,
    roles: &[&str],
    state: &AriaMiscState,
    msgs: &mut Vec<VNuMessage>,
) {
    validate_element_prohibitions(ctx, tag, elem_name, roles, msgs);
    validate_context_roles(ctx, tag, elem_name, roles, state, msgs);
    validate_presentational_descendants(ctx, tag, elem_name, msgs);
}
