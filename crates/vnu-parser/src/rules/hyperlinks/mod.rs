use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use crate::rules::datatypes::tokens::is_valid_bcp47_lang_tag;
use vnu_core::VNuMessage;

fn is_valid_mime_type(val: &str) -> bool {
    let trimmed = val.trim();
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() != 2 {
        return false;
    }
    let (t, st) = (parts[0], parts[1]);
    !t.is_empty()
        && !st.is_empty()
        && !t.contains(' ')
        && !st.contains(' ')
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '+' | '_'))
        && st
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '+' | '_'))
}

#[derive(Default)]
pub struct HyperlinksRule;

impl Rule for HyperlinksRule {
    fn name(&self) -> &'static str {
        "hyperlinks"
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "a" {
            if (tag.has_attr("download") || tag.has_attr("ping")) && !tag.has_attr("href") {
                msgs.push(VNuMessage::error(
                    "Element “a” is missing required attribute “href”.",
                    tag.line,
                    tag.col,
                ));
            }
            if tag.has_attr("media") {
                msgs.push(VNuMessage::error(
                    "Attribute “media” not allowed on element “a” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
        } else if name_lower == "area" {
            if !ctx.is_descendant_of("map") {
                msgs.push(VNuMessage::error(
                    "The “area” element must have a “map” ancestor.",
                    tag.line,
                    tag.col,
                ));
            }

            if tag.has_attr("media") {
                msgs.push(VNuMessage::error(
                    "Attribute “media” not allowed on element “area” at this point.",
                    tag.line,
                    tag.col,
                ));
            }

            if tag
                .get_attr("shape")
                .is_some_and(|s| s.eq_ignore_ascii_case("default"))
                && tag.has_attr("coords")
            {
                msgs.push(VNuMessage::error(
                    "Attribute “coords” not allowed on element “area” at this point.",
                    tag.line,
                    tag.col,
                ));
            }

            if tag.has_attr("download") && !tag.has_attr("href") {
                msgs.push(VNuMessage::error(
                    "Element “area” is missing required attribute “href”.",
                    tag.line,
                    tag.col,
                ));
            }

            if let Some(href) = tag.get_attr("href") {
                if href.is_empty() {
                    msgs.push(VNuMessage::error(
                        "Element “area” is missing required attribute “href”.",
                        tag.line,
                        tag.col,
                    ));
                }
                if !tag.has_attr("alt") {
                    msgs.push(VNuMessage::error(
                        "Element “area” is missing required attribute “alt”.",
                        tag.line,
                        tag.col,
                    ));
                }
            } else {
                if tag.has_attr("alt") {
                    msgs.push(VNuMessage::error(
                        "Element “area” is missing required attribute “href”.",
                        tag.line,
                        tag.col,
                    ));
                }
                let has_link_attrs = tag.has_attr("rel")
                    || tag.has_attr("target")
                    || tag.has_attr("type")
                    || tag.has_attr("hreflang")
                    || tag.has_attr("ping");
                if has_link_attrs && !tag.has_attr("alt") {
                    msgs.push(VNuMessage::error(
                        "Element “area” is missing required attribute “alt”.",
                        tag.line,
                        tag.col,
                    ));
                }
            }

            if let Some(hl) = tag.get_attr("hreflang") {
                if !is_valid_bcp47_lang_tag(hl) {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Bad value “{}” for attribute “hreflang” on element “area”.",
                            hl
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            }

            if let Some(t) = tag.get_attr("type") {
                if !is_valid_mime_type(t) {
                    msgs.push(VNuMessage::error(
                        format!("Bad value “{}” for attribute “type” on element “area”.", t),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        msgs
    }
}
