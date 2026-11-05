pub fn quoted_parts(message: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = message;
    while let Some(start) = rest.find(['“', '"']) {
        let Some(open) = rest[start..].chars().next() else {
            break;
        };
        rest = &rest[start + open.len_utf8()..];
        let close = if open == '“' { '”' } else { '"' };
        let Some(end) = rest.find(close) else {
            break;
        };
        parts.push(&rest[..end]);
        rest = &rest[end + close.len_utf8()..];
    }
    parts
}
