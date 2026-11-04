use crate::context::ElementContext;
use crate::rules::datatypes::tokens::is_valid_autocomplete;
use vnu_core::VNuMessage;

pub fn validate_textarea(tag: &ElementContext, msgs: &mut Vec<VNuMessage>) {
    if let Some(ac) = tag.get_attr("autocomplete") {
        if !is_valid_autocomplete(ac) {
            msgs.push(VNuMessage::error(
                format!(
                    "Bad value “{}” for attribute “autocomplete” on element “textarea”.",
                    ac
                ),
                tag.line,
                tag.col,
            ));
        }
    }
}
