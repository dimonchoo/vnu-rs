use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct NestingRule {
    optgroup_missing_label: Vec<Option<(usize, usize)>>,
}

impl Rule for NestingRule {
    fn name(&self) -> &'static str {
        "nesting"
    }

    fn reset(&mut self) {
        self.optgroup_missing_label.clear();
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "button" && ctx.is_descendant_of("button") {
            msgs.push(VNuMessage::error(
                "The element “button” must not appear as a descendant of the “button” element.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "button" && ctx.is_descendant_of("a") {
            msgs.push(VNuMessage::error(
                "The element “button” must not appear as a descendant of the “a” element.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "address" && ctx.is_descendant_of("address") {
            msgs.push(VNuMessage::error(
                "The element “address” must not appear as a descendant of the “address” element.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "header" {
            if ctx.is_descendant_of("header") {
                msgs.push(VNuMessage::error(
                    "The element “header” must not appear as a descendant of the “header” element.",
                    tag.line,
                    tag.col,
                ));
            }
            if ctx.is_descendant_of("footer") {
                msgs.push(VNuMessage::error(
                    "The element “header” must not appear as a descendant of the “footer” element.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        if name_lower == "footer" {
            if ctx.is_descendant_of("footer") {
                msgs.push(VNuMessage::error(
                    "The element “footer” must not appear as a descendant of the “footer” element.",
                    tag.line,
                    tag.col,
                ));
            }
            if ctx.is_descendant_of("header") {
                msgs.push(VNuMessage::error(
                    "The element “footer” must not appear as a descendant of the “header” element.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        if name_lower == "audio" && tag.has_attr("controls") && ctx.is_descendant_of("button") {
            msgs.push(VNuMessage::error(
                "The element “audio” with the attribute “controls” must not appear as a descendant of the “button” element.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "bdo" {
            if let Some(dir) = tag.get_attr("dir") {
                if dir.eq_ignore_ascii_case("auto")
                    || !matches!(dir.to_ascii_lowercase().as_str(), "ltr" | "rtl")
                {
                    msgs.push(VNuMessage::error(
                        "The value of “dir” attribute for the “bdo” element must not be “auto”.",
                        tag.line,
                        tag.col,
                    ));
                }
            } else {
                msgs.push(VNuMessage::error(
                    "Element “bdo” must have attribute “dir”.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        if name_lower == "optgroup" {
            if tag.has_attr("label") {
                self.optgroup_missing_label.push(None);
            } else {
                self.optgroup_missing_label.push(Some((tag.line, tag.col)));
            }
        } else if name_lower == "legend"
            && ctx
                .parent()
                .is_some_and(|p| p.name.eq_ignore_ascii_case("optgroup"))
        {
            if let Some(last) = self.optgroup_missing_label.last_mut() {
                *last = None;
            }
        }

        if name_lower == "a" && ctx.is_descendant_of("button") {
            let msg = if tag.has_attr("href") {
                "The element “a” with the attribute “href” must not appear as a descendant of the “button” element."
            } else {
                "The element “a” must not appear as a descendant of the “button” element."
            };
            msgs.push(VNuMessage::error(msg, tag.line, tag.col));
        }

        if name_lower == "main" {
            for ancestor in ctx.open_elements.iter().rev() {
                let a = ancestor.name.to_ascii_lowercase();
                if matches!(
                    a.as_str(),
                    "article" | "aside" | "footer" | "header" | "nav"
                ) {
                    msgs.push(VNuMessage::error(
                        format!("The “main” element must not appear as a descendant of the “{}” element.", a),
                        tag.line,
                        tag.col,
                    ));
                    break;
                }
            }
        }

        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if name.eq_ignore_ascii_case("optgroup") {
            if let Some(Some((line, col))) = self.optgroup_missing_label.pop() {
                msgs.push(VNuMessage::error(
                    "An “optgroup” element with no child “legend” element must have a “label” attribute.",
                    line,
                    col,
                ));
            }
        }
        msgs
    }
}
