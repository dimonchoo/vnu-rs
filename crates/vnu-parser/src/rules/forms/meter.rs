use crate::context::ElementContext;
use vnu_core::VNuMessage;

pub fn validate_meter(tag: &ElementContext, msgs: &mut Vec<VNuMessage>) {
    if !tag.has_attr("value") {
        msgs.push(VNuMessage::error(
            "Element “meter” is missing required attribute “value”.",
            tag.line,
            tag.col,
        ));
    }

    let parse_double = |attr: &str| -> Option<f64> {
        tag.get_attr(attr)
            .and_then(|v| v.trim().parse::<f64>().ok())
    };

    let value = parse_double("value");
    let min = parse_double("min");
    let max = parse_double("max");
    let optimum = parse_double("optimum");
    let low = parse_double("low");
    let high = parse_double("high");

    if let (Some(m), Some(v)) = (min, value) {
        if m > v {
            msgs.push(VNuMessage::error(
                "The value of the “min” attribute must be less than or equal to the value of the “value” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if min.is_none() && value.is_some_and(|v| v < 0.0) {
        msgs.push(VNuMessage::error(
            "The value of the “value” attribute must be greater than or equal to zero when the “min” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }
    if let (Some(v), Some(m)) = (value, max) {
        if v > m {
            msgs.push(VNuMessage::error(
                "The value of the “value” attribute must be less than or equal to the value of the “max” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if max.is_none() && value.is_some_and(|v| v > 1.0) {
        msgs.push(VNuMessage::error(
            "The value of the “value” attribute must be less than or equal to one when the “max” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }
    if let (Some(mi), Some(ma)) = (min, max) {
        if mi > ma {
            msgs.push(VNuMessage::error(
                "The value of the “min” attribute must be less than or equal to the value of the “max” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if min.is_none() && max.is_some_and(|m| m < 0.0) {
        msgs.push(VNuMessage::error(
            "The value of the “max” attribute must be greater than or equal to zero when the “min” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }
    if max.is_none() && min.is_some_and(|m| m > 1.0) {
        msgs.push(VNuMessage::error(
            "The value of the “min” attribute must be less than or equal to one when the “max” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }
    if let (Some(m), Some(l)) = (min, low) {
        if m > l {
            msgs.push(VNuMessage::error(
                "The value of the “min” attribute must be less than or equal to the value of the “low” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if min.is_none() && low.is_some_and(|l| l < 0.0) {
        msgs.push(VNuMessage::error(
            "The value of the “low” attribute must be greater than or equal to zero when the “min” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }
    if let (Some(m), Some(h)) = (min, high) {
        if m > h {
            msgs.push(VNuMessage::error(
                "The value of the “min” attribute must be less than or equal to the value of the “high” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if min.is_none() && high.is_some_and(|h| h < 0.0) {
        msgs.push(VNuMessage::error(
            "The value of the “high” attribute must be greater than or equal to zero when the “min” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }
    if let (Some(l), Some(h)) = (low, high) {
        if l > h {
            msgs.push(VNuMessage::error(
                "The value of the “low” attribute must be less than or equal to the value of the “high” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if let (Some(h), Some(m)) = (high, max) {
        if h > m {
            msgs.push(VNuMessage::error(
                "The value of the “high” attribute must be less than or equal to the value of the “max” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if max.is_none() && high.is_some_and(|h| h > 1.0) {
        msgs.push(VNuMessage::error(
            "The value of the “high” attribute must be less than or equal to one when the “max” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }
    if let (Some(l), Some(m)) = (low, max) {
        if l > m {
            msgs.push(VNuMessage::error(
                "The value of the “low” attribute must be less than or equal to the value of the “max” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if max.is_none() && low.is_some_and(|l| l > 1.0) {
        msgs.push(VNuMessage::error(
            "The value of the “low” attribute must be less than or equal to one when the “max” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }
    if let (Some(m), Some(o)) = (min, optimum) {
        if m > o {
            msgs.push(VNuMessage::error(
                "The value of the “min” attribute must be less than or equal to the value of the “optimum” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if min.is_none() && optimum.is_some_and(|o| o < 0.0) {
        msgs.push(VNuMessage::error(
            "The value of the “optimum” attribute must be greater than or equal to zero when the “min” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }
    if let (Some(o), Some(m)) = (optimum, max) {
        if o > m {
            msgs.push(VNuMessage::error(
                "The value of the “optimum” attribute must be less than or equal to the value of the “max” attribute.",
                tag.line,
                tag.col,
            ));
        }
    }
    if max.is_none() && optimum.is_some_and(|o| o > 1.0) {
        msgs.push(VNuMessage::error(
            "The value of the “optimum” attribute must be less than or equal to one when the “max” attribute is absent.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("aria-valuemax") {
        if tag.has_attr("max") {
            msgs.push(VNuMessage::error(
                "The “aria-valuemax” attribute must not be used on an element which has a “max” attribute.",
                tag.line,
                tag.col,
            ));
        } else {
            msgs.push(VNuMessage::warning(
                "The “aria-valuemax” attribute should not be used on a “meter” element.",
                tag.line,
                tag.col,
            ));
        }
    }

    if tag.has_attr("aria-valuemin") {
        if tag.has_attr("min") {
            msgs.push(VNuMessage::error(
                "The “aria-valuemin” attribute must not be used on an element which has a “min” attribute.",
                tag.line,
                tag.col,
            ));
        } else {
            msgs.push(VNuMessage::warning(
                "The “aria-valuemin” attribute should not be used on a “meter” element.",
                tag.line,
                tag.col,
            ));
        }
    }
}
