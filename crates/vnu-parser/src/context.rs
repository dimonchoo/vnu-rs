#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct LineIndex {
    offsets: Vec<usize>,
    len: usize,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut offsets = vec![0];
        for (idx, b) in source.bytes().enumerate() {
            if b == b'\n' {
                offsets.push(idx + 1);
            }
        }
        Self {
            offsets,
            len: source.len(),
        }
    }

    pub fn location(&self, byte_offset: usize) -> SourceLocation {
        let offset = byte_offset.min(self.len);
        let line_idx = match self.offsets.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.offsets[line_idx];
        let col = offset - line_start + 1;
        SourceLocation {
            line: line_idx + 1,
            col,
        }
    }

    pub fn line_snippet<'a>(&self, source: &'a str, line: usize) -> &'a str {
        if line == 0 || line > self.offsets.len() {
            return "";
        }
        let start = self.offsets[line - 1];
        let end = if line < self.offsets.len() {
            self.offsets[line].saturating_sub(1)
        } else {
            source.len()
        };
        &source[start..end]
    }
}

#[derive(Debug, Clone)]
pub struct ElementContext {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub line: usize,
    pub col: usize,
    pub byte_offset: usize,
}

impl ElementContext {
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Default)]
pub struct ValidationContext<'a> {
    pub source: &'a str,
    pub open_elements: Vec<ElementContext>,
    pub doctype_seen: bool,
    pub html_seen: bool,
    pub head_seen: bool,
    pub title_seen: bool,
    pub body_seen: bool,
}

impl<'a> ValidationContext<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            open_elements: Vec::new(),
            doctype_seen: false,
            html_seen: false,
            head_seen: false,
            title_seen: false,
            body_seen: false,
        }
    }

    pub fn is_descendant_of(&self, tag_name: &str) -> bool {
        self.open_elements.iter().any(|elem| elem.name.eq_ignore_ascii_case(tag_name))
    }

    pub fn parent(&self) -> Option<&ElementContext> {
        self.open_elements.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_index() {
        let src = "hello\nworld\nfoo";
        let idx = LineIndex::new(src);
        assert_eq!(idx.location(0), SourceLocation { line: 1, col: 1 });
        assert_eq!(idx.location(5), SourceLocation { line: 1, col: 6 });
        assert_eq!(idx.location(6), SourceLocation { line: 2, col: 1 });
        assert_eq!(idx.location(12), SourceLocation { line: 3, col: 1 });
        assert_eq!(idx.line_snippet(src, 1), "hello");
        assert_eq!(idx.line_snippet(src, 2), "world");
        assert_eq!(idx.line_snippet(src, 3), "foo");
    }
}
