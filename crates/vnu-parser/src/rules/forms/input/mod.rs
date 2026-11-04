pub mod allowed;
pub mod values;

use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use super::types::VALID_INPUT_TYPES;
use allowed::*;
use values::validate_input_values;

#[derive(Default)]
pub struct InputRule {
    pending_lists: Vec<(String, usize, usize)>,
    pending_forms: Vec<(String, usize, usize)>,
}

impl Rule for InputRule {
    fn name(&self) -> &'static str {
        "forms::input"
    }

    fn reset(&mut self) {
        self.pending_lists.clear();
        self.pending_forms.clear();
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        if !tag.name.eq_ignore_ascii_case("input") {
            return Vec::new();
        }
        let mut msgs = Vec::new();

        let effective_type = super::types::effective_input_type(tag);
        if let Some(t) = tag.get_attr("type") {
            let t_lower = t.to_ascii_lowercase();
            if !VALID_INPUT_TYPES.contains(&t_lower.as_str()) {
                msgs.push(VNuMessage::error(
                    format!("Bad value “{}” for attribute “type” on element “input”.", t),
                    tag.line,
                    tag.col,
                ));
            }
        }

        // 1. Element-level special rules
        if effective_type == "button" {
            let val = tag.get_attr("value");
            if val.is_none_or(|v| v.is_empty()) {
                msgs.push(VNuMessage::error(
                    "Element “input” with attribute “type” whose value is “button” must have non-empty attribute “value”.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        if effective_type == "image" && tag.get_attr("alt").is_none() {
            msgs.push(VNuMessage::error(
                "Element “input” is missing required attribute “alt”.",
                tag.line,
                tag.col,
            ));
        }

        if let Some(name) = tag.get_attr("name") {
            if name == "isindex" {
                msgs.push(VNuMessage::error(
                    "The value “isindex” for the “name” attribute of the “input” element is not allowed.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        if effective_type == "hidden" {
            if let Some(ac) = tag.get_attr("autocomplete") {
                let ac_trim = ac.trim().to_ascii_lowercase();
                if ac_trim == "on" || ac_trim == "off" {
                    msgs.push(VNuMessage::error(
                        "An “input” element with a “type” attribute whose value is “hidden” must not have an “autocomplete” attribute whose value is “on” or “off”.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }

        // 2. Attribute-level restrictions
        for (attr, _) in &tag.attrs {
            let attr_lower = attr.to_ascii_lowercase();

            // Layer 1 overrides
            if effective_type == "hidden"
                && matches!(
                    attr_lower.as_str(),
                    "list"
                        | "maxlength"
                        | "minlength"
                        | "pattern"
                        | "placeholder"
                        | "readonly"
                        | "required"
                        | "size"
                )
            {
                msgs.push(VNuMessage::error(
                    format!("Attribute “{}” not allowed on element “input” at this point.", attr),
                    tag.line,
                    tag.col,
                ));
                continue;
            }

            if matches!(effective_type, "number" | "range") && attr_lower == "size" {
                msgs.push(VNuMessage::error(
                    "Attribute “size” not allowed on element “input” at this point.",
                    tag.line,
                    tag.col,
                ));
                continue;
            }

            // Schematron INPUT_ATTRIBUTES
            if let Some(allowed) = get_input_attribute_allowed(&attr_lower) {
                if !allowed.contains(&effective_type) {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Attribute “{}” is only allowed when the input type is {}.",
                            attr_lower,
                            render_type_list(allowed)
                        ),
                        tag.line,
                        tag.col,
                    ));
                    continue;
                }
            }

            if is_global_or_common_attr(&attr_lower) {
                continue;
            }

            if !is_attr_allowed_for_type(&attr_lower, effective_type) {
                msgs.push(VNuMessage::error(
                    format!("Attribute “{}” not allowed on element “input” at this point.", attr),
                    tag.line,
                    tag.col,
                ));
            }
        }

        // Hidden input must not have aria-* attributes
        if effective_type == "hidden" {
            for (attr, _) in &tag.attrs {
                if attr.starts_with("aria-") {
                    msgs.push(VNuMessage::error(
                        "An “input” element with a “type” attribute whose value is “hidden” must not have any “aria-*” attributes.".to_string(),
                        tag.line,
                        tag.col,
                    ));
                    break;
                }
            }
        }

        // Checkbox with role="button" must have aria-pressed
        if effective_type == "checkbox" && tag.get_attr("role") == Some("button") && tag.get_attr("aria-pressed").is_none() {
            msgs.push(VNuMessage::error(
                "An “input” element with a “type” attribute whose value is “checkbox” and with a “role” attribute whose value is “button” must have an “aria-pressed” attribute.".to_string(),
                tag.line,
                tag.col,
            ));
        }

        // 3. Attribute values format validation
        validate_input_values(tag, effective_type, &mut msgs);

        // 4. Track list and form references
        if let Some(list_id) = tag.get_attr("list") {
            if is_attr_allowed_for_type("list", effective_type) {
                self.pending_lists.push((list_id.to_string(), tag.line, tag.col));
            }
        }
        if let Some(form_id) = tag.get_attr("form") {
            self.pending_forms.push((form_id.to_string(), tag.line, tag.col));
        }

        msgs
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        for (list_id, line, col) in &self.pending_lists {
            if ctx.id_to_tag.get(list_id).map(|s| s.as_str()) != Some("datalist") {
                msgs.push(VNuMessage::error(
                    "The “list” attribute of the “input” element must refer to a “datalist” element.".to_string(),
                    *line,
                    *col,
                ));
            }
        }
        for (form_id, line, col) in &self.pending_forms {
            if ctx.id_to_tag.get(form_id).map(|s| s.as_str()) != Some("form") {
                msgs.push(VNuMessage::error(
                    "The “form” attribute must refer to a form element.".to_string(),
                    *line,
                    *col,
                ));
            }
        }
        msgs
    }
}
