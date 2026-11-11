use vnu_core::VNuMessage;

struct Span {
    start: usize,
    end: usize,
    value: Option<(usize, usize)>,
}

pub fn apply_highlight(line: &str, extract: &str, message: &mut VNuMessage) {
    let already_ranged = message
        .first_column
        .zip(message.last_column)
        .is_some_and(|(first, last)| first != last);
    if already_ranged {
        return;
    }
    let Some((first, last)) = highlight_columns(line, &message.message) else {
        return;
    };
    message.first_column = Some(first);
    message.last_column = Some(last);
    if message.first_line.is_none() {
        message.first_line = message.last_line;
    }
    let visible_chars = extract.chars().count();
    if first >= 1 && last <= visible_chars {
        message.hilite_start = Some(first - 1);
        message.hilite_length = Some(last - first + 1);
    }
}

pub fn highlight_columns(line: &str, message: &str) -> Option<(usize, usize)> {
    let quoted = crate::quotes::quoted_parts(message);
    if message.contains("Bad value") {
        if let (Some(value), Some(attr)) = (quoted.first().copied(), quoted.get(1).copied()) {
            if let Some(span) = find_attribute(line, attr) {
                if !value.is_empty() {
                    if let Some((start, end)) = span.value {
                        if line.get(start..end).is_some_and(|text| text == value) {
                            return Some(columns(line, start, end));
                        }
                    }
                }
                return Some(columns(line, span.start, span.end));
            }
        }
    }

    if message.contains("attribute") {
        for name in &quoted {
            if let Some(span) = find_attribute(line, name) {
                return Some(columns(line, span.start, span.end));
            }
        }
    }

    for name in &quoted {
        if let Some((start, end)) = find_tag(line, name) {
            return Some(columns(line, start, end));
        }
    }
    find_tag_anywhere(line).map(|(start, end)| columns(line, start, end))
}

fn find_attribute(line: &str, name: &str) -> Option<Span> {
    if name.is_empty() {
        return None;
    }
    let bytes = line.as_bytes();
    let name_bytes = name.as_bytes();
    let mut index = 0;
    while index + name_bytes.len() <= bytes.len() {
        if bytes[index..index + name_bytes.len()].eq_ignore_ascii_case(name_bytes) {
            let before_ok = index > 0 && bytes[index - 1].is_ascii_whitespace();
            let after = index + name_bytes.len();
            let after_ok = after == bytes.len() || is_name_end(bytes[after]);
            if before_ok && after_ok {
                return Some(attribute_span(bytes, index, after));
            }
        }
        index += 1;
    }
    None
}

fn attribute_span(bytes: &[u8], start: usize, after_name: usize) -> Span {
    let mut cursor = after_name;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if cursor >= bytes.len() || bytes[cursor] != b'=' {
        return Span {
            start,
            end: after_name,
            value: None,
        };
    }
    cursor += 1;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if cursor < bytes.len() && (bytes[cursor] == b'"' || bytes[cursor] == b'\'') {
        let quote = bytes[cursor];
        let value_start = cursor + 1;
        let mut value_end = value_start;
        while value_end < bytes.len() && bytes[value_end] != quote {
            value_end += 1;
        }
        let end = if value_end < bytes.len() {
            value_end + 1
        } else {
            value_end
        };
        return Span {
            start,
            end,
            value: Some((value_start, value_end)),
        };
    }
    let value_start = cursor;
    while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() && bytes[cursor] != b'>' {
        cursor += 1;
    }
    Span {
        start,
        end: cursor,
        value: Some((value_start, cursor)),
    }
}

fn find_tag(line: &str, name: &str) -> Option<(usize, usize)> {
    if name.is_empty() || name.chars().any(|ch| ch.is_whitespace() || ch == '<') {
        return None;
    }
    let bytes = line.as_bytes();
    let name_bytes = name.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'<' {
            let mut cursor = index + 1;
            if cursor < bytes.len() && bytes[cursor] == b'/' {
                cursor += 1;
            }
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if bytes
                .get(cursor..cursor + name_bytes.len())
                .is_some_and(|found| found.eq_ignore_ascii_case(name_bytes))
            {
                let after = cursor + name_bytes.len();
                if after == bytes.len() || is_name_end(bytes[after]) {
                    return Some(tag_extent(bytes, index, after));
                }
            }
        }
        index += 1;
    }
    None
}

fn find_tag_anywhere(line: &str) -> Option<(usize, usize)> {
    let bytes = line.as_bytes();
    let start = bytes.iter().position(|byte| *byte == b'<')?;
    let mut cursor = start + 1;
    if cursor < bytes.len() && bytes[cursor] == b'/' {
        cursor += 1;
    }
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    let name_start = cursor;
    while cursor < bytes.len() && is_name_char(bytes[cursor]) {
        cursor += 1;
    }
    if cursor == name_start {
        return None;
    }
    Some(tag_extent(bytes, start, cursor))
}

fn tag_extent(bytes: &[u8], start: usize, after_name: usize) -> (usize, usize) {
    if let Some(relative) = bytes[after_name..].iter().position(|byte| *byte == b'>') {
        if relative <= 80 {
            return (start, after_name + relative + 1);
        }
    }
    (start, after_name)
}

fn columns(line: &str, start: usize, end: usize) -> (usize, usize) {
    let first = line[..start].chars().count() + 1;
    let last = line[..end].chars().count().max(first);
    (first, last)
}

fn is_name_end(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'=' | b'/' | b'>')
}

fn is_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_lang_value_is_the_highlighted_span() {
        let line = "<p lang=\"zzz\">text</p>";
        let message = "Bad value “zzz” for attribute “lang” on element “p”.";
        assert_eq!(highlight_columns(line, message), Some((10, 12)));
    }

    #[test]
    fn indented_bad_value_counts_leading_spaces() {
        let line = "    <p lang=\"zzz\">";
        let message = "Bad value “zzz” for attribute “lang” on element “p”.";
        assert_eq!(highlight_columns(line, message), Some((14, 16)));
    }

    #[test]
    fn missing_alt_highlights_the_img_tag() {
        let line = "<img src=\"x.png\">";
        let message =
            "An “img” element must have an “alt” attribute, except under certain conditions.";
        assert_eq!(highlight_columns(line, message), Some((1, 17)));
    }

    #[test]
    fn empty_bad_value_highlights_the_attribute() {
        let line = "<div itemtype=\"\" itemscope></div>";
        let message = "Bad value “” for attribute “itemtype” on element “div”.";
        assert_eq!(highlight_columns(line, message), Some((6, 16)));
    }

    #[test]
    fn start_tag_without_a_named_element_uses_the_tag_on_the_line() {
        let line = "<html>";
        let message = "Start tag seen without seeing a doctype first. Expected “<!DOCTYPE html>”.";
        assert_eq!(highlight_columns(line, message), Some((1, 6)));
    }
}
