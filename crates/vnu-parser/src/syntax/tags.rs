use vnu_core::VNuMessage;

pub fn handle_tag_open(bytes: &[u8], idx: usize, src: &str, msgs: &mut Vec<VNuMessage>) -> usize {
    if idx + 1 >= bytes.len() {
        return 1;
    }

    let next = bytes[idx + 1];

    if next == b'?' {
        if !src.trim_start().starts_with("<?xml")
            && memchr::memmem::find(src.as_bytes(), b"http://www.w3.org/1999/xhtml").is_none()
        {
            msgs.push(VNuMessage::error(
                "Saw “<?”. Probable cause: Attempt to use an XML processing instruction in HTML. (XML processing instructions are not supported in HTML.)".to_string(),
                1,
                1,
            ));
        }
        return 2;
    }

    if next == b'>' {
        msgs.push(VNuMessage::error(
            "Saw “<>”. Probable causes: Unescaped “<” (escape as “&lt;”) or mistyped start tag."
                .to_string(),
            1,
            1,
        ));
        return 2;
    }

    if next == b'/' {
        return handle_end_tag(bytes, idx, src, msgs);
    }

    let rest = &bytes[idx + 1..];
    if (rest.starts_with(b"image ") || rest.starts_with(b"image>") || rest.starts_with(b"image/"))
        && memchr::memmem::find(src.as_bytes(), b"<svg").is_none()
    {
        msgs.push(VNuMessage::error(
            "Saw a start tag “image”.".to_string(),
            1,
            1,
        ));
    }
    if rest.starts_with(b"frameset ")
        || rest.starts_with(b"frameset>")
        || rest.starts_with(b"frameset/")
    {
        msgs.push(VNuMessage::error(
            "The “frameset” element is obsolete. Use the “iframe” element and CSS instead, or use server-side includes.".to_string(),
            1,
            1,
        ));
    }

    if !next.is_ascii_alphabetic() && next != b'!' && !next.is_ascii_whitespace() {
        msgs.push(VNuMessage::error(
            format!("Bad character “{}” after “<”. Probable cause: Unescaped “<”. Try escaping it as “&lt;”.", next as char),
            1,
            1,
        ));
    }

    scan_start_tag(bytes, idx, src, msgs)
}

fn handle_end_tag(bytes: &[u8], idx: usize, src: &str, msgs: &mut Vec<VNuMessage>) -> usize {
    if idx + 2 >= bytes.len() {
        return 2;
    }

    if bytes[idx + 2] == b'>' {
        msgs.push(VNuMessage::error(
            "Saw “</>”. Probable causes: Unescaped “<” (escape as “&lt;”) or mistyped end tag."
                .to_string(),
            1,
            1,
        ));
        return 3;
    }

    if bytes[idx + 2] == b' ' || bytes[idx + 2] == b'\t' || bytes[idx + 2] == b'\n' {
        msgs.push(VNuMessage::error("Garbage after “</”.".to_string(), 1, 1));
    }

    let end_tag_close = bytes[idx + 2..].iter().position(|&b| b == b'>');
    if let Some(pos) = end_tag_close {
        let tag_slice = &bytes[idx + 2..idx + 2 + pos];
        if tag_slice.ends_with(b"/") {
            msgs.push(VNuMessage::error(
                "Stray “/” at the end of an end tag.".to_string(),
                1,
                1,
            ));
        }
        let tag_str = String::from_utf8_lossy(tag_slice);
        if tag_str.split_whitespace().count() > 1 {
            msgs.push(VNuMessage::error(
                "End tag had attributes.".to_string(),
                1,
                1,
            ));
        }
        if tag_str.trim().eq_ignore_ascii_case("br") {
            msgs.push(VNuMessage::error("End tag “br”.".to_string(), 1, 1));
        }
        if (tag_str.trim().eq_ignore_ascii_case("src")
            || tag_str.trim().eq_ignore_ascii_case("style/")
            || tag_str.trim().eq_ignore_ascii_case("style"))
            && src.contains("<script")
            && !src.contains("</script")
        {
            msgs.push(VNuMessage::error(
                "End of file seen when expecting text or an end tag.".to_string(),
                1,
                1,
            ));
        }
        2 + pos + 1
    } else {
        2
    }
}

fn scan_start_tag(bytes: &[u8], idx: usize, src: &str, msgs: &mut Vec<VNuMessage>) -> usize {
    let mut j = idx + 1;
    let len = bytes.len();

    while j < len && bytes[j] != b'>' {
        let b = bytes[j];

        if b == b'/' && j + 1 < len && bytes[j + 1] == b' ' {
            msgs.push(VNuMessage::error(
                "A slash was not immediately followed by “>”.".to_string(),
                1,
                1,
            ));
        }

        if b == b'=' {
            if j > 0 && (bytes[j - 1] == b' ' || bytes[j - 1] == b'\t') {
                msgs.push(VNuMessage::error("Saw “=” when expecting an attribute name. Probable cause: Attribute name missing.".to_string(), 1, 1));
            }
            let next_j = check_equals_sign(bytes, j, src, msgs);
            j = next_j;
            continue;
        }

        if b == b'<' {
            if j > 0 && bytes[j - 1].is_ascii_alphabetic() {
                msgs.push(VNuMessage::error(
                    "“<” in attribute name. Probable cause: “>” missing immediately before."
                        .to_string(),
                    1,
                    1,
                ));
            } else {
                msgs.push(VNuMessage::error("Saw “<” when expecting an attribute name. Probable cause: Missing “>” immediately before.".to_string(), 1, 1));
            }
        }

        if b == b'"' || b == b'\'' {
            msgs.push(VNuMessage::error("Quote “\"” in attribute name. Probable cause: Matching quote missing somewhere earlier.".to_string(), 1, 1));
        }

        j += 1;
    }

    if j < len && bytes[j] == b'>' {
        j - idx + 1
    } else {
        j - idx
    }
}

fn check_equals_sign(bytes: &[u8], eq_idx: usize, src: &str, msgs: &mut Vec<VNuMessage>) -> usize {
    let mut k = eq_idx + 1;
    while k < bytes.len()
        && (bytes[k] == b' ' || bytes[k] == b'\t' || bytes[k] == b'\n' || bytes[k] == b'\r')
    {
        k += 1;
    }
    if k >= bytes.len() {
        return k;
    }

    let val_first = bytes[k];
    if val_first == b'`' {
        msgs.push(VNuMessage::error(
            "“`” at the start of an unquoted attribute value. Probable cause: Using the wrong character as a quote.".to_string(),
            1,
            1,
        ));
        k + 1
    } else if val_first == b'=' {
        msgs.push(VNuMessage::error(
            "“=” at the start of an unquoted attribute value. Probable cause: Stray duplicate equals sign.".to_string(),
            1,
            1,
        ));
        k + 1
    } else if val_first == b'<' {
        msgs.push(VNuMessage::error(
            "“<” at the start of an unquoted attribute value. Probable cause: Missing “>” immediately before.".to_string(),
            1,
            1,
        ));
        k + 1
    } else if val_first == b'>' {
        msgs.push(VNuMessage::error(
            "Attribute value missing.".to_string(),
            1,
            1,
        ));
        k
    } else if val_first == b'"' || val_first == b'\'' {
        let quote = val_first;
        let mut close_idx = k + 1;
        while close_idx < bytes.len() && bytes[close_idx] != quote {
            if bytes[close_idx] == b'&' {
                super::charrefs::handle_charref(bytes, close_idx, src, msgs);
            }
            close_idx += 1;
        }
        if close_idx < bytes.len() {
            if close_idx + 1 < bytes.len() && bytes[close_idx + 1].is_ascii_alphabetic() {
                msgs.push(VNuMessage::error(
                    "No space between attributes.".to_string(),
                    1,
                    1,
                ));
            }
            close_idx + 1
        } else {
            bytes.len()
        }
    } else {
        let mut m = k;
        while m < bytes.len() && !bytes[m].is_ascii_whitespace() && bytes[m] != b'>' {
            let vb = bytes[m];
            if vb == b'&' {
                super::charrefs::handle_charref(bytes, m, src, msgs);
            } else if vb == b'`' {
                msgs.push(VNuMessage::error(
                    "“`” in an unquoted attribute value. Probable cause: Using the wrong character as a quote.".to_string(),
                    1,
                    1,
                ));
                break;
            } else if vb == b'<' {
                msgs.push(VNuMessage::error(
                    "“<” in an unquoted attribute value. Probable cause: Missing “>” immediately before.".to_string(),
                    1,
                    1,
                ));
                break;
            } else if vb == b'"' {
                msgs.push(VNuMessage::error(
                    "“\"” in an unquoted attribute value. Probable causes: Attributes running together or a URL query string in an unquoted attribute value.".to_string(),
                    1,
                    1,
                ));
                break;
            }
            m += 1;
        }
        m
    }
}
