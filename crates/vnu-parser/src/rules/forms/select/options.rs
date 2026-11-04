use vnu_core::VNuMessage;
use crate::context::ElementContext;

pub fn check_option_start(
    tag: &ElementContext,
    msgs: &mut Vec<VNuMessage>,
) {
    if tag.has_attr("aria-selected") {
        msgs.push(VNuMessage::warning(
            "The “aria-selected” attribute should not be used on the “option” element.",
            tag.line,
            tag.col,
        ));
    }

    if let Some(label) = tag.get_attr("label") {
        if label.is_empty() {
            msgs.push(VNuMessage::error(
                "Bad value “” for attribute “label” on element “option”.",
                tag.line,
                tag.col,
            ));
        }
    }
}
