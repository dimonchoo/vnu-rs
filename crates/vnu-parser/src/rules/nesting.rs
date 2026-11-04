use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct NestingRule;

impl Rule for NestingRule {
    fn name(&self) -> &'static str {
        "nesting"
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, ctx: &ValidationContext) -> bool {
        ctx.open_elements
            .last()
            .is_some_and(|top| top.name.eq_ignore_ascii_case("table"))
            || (ctx.head_seen && !ctx.body_seen && ctx.is_descendant_of("noscript"))
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "a" && ctx.is_descendant_of("a") {
            msgs.push(VNuMessage::error(
                "Start tag “a” seen but an element of the same type was already open.",
                tag.line,
                tag.col,
            ));
        }

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

        let is_heading = |n: &str| matches!(n, "h1" | "h2" | "h3" | "h4" | "h5" | "h6");
        if is_heading(&name_lower) && ctx.open_elements.iter().any(|e| is_heading(&e.name)) {
            msgs.push(VNuMessage::error(
                "Heading cannot be a child of another heading.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "table" && ctx.is_descendant_of("table") {
            msgs.push(VNuMessage::error(
                "Start tag for “table” seen but the previous “table” is still open.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "select"
            && ctx
                .open_elements
                .last()
                .map(|e| e.name.eq_ignore_ascii_case("table"))
                .unwrap_or(false)
        {
            msgs.push(VNuMessage::error(
                "Start tag “select” seen in “table”.",
                tag.line,
                tag.col,
            ));
        }

        msgs
    }

    fn on_characters(
        &mut self,
        ctx: &ValidationContext,
        text: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if text.trim().is_empty() {
            return msgs;
        }

        if let Some(top) = ctx.open_elements.last() {
            if top.name.eq_ignore_ascii_case("table") {
                msgs.push(VNuMessage::error(
                    "Misplaced non-space characters inside a table.",
                    line,
                    col,
                ));
            }
        }

        if ctx.is_descendant_of("noscript") && ctx.is_descendant_of("head") {
            msgs.push(VNuMessage::error(
                "Non-space character inside “noscript” inside “head”.",
                line,
                col,
            ));
        }

        msgs
    }
}
