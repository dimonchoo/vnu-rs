use crate::context::ElementContext;

pub const VALID_BUTTON_TYPES: &[&str] = &["submit", "reset", "button"];

pub const VALID_INPUT_TYPES: &[&str] = &[
    "button", "checkbox", "color", "date", "datetime-local", "email", "file", "hidden",
    "image", "month", "number", "password", "radio", "range", "reset", "search", "submit",
    "tel", "text", "time", "url", "week",
];

#[inline]
pub fn effective_input_type(tag: &ElementContext) -> &'static str {
    let raw = tag.get_attr("type").unwrap_or("text").trim();
    VALID_INPUT_TYPES
        .iter()
        .find(|&&vt| vt.eq_ignore_ascii_case(raw))
        .copied()
        .unwrap_or("text")
}
