use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct HeadingsRule {
    section_stack: Vec<(String, bool)>, // (tag_name, has_heading)
    current_heading: Option<HeadingCapture>,
    has_level_1: bool,
    heading_count: usize,
    last_heading: Option<(String, usize)>,
}

struct HeadingCapture {
    line: usize,
    col: usize,
    has_content: bool,
}

impl Rule for HeadingsRule {
    fn name(&self) -> &'static str {
        "headings"
    }

    fn reset(&mut self) {
        self.section_stack.clear();
        self.current_heading = None;
        self.has_level_1 = false;
        self.heading_count = 0;
        self.last_heading = None;
    }

    fn interested_in_start_tag(&self, tag: &ElementContext) -> bool {
        let n = tag.name.as_str();
        (n.len() == 2
            && (n.as_bytes()[0] == b'h' || n.as_bytes()[0] == b'H')
            && (b'1'..=b'6').contains(&n.as_bytes()[1]))
            || n.eq_ignore_ascii_case("article")
            || n.eq_ignore_ascii_case("section")
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "article" || name_lower == "section" {
            self.section_stack.push((name_lower.clone(), false));
        }

        let heading_level = match name_lower.as_str() {
            "h1" => Some(1),
            "h2" => Some(2),
            "h3" => Some(3),
            "h4" => Some(4),
            "h5" => Some(5),
            "h6" => Some(6),
            _ => None,
        };

        if let Some(level) = heading_level {
            self.heading_count += 1;
            if level == 1 {
                self.has_level_1 = true;
            }

            if let Some((prev_tag, prev_level)) = &self.last_heading {
                if level > *prev_level + 1 {
                    let skipped = level - prev_level - 1;
                    let plural = if skipped == 1 { "" } else { "s" };
                    msgs.push(VNuMessage::error(
                        format!(
                            "The heading “{}” (with computed level {}) follows the heading “{}” (with computed level {}), skipping {} heading level{}.",
                            name_lower, level, prev_tag, prev_level, skipped, plural
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
            }
            self.last_heading = Some((name_lower.clone(), level));

            for (_, has_heading) in &mut self.section_stack {
                *has_heading = true;
            }
            self.current_heading = Some(HeadingCapture {
                line: tag.line,
                col: tag.col,
                has_content: false,
            });
        } else if let Some(ref mut capture) = self.current_heading {
            // Child element inside heading provides content (e.g. <img>)
            capture.has_content = true;
        }

        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        name: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = name.to_ascii_lowercase();

        if matches!(name_lower.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
            if let Some(capture) = self.current_heading.take() {
                if !capture.has_content {
                    msgs.push(VNuMessage::warning(
                        "Empty heading.",
                        capture.line,
                        capture.col,
                    ));
                }
            }
        } else if name_lower == "article" {
            if let Some((_, has_heading)) = self.section_stack.pop() {
                if !has_heading {
                    msgs.push(VNuMessage::warning(
                        "Article lacks heading. Consider using “h2”-“h6” elements to add identifying headings to all articles.",
                        line,
                        col,
                    ));
                }
            }
        } else if name_lower == "section" {
            if let Some((_, has_heading)) = self.section_stack.pop() {
                if !has_heading {
                    msgs.push(VNuMessage::warning(
                        "Section lacks heading. Consider using “h2”-“h6” elements to add identifying headings to all sections, or else use a “div” element instead for any cases where no heading is needed.",
                        line,
                        col,
                    ));
                }
            }
        }

        msgs
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, _ctx: &ValidationContext) -> bool {
        self.current_heading.is_some()
    }

    fn on_characters(
        &mut self,
        _ctx: &ValidationContext,
        text: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        if let Some(ref mut capture) = self.current_heading {
            if text.chars().any(|c| !c.is_ascii_whitespace()) {
                capture.has_content = true;
            }
        }
        vec![]
    }

    fn on_finish(&mut self, _ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if self.heading_count > 0 && !self.has_level_1 {
            msgs.push(VNuMessage::warning(
                "This document has heading elements but none of them has a computed heading level of 1.",
                1,
                1,
            ));
        }
        msgs
    }
}
