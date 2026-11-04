use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;

const DISALLOWED_ON_PICTURE: &[&str] = &[
    "align", "alt", "border", "crossorigin", "height", "hspace", "ismap", "longdesc",
    "lowsrc", "media", "name", "role", "sizes", "src", "vspace", "width",
];

const DISALLOWED_ON_SOURCE_IN_PICTURE: &[&str] = &[
    "align", "alt", "border", "crossorigin", "hspace", "ismap", "longdesc", "role", "vspace",
];

fn is_valid_srcset(val: &str) -> bool {
    let s = val.trim();
    if s.is_empty() || s != val {
        return false;
    }
    if s.starts_with(',') || s.ends_with(',') || s.contains(",,") {
        return false;
    }
    if s.contains("/*") || s.contains("*/") {
        return false;
    }
    if s.chars().any(|c| matches!(c, '{' | '}' | '[' | ']' | '(' | ')' | '|')) {
        return false;
    }

    let mut seen_w: Vec<u64> = Vec::new();
    let mut seen_x: Vec<u64> = Vec::new(); // store fixed-point 1000x float to avoid float eq
    let mut has_w = false;
    let mut has_x = false;
    let mut has_unadorned = false;

    for cand in s.split(',') {
        let tokens: Vec<&str> = cand.split_whitespace().collect();
        if tokens.is_empty() {
            return false;
        }
        let descriptors = &tokens[1..];
        if descriptors.len() > 1 {
            return false;
        }
        if descriptors.is_empty() {
            if has_unadorned || seen_x.contains(&1000) {
                return false;
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
                    return false;
                }
                let w: u64 = match num_str.parse() {
                    Ok(val) if val > 0 => val,
                    _ => return false,
                };
                if seen_w.contains(&w) {
                    return false;
                }
                seen_w.push(w);
                has_w = true;
            } else if let Some(num_str) = desc.strip_suffix('x') {
                if num_str.is_empty() || num_str.starts_with('+') || num_str.starts_with('-') {
                    return false;
                }
                let x_float: f64 = match num_str.parse::<f64>() {
                    Ok(val) if val > 0.0 && val.is_finite() => val,
                    _ => return false,
                };
                let x_key = (x_float * 1000.0).round() as u64;
                if seen_x.contains(&x_key) || (x_key == 1000 && has_unadorned) {
                    return false;
                }
                seen_x.push(x_key);
                has_x = true;
            } else {
                return false;
            }
        }

        if has_w && has_x {
            return false;
        }
    }

    true
}

#[derive(Debug, Clone)]
struct SourceInfo {
    has_media: bool,
    has_type: bool,
    sizes_starts_with_auto: bool,
    line: usize,
    col: usize,
}

#[derive(Debug, Clone, Default)]
struct PictureState {
    seen_img: bool,
    sources: Vec<SourceInfo>,
}

#[derive(Default)]
pub struct MediaRule {
    picture_stack: Vec<PictureState>,
}

impl Rule for MediaRule {
    fn name(&self) -> &'static str {
        "media"
    }

    fn reset(&mut self) {
        self.picture_stack.clear();
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let el = tag.name.to_ascii_lowercase();

        // 1. Srcset validation on img and source
        if matches!(el.as_str(), "img" | "source") {
            if let Some(ss) = tag.get_attr("srcset") {
                if !is_valid_srcset(ss) {
                    msgs.push(VNuMessage::error(
                        format!("Bad value “{}” for attribute “srcset” on element “{}”.", ss, el),
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        // 2. Audio element constraints
        if el == "audio" && tag.has_attr("srcset") {
            msgs.push(VNuMessage::error(
                "Attribute “srcset” not allowed on element “audio” at this point.",
                tag.line,
                tag.col,
            ));
        }

        // 3. Img constraints
        if el == "img" {
            if tag.has_attr("type") {
                msgs.push(VNuMessage::error(
                    "Attribute “type” not allowed on element “img” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
            if !tag.has_attr("src") && !tag.has_attr("srcset") {
                msgs.push(VNuMessage::error(
                    "Element “img” is missing one or more of the following attributes: “src”, “srcset”.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        // 4. Picture element opening
        if el == "picture" {
            for attr in DISALLOWED_ON_PICTURE {
                if tag.has_attr(attr) {
                    msgs.push(VNuMessage::error(
                        format!("Attribute “{}” not allowed on element “picture” at this point.", attr),
                        tag.line,
                        tag.col,
                    ));
                }
            }
            self.picture_stack.push(PictureState::default());
            return msgs;
        }

        // 5. Children of picture
        let is_in_picture = ctx
            .open_elements
            .last()
            .map(|e| e.name.eq_ignore_ascii_case("picture"))
            .unwrap_or(false);

        if is_in_picture {
            if let Some(state) = self.picture_stack.last_mut() {
                if !matches!(el.as_str(), "source" | "img" | "script" | "template") {
                    msgs.push(VNuMessage::error(
                        format!("Element “{}” not allowed as child of “picture” in this context.", el),
                        tag.line,
                        tag.col,
                    ));
                } else if el == "source" {
                    for attr in DISALLOWED_ON_SOURCE_IN_PICTURE {
                        if tag.has_attr(attr) {
                            msgs.push(VNuMessage::error(
                                format!("Attribute “{}” not allowed on element “source” at this point.", attr),
                                tag.line,
                                tag.col,
                            ));
                        }
                    }
                    if state.seen_img {
                        msgs.push(VNuMessage::error(
                            "Element “source” not allowed as child of “picture” in this context.",
                            tag.line,
                            tag.col,
                        ));
                    }
                    if let Some(media) = tag.get_attr("media") {
                        let trimmed = media.trim();
                        if trimmed.is_empty() {
                            msgs.push(VNuMessage::error(
                                "Value of “media” attribute here must not be empty.",
                                tag.line,
                                tag.col,
                            ));
                        } else if trimmed.eq_ignore_ascii_case("all") {
                            msgs.push(VNuMessage::error(
                                "Value of “media” attribute here must not be “all”.",
                                tag.line,
                                tag.col,
                            ));
                        }
                    }
                    let sizes_auto = tag
                        .get_attr("sizes")
                        .map(|s| s.trim().to_ascii_lowercase().starts_with("auto"))
                        .unwrap_or(false);

                    state.sources.push(SourceInfo {
                        has_media: tag.has_attr("media"),
                        has_type: tag.has_attr("type"),
                        sizes_starts_with_auto: sizes_auto,
                        line: tag.line,
                        col: tag.col,
                    });
                } else if el == "img" {
                    if state.seen_img {
                        msgs.push(VNuMessage::error(
                            "Element “img” not allowed as child of “picture” in this context.",
                            tag.line,
                            tag.col,
                        ));
                    } else {
                        state.seen_img = true;
                        let img_has_lazy = tag.has_attr_val("loading", "lazy");
                        let img_sizes_auto = tag
                            .get_attr("sizes")
                            .map(|s| s.trim().to_ascii_lowercase().starts_with("auto"))
                            .unwrap_or(false);

                        for src in &state.sources {
                            if !src.has_media && !src.has_type {
                                msgs.push(VNuMessage::error(
                                    "A “source” element that has a following sibling “source” element or “img” element with a “srcset” attribute must have a “media” attribute and/or “type” attribute.",
                                    src.line,
                                    src.col,
                                ));
                            }
                            if src.sizes_starts_with_auto && (!img_has_lazy || !img_sizes_auto) {
                                msgs.push(VNuMessage::error(
                                    "A “sizes” attribute value starting with “auto” on a “source” element requires the following “img” element to have a “loading” attribute set to “lazy” and a “sizes” attribute value starting with “auto”.",
                                    src.line,
                                    src.col,
                                ));
                            }
                        }
                    }
                }
            }
        }

        msgs
    }

    fn on_end_tag(&mut self, _ctx: &ValidationContext, name: &str, line: usize, col: usize) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if name.eq_ignore_ascii_case("picture") {
            if let Some(state) = self.picture_stack.pop() {
                if !state.seen_img {
                    msgs.push(VNuMessage::error(
                        "Element “picture” is missing a required instance of child element “img”.",
                        line,
                        col,
                    ));
                }
            }
        }
        msgs
    }
}
