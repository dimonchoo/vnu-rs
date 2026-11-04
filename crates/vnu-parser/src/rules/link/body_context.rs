use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

pub fn check_link_body_context(
    ctx: &ValidationContext,
    tag: &ElementContext,
    rel_tokens: &[&str],
    msgs: &mut Vec<VNuMessage>,
) {
    if !ctx.is_descendant_of("body") {
        return;
    }

    if tag.has_attr("itemprop") || tag.has_attr("property") {
        return;
    }

    let allowed_body_rels = [
        "dns-prefetch",
        "modulepreload",
        "pingback",
        "preconnect",
        "prefetch",
        "preload",
        "prerender",
        "stylesheet",
    ];

    let has_allowed_rel = rel_tokens.iter().any(|r| allowed_body_rels.contains(r));
    if !has_allowed_rel {
        msgs.push(VNuMessage::error(
            "A “link” element must not appear as a descendant of a “body” element unless the “link” element has an “itemprop” attribute or has a “rel” attribute whose value contains “dns-prefetch”, “modulepreload”, “pingback”, “preconnect”, “prefetch”, “preload”, “prerender”, or “stylesheet”.",
            tag.line,
            tag.col,
        ));
    }
}
