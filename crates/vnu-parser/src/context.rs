use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LineIndex {
    offsets: Vec<usize>,
    len: usize,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut idx = Self {
            offsets: Vec::with_capacity((source.len() / 40).max(16)),
            len: 0,
        };
        idx.reset(source);
        idx
    }

    #[inline]
    pub fn reset(&mut self, source: &str) {
        self.offsets.clear();
        self.offsets.push(0);
        for pos in memchr::memchr_iter(b'\n', source.as_bytes()) {
            self.offsets.push(pos + 1);
        }
        self.len = source.len();
    }

    pub fn line_offset(&self, line: usize) -> usize {
        if line == 0 {
            0
        } else if line <= self.offsets.len() {
            self.offsets[line - 1]
        } else {
            self.len
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
    /// Shared with the tree builder, which keeps the attributes of open elements.
    pub attrs: Rc<[(String, String)]>,
    pub line: usize,
    pub col: usize,
    pub byte_offset: usize,
}

impl ElementContext {
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case(name))
    }

    pub fn has_attr_val(&self, name: &str, val: &str) -> bool {
        self.get_attr(name)
            .map(|v| v.eq_ignore_ascii_case(val))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DocumentMode {
    #[default]
    Html,
    Svg,
}

#[derive(Debug, Default)]
pub struct ValidationContext<'a> {
    pub source: &'a str,
    pub mode: DocumentMode,
    pub open_elements: Vec<ElementContext>,
    pub doctype_seen: bool,
    pub html_seen: bool,
    pub head_seen: bool,
    pub title_seen: bool,
    pub body_seen: bool,
    pub is_xhtml: bool,
    /// True when the document goes through the HTML tree builder (not XHTML or SVG,
    /// which Java parses as XML).
    pub html_parser: bool,
    pub id_to_tag: std::collections::HashMap<String, String>,
    pub line_index: Option<&'a LineIndex>,
}

impl<'a> ValidationContext<'a> {
    pub fn new(source: &'a str) -> Self {
        let is_xhtml = source.trim_start().starts_with("<?xml")
            || memchr::memmem::find(source.as_bytes(), b"http://www.w3.org/1999/xhtml").is_some();
        Self {
            source,
            mode: DocumentMode::Html,
            open_elements: Vec::new(),
            doctype_seen: false,
            html_seen: false,
            head_seen: false,
            title_seen: false,
            body_seen: false,
            is_xhtml,
            html_parser: false,
            id_to_tag: std::collections::HashMap::new(),
            line_index: None,
        }
    }

    pub fn has_ancestor_role(&self, role_name: &str) -> bool {
        self.open_elements.iter().rev().any(|e| {
            e.get_attr("role")
                .map(|r| {
                    r.split_whitespace()
                        .any(|token| token.eq_ignore_ascii_case(role_name))
                })
                .unwrap_or(false)
        })
    }

    pub fn is_descendant_of(&self, tag_name: &str) -> bool {
        self.open_elements
            .iter()
            .any(|elem| elem.name.eq_ignore_ascii_case(tag_name))
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
