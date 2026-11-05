use crate::context::{ElementContext, ValidationContext};
use std::collections::HashSet;
use vnu_core::VNuMessage;

pub fn validate_microdata(
    ctx: &ValidationContext,
    tag: &ElementContext,
    all_itemref_ids: &mut HashSet<String>,
    pending_itemref_checks: &mut Vec<(String, usize, usize)>,
    pending_itemprop_checks: &mut Vec<(usize, usize, Option<String>)>,
    msgs: &mut Vec<VNuMessage>,
) {
    if let Some(itemref) = tag.get_attr("itemref") {
        if !tag.has_attr("itemscope") {
            msgs.push(VNuMessage::error(
                "The “itemref” attribute must not be specified on elements that do not have an “itemscope” attribute specified.",
                tag.line,
                tag.col,
            ));
        }
        let tokens: Vec<&str> = itemref.split_whitespace().collect();
        let mut seen = HashSet::new();
        for tok in &tokens {
            if !seen.insert(*tok) {
                msgs.push(VNuMessage::error(
                    "The “itemref” attribute contained redundant references.",
                    tag.line,
                    tag.col,
                ));
                break;
            }
        }
        for tok in &tokens {
            all_itemref_ids.insert(tok.to_string());
            pending_itemref_checks.push((tok.to_string(), tag.line, tag.col));
        }
    }

    if tag.has_attr("itemprop") {
        let in_item = ctx.open_elements.iter().any(|e| e.has_attr("itemscope"));
        if !in_item {
            pending_itemprop_checks.push((
                tag.line,
                tag.col,
                tag.get_attr("id").map(|s| s.to_string()),
            ));
        }
    }

    if tag.has_attr("itemid") && (!tag.has_attr("itemscope") || !tag.has_attr("itemtype")) {
        msgs.push(VNuMessage::error(
            "The “itemid” attribute must not be specified on elements that do not have both an “itemscope” attribute and an “itemtype” attribute specified.",
            tag.line,
            tag.col,
        ));
    }
}
