use vnu_core::VNuMessage;

pub fn handle_doctype(
    bytes: &[u8],
    idx: usize,
    src: &str,
    doctype_count: &mut usize,
    msgs: &mut Vec<VNuMessage>,
) -> usize {
    *doctype_count += 1;
    let rest = &src[idx + 9..];
    let mut in_quote = false;
    let mut end_dt_opt = None;

    for (i, b) in rest.bytes().enumerate() {
        if b == b'"' {
            in_quote = !in_quote;
        } else if b == b'>' && !in_quote {
            end_dt_opt = Some(i);
            break;
        }
    }

    if let Some(end_dt) = end_dt_opt {
        let dt_content = &rest[..end_dt];
        validate_doctype_content(dt_content, msgs);
        9 + end_dt + 1
    } else {
        if rest.contains("SYSTEM") {
            msgs.push(VNuMessage::error(
                "End of file inside system identifier.".to_string(),
                1,
                1,
            ));
        } else if rest.contains("PUBLIC") {
            msgs.push(VNuMessage::error(
                "End of file inside public identifier.".to_string(),
                1,
                1,
            ));
        } else {
            msgs.push(VNuMessage::error(
                "End of file inside doctype.".to_string(),
                1,
                1,
            ));
        }
        bytes.len() - idx
    }
}

fn validate_doctype_content(dt_content: &str, msgs: &mut Vec<VNuMessage>) {
    if dt_content.is_empty()
        || (dt_content.to_ascii_lowercase().starts_with("html") && !dt_content.starts_with(' '))
    {
        msgs.push(VNuMessage::error(
            "Missing space before doctype name.".to_string(),
            1,
            1,
        ));
    }
    if dt_content.contains("PUBLIC") && dt_content.contains("\"\"") {
        msgs.push(VNuMessage::error(
            "No space between the doctype public and system identifiers.".to_string(),
            1,
            1,
        ));
    }
    if dt_content.contains("PUBLIC\"") {
        msgs.push(VNuMessage::error(
            "No space between the doctype “PUBLIC” keyword and the quote.".to_string(),
            1,
            1,
        ));
    }
    if dt_content.contains("SYSTEM\"") {
        msgs.push(VNuMessage::error(
            "No space between the doctype “SYSTEM” keyword and the quote.".to_string(),
            1,
            1,
        ));
    }
    if dt_content.contains("BOGUS") {
        msgs.push(VNuMessage::error("Bogus doctype.".to_string(), 1, 1));
    }
    if dt_content.contains("PUBLIC") && dt_content.trim_end().ends_with("PUBLIC") {
        msgs.push(VNuMessage::error(
            "Expected a public identifier but the doctype ended.".to_string(),
            1,
            1,
        ));
    }
    if dt_content.contains("SYSTEM") && dt_content.trim_end().ends_with("SYSTEM") {
        msgs.push(VNuMessage::error(
            "Expected a system identifier but the doctype ended.".to_string(),
            1,
            1,
        ));
    }
    if dt_content.contains("PUBLIC") && dt_content.contains('>') {
        msgs.push(VNuMessage::error(
            "“>” in public identifier.".to_string(),
            1,
            1,
        ));
    }
    if dt_content.contains("SYSTEM") && dt_content.contains('>') {
        msgs.push(VNuMessage::error(
            "“>” in system identifier.".to_string(),
            1,
            1,
        ));
    }
}
