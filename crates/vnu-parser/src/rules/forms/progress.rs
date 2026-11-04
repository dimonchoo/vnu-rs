use crate::context::ElementContext;
use vnu_core::VNuMessage;

pub fn validate_progress(tag: &ElementContext, msgs: &mut Vec<VNuMessage>) {
    let parse_double = |attr: &str| -> Option<f64> {
        tag.get_attr(attr)
            .and_then(|v| v.trim().parse::<f64>().ok())
    };

    let value = parse_double("value");
    let max = parse_double("max");

    if let (Some(v), Some(m)) = (value, max) {
        if v > m {
            msgs.push(VNuMessage::error(
                "The value of the  “value” attribute must be less than or equal to the value of the “max” attribute.",
                tag.line,
                tag.col,
            ));
        }
    } else if max.is_none() && value.is_some_and(|v| v > 1.0) {
        msgs.push(VNuMessage::error(
            "The value of the  “value” attribute must be less than or equal to one when the “max” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("aria-valuemax") {
        msgs.push(VNuMessage::warning(
            "The “aria-valuemax” attribute should not be used on a “progress” element.",
            tag.line,
            tag.col,
        ));
    }
}
