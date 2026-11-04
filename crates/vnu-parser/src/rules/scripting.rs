use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

const JAVASCRIPT_MIME_TYPES: &[&str] = &[
    "application/ecmascript",
    "application/javascript",
    "application/x-ecmascript",
    "application/x-javascript",
    "text/ecmascript",
    "text/javascript",
    "text/javascript1.0",
    "text/javascript1.1",
    "text/javascript1.2",
    "text/javascript1.3",
    "text/javascript1.4",
    "text/javascript1.5",
    "text/jscript",
    "text/livescript",
    "text/x-ecmascript",
    "text/x-javascript",
];

#[inline]
fn is_js_mime_type(t: &str) -> bool {
    JAVASCRIPT_MIME_TYPES.iter().any(|m| m.eq_ignore_ascii_case(t))
}

#[derive(Default)]
pub struct ScriptingRule;

impl Rule for ScriptingRule {
    fn name(&self) -> &'static str {
        "scripting"
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if !tag.name.eq_ignore_ascii_case("script") {
            return msgs;
        }

        let script_type = tag.get_attr("type").map(|s| s.trim()).unwrap_or("");
        let has_src = tag.has_attr("src");

        // 1. Language attribute checks
        if tag.has_attr("language") {
            if tag.has_attr("type") && !script_type.eq_ignore_ascii_case("text/javascript") {
                msgs.push(VNuMessage::error(
                    "A “script” element with the “language=\"JavaScript\"” attribute set must not have a “type” attribute whose value is not “text/javascript”.",
                    tag.line,
                    tag.col,
                ));
            } else {
                msgs.push(VNuMessage::warning(
                    "The “language” attribute on the “script” element is obsolete. Use the “type” attribute instead.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        // 2. Charset attribute checks
        if let Some(charset) = tag.get_attr("charset") {
            if !has_src {
                msgs.push(VNuMessage::error(
                    "Element “script” must not have attribute “charset” unless attribute “src” is also specified.",
                    tag.line,
                    tag.col,
                ));
            } else if !charset.eq_ignore_ascii_case("utf-8") {
                msgs.push(VNuMessage::error(
                    "The only allowed value for the “charset” attribute for the “script” element is “utf-8”. (But the attribute is not needed and should be omitted altogether.)",
                    tag.line,
                    tag.col,
                ));
            }
        }

        let is_import_map = script_type.eq_ignore_ascii_case("importmap");
        let is_speculation_rules = script_type.eq_ignore_ascii_case("speculationrules");
        let is_module = script_type.eq_ignore_ascii_case("module");
        let is_classic = script_type.is_empty() || is_js_mime_type(script_type);
        let is_data_block = !is_import_map && !is_speculation_rules && !is_module && !is_classic;

        if is_import_map {
            if has_src {
                msgs.push(VNuMessage::error(
                    "A “script” element with a “type” attribute whose value is “importmap” must not have a “src” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
            for attr in &[
                "async",
                "nomodule",
                "defer",
                "crossorigin",
                "integrity",
                "referrerpolicy",
                "fetchpriority",
                "blocking",
            ] {
                if tag.has_attr(attr) {
                    let article = if *attr == "async" || *attr == "integrity" {
                        "an"
                    } else {
                        "a"
                    };
                    msgs.push(VNuMessage::error(
                        format!(
                            "A “script” element with “type=importmap” must not have {} “{}” attribute.",
                            article, attr
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        } else if is_speculation_rules {
            for attr in &[
                "nomodule",
                "async",
                "defer",
                "blocking",
                "crossorigin",
                "referrerpolicy",
                "integrity",
                "fetchpriority",
            ] {
                if tag.has_attr(attr) {
                    let article = if *attr == "async" || *attr == "integrity" {
                        "an"
                    } else {
                        "a"
                    };
                    msgs.push(VNuMessage::error(
                        format!(
                            "A “script” element with “type=speculationrules” must not have {} “{}” attribute.",
                            article, attr
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        } else if is_data_block {
            if has_src {
                msgs.push(VNuMessage::error(
                    "A “script” element with a “type” attribute whose value is neither a JavaScript MIME type, “module”, “importmap”, nor “speculationrules” (i.e., a data block) must not have a “src” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
            for attr in &[
                "async",
                "nomodule",
                "defer",
                "crossorigin",
                "integrity",
                "referrerpolicy",
                "fetchpriority",
                "blocking",
            ] {
                if tag.has_attr(attr) {
                    let article = if *attr == "async" || *attr == "integrity" {
                        "an"
                    } else {
                        "a"
                    };
                    msgs.push(VNuMessage::error(
                        format!(
                            "A “script” element with a “type” attribute whose value is neither a JavaScript MIME type, “module”, “importmap”, nor “speculationrules” (i.e., a data block) must not have {} “{}” attribute.",
                            article, attr
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        } else if is_module {
            if tag.has_attr("nomodule") {
                msgs.push(VNuMessage::error(
                    "A “script” element with a “nomodule” attribute must not have a “type” attribute with the value “module”.",
                    tag.line,
                    tag.col,
                ));
            }
            if tag.has_attr("defer") {
                msgs.push(VNuMessage::error(
                    "A “script” element with “type=module” must not have a “defer” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
            if !has_src {
                for attr in &["integrity", "fetchpriority", "blocking"] {
                    if tag.has_attr(attr) {
                        let article = if *attr == "integrity" { "an" } else { "a" };
                        msgs.push(VNuMessage::error(
                            format!(
                                "An inline “script” element with “type=module” must not have {} “{}” attribute.",
                                article, attr
                            ),
                            tag.line,
                            tag.col,
                        ));
                    }
                }
            }
        } else if is_classic && !has_src {
            if tag.has_attr("defer") {
                msgs.push(VNuMessage::error(
                    "An inline “script” element (i.e., a “script” element without a “src” attribute and with a “type” attribute that is either unspecified, empty, or a JavaScript MIME type) must not have a “defer” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
            if tag.has_attr("async") {
                msgs.push(VNuMessage::error(
                    "An inline classic “script” element (i.e., a “script” element without a “src” attribute and with a “type” attribute that is either unspecified, empty, or a JavaScript MIME type) must not have an “async” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
            if tag.has_attr("integrity") {
                msgs.push(VNuMessage::error(
                    "An inline classic “script” element (i.e., a “script” element without a “src” attribute and with a “type” attribute that is either unspecified, empty, or a JavaScript MIME type) must not have an “integrity” attribute.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        msgs
    }
}
