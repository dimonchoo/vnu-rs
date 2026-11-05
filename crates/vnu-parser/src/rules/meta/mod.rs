pub mod attributes;
pub mod directives;

use self::attributes::check_meta_attributes;
use self::directives::{validate_meta_csp, validate_meta_refresh};
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct MetaRule {
    has_meta_charset: bool,
    has_content_type_pragma: bool,
    has_meta_description: bool,
}

impl Rule for MetaRule {
    fn name(&self) -> &'static str {
        "meta"
    }

    fn reset(&mut self) {
        self.has_meta_charset = false;
        self.has_content_type_pragma = false;
        self.has_meta_description = false;
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if !tag.name.eq_ignore_ascii_case("meta") {
            return msgs;
        }

        if tag.has_attr("charset") {
            if self.has_meta_charset {
                msgs.push(VNuMessage::error(
                    "A document must not include more than one “meta” element with a “charset” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
            if self.has_content_type_pragma {
                msgs.push(VNuMessage::error(
                    "A document must not include both a “meta” element with an “http-equiv” attribute whose value is “content-type”, and a “meta” element with a “charset” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
            self.has_meta_charset = true;
        }

        if let Some(equiv) = tag.get_attr("http-equiv") {
            if equiv.eq_ignore_ascii_case("content-type") {
                if self.has_meta_charset {
                    msgs.push(VNuMessage::error(
                        "A document must not include both a “meta” element with an “http-equiv” attribute whose value is “content-type”, and a “meta” element with a “charset” attribute.",
                        tag.line,
                        tag.col,
                    ));
                }
                if self.has_content_type_pragma {
                    msgs.push(VNuMessage::error(
                        "A document must not include more than one “meta” element with an “http-equiv” attribute whose value is “content-type”.",
                        tag.line,
                        tag.col,
                    ));
                }
                self.has_content_type_pragma = true;
            }

            if let Some(content) = tag.get_attr("content") {
                if equiv.eq_ignore_ascii_case("refresh") {
                    validate_meta_refresh(tag, content, &mut msgs);
                } else if equiv.eq_ignore_ascii_case("content-security-policy") {
                    validate_meta_csp(tag, content, &mut msgs);
                }
            }
        }

        if tag.get_attr("name") == Some("description") {
            if self.has_meta_description {
                msgs.push(VNuMessage::error(
                    "A document must not include more than one “meta” element with its “name” attribute set to the value “description”.",
                    tag.line,
                    tag.col,
                ));
            }
            self.has_meta_description = true;
        }

        check_meta_attributes(tag, &mut msgs);

        msgs
    }
}
