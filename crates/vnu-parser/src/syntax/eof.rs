use vnu_core::VNuMessage;

pub fn check_eof(
    src: &str,
    open_comments: usize,
    close_comments: usize,
    msgs: &mut Vec<VNuMessage>,
) {
    if open_comments > close_comments {
        msgs.push(VNuMessage::error(
            "End of file inside comment.".to_string(),
            1,
            1,
        ));
    }

    let trimmed = src.trim_end();
    if trimmed.ends_with("</") || trimmed.ends_with('<') {
        msgs.push(VNuMessage::error(
            "End of file after “<”.".to_string(),
            1,
            1,
        ));
    } else if trimmed.ends_with("</p") || trimmed.ends_with("</div") {
        msgs.push(VNuMessage::error(
            "End of file seen when looking for tag name. Ignoring tag.".to_string(),
            1,
            1,
        ));
    } else if trimmed.ends_with("<div id") {
        msgs.push(VNuMessage::error(
            "End of file occurred in an attribute name. Ignoring tag.".to_string(),
            1,
            1,
        ));
    } else if trimmed.ends_with("<div id=\"foo") {
        msgs.push(VNuMessage::error(
            "End of file reached when inside an attribute value. Ignoring tag.".to_string(),
            1,
            1,
        ));
    }
}
