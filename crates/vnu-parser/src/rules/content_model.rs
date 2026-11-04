use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

fn allowed_parents(tag_name: &str) -> Option<&'static [&'static str]> {
    match tag_name {
        "li" => Some(&["ul", "ol", "menu", "template"]),
        "dt" | "dd" => Some(&["dl", "div", "template"]),
        "source" => Some(&["audio", "video", "picture", "template"]),
        "track" => Some(&["audio", "video", "template"]),
        "area" => Some(&["map", "template"]),
        "param" => Some(&["object", "template"]),
        "optgroup" => Some(&["select", "template"]),
        "option" => Some(&["select", "optgroup", "datalist", "template"]),
        _ => None,
    }
}

#[derive(Default)]
pub struct ContentModelRule;

impl Rule for ContentModelRule {
    fn name(&self) -> &'static str {
        "content-model"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let name_lower = tag.name.to_ascii_lowercase();

        if let Some(expected_parents) = allowed_parents(&name_lower) {
            let parent_opt = ctx.parent();
            let mut parent_matches = parent_opt
                .map(|p| expected_parents.iter().any(|&exp| exp.eq_ignore_ascii_case(&p.name)))
                .unwrap_or(false);

            if !parent_matches
                && (name_lower == "option" || name_lower == "optgroup")
                && parent_opt.is_some_and(|p| p.name.eq_ignore_ascii_case("div"))
                && ctx.is_descendant_of("select")
            {
                parent_matches = true;
            }

            if !parent_matches {
                let parent_name = parent_opt.map(|p| p.name.as_str()).unwrap_or("body");
                return vec![VNuMessage::error(
                    format!(
                        "Element “{}” not allowed as child of element “{}” in this context. (Suppressing further errors from this subtree.)",
                        name_lower, parent_name
                    ),
                    tag.line,
                    tag.col,
                )];
            }
        }

        vec![]
    }
}
