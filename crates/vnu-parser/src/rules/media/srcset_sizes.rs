use crate::context::ElementContext;
use vnu_core::VNuMessage;

pub fn parse_srcset(val: &str) -> Option<(bool, bool)> {
    let s = val.trim();
    if s.is_empty() || s != val {
        return None;
    }
    if s.starts_with(',') || s.ends_with(',') || s.contains(",,") {
        return None;
    }
    if s.contains("/*") || s.contains("*/") {
        return None;
    }
    if s.chars()
        .any(|c| matches!(c, '{' | '}' | '[' | ']' | '(' | ')' | '|'))
    {
        return None;
    }

    if !s.contains(char::is_whitespace) {
        if (s.starts_with("http:") && !s.starts_with("http://"))
            || (s.starts_with("https:") && !s.starts_with("https://"))
            || (s.starts_with("ftp:") && !s.starts_with("ftp://"))
        {
            return None;
        }
        return Some((false, true));
    }

    let mut seen_w: Vec<u64> = Vec::new();
    let mut seen_x: Vec<u64> = Vec::new();
    let mut has_w = false;
    let mut has_x = false;
    let mut has_unadorned = false;

    for cand in s.split(',') {
        let tokens: Vec<&str> = cand.split_whitespace().collect();
        if tokens.is_empty() {
            return None;
        }
        let url = tokens[0];
        if (url.starts_with("http:") && !url.starts_with("http://"))
            || (url.starts_with("https:") && !url.starts_with("https://"))
            || (url.starts_with("ftp:") && !url.starts_with("ftp://"))
        {
            return None;
        }

        let descriptors = &tokens[1..];
        if descriptors.len() > 1 {
            return None;
        }
        if descriptors.is_empty() {
            if has_unadorned || seen_x.contains(&1000) {
                return None;
            }
            has_unadorned = true;
            seen_x.push(1000);
        } else {
            let desc = descriptors[0];
            if let Some(num_str) = desc.strip_suffix('w') {
                if num_str.is_empty()
                    || num_str.starts_with('+')
                    || num_str.starts_with('-')
                    || !num_str.chars().all(|c| c.is_ascii_digit())
                {
                    return None;
                }
                let w: u64 = match num_str.parse() {
                    Ok(val) if val > 0 => val,
                    _ => return None,
                };
                if seen_w.contains(&w) {
                    return None;
                }
                seen_w.push(w);
                has_w = true;
            } else if let Some(num_str) = desc.strip_suffix('x') {
                if num_str.is_empty() || num_str.starts_with('+') || num_str.starts_with('-') {
                    return None;
                }
                let x_float: f64 = match num_str.parse::<f64>() {
                    Ok(val) if val > 0.0 && val.is_finite() => val,
                    _ => return None,
                };
                let x_key = (x_float * 1000.0).round() as u64;
                if seen_x.contains(&x_key) || (x_key == 1000 && has_unadorned) {
                    return None;
                }
                seen_x.push(x_key);
                has_x = true;
            } else {
                return None;
            }
        }

        if has_w && (has_x || has_unadorned) {
            return None;
        }
    }

    Some((has_w, has_x || has_unadorned))
}

pub fn is_valid_sizes(val: &str) -> bool {
    let mut s = val.trim();
    if s.is_empty() {
        return false;
    }
    if s.contains("+/**/") || s.contains("/**/vw") {
        return false;
    }
    while let Some(stripped) = s.strip_prefix("/**/") {
        s = stripped.trim();
    }
    while let Some(stripped) = s.strip_suffix("/**/") {
        s = stripped.trim();
    }
    if s.starts_with(',') || s.ends_with(',') || s.contains(",,") {
        return false;
    }
    if s.contains("/*") || s.contains("*/") {
        return false;
    }
    let lower = s.to_ascii_lowercase();
    for bad in &[
        "deg",
        "grad",
        "rad",
        "turn",
        "hz",
        "khz",
        "ms",
        "s ",
        "dpi",
        "dpcm",
        "dppx",
        "%",
        "default",
        "inherit",
        "initial",
        "foo",
        "bar",
        "all and",
        "all ",
        "(})",
        "(123)",
        "e+1.5",
        "-1e",
        "(min-width:)",
    ] {
        if lower.contains(bad) {
            return false;
        }
    }
    if lower == "1s" {
        return false;
    }

    let mut parts: Vec<&str> = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (idx, ch) in s.char_indices() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
        } else if ch == ',' && depth == 0 {
            parts.push(s[start..idx].trim());
            start = idx + 1;
        }
    }
    parts.push(s[start..].trim());

    if parts.is_empty() {
        return false;
    }

    for (i, &part) in parts.iter().enumerate() {
        if part.is_empty() {
            return false;
        }
        if i + 1 < parts.len() && (!part.starts_with('(') || !part.contains(')')) {
            return false;
        }

        let tokens: Vec<&str> = part.split_whitespace().collect();
        if let Some(&last_tok) = tokens.last() {
            if last_tok.starts_with('-')
                && last_tok != "-0"
                && !last_tok.starts_with("-0e")
                && !last_tok.starts_with("-0E")
            {
                return false;
            }
            if !last_tok.contains('(') && !last_tok.ends_with(')') {
                let is_valid_unit = [
                    "px", "em", "rem", "ex", "ch", "vw", "vh", "vmin", "vmax", "cm", "mm", "in",
                    "pt", "pc", "q", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax", "vi", "vb",
                    "auto",
                ]
                .iter()
                .any(|&u| last_tok.to_ascii_lowercase().ends_with(u));

                if !is_valid_unit && last_tok != "0" && last_tok != "-0" {
                    return false;
                }
            }
        }
    }

    true
}

pub fn validate_media_attributes(
    tag: &ElementContext,
    el: &str,
    is_in_picture: bool,
    is_in_video_or_audio: bool,
    msgs: &mut Vec<VNuMessage>,
) {
    // 1. Disallowed srcset on elements where not allowed
    if matches!(
        el,
        "audio" | "link" | "object" | "picture" | "track" | "video" | "image"
    ) && tag.has_attr("srcset")
    {
        msgs.push(VNuMessage::error(
            format!(
                "Attribute “srcset” not allowed on element “{}” at this point.",
                el
            ),
            tag.line,
            tag.col,
        ));
    }

    // 2. Disallowed usemap on picture and source
    if matches!(el, "picture" | "source") && tag.has_attr("usemap") {
        msgs.push(VNuMessage::error(
            format!(
                "Attribute “usemap” not allowed on element “{}” at this point.",
                el
            ),
            tag.line,
            tag.col,
        ));
    }

    // 3. Source in video or audio: cannot have srcset or sizes
    if el == "source" && is_in_video_or_audio {
        if tag.has_attr("srcset") {
            msgs.push(VNuMessage::error(
                "Attribute “srcset” not allowed on element “source” at this point.",
                tag.line,
                tag.col,
            ));
        }
        if tag.has_attr("sizes") {
            msgs.push(VNuMessage::error(
                "Attribute “sizes” not allowed on element “source” at this point.",
                tag.line,
                tag.col,
            ));
        }
    }

    // 4. Srcset validation on img and source
    if matches!(el, "img" | "source") && !(el == "source" && is_in_video_or_audio) {
        let has_sizes = tag.has_attr("sizes");
        let mut has_w = false;

        if let Some(ss) = tag.get_attr("srcset") {
            if let Some((parsed_w, has_x_or_unadorned)) = parse_srcset(ss) {
                has_w = parsed_w;
                if has_sizes && has_x_or_unadorned {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Bad value “{}” for attribute “srcset” on element “{}”.",
                            ss, el
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            } else {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for attribute “srcset” on element “{}”.",
                        ss, el
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }

        if el == "img" {
            let sizes_auto_lazy = tag
                .get_attr("sizes")
                .is_some_and(|s| s.trim().eq_ignore_ascii_case("auto"))
                && tag.has_attr_val("loading", "lazy");
            if has_sizes && !tag.has_attr("srcset") && !sizes_auto_lazy {
                msgs.push(VNuMessage::error(
                    "The “sizes” attribute must only be specified if the “srcset” attribute is also specified.",
                    tag.line,
                    tag.col,
                ));
            }
            if has_w && !has_sizes {
                msgs.push(VNuMessage::error(
                    "When the “srcset” attribute has any image candidate string with a width descriptor, the “sizes” attribute must also be specified.",
                    tag.line,
                    tag.col,
                ));
            }
        } else if el == "source" && !is_in_picture && has_w && !has_sizes {
            msgs.push(VNuMessage::error(
                "When the “srcset” attribute has any image candidate string with a width descriptor, the “sizes” attribute must also be specified — unless the following “img” element has a “loading” attribute set to “lazy” and a “sizes” attribute value starting with “auto”.",
                tag.line,
                tag.col,
            ));
        }

        // Sizes attribute validity
        if let Some(sizes_val) = tag.get_attr("sizes") {
            let trimmed = sizes_val.trim();
            if trimmed.to_ascii_lowercase().starts_with("auto") {
                if el == "img" && !tag.has_attr_val("loading", "lazy") {
                    msgs.push(VNuMessage::error(
                        "The “sizes” attribute value starting with “auto” is only valid for lazy-loaded images. Add “loading=\"lazy\"” to this element.",
                        tag.line,
                        tag.col,
                    ));
                }
            } else if !is_valid_sizes(sizes_val) {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for attribute “sizes” on element “{}”.",
                        sizes_val, el
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }
}
