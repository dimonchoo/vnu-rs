use crate::context::{ElementContext, ValidationContext};
use std::collections::HashSet;
use vnu_core::VNuMessage;

pub fn validate_usemap(
    ctx: &ValidationContext,
    tag: &ElementContext,
    el: &str,
    pending_usemap_checks: &mut Vec<(String, usize, usize)>,
    defined_map_names: &mut HashSet<String>,
    msgs: &mut Vec<VNuMessage>,
) {
    if let Some(usemap) = tag.get_attr("usemap") {
        if el == "img" && ctx.is_descendant_of("a") {
            msgs.push(VNuMessage::error(
                "The element “img” with the attribute “usemap” must not appear as a descendant of the “a” element.",
                tag.line,
                tag.col,
            ));
        }
        if let Some(target) = usemap.strip_prefix('#') {
            pending_usemap_checks.push((target.to_string(), tag.line, tag.col));
        }
    }
    if el == "map" {
        if let Some(name) = tag.get_attr("name") {
            defined_map_names.insert(name.to_string());
        }
    }
}
