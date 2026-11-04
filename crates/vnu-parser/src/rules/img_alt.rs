use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct ImgAltRule;

impl Rule for ImgAltRule {
    fn name(&self) -> &'static str {
        "img-alt"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if !tag.name.eq_ignore_ascii_case("img") {
            return msgs;
        }

        let alt_opt = tag.get_attr("alt");
        let has_alt = alt_opt.is_some();
        let alt_empty = alt_opt.is_some_and(|a| a.is_empty());
        let has_role = tag.has_attr("role");
        let has_aria_label = tag.has_attr("aria-label") || tag.has_attr("aria-labelledby");

        if tag.has_attr("ismap")
            && !ctx
                .open_elements
                .iter()
                .any(|e| e.name.eq_ignore_ascii_case("a") && e.has_attr("href"))
        {
            msgs.push(VNuMessage::error(
                "The “img” element with the “ismap” attribute set must have an “a” ancestor with the “href” attribute.",
                tag.line,
                tag.col,
            ));
        }

        if tag.has_attr("controls") && (!has_alt || alt_empty) {
            msgs.push(VNuMessage::error(
                "The “controls” attribute must not be specified on an “img” element that does not have an “alt” attribute, or whose “alt” attribute’s value is the empty string.",
                tag.line,
                tag.col,
            ));
        }

        if !has_alt && !has_aria_label {
            if has_role {
                msgs.push(VNuMessage::error(
                    "An “img” element with a “role” attribute must also have an accessible name (e.g., an “alt” attribute).",
                    tag.line,
                    tag.col,
                ));
            } else {
                let has_other_aria = tag.attrs.iter().any(|(k, _)| {
                    let lower = k.to_ascii_lowercase();
                    lower.starts_with("aria-") && lower != "aria-hidden"
                });
                if has_other_aria {
                    msgs.push(VNuMessage::error(
                        "An “img” element with any “aria-*” attributes other than “aria-hidden” must also have an accessible name. (e.g., an “alt” attribute).",
                        tag.line,
                        tag.col,
                    ));
                } else {
                    msgs.push(VNuMessage::error(
                        "An “img” element must have an “alt” attribute, except under certain conditions. For details, consult guidance on providing text alternatives for images.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        } else if has_role {
            if alt_empty {
                msgs.push(VNuMessage::error(
                    "An “img” element with a “role” attribute must not have an “alt” attribute whose value is the empty string.",
                    tag.line,
                    tag.col,
                ));
            } else if let Some(role_val) = tag.get_attr("role") {
                for r in role_val.split_whitespace() {
                    if r.eq_ignore_ascii_case("none") || r.eq_ignore_ascii_case("presentation") {
                        msgs.push(VNuMessage::error(
                            format!("Bad value “{}” for attribute “role” on element “img”", r),
                            tag.line,
                            tag.col,
                        ));
                    }
                }
            }
        }

        msgs
    }
}
