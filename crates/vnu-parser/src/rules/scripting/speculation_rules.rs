use vnu_core::VNuMessage;

pub fn validate_speculationrules(text: &str, line: usize, col: usize, msgs: &mut Vec<VNuMessage>) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        msgs.push(VNuMessage::error(
            "A “script” element with a “type” attribute whose value is “speculationrules” must have valid JSON content.",
            line,
            col,
        ));
        return;
    }

    let val: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => {
            msgs.push(VNuMessage::error(
                "A “script” element with a “type” attribute whose value is “speculationrules” must have valid JSON content.",
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
                "A “script” element with a “type” attribute whose value is “speculationrules” must contain a JSON object.",
                line,
                col,
            ));
            return;
        }
    };

    let has_prefetch = obj.contains_key("prefetch");
    let has_prerender = obj.contains_key("prerender");

    if !has_prefetch && !has_prerender {
        msgs.push(VNuMessage::error(
            "A “script” element with a “type” attribute whose value is “speculationrules” must contain a JSON object with at least one of the properties “prefetch” or “prerender”.",
            line,
            col,
        ));
        return;
    }

    for k in obj.keys() {
        if !matches!(k.as_str(), "prefetch" | "prerender") {
            msgs.push(VNuMessage::error(
                "A “script” element with a “type” attribute whose value is “speculationrules” must contain a JSON object with only “prefetch” and/or “prerender” as properties.",
                line,
                col,
            ));
            return;
        }
    }

    for prop_name in &["prefetch", "prerender"] {
        if let Some(prop_val) = obj.get(*prop_name) {
            let rules_arr = match prop_val.as_array() {
                Some(a) => a,
                None => {
                    msgs.push(VNuMessage::error(
                        format!(
                            "The “{}” property within the content of a “script” element with a “type” attribute whose value is “speculationrules” must be a JSON array.",
                            prop_name
                        ),
                        line,
                        col,
                    ));
                    continue;
                }
            };

            for item in rules_arr {
                let rule_obj = match item.as_object() {
                    Some(o) => o,
                    None => {
                        msgs.push(VNuMessage::error(
                            format!(
                                "Each item in the “{}” array within the content of a “script” element with a “type” attribute whose value is “speculationrules” must be a JSON object.",
                                prop_name
                            ),
                            line,
                            col,
                        ));
                        continue;
                    }
                };

                for rk in rule_obj.keys() {
                    if !matches!(rk.as_str(), "source" | "urls" | "where" | "eagerness") {
                        msgs.push(VNuMessage::error(
                            format!("Each rule in the “{}” array must only contain the properties “source”, “urls”, “where”, and “eagerness”.", prop_name),
                            line,
                            col,
                        ));
                        break;
                    }
                }

                if let Some(eagerness) = rule_obj.get("eagerness") {
                    match eagerness.as_str() {
                        Some(e) => {
                            if !matches!(e, "eager" | "moderate" | "conservative") {
                                msgs.push(VNuMessage::error(
                                    "The “eagerness” property in a speculation rule must be one of “eager”, “moderate”, or “conservative”.",
                                    line,
                                    col,
                                ));
                            }
                        }
                        None => {
                            msgs.push(VNuMessage::error(
                                "The “eagerness” property in a speculation rule must be a string.",
                                line,
                                col,
                            ));
                        }
                    }
                }

                let source = rule_obj.get("source");
                let has_urls = rule_obj.contains_key("urls");
                let has_where = rule_obj.contains_key("where");

                if let Some(src_val) = source {
                    match src_val.as_str() {
                        Some("list") => {
                            if has_where {
                                msgs.push(VNuMessage::error(
                                    "A speculation rule with “source” set to “list” must not have a “where” property.",
                                    line,
                                    col,
                                ));
                            }
                            if !has_urls {
                                msgs.push(VNuMessage::error(
                                    "A speculation rule with “source” set to “list” must have a “urls” property.",
                                    line,
                                    col,
                                ));
                            }
                        }
                        Some("document") => {
                            if has_urls {
                                msgs.push(VNuMessage::error(
                                    "A speculation rule with “source” set to “document” must not have a “urls” property.",
                                    line,
                                    col,
                                ));
                            }
                            if !has_where {
                                msgs.push(VNuMessage::error(
                                    "A speculation rule with “source” set to “document” must have a “where” property.",
                                    line,
                                    col,
                                ));
                            }
                        }
                        Some(_) => {
                            msgs.push(VNuMessage::error(
                                "The “source” property in a speculation rule must be either “list” or “document”.",
                                line,
                                col,
                            ));
                        }
                        None => {
                            msgs.push(VNuMessage::error(
                                "The “source” property in a speculation rule must be a string.",
                                line,
                                col,
                            ));
                        }
                    }
                } else if !has_urls && !has_where {
                    msgs.push(VNuMessage::error(
                        "A speculation rule must have a “source” property, or a “urls” property (for list rules), or a “where” property (for document rules).",
                        line,
                        col,
                    ));
                }

                if let Some(urls_val) = rule_obj.get("urls") {
                    match urls_val.as_array() {
                        Some(arr) => {
                            if arr.is_empty() {
                                msgs.push(VNuMessage::error(
                                    "The “urls” property in a speculation rule must contain at least one URL.",
                                    line,
                                    col,
                                ));
                            }
                            for u in arr {
                                match u.as_str() {
                                    Some("") => {
                                        msgs.push(VNuMessage::error(
                                            "Each URL in the “urls” array must be a non-empty string.",
                                            line,
                                            col,
                                        ));
                                    }
                                    Some(_) => {}
                                    None => {
                                        msgs.push(VNuMessage::error(
                                            "Each item in the “urls” array must be a string.",
                                            line,
                                            col,
                                        ));
                                    }
                                }
                            }
                        }
                        None => {
                            msgs.push(VNuMessage::error(
                                "The “urls” property in a speculation rule must be a JSON array.",
                                line,
                                col,
                            ));
                        }
                    }
                }

                if let Some(where_val) = rule_obj.get("where") {
                    match where_val.as_object() {
                        Some(w_obj) => {
                            validate_predicate(w_obj, line, col, msgs);
                        }
                        None => {
                            msgs.push(VNuMessage::error(
                                "The “where” property in a speculation rule must be a JSON object.",
                                line,
                                col,
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn validate_predicate(pred: &serde_json::Map<String, serde_json::Value>, line: usize, col: usize, msgs: &mut Vec<VNuMessage>) {
    let pred_keys: Vec<&str> = pred
        .keys()
        .filter(|k| matches!(k.as_str(), "and" | "or" | "not" | "href_matches" | "selector_matches"))
        .map(|k| k.as_str())
        .collect();

    if pred_keys.is_empty() {
        msgs.push(VNuMessage::error(
            "A document rule predicate must have one of the properties “and”, “or”, “not”, “href_matches”, or “selector_matches”.",
            line,
            col,
        ));
        return;
    }
    if pred_keys.len() > 1 {
        msgs.push(VNuMessage::error(
            "A document rule predicate must have only one of the properties “and”, “or”, “not”, “href_matches”, or “selector_matches”.",
            line,
            col,
        ));
        return;
    }

    let key = pred_keys[0];
    let val = &pred[key];

    match key {
        "and" | "or" => {
            match val.as_array() {
                Some(arr) => {
                    if arr.is_empty() {
                        msgs.push(VNuMessage::error(
                            format!("The “{}” property in a document rule must contain at least one item.", key),
                            line,
                            col,
                        ));
                    } else {
                        for item in arr {
                            if let Some(inner) = item.as_object() {
                                validate_predicate(inner, line, col, msgs);
                            } else {
                                msgs.push(VNuMessage::error(
                                    format!("Each item in the “{}” array in a document rule must be a JSON object.", key),
                                    line,
                                    col,
                                ));
                            }
                        }
                    }
                }
                None => {
                    msgs.push(VNuMessage::error(
                        format!("The “{}” property in a document rule must be a JSON array.", key),
                        line,
                        col,
                    ));
                }
            }
        }
        "not" => {
            if let Some(inner) = val.as_object() {
                validate_predicate(inner, line, col, msgs);
            } else {
                msgs.push(VNuMessage::error(
                    "The “not” property in a document rule must be a JSON object.",
                    line,
                    col,
                ));
            }
        }
        "href_matches" => {
            if let Some(s) = val.as_str() {
                if s.is_empty() {
                    msgs.push(VNuMessage::error(
                        "The “href_matches” property in a document rule must be a non-empty string.",
                        line,
                        col,
                    ));
                }
            } else if let Some(arr) = val.as_array() {
                if arr.is_empty() {
                    msgs.push(VNuMessage::error(
                        "The “href_matches” property in a document rule must contain at least one pattern.",
                        line,
                        col,
                    ));
                }
            } else {
                msgs.push(VNuMessage::error(
                    "The “href_matches” property in a document rule must be a string or an array of strings.",
                    line,
                    col,
                ));
            }
        }
        "selector_matches" => {
            if let Some(s) = val.as_str() {
                if s.is_empty() {
                    msgs.push(VNuMessage::error(
                        "The “selector_matches” property in a document rule must be a non-empty string.",
                        line,
                        col,
                    ));
                }
            } else if val.as_array().is_none() {
                msgs.push(VNuMessage::error(
                    "The “selector_matches” property in a document rule must be a string or an array of strings.",
                    line,
                    col,
                ));
            }
        }
        _ => {}
    }
}
