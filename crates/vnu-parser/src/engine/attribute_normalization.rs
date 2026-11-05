use std::collections::HashMap;

#[derive(Default)]
pub struct XhtmlAttributeNormalizer {
    casing_by_line: HashMap<usize, Vec<(String, String)>>,
}

impl XhtmlAttributeNormalizer {
    pub fn build(source: &str) -> Self {
        let mut casing_by_line = HashMap::new();
        if !source.contains("http://www.w3.org/1999/xhtml")
            && !source.trim_start().starts_with("<?xml")
        {
            return Self { casing_by_line };
        }

        for (line_idx, line) in source.lines().enumerate() {
            if !line.contains("data-") {
                continue;
            }
            let line_num = line_idx + 1;
            let mut entries = Vec::new();
            for word in line.split(|c: char| c.is_whitespace() || c == '<' || c == '>') {
                let name = word.split('=').next().unwrap_or("");
                if name.starts_with("data-") && name.chars().any(|c| c.is_ascii_uppercase()) {
                    entries.push((name.to_ascii_lowercase(), name.to_string()));
                }
            }
            if !entries.is_empty() {
                casing_by_line.insert(line_num, entries);
            }
        }

        Self { casing_by_line }
    }

    #[inline]
    pub fn normalize_attributes(&self, line: usize, attrs: &mut [(String, String)]) {
        if let Some(entries) = self.casing_by_line.get(&line) {
            for (attr_name, _) in attrs.iter_mut() {
                for (lower, original) in entries {
                    if attr_name == lower {
                        *attr_name = original.clone();
                    }
                }
            }
        }
    }
}
