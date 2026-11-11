pub mod customizable;
pub mod options;

use self::customizable::{check_button_in_select, check_selectedcontent};
use self::options::check_option_start;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use crate::rules::datatypes::tokens::is_valid_autocomplete;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct SelectState {
    pub is_required: bool,
    pub has_multiple: bool,
    pub size_greater_than_one: bool,
    pub selected_count: usize,
    pub option_count: usize,
    pub first_option_in_optgroup: bool,
    pub first_option_valid_placeholder: bool,
    pub first_option_line: usize,
    pub first_option_col: usize,
    pub current_option_text: String,
    pub current_option_has_empty_val: bool,
    pub current_option_has_label: bool,
    pub current_option_line: usize,
    pub current_option_col: usize,
    pub line: usize,
    pub col: usize,
}

#[derive(Default)]
pub struct SelectRule {
    select_stack: Vec<SelectState>,
    standalone_option_text: String,
    standalone_option_has_label: bool,
    standalone_option_line: usize,
    standalone_option_col: usize,
    in_option_depth: usize,
}

impl Rule for SelectRule {
    fn name(&self) -> &'static str {
        "select"
    }

    fn reset(&mut self) {
        self.select_stack.clear();
        self.standalone_option_text.clear();
        self.standalone_option_has_label = false;
        self.standalone_option_line = 0;
        self.standalone_option_col = 0;
        self.in_option_depth = 0;
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, _ctx: &ValidationContext) -> bool {
        self.in_option_depth > 0
    }

    fn on_characters(
        &mut self,
        _ctx: &ValidationContext,
        text: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        if let Some(state) = self.select_stack.last_mut() {
            state.current_option_text.push_str(text);
        } else {
            self.standalone_option_text.push_str(text);
        }
        vec![]
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "button" {
            check_button_in_select(ctx, tag, &mut msgs);
            return msgs;
        }

        if name_lower == "selectedcontent" {
            check_selectedcontent(ctx, tag, &mut msgs);
            return msgs;
        }

        if name_lower == "option" {
            self.in_option_depth += 1;
            check_option_start(tag, &mut msgs);

            let has_label = tag.get_attr("label").is_some_and(|l| !l.is_empty());
            let has_empty_val = tag.get_attr("value").is_some_and(|v| v.is_empty());
            let is_selected = tag.has_attr("selected");

            if let Some(state) = self.select_stack.last_mut() {
                state.option_count += 1;
                if is_selected {
                    state.selected_count += 1;
                }
                if state.option_count == 1 {
                    state.first_option_line = tag.line;
                    state.first_option_col = tag.col;
                    state.first_option_in_optgroup = ctx.is_descendant_of("optgroup");
                }
                state.current_option_text.clear();
                state.current_option_has_label = has_label;
                state.current_option_has_empty_val = has_empty_val;
                state.current_option_line = tag.line;
                state.current_option_col = tag.col;
            } else {
                self.standalone_option_text.clear();
                // An option child of datalist may be empty (HTML content model).
                self.standalone_option_has_label = has_label
                    || ctx
                        .parent()
                        .is_some_and(|p| p.name.eq_ignore_ascii_case("datalist"));
                self.standalone_option_line = tag.line;
                self.standalone_option_col = tag.col;
            }

            return msgs;
        }

        if name_lower == "select" {
            let has_multiple = tag.has_attr("multiple");
            let mut size_greater_than_one = false;

            if let Some(size) = tag.get_attr("size") {
                let trimmed = size.trim();
                if trimmed == "0" {
                    msgs.push(VNuMessage::error(
                        "Bad value “0” for attribute “size” on element “select”.",
                        tag.line,
                        tag.col,
                    ));
                } else if let Ok(n) = trimmed.parse::<usize>() {
                    if n > 1 {
                        size_greater_than_one = true;
                    }
                }
            }

            if let Some(ac) = tag.get_attr("autocomplete") {
                if !is_valid_autocomplete(ac) {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Bad value “{}” for attribute “autocomplete” on element “select”.",
                            ac
                        ),
                        tag.line,
                        tag.col,
                    ));
                }
                if ac.contains("webauthn") {
                    msgs.push(VNuMessage::error(
                        "The value of the “autocomplete” attribute for the “select” element must not contain “webauthn”.",
                        tag.line,
                        tag.col,
                    ));
                }
            }

            if let Some(role) = tag.get_attr("role") {
                if role.eq_ignore_ascii_case("combobox") && !has_multiple && !size_greater_than_one
                {
                    msgs.push(VNuMessage::warning(
                        "The “combobox” role is unnecessary for element “select” without a “multiple” attribute and without a “size” attribute whose value is greater than 1.",
                        tag.line,
                        tag.col,
                    ));
                }
            }

            self.select_stack.push(SelectState {
                is_required: tag.has_attr("required"),
                has_multiple,
                size_greater_than_one,
                line: tag.line,
                col: tag.col,
                ..Default::default()
            });
        }

        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = name.to_ascii_lowercase();

        if name_lower == "option" {
            self.in_option_depth = self.in_option_depth.saturating_sub(1);
            if let Some(state) = self.select_stack.last_mut() {
                if !state.current_option_has_label && state.current_option_text.trim().is_empty() {
                    msgs.push(VNuMessage::error(
                        "Element “option” without attribute “label” must not be empty.",
                        state.current_option_line,
                        state.current_option_col,
                    ));
                }
                if state.option_count == 1 {
                    state.first_option_valid_placeholder = state.current_option_has_empty_val
                        || state.current_option_text.trim().is_empty();
                }
            } else if !self.standalone_option_has_label
                && self.standalone_option_text.trim().is_empty()
            {
                msgs.push(VNuMessage::error(
                    "Element “option” without attribute “label” must not be empty.",
                    self.standalone_option_line,
                    self.standalone_option_col,
                ));
            }
        } else if name_lower == "select" {
            if let Some(state) = self.select_stack.pop() {
                if !state.has_multiple && state.selected_count > 1 {
                    msgs.push(VNuMessage::error(
                        "The “select” element cannot have more than one selected “option” descendant unless the “multiple” attribute is specified.",
                        state.line,
                        state.col,
                    ));
                }

                if state.is_required && !state.has_multiple && !state.size_greater_than_one {
                    if state.option_count == 0 {
                        msgs.push(VNuMessage::error(
                            "A “select” element with a “required” attribute, and without a “multiple” attribute, and without a “size” attribute whose value is greater than “1”, must have a descendant “option” element.",
                            state.line,
                            state.col,
                        ));
                    } else if state.first_option_in_optgroup {
                        msgs.push(VNuMessage::error(
                            "The first “option” element in a “select” element with a “required” attribute, and without a “multiple” attribute, and without a “size” attribute whose value is greater than “1”, must not be a descendant of an “optgroup” element. Consider either adding an “option” element with an empty “value” attribute before the “optgroup” element, or adding a “size” attribute with a value equal to the number of “option” elements.",
                            state.first_option_line,
                            state.first_option_col,
                        ));
                    } else if !state.first_option_valid_placeholder {
                        msgs.push(VNuMessage::error(
                            "The first “option” element in a “select” element with a “required” attribute, and without a “multiple” attribute, and without a “size” attribute whose value is greater than “1”, must have either an empty “value” attribute, or must have no text content. Consider either adding a placeholder option label, or adding a “size” attribute with a value equal to the number of “option” elements.",
                            state.first_option_line,
                            state.first_option_col,
                        ));
                    }
                }
            }
        }

        msgs
    }
}
