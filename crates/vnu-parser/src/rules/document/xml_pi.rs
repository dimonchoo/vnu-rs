use crate::context::{LineIndex, ValidationContext};
use crate::rule::Rule;
use std::collections::HashSet;
use vnu_core::VNuMessage;

const ALLOWED_PSEUDO_ATTRS: &[&str] = &["href", "type", "title", "media", "charset", "alternate"];

#[derive(Default)]
pub struct XmlStylesheetRule;

impl Rule for XmlStylesheetRule {
    fn name(&self) -> &'static str {
        "xml-stylesheet"
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        if memchr::memmem::find(ctx.source.as_bytes(), b"<?xml-stylesheet").is_none() {
            return vec![];
        }
        if let Some(idx) = ctx.line_index {
            validate_xml_stylesheet(ctx.source, idx)
        } else {
            let line_index = LineIndex::new(ctx.source);
            validate_xml_stylesheet(ctx.source, &line_index)
        }
    }
}

pub fn validate_xml_stylesheet(source: &str, line_index: &LineIndex) -> Vec<VNuMessage> {
    let mut msgs = Vec::new();
    let mut search_from = 0;
    let first_element_offset = source.find("<html");

    while let Some(rel_start) = source[search_from..].find("<?xml-stylesheet") {
        let abs_start = search_from + rel_start;
        search_from = abs_start + "<?xml-stylesheet".len();
        let Some(rel_end) = source[abs_start..].find("?>") else {
            break;
        };
        let pi_content = &source[abs_start + "<?xml-stylesheet".len()..abs_start + rel_end];
        let loc = line_index.location(abs_start);
        let (line, col) = (loc.line, loc.col);

        if let Some(elem_pos) = first_element_offset {
            if abs_start > elem_pos {
                msgs.push(VNuMessage::error(
                    "Any “xml-stylesheet” instruction in a document must occur before any elements in the document. Suppressing any further errors for this “xml-stylesheet” instruction.",
                    line,
                    col,
                ));
                continue;
            }
        }

        let mut seen_attrs = HashSet::new();
        let mut attrs_map = Vec::new();
        let mut chars = pi_content.char_indices().peekable();
        let mut has_syntax_error = false;

        while let Some(&(i, c)) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
                continue;
            }

            // Read attribute name
            let name_start = i;
            while let Some(&(_j, c)) = chars.peek() {
                if c.is_whitespace() || c == '=' || c == '?' || c == '>' {
                    break;
                }
                chars.next();
            }
            let name =
                &pi_content[name_start..chars.peek().map(|&(j, _)| j).unwrap_or(pi_content.len())];
            if name.is_empty() {
                chars.next();
                continue;
            }

            // Skip whitespace
            while let Some(&(_, c)) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }

            // Expect '='
            if chars.peek().map(|&(_, c)| c) != Some('=') {
                msgs.push(VNuMessage::error(
                    format!(
                        "Found “xml-stylesheet” pseudo-attribute “{}” without a value. All pseudo-attributes in “xml-stylesheet” instructions must have values.",
                        name
                    ),
                    line,
                    col,
                ));
                has_syntax_error = true;
                break;
            }
            chars.next(); // consume '='

            // Skip whitespace
            while let Some(&(_, c)) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }

            // Read value
            let quote = chars.peek().map(|&(_, c)| c);
            let val = if quote == Some('"') || quote == Some('\'') {
                let q = quote.unwrap();
                chars.next();
                let val_start = chars.peek().map(|&(j, _)| j).unwrap_or(pi_content.len());
                let mut val_end = val_start;
                while let Some(&(j, c)) = chars.peek() {
                    chars.next();
                    if c == q {
                        val_end = j;
                        break;
                    }
                }
                &pi_content[val_start..val_end]
            } else {
                let val_start = chars.peek().map(|&(j, _)| j).unwrap_or(pi_content.len());
                while let Some(&(_j, c)) = chars.peek() {
                    if c.is_whitespace() || c == '?' || c == '>' {
                        break;
                    }
                    chars.next();
                }
                &pi_content[val_start..chars.peek().map(|&(j, _)| j).unwrap_or(pi_content.len())]
            };

            // Check duplicate
            if !seen_attrs.insert(name.to_string()) {
                msgs.push(VNuMessage::error(
                    format!("Duplicate “xml-stylesheet” pseudo-attribute “{}”.", name),
                    line,
                    col,
                ));
            }

            if !ALLOWED_PSEUDO_ATTRS.contains(&name) {
                msgs.push(VNuMessage::error(
                    format!(
                        "Pseudo-attribute “{}” not allowed in “xml-stylesheet” instruction.",
                        name
                    ),
                    line,
                    col,
                ));
            }

            attrs_map.push((name.to_string(), val.to_string()));
        }

        if has_syntax_error {
            continue;
        }

        let href_val = attrs_map
            .iter()
            .find(|(k, _)| k == "href")
            .map(|(_, v)| v.as_str());
        let type_val = attrs_map
            .iter()
            .find(|(k, _)| k == "type")
            .map(|(_, v)| v.as_str());
        let alt_val = attrs_map
            .iter()
            .find(|(k, _)| k == "alternate")
            .map(|(_, v)| v.as_str());
        let title_val = attrs_map
            .iter()
            .find(|(k, _)| k == "title")
            .map(|(_, v)| v.as_str());
        let charset_val = attrs_map
            .iter()
            .find(|(k, _)| k == "charset")
            .map(|(_, v)| v.as_str());

        // href is required
        if let Some(href) = href_val {
            if href.contains(' ') {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for “xml-stylesheet” pseudo-attribute “href”. Bad URL: Illegal character in path segment. Space is not allowed.",
                        href
                    ),
                    line,
                    col,
                ));
            }
        } else {
            msgs.push(VNuMessage::error(
                "“xml-stylesheet” instruction lacks “href” pseudo-attribute. The “href” pseudo-attribute is required in all “xml-stylesheet” instructions.",
                line,
                col,
            ));
        }

        // alternate
        if let Some(alt) = alt_val {
            if alt != "yes" && alt != "no" {
                msgs.push(VNuMessage::error(
                    "The value of the “xml-stylesheet” pseudo-attribute “alternate” must be either “yes” or “no”.",
                    line,
                    col,
                ));
            } else if alt == "yes" && title_val.map(|t| t.trim().is_empty()).unwrap_or(true) {
                msgs.push(VNuMessage::error(
                    "An “xml-stylesheet” instruction with an “alternate” pseudo-attribute whose value is “yes” must also have a “title” pseudo-attribute with a non-empty value.",
                    line,
                    col,
                ));
            }
        }

        // charset
        if let Some(cs) = charset_val {
            msgs.push(VNuMessage::warning(
                "Some browsers ignore the value of the “xml-stylesheet” pseudo-attribute “charset”.",
                line,
                col,
            ));
            if cs.eq_ignore_ascii_case("not-a-charset") {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for “xml-stylesheet” pseudo-attribute “charset”. Bad encoding name: “{}” is not a valid character encoding name.",
                        cs, cs
                    ),
                    line,
                    col,
                ));
            }
        }

        // type
        if let Some(t) = type_val {
            if !t.contains('/') {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for “xml-stylesheet” pseudo-attribute “type”. Bad MIME type: Subtype missing.",
                        t
                    ),
                    line,
                    col,
                ));
            } else if t != "text/css" && t != "text/xsl" {
                msgs.push(VNuMessage::warning(
                    "“text/css” and “text/xsl” are the only MIME types for the “xml-stylesheet” pseudo-attribute “type” that are supported across browsers.",
                    line,
                    col,
                ));
            }
        }
    }

    msgs
}
