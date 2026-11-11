pub mod body_scope;
pub mod css_validator;
pub mod tokenizer;

use self::css_validator::CssValidator;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct StyleRule {
    parent_child_counts: Vec<usize>,
    current_style: Option<StyleCapture>,
}

struct StyleCapture {
    in_body: bool,
    in_template: bool,
    line: usize,
    col: usize,
    buffer: String,
}

impl Rule for StyleRule {
    fn name(&self) -> &'static str {
        "style"
    }

    fn reset(&mut self) {
        self.parent_child_counts.clear();
        self.current_style = None;
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        // Increment current parent child count
        let parent_child_count = if let Some(count) = self.parent_child_counts.last_mut() {
            let prev = *count;
            *count += 1;
            prev
        } else {
            0
        };

        // Push new child count for non-void, non-self-closing elements
        if !crate::engine::html_stack::is_void_element(&name_lower) {
            self.parent_child_counts.push(0);
        }

        if name_lower == "style" {
            // 1. type attribute
            if !ctx.is_descendant_of("svg") {
                if let Some(t) = tag.get_attr("type") {
                    if t.trim().eq_ignore_ascii_case("text/css") {
                        msgs.push(VNuMessage::warning(
                            "The “type” attribute for the “style” element is not needed and should be omitted.",
                            tag.line,
                            tag.col,
                        ));
                    } else {
                        msgs.push(VNuMessage::error(
                            "The only allowed value for the “type” attribute for the “style” element is “text/css” (with no parameters). (But the attribute is not needed and should be omitted altogether.)",
                            tag.line,
                            tag.col,
                        ));
                    }
                }
            }

            // 2. scoped attribute
            if tag.has_attr("scoped") {
                msgs.push(VNuMessage::error(
                    "Attribute “scoped” not allowed on element “style” at this point.",
                    tag.line,
                    tag.col,
                ));
            }

            // 3. Placement in body (HTML only, outside SVG/MathML)
            let in_body = ctx.is_descendant_of("body")
                && !ctx.is_descendant_of("svg")
                && !ctx.is_descendant_of("math");
            let in_template = ctx.is_descendant_of("template");

            if in_body && !in_template {
                let parent_name = ctx
                    .parent()
                    .map(|p| p.name.to_ascii_lowercase())
                    .unwrap_or_default();

                // Prohibited phrasing / structured parents
                match parent_name.as_str() {
                    "a" => {
                        let grandparent_phrasing = ctx.open_elements.len() >= 2
                            && matches!(
                                ctx.open_elements[ctx.open_elements.len() - 2]
                                    .name
                                    .to_ascii_lowercase()
                                    .as_str(),
                                "p" | "span"
                                    | "em"
                                    | "strong"
                                    | "small"
                                    | "b"
                                    | "i"
                                    | "u"
                                    | "q"
                                    | "label"
                            );
                        if grandparent_phrasing {
                            msgs.push(VNuMessage::error(
                                "Element “style” not allowed as child of “a” in this context. Note: The “a” element has a transparent content model; its allowed content is inherited from its parent element.",
                                tag.line,
                                tag.col,
                            ));
                        }
                    }
                    "p" | "span" | "time" | "select" | "ul" | "table" => {
                        msgs.push(VNuMessage::error(
                            format!(
                                "Element “style” not allowed as child of “{}” in this context.",
                                parent_name
                            ),
                            tag.line,
                            tag.col,
                        ));
                    }
                    _ => {}
                }

                // First child rule in body
                if parent_child_count > 0 {
                    msgs.push(VNuMessage::error(
                        "A “style” element in “body” must be the first child of its parent.",
                        tag.line,
                        tag.col,
                    ));
                }
            }

            self.current_style = Some(StyleCapture {
                in_body,
                in_template,
                line: tag.line,
                col: tag.col,
                buffer: String::new(),
            });
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
        self.parent_child_counts.pop();

        if name.eq_ignore_ascii_case("style") {
            if let Some(capture) = self.current_style.take() {
                msgs.extend(CssValidator::validate(
                    &capture.buffer,
                    capture.in_body,
                    capture.in_template,
                    capture.line,
                    capture.col,
                ));
            }
        }

        msgs
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, ctx: &ValidationContext) -> bool {
        self.current_style.is_some()
            || (ctx.body_seen
                && self.parent_child_counts.last().copied() == Some(0)
                && ctx.parent().is_some_and(|p| {
                    matches!(
                        p.name.as_str(),
                        "div" | "body" | "section" | "article" | "main" | "aside" | "nav"
                    )
                }))
    }

    fn on_characters(
        &mut self,
        _ctx: &ValidationContext,
        text: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        if let Some(ref mut capture) = self.current_style {
            capture.buffer.push_str(text);
        } else if text.chars().any(|c| !c.is_ascii_whitespace()) {
            if let Some(count) = self.parent_child_counts.last_mut() {
                *count += 1;
            }
        }
        vec![]
    }
}
