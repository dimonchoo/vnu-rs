use vnu_core::VNuMessage;
use crate::rules::datatypes::tokens::{validate_url_syntax, UrlValidationResult};

fn is_valid_url(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    let is_url_prefix = trimmed.starts_with('/')
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
        || (trimmed.contains("://") && !trimmed.starts_with("://"))
        || trimmed.starts_with("data:")
        || trimmed.starts_with("blob:");

    is_url_prefix && matches!(validate_url_syntax(trimmed), UrlValidationResult::Valid)
}

pub fn validate_importmap(text: &str, line: usize, col: usize, msgs: &mut Vec<VNuMessage>) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        msgs.push(VNuMessage::error(
            "A script “script” with a “type” attribute whose value is “importmap” must have valid JSON content.",
            line,
            col,
        ));
        return;
    }

    let val: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => {
            msgs.push(VNuMessage::error(
                "A script “script” with a “type” attribute whose value is “importmap” must have valid JSON content.",
                line,
                col,
            ));
            return;
        }
    };

    let obj = match val.as_object() {
        Some(o) => o,
        None => {
            msgs.push(VNuMessage::error(
                "A “script” element with a “type” attribute whose value is “importmap” must contain a JSON object with no properties other than “imports”, “scopes”, and “integrity”.",
                line,
                col,
            ));
            return;
        }
    };

    for k in obj.keys() {
        if !matches!(k.as_str(), "imports" | "scopes" | "integrity") {
            msgs.push(VNuMessage::error(
                "A “script” element with a “type” attribute whose value is “importmap” must contain a JSON object with no properties other than “imports”, “scopes”, and “integrity”.",
                line,
                col,
            ));
            return;
        }
    }

    if let Some(imports) = obj.get("imports") {
        let imports_obj = match imports.as_object() {
            Some(o) => o,
            None => {
                msgs.push(VNuMessage::error(
                    "The value of the “imports” property within the content of a “script” element with a “type” attribute whose value is “importmap” must be a JSON object.",
                    line,
                    col,
                ));
                return;
            }
        };

        for (k, v) in imports_obj {
            if k.is_empty() {
                msgs.push(VNuMessage::error(
                    "A specifier map defined in a “imports” property within the content of a “script” element with a “type” attribute whose value is “importmap” must only contain non-empty keys.",
                    line,
                    col,
                ));
            }
            match v.as_str() {
                Some(s) => {
                    if k.ends_with('/') && !s.ends_with('/') {
                        msgs.push(VNuMessage::error(
                            "A specifier map defined in a “imports” property within the content of a “script” element with a “type” attribute whose value is “importmap” must have values that end with “/” when its corresponding key ends with “/”.",
                            line,
                            col,
                        ));
                    }
                }
                None => {
                    msgs.push(VNuMessage::error(
                        "A specifier map defined in a “imports” property within the content of a “script” element with a “type” attribute whose value is “importmap” must only contain string values.",
                        line,
                        col,
                    ));
                }
            }
        }
    }

    if let Some(scopes) = obj.get("scopes") {
        let scopes_obj = match scopes.as_object() {
            Some(o) => o,
            None => {
                msgs.push(VNuMessage::error(
                    "The value of the “scopes” property within the content of a “script” element with a “type” attribute whose value is “importmap” must be a JSON object whose values are also JSON objects.",
                    line,
                    col,
                ));
                return;
            }
        };

        for (k, v) in scopes_obj {
            if !is_valid_url(k) {
                msgs.push(VNuMessage::error(
                    "The value of the “scopes” property within the content of a “script” element with a “type” attribute whose value is “importmap” must be a JSON object whose keys are valid URL strings.",
                    line,
                    col,
                ));
            }
            let inner_obj = match v.as_object() {
                Some(o) => o,
                None => {
                    msgs.push(VNuMessage::error(
                        "The value of the “scopes” property within the content of a “script” element with a “type” attribute whose value is “importmap” must be a JSON object whose values are also JSON objects.",
                        line,
                        col,
                    ));
                    continue;
                }
            };
            for (_ik, iv) in inner_obj {
                match iv.as_str() {
                    Some(s) if is_valid_url(s) => {}
                    _ => {
                        msgs.push(VNuMessage::error(
                            "A specifier map defined in a “scopes” property within the content of a “script” element with a “type” attribute whose value is “importmap” must only contain valid URL values.",
                            line,
                            col,
                        ));
                    }
                }
            }
        }
    }
}
