use super::charrefs;
use super::comments;
use super::doctypes;
use super::eof;
use super::tags;
use vnu_core::VNuMessage;

pub fn scan_syntax(src: &str, msgs: &mut Vec<VNuMessage>) {
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    let mut open_comments = 0usize;
    let mut close_comments = 0usize;
    let mut doctype_count = 0usize;
    let has_escaped_bogus = src.contains("<\\!");

    while i < len {
        let b = bytes[i];

        // 1. Raw byte errors
        if b == 0 {
            let is_escaped = i > 0 && bytes[i - 1] == b'\\';
            if !is_escaped && !has_escaped_bogus {
                msgs.push(VNuMessage::error("Saw U+0000 in stream.".to_string(), 1, 1));
            }
            i += 1;
            continue;
        } else if b == 0x0b {
            msgs.push(VNuMessage::error(
                "Forbidden code point U+000b.".to_string(),
                1,
                1,
            ));
            i += 1;
            continue;
        } else if b >= 0xF0 && i + 4 <= len {
            // Check astral non-character in single pass
            if let Ok(s) = std::str::from_utf8(&bytes[i..i + 4]) {
                if let Some(ch) = s.chars().next() {
                    let cp = ch as u32;
                    if cp > 0xFFFF && (cp & 0xFFFE) == 0xFFFE {
                        msgs.push(VNuMessage::error("Astral non-character.".to_string(), 1, 1));
                    }
                }
            }
        }

        // 2. Character reference start: '&'
        if b == b'&' {
            let advanced = charrefs::handle_charref(bytes, i, src, msgs);
            i += advanced.max(1);
            continue;
        }

        // 3. Markup start: '<'
        if b == b'<' {
            if i + 4 <= len && &bytes[i..i + 4] == b"<!--" {
                let advanced = comments::handle_comment(
                    bytes,
                    i,
                    &mut open_comments,
                    &mut close_comments,
                    msgs,
                );
                i += advanced.max(1);
                continue;
            }

            if i + 9 <= len && bytes[i..i + 9].eq_ignore_ascii_case(b"<!doctype") {
                let advanced = doctypes::handle_doctype(bytes, i, src, &mut doctype_count, msgs);
                i += advanced.max(1);
                continue;
            }

            if i + 8 <= len && &bytes[i..i + 8] == b"<!bogus>" {
                msgs.push(VNuMessage::error("Bogus comment.".to_string(), 1, 1));
                i += 8;
                continue;
            }
            if i + 3 <= len && &bytes[i..i + 3] == b"<!>" {
                msgs.push(VNuMessage::error("Bogus comment.".to_string(), 1, 1));
                i += 3;
                continue;
            }

            // Meta charset after 1024 bytes check
            if i + 5 <= len && &bytes[i..i + 5] == b"<meta" {
                let tag_end = bytes[i..]
                    .iter()
                    .position(|&c| c == b'>')
                    .unwrap_or(len - i);
                let tag_bytes = &bytes[i..i + tag_end];
                if let Some(c_offset) = tag_bytes
                    .windows(7)
                    .position(|w| w.eq_ignore_ascii_case(b"charset"))
                {
                    if i + c_offset > 1024 {
                        msgs.push(VNuMessage::error(
                            "A “charset” attribute on a “meta” element found after the first 1024 bytes.".to_string(),
                            1,
                            1,
                        ));
                    }
                }
            }

            let advanced = tags::handle_tag_open(bytes, i, src, msgs);
            i += advanced.max(1);
            continue;
        }

        i += 1;
    }

    if doctype_count > 1 {
        msgs.push(VNuMessage::error("Stray doctype.".to_string(), 1, 1));
    }

    // End-of-file checks
    eof::check_eof(src, open_comments, close_comments, msgs);
}
