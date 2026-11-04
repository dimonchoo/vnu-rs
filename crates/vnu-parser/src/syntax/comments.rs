use vnu_core::VNuMessage;

pub fn handle_comment(
    bytes: &[u8],
    start: usize,
    open_comments: &mut usize,
    close_comments: &mut usize,
    msgs: &mut Vec<VNuMessage>,
) -> usize {
    *open_comments += 1;
    let after_open = &bytes[start + 4..];

    // Find end of comment
    let end_pos = after_open
        .windows(3)
        .position(|w| w == b"-->");

    if let Some(pos) = end_pos {
        *close_comments += 1;
        let comment_body = &after_open[..pos];
        if comment_body.windows(4).any(|w| w == b"<!--") {
            msgs.push(VNuMessage::error(
                "Saw “<!--” within a comment. Probable cause: Nested comment (not allowed).".to_string(),
                1,
                1,
            ));
        }
        if comment_body.windows(4).any(|w| w == b"--!>") || after_open.windows(4).any(|w| w == b"--!>") {
            msgs.push(VNuMessage::error(
                "Premature end of comment. Use “-->” to end a comment properly.".to_string(),
                1,
                1,
            ));
        }
        4 + pos + 3
    } else {
        // EOF inside comment
        if after_open.windows(4).any(|w| w == b"<!--") {
            msgs.push(VNuMessage::error(
                "Saw “<!--” within a comment. Probable cause: Nested comment (not allowed).".to_string(),
                1,
                1,
            ));
        }
        bytes.len() - start
    }
}
