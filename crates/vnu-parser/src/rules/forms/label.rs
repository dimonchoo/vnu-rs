use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use std::collections::HashSet;
use vnu_core::VNuMessage;

pub const LABELABLE_ELEMENTS: &[&str] = &[
    "button", "input", "meter", "output", "progress", "select", "textarea",
];

struct ActiveLabel {
    for_attr: Option<String>,
    labelable_count: usize,
    has_role: bool,
    has_aria_hidden: bool,
    line: usize,
    col: usize,
}

#[derive(Default)]
pub struct LabelRule {
    label_stack: Vec<ActiveLabel>,
    pending_for_checks: Vec<(String, usize, usize)>,
    labelable_ids: HashSet<String>,
}

impl Rule for LabelRule {
    fn name(&self) -> &'static str {
        "forms::label"
    }

    fn reset(&mut self) {
        self.label_stack.clear();
        self.pending_for_checks.clear();
        self.labelable_ids.clear();
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        let is_hidden_input = name_lower == "input"
            && tag
                .get_attr("type")
                .is_some_and(|t| t.eq_ignore_ascii_case("hidden"));
        let is_labelable = LABELABLE_ELEMENTS.contains(&name_lower.as_str()) && !is_hidden_input;

        if is_labelable {
            if let Some(id) = tag.get_attr("id") {
                self.labelable_ids.insert(id.to_string());
            }

            for label in &mut self.label_stack {
                label.labelable_count += 1;
                if label.labelable_count == 2 {
                    msgs.push(VNuMessage::error(
                        "The “label” element may contain at most one “button”, “input”, “meter”, “output”, “progress”, “select”, or “textarea” descendant.",
                        tag.line,
                        tag.col,
                    ));
                }

                if label.has_role {
                    msgs.push(VNuMessage::error(
                        "The “role” attribute must not be used on any “label” element that is an ancestor of a labelable element.",
                        label.line,
                        label.col,
                    ));
                    label.has_role = false;
                }

                if label.has_aria_hidden {
                    msgs.push(VNuMessage::error(
                        "The “aria-hidden” attribute must not be used on any “label” element that is an ancestor of a labelable element.",
                        label.line,
                        label.col,
                    ));
                    label.has_aria_hidden = false;
                }

                if let Some(ref fid) = label.for_attr {
                    let id = tag.get_attr("id");
                    if id != Some(fid.as_str()) {
                        msgs.push(VNuMessage::error(
                            format!(
                                "Any “{}” descendant of a “label” element with a “for” attribute must have an ID value that matches that “for” attribute.",
                                name_lower
                            ),
                            tag.line,
                            tag.col,
                        ));
                    }
                }
            }
        }

        if name_lower == "label" {
            let for_attr = tag.get_attr("for").map(|s| s.to_string());
            if let Some(ref fid) = for_attr {
                self.pending_for_checks
                    .push((fid.clone(), tag.line, tag.col));
            }

            self.label_stack.push(ActiveLabel {
                for_attr,
                labelable_count: 0,
                has_role: tag.has_attr("role"),
                has_aria_hidden: tag.has_attr("aria-hidden"),
                line: tag.line,
                col: tag.col,
            });
        }

        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        tag_name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        if tag_name.eq_ignore_ascii_case("label") {
            self.label_stack.pop();
        }
        vec![]
    }

    fn on_finish(&mut self, _ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        for (fid, line, col) in &self.pending_for_checks {
            if !self.labelable_ids.contains(fid) {
                msgs.push(VNuMessage::error(
                    "The value of the “for” attribute of the “label” element must be the ID of a non-hidden form control.",
                    *line,
                    *col,
                ));
            }
        }
        msgs
    }
}
