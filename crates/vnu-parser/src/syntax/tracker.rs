use html5ever::tokenizer::Doctype;

#[derive(Default)]
pub struct SyntaxTracker {
    pub needs_scan: bool,
}

impl SyntaxTracker {
    #[inline]
    pub fn on_doctype(&mut self, d: &Doctype, doctype_seen: bool) {
        if self.needs_scan {
            return;
        }
        if doctype_seen
            || d.name.as_deref() != Some("html")
            || d.public_id.is_some()
            || d.system_id.is_some()
            || d.force_quirks
        {
            self.needs_scan = true;
        }
    }

    #[inline]
    pub fn on_start_tag(&mut self, name: &str, attrs: &[(String, String)], byte_offset: usize) {
        if self.needs_scan {
            return;
        }
        if name.eq_ignore_ascii_case("image") || name.eq_ignore_ascii_case("frameset") {
            self.needs_scan = true;
            return;
        }
        // data-* is normal HTML. It is not a reason to rescan the whole document.
        for (_, val) in attrs {
            let b = val.as_bytes();
            if b.starts_with(b"=") || b.contains(&b'`') || b.contains(&b'<') || b.contains(&b'"') {
                self.needs_scan = true;
                return;
            }
        }
        if name.eq_ignore_ascii_case("meta")
            && attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("charset"))
            && byte_offset > 1024
        {
            self.needs_scan = true;
        }
    }

    #[inline]
    pub fn on_end_tag(&mut self, name: &str, is_void: bool) {
        if self.needs_scan {
            return;
        }
        if name.eq_ignore_ascii_case("br") || is_void {
            self.needs_scan = true;
        }
    }

    #[inline]
    pub fn on_characters(&mut self, s: &str, in_script_or_style: bool) {
        if self.needs_scan {
            return;
        }
        if in_script_or_style && s.contains("</") {
            self.needs_scan = true;
            return;
        }
        if memchr::memchr2(0, 0x0b, s.as_bytes()).is_some() {
            self.needs_scan = true;
            return;
        }
        if !s.is_ascii()
            && s.chars().any(|c| {
                let u = c as u32;
                u > 0xFFFF && (u & 0xFFFE) == 0xFFFE
            })
        {
            self.needs_scan = true;
        }
    }

    #[inline]
    pub fn on_parse_error(&mut self) {
        self.needs_scan = true;
    }

    #[inline]
    pub fn check_raw_script_step(&mut self, is_script_step: bool, html: &str) {
        if !self.needs_scan
            && is_script_step
            && (html.contains("</src>") || html.contains("</style>"))
        {
            self.needs_scan = true;
        }
    }
}
