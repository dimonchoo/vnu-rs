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
            if i + 9 <= len && &bytes[i..i + 9] == b"<![CDATA[" {
                if let Some(end_cdata) = memchr::memmem::find(&bytes[i + 9..], b"]]>") {
                    i += 9 + end_cdata + 3;
                    continue;
                }
            }
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

            let advanced = tags::handle_tag_open(bytes, i, src, msgs).max(1);
            if let Some(name) = raw_text_start_tag(bytes, i) {
                // Script and style source is not markup. A "<" inside it is not a tag.
                i = raw_element_end(bytes, i + advanced, name);
                continue;
            }
            i += advanced;
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

const RAW_TEXT_ELEMENTS: [&str; 9] = [
    "script",
    "style",
    "iframe",
    "xmp",
    "noembed",
    "noframes",
    "textarea",
    "title",
    "plaintext",
];

/// Name of the raw text element whose start tag begins at `idx`, without allocating.
fn raw_text_start_tag(bytes: &[u8], idx: usize) -> Option<&'static str> {
    if bytes.get(idx) != Some(&b'<') {
        return None;
    }
    let start = idx + 1;
    let mut j = start;
    while j < bytes.len()
        && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-' || bytes[j] == b':')
    {
        j += 1;
    }
    let name = &bytes[start..j];
    if name.is_empty() || !name[0].is_ascii_alphabetic() {
        return None;
    }
    RAW_TEXT_ELEMENTS
        .into_iter()
        .find(|el| el.as_bytes().eq_ignore_ascii_case(name))
}

/// Index just after the matching end tag. Raw text runs to EOF when the end tag is missing.
fn raw_element_end(bytes: &[u8], content_at: usize, name: &str) -> usize {
    if name == "plaintext" || content_at >= bytes.len() {
        return bytes.len();
    }
    let name_bytes = name.as_bytes();
    let mut i = content_at;
    while i + 2 + name_bytes.len() <= bytes.len() {
        if bytes[i] == b'<'
            && bytes[i + 1] == b'/'
            && bytes[i + 2..i + 2 + name_bytes.len()].eq_ignore_ascii_case(name_bytes)
        {
            let after = i + 2 + name_bytes.len();
            let boundary = after >= bytes.len()
                || bytes[after].is_ascii_whitespace()
                || bytes[after] == b'>'
                || bytes[after] == b'/';
            if boundary {
                return bytes[after..]
                    .iter()
                    .position(|byte| *byte == b'>')
                    .map(|rel| after + rel + 1)
                    .unwrap_or(bytes.len());
            }
        }
        i += 1;
    }
    bytes.len()
}
