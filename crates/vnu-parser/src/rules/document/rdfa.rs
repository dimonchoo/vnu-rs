use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct RdfaRule;

impl Rule for RdfaRule {
    fn name(&self) -> &'static str {
        "rdfa"
    }

    fn interested_in_start_tag(&self, tag: &ElementContext) -> bool {
        !tag.attrs.is_empty()
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let el = tag.name.to_ascii_lowercase();

        // 1. Disallowed href on non-linking elements in RDFa
        if tag.has_attr("href") && matches!(el.as_str(), "div" | "img" | "p") {
            msgs.push(VNuMessage::error(
                format!(
                    "Attribute “href” not allowed on element “{}” at this point.",
                    el
                ),
                tag.line,
                tag.col,
            ));
        }

        // 2. Disallowed src on <a>
        if el == "a" && tag.has_attr("src") {
            msgs.push(VNuMessage::error(
                "Attribute “src” not allowed on element “a” at this point.",
                tag.line,
                tag.col,
            ));
        }

        // 3. Disallowed xml:base on <html>
        if el == "html" && tag.has_attr("xml:base") {
            msgs.push(VNuMessage::error(
                "Attribute “xml:base” not allowed on element “html” at this point.",
                tag.line,
                tag.col,
            ));
        }

        // 4. xmlns:* declarations in XHTML-ish documents; with HTML parsing the tree
        //    builder's checkAttributes reports them.
        if !ctx.html_parser
            && el != "svg"
            && el != "math"
            && !ctx.is_descendant_of("svg")
            && !ctx.is_descendant_of("math")
            && !ctx.is_xhtml
        {
            for (attr_name, _) in tag.attrs.iter() {
                let attr_lower = attr_name.to_ascii_lowercase();
                if attr_lower.starts_with("xmlns:") {
                    msgs.push(VNuMessage::error(
                        format!("Attribute “{}” not allowed here.", attr_name),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        // 5. prefix attribute syntax
        if let Some(prefix_val) = tag.get_attr("prefix") {
            let mut valid = true;
            let trimmed = prefix_val.trim();
            if trimmed.is_empty() {
                valid = false;
            } else {
                let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                // Tokens are pairs of `prefix: URI`
                let mut i = 0;
                while i < tokens.len() {
                    let p = tokens[i];
                    if let Some(colon_pos) = p.find(':') {
                        let ncname = &p[..colon_pos];
                        if ncname.is_empty()
                            || !ncname
                                .chars()
                                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                            || ncname.starts_with('$')
                        {
                            valid = false;
                            break;
                        }
                        if colon_pos + 1 < p.len() {
                            // URL was attached without space
                            i += 1;
                        } else {
                            // Expect next token to be URL
                            if i + 1 < tokens.len() {
                                i += 2;
                            } else {
                                valid = false;
                                break;
                            }
                        }
                    } else {
                        valid = false;
                        break;
                    }
                }
            }
            if !valid {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for attribute “prefix” on element “{}”.",
                        prefix_val, el
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }

        // 6. Block <p> in phrasing elements (strong, span) in RDFa
        if el == "p" {
            if let Some(parent) = ctx.parent() {
                let p_name = parent.name.to_ascii_lowercase();
                if matches!(p_name.as_str(), "strong" | "span") {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Element “p” not allowed as child of “{}” in this context.",
                            p_name
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        msgs
    }
}
