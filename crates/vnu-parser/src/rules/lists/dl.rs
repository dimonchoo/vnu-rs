use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use std::collections::HashSet;
use vnu_core::VNuMessage;

const DISALLOWED_IN_DT: &[&str] = &[
    "article", "nav", "section", "header", "footer", "h1", "h2", "h3", "h4", "h5", "h6", "hgroup",
];

#[derive(Default)]
pub struct DlRule {
    dl_stack: Vec<DlState>,
    in_dt: bool,
    current_dt_text: String,
    dt_line: usize,
    dt_col: usize,
}

#[derive(Default)]
struct DlState {
    mode: DlMode,
    div_state: Option<DivState>,
    seen_dt_names: HashSet<String>,
    seen_template: bool,
}

#[derive(Default, PartialEq, Eq)]
enum DlMode {
    #[default]
    Undecided,
    DtDd(bool), // true if last child was dt
    DivOnly,
}

#[derive(Default)]
struct DivState {
    has_dt: bool,
    has_dd: bool,
    after_dd: bool,
}

impl Rule for DlRule {
    fn name(&self) -> &'static str {
        "dl"
    }

    fn reset(&mut self) {
        self.dl_stack.clear();
        self.in_dt = false;
        self.current_dt_text.clear();
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, ctx: &ValidationContext) -> bool {
        self.in_dt
            || ctx.open_elements.last().is_some_and(|top| {
                top.name.eq_ignore_ascii_case("dl")
                    || (top.name.eq_ignore_ascii_case("div")
                        && ctx.open_elements.len() >= 2
                        && ctx.open_elements[ctx.open_elements.len() - 2]
                            .name
                            .eq_ignore_ascii_case("dl"))
            })
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let el = tag.name.to_ascii_lowercase();

        // 1. Disallowed descendants in dt
        if DISALLOWED_IN_DT.contains(&el.as_str()) && ctx.is_descendant_of("dt") {
            msgs.push(VNuMessage::error(
                format!(
                    "The element “{}” must not appear as a descendant of the “dt” element.",
                    el
                ),
                tag.line,
                tag.col,
            ));
        }

        // 2. Dl tag opening
        if el == "dl" {
            let parent_name = ctx
                .open_elements
                .last()
                .map(|e| e.name.as_str())
                .unwrap_or("");
            if parent_name.eq_ignore_ascii_case("dl") {
                msgs.push(VNuMessage::error(
                    "Element “dl” not allowed as child of “dl” in this context.",
                    tag.line,
                    tag.col,
                ));
            } else if parent_name.eq_ignore_ascii_case("div")
                && ctx.open_elements.len() >= 2
                && ctx.open_elements[ctx.open_elements.len() - 2]
                    .name
                    .eq_ignore_ascii_case("dl")
            {
                msgs.push(VNuMessage::error(
                    "Element “dl” not allowed as child of “div” in this context.",
                    tag.line,
                    tag.col,
                ));
            }
            self.dl_stack.push(DlState::default());
            return msgs;
        }

        // 3. Track dt text
        if el == "dt" {
            self.in_dt = true;
            self.current_dt_text.clear();
            self.dt_line = tag.line;
            self.dt_col = tag.col;
        }

        if el == "template"
            && ctx
                .open_elements
                .last()
                .map(|e| e.name.eq_ignore_ascii_case("dl"))
                .unwrap_or(false)
        {
            if let Some(dl) = self.dl_stack.last_mut() {
                dl.seen_template = true;
            }
        }

        if ctx.is_descendant_of("template") {
            return msgs;
        }

        // 4. Direct child of dl
        let is_direct_child_of_dl = ctx
            .open_elements
            .last()
            .map(|e| e.name.eq_ignore_ascii_case("dl"))
            .unwrap_or(false);

        if is_direct_child_of_dl {
            if let Some(dl) = self.dl_stack.last_mut() {
                if el == "div" {
                    if matches!(dl.mode, DlMode::DtDd(_)) {
                        msgs.push(VNuMessage::error(
                            "Element “div” not allowed as child of “dl” in this context.",
                            tag.line,
                            tag.col,
                        ));
                    } else {
                        dl.mode = DlMode::DivOnly;
                        dl.div_state = Some(DivState::default());
                    }
                } else if el == "dt" {
                    if dl.mode == DlMode::DivOnly {
                        msgs.push(VNuMessage::error(
                            "Element “dt” not allowed as child of “dl” in this context.",
                            tag.line,
                            tag.col,
                        ));
                    } else {
                        dl.mode = DlMode::DtDd(true);
                    }
                } else if el == "dd" {
                    if dl.mode == DlMode::DivOnly {
                        msgs.push(VNuMessage::error(
                            "Element “dd” not allowed as child of “dl” in this context.",
                            tag.line,
                            tag.col,
                        ));
                    } else if dl.mode == DlMode::Undecided {
                        msgs.push(VNuMessage::error(
                            "Element “dl” is missing a required child element.",
                            tag.line,
                            tag.col,
                        ));
                        dl.mode = DlMode::DtDd(false);
                    } else {
                        dl.mode = DlMode::DtDd(false);
                    }
                }
            }
        }

        // 5. Child of div in dl
        let is_child_of_div_in_dl = ctx.open_elements.len() >= 2
            && ctx.open_elements[ctx.open_elements.len() - 1]
                .name
                .eq_ignore_ascii_case("div")
            && ctx.open_elements[ctx.open_elements.len() - 2]
                .name
                .eq_ignore_ascii_case("dl");

        if is_child_of_div_in_dl {
            if let Some(dl) = self.dl_stack.last_mut() {
                if let Some(div) = dl.div_state.as_mut() {
                    if el == "div" {
                        msgs.push(VNuMessage::error(
                            "Element “div” not allowed as child of “div” in this context.",
                            tag.line,
                            tag.col,
                        ));
                    } else if el == "dl" {
                        msgs.push(VNuMessage::error(
                            "Element “dl” not allowed as child of “div” in this context.",
                            tag.line,
                            tag.col,
                        ));
                    } else if !matches!(el.as_str(), "dt" | "dd" | "script" | "template") {
                        msgs.push(VNuMessage::error(
                            format!(
                                "Element “{}” not allowed as child of “div” in this context.",
                                el
                            ),
                            tag.line,
                            tag.col,
                        ));
                    } else if el == "dt" {
                        if div.after_dd {
                            msgs.push(VNuMessage::error(
                                "Element “dt” not allowed as child of “div” in this context.",
                                tag.line,
                                tag.col,
                            ));
                        }
                        div.has_dt = true;
                    } else if el == "dd" {
                        div.has_dd = true;
                        div.after_dd = true;
                    }
                }
            }
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
        if self.in_dt {
            self.current_dt_text.push_str(text);
        }

        if !text.trim().is_empty() {
            if let Some(top) = ctx.open_elements.last() {
                if top.name.eq_ignore_ascii_case("dl") {
                    msgs.push(VNuMessage::error(
                        "Text not allowed in “dl” in this context.",
                        line,
                        col,
                    ));
                } else if top.name.eq_ignore_ascii_case("div")
                    && ctx.open_elements.len() >= 2
                    && ctx.open_elements[ctx.open_elements.len() - 2]
                        .name
                        .eq_ignore_ascii_case("dl")
                {
                    msgs.push(VNuMessage::error(
                        "Text not allowed in “div” in this context.",
                        line,
                        col,
                    ));
                }
            }
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

        if name.eq_ignore_ascii_case("dt") {
            self.in_dt = false;
            let dt_name = self.current_dt_text.trim();
            if !dt_name.is_empty() {
                if let Some(dl) = self.dl_stack.last_mut() {
                    if !dl.seen_dt_names.insert(dt_name.to_string()) {
                        msgs.push(VNuMessage::warning(
                            format!(
                                "Duplicate “dt” name “{}” in “dl” element. Within a single “dl” element, there should not be more than one “dt” element for each name.",
                                dt_name
                            ),
                            self.dt_line,
                            self.dt_col,
                        ));
                    }
                }
            }
        } else if name.eq_ignore_ascii_case("div") {
            if let Some(dl) = self.dl_stack.last_mut() {
                if let Some(div) = dl.div_state.take() {
                    if !div.has_dd {
                        msgs.push(VNuMessage::error(
                            "Element “div” is missing a required instance of child element “dd”.",
                            line,
                            col,
                        ));
                    } else if !div.has_dt {
                        msgs.push(VNuMessage::error(
                            "Element “div” is missing a required instance of child element “dt”.",
                            line,
                            col,
                        ));
                    }
                }
            }
        } else if name.eq_ignore_ascii_case("dl") {
            if let Some(dl) = self.dl_stack.pop() {
                if let DlMode::DtDd(true) = dl.mode {
                    if dl.seen_template {
                        msgs.push(VNuMessage::error(
                            "Element “dl” is missing a required instance of one or more of the following child elements: “dd”.",
                            line,
                            col,
                        ));
                    } else {
                        msgs.push(VNuMessage::error(
                            "Element “dl” is missing a required instance of child element “dd”.",
                            line,
                            col,
                        ));
                    }
                }
            }
        }

        msgs
    }
}
