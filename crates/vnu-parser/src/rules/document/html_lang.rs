use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use crate::rules::datatypes::tokens::is_valid_bcp47_lang_tag;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct HtmlLangRule {
    html_lang_seen: bool,
    warning_emitted: bool,
}

impl Rule for HtmlLangRule {
    fn name(&self) -> &'static str {
        "html-lang"
    }

    fn reset(&mut self) {
        self.html_lang_seen = false;
        self.warning_emitted = false;
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();

        if let Some(xml_lang) = tag.get_attr("xml:lang") {
            let lang = tag.get_attr("lang");
            let is_foreign_or_xhtml =
                ctx.is_xhtml || ctx.is_descendant_of("svg") || ctx.is_descendant_of("math");
            if (!is_foreign_or_xhtml && lang != Some(xml_lang))
                || (is_foreign_or_xhtml && lang.is_some_and(|l| l != xml_lang))
            {
                msgs.push(VNuMessage::error(
                    "When the attribute “xml:lang” in no namespace is specified, the element must also have the attribute “lang” present with the same value.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        if let Some(val) = tag.get_attr("lang") {
            if !val.is_empty() && !is_valid_bcp47_lang_tag(val) {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for attribute “lang” on element “{}”.",
                        val,
                        tag.name.to_ascii_lowercase()
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }

        if tag.name.eq_ignore_ascii_case("html") {
            if tag.has_attr("lang") {
                self.html_lang_seen = true;
            } else if !self.warning_emitted {
                msgs.push(VNuMessage::warning(
                    "Consider adding a “lang” attribute to the “html” start tag to declare the language of this document.",
                    tag.line,
                    tag.col,
                ));
                self.warning_emitted = true;
            }
        }

        msgs
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if ctx.mode == crate::context::DocumentMode::Svg {
            return msgs;
        }
        if !self.html_lang_seen && !self.warning_emitted {
            msgs.push(VNuMessage::warning(
                "Consider adding a “lang” attribute to the “html” start tag to declare the language of this document.",
                1,
                1,
            ));
            self.warning_emitted = true;
        }
        msgs
    }
}
