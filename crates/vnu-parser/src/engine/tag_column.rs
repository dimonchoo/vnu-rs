/// Locate a tag on one source line. html5ever reports the line, not the column.
pub fn find_tag(line: &str, from_byte: usize, name: &str, end_tag: bool) -> Option<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut i = from_byte.min(bytes.len());
    let name_bytes = name.as_bytes();
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        if end_tag {
            if j >= bytes.len() || bytes[j] != b'/' {
                i += 1;
                continue;
            }
            j += 1;
        } else if j < bytes.len() && (bytes[j] == b'/' || bytes[j] == b'!' || bytes[j] == b'?') {
            i += 1;
            continue;
        }
        if j + name_bytes.len() > bytes.len() {
            break;
        }
        if line[j..j + name_bytes.len()].eq_ignore_ascii_case(name) {
            let after = j + name_bytes.len();
            let boundary = after >= bytes.len()
                || !(bytes[after].is_ascii_alphanumeric()
                    || bytes[after] == b'-'
                    || bytes[after] == b':');
            if boundary {
                return Some((i + 1, after));
            }
        }
        i += 1;
    }
    None
}

pub fn find_marker(line: &str, from_byte: usize, marker: &str) -> Option<(usize, usize)> {
    let start = from_byte.min(line.len());
    line[start..].find(marker).map(|rel| {
        let at = start + rel;
        (at + 1, at + marker.len())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_tag_on_a_line_is_not_the_first() {
        let line = "<div></div><p>";
        let (col, next) = find_tag(line, 0, "div", false).expect("div");
        assert_eq!(col, 1);
        let (col, _) = find_tag(line, next, "p", false).expect("p");
        assert_eq!(col, 12);
    }

    #[test]
    fn end_tag_is_not_a_start_tag() {
        let line = "<p></p>";
        let (col, next) = find_tag(line, 0, "p", false).expect("start");
        assert_eq!(col, 1);
        let (col, _) = find_tag(line, next, "p", true).expect("end");
        assert_eq!(col, 4);
    }
}
