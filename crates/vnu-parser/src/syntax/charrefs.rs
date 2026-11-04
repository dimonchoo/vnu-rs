use vnu_core::VNuMessage;

pub fn handle_charref(bytes: &[u8], idx: usize, _src: &str, msgs: &mut Vec<VNuMessage>) -> usize {
    if idx + 1 >= bytes.len() {
        return 1;
    }

    if bytes[idx + 1] == b'#' {
        let after_hash = &bytes[idx + 2..];
        if after_hash.is_empty() || after_hash[0] == b';' {
            msgs.push(VNuMessage::error("No digits after “&#”.".to_string(), 1, 1));
            return 2;
        }

        let is_hex = after_hash[0] == b'x' || after_hash[0] == b'X';
        if is_hex {
            if after_hash.len() > 1 && !after_hash[1].is_ascii_hexdigit() {
                msgs.push(VNuMessage::error("No digits after “”.".to_string(), 1, 1));
                return 3;
            }
            if after_hash.len() == 1 || after_hash[1] == b';' {
                msgs.push(VNuMessage::error("No digits after “&#x”.".to_string(), 1, 1));
                return 3;
            }
        }

        let mut end = idx + 2 + if is_hex { 1 } else { 0 };
        while end < bytes.len() && ((is_hex && bytes[end].is_ascii_hexdigit()) || (!is_hex && bytes[end].is_ascii_digit())) {
            end += 1;
        }

        let has_semi = end < bytes.len() && bytes[end] == b';';
        let num_bytes = &bytes[idx + 2 + if is_hex { 1 } else { 0 }..end];
        let num_str = std::str::from_utf8(num_bytes).unwrap_or("");
        let cp_opt = if is_hex {
            u32::from_str_radix(num_str, 16).ok()
        } else {
            num_str.parse::<u32>().ok()
        };

        if let Some(cp) = cp_opt {
            if !has_semi {
                msgs.push(VNuMessage::error("Character reference was not terminated by a semicolon.".to_string(), 1, 1));
            }
            validate_codepoint(cp, msgs);
        }

        (end - idx) + if has_semi { 1 } else { 0 }
    } else {
        if bytes[idx..].starts_with(b"&amp") && !bytes[idx..].starts_with(b"&amp;") {
            msgs.push(VNuMessage::error(
                "Named character reference was not terminated by a semicolon. (Or “&” should have been escaped as “&amp;”.)".to_string(),
                1,
                1,
            ));
        }
        1
    }
}

fn validate_codepoint(cp: u32, msgs: &mut Vec<VNuMessage>) {
    if cp == 0 {
        msgs.push(VNuMessage::error("Character reference expands to zero.".to_string(), 1, 1));
    } else if cp == 13 {
        msgs.push(VNuMessage::error("A numeric character reference expanded to carriage return.".to_string(), 1, 1));
    } else if (0x80..=0x9F).contains(&cp) {
        msgs.push(VNuMessage::error("A numeric character reference expanded to the C1 controls range.".to_string(), 1, 1));
    } else if (0xD800..=0xDFFF).contains(&cp) {
        msgs.push(VNuMessage::error("Character reference expands to a surrogate.".to_string(), 1, 1));
    } else if cp > 0x10FFFF {
        msgs.push(VNuMessage::error("Character reference outside the permissible Unicode range.".to_string(), 1, 1));
    } else if (0xFDD0..=0xFDEF).contains(&cp) {
        msgs.push(VNuMessage::error("Character reference expands to a permanently unassigned code point.".to_string(), 1, 1));
    } else if (cp & 0xFFFE) == 0xFFFE || (cp & 0xFFFF) == 0xFFFF {
        if cp > 0xFFFF {
            msgs.push(VNuMessage::error(format!("Character reference expands to an astral non-character (U+{:x}).", cp), 1, 1));
        } else {
            msgs.push(VNuMessage::error(format!("Character reference expands to a non-character (U+{:x}).", cp), 1, 1));
        }
    } else if (cp < 0x20 && cp != 9 && cp != 10 && cp != 12 && cp != 13) || cp == 0x7F {
        msgs.push(VNuMessage::error(format!("Character reference expands to a control character (U+{:04x}).", cp), 1, 1));
    }
}
