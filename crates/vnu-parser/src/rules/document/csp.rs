use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct CspRule {
    csp_script_src_blocks_inline: bool,
    csp_style_src_blocks_inline: bool,
}

impl Rule for CspRule {
    fn name(&self) -> &'static str {
        "csp"
    }

    fn reset(&mut self) {
        self.csp_script_src_blocks_inline = false;
        self.csp_style_src_blocks_inline = false;
    }

    fn interested_in_start_tag(&self, tag: &ElementContext) -> bool {
        if tag.name.eq_ignore_ascii_case("meta") {
            return true;
        }
        if self.csp_script_src_blocks_inline
            && (tag.name.eq_ignore_ascii_case("script")
                || tag
                    .attrs
                    .iter()
                    .any(|(a, _)| a.starts_with("on") && a.len() > 2))
        {
            return true;
        }
        if self.csp_style_src_blocks_inline
            && (tag.name.eq_ignore_ascii_case("style")
                || tag
                    .attrs
                    .iter()
                    .any(|(a, _)| a.eq_ignore_ascii_case("style")))
        {
            return true;
        }
        false
    }

    fn on_start_tag(&mut self, _ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();

        if tag.name.eq_ignore_ascii_case("meta")
            && tag.has_attr_val("http-equiv", "Content-Security-Policy")
        {
            if let Some(content) = tag.get_attr("content") {
                for directive in content.split(';') {
                    let mut tokens = directive.split_whitespace();
                    let Some(dir_name) = tokens.next() else {
                        continue;
                    };
                    if dir_name.eq_ignore_ascii_case("script-src") {
                        let has_unsafe = tokens.any(|t| t.eq_ignore_ascii_case("'unsafe-inline'"));
                        if !has_unsafe {
                            self.csp_script_src_blocks_inline = true;
                        }
                    } else if dir_name.eq_ignore_ascii_case("style-src") {
                        let has_unsafe = tokens.any(|t| t.eq_ignore_ascii_case("'unsafe-inline'"));
                        if !has_unsafe {
                            self.csp_style_src_blocks_inline = true;
                        }
                    }
                }
            }
        }

        if self.csp_script_src_blocks_inline
            && tag.name.eq_ignore_ascii_case("script")
            && tag.get_attr("src").is_none()
        {
            msgs.push(VNuMessage::warning(
                "Inline script violates Content Security Policy (meta tag): blocked by “script-src” directive (missing “‘unsafe-inline’” or nonce/hash).",
                tag.line,
                tag.col,
            ));
        }

        if self.csp_style_src_blocks_inline && tag.name.eq_ignore_ascii_case("style") {
            msgs.push(VNuMessage::warning(
                "Inline style violates Content Security Policy (meta tag): blocked by “style-src” directive (missing “‘unsafe-inline’” or nonce/hash).",
                tag.line,
                tag.col,
            ));
        }

        for (attr_name, _) in tag.attrs.iter() {
            if self.csp_script_src_blocks_inline
                && attr_name.starts_with("on")
                && attr_name.len() > 2
            {
                msgs.push(VNuMessage::warning(
                    format!(
                        "Event handler attribute “{}” violates Content Security Policy (meta tag): blocked by “script-src” directive.",
                        attr_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
            if self.csp_style_src_blocks_inline && attr_name.eq_ignore_ascii_case("style") {
                msgs.push(VNuMessage::warning(
                    "The “style” attribute violates Content Security Policy (meta tag): blocked by “style-src” directive.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        msgs
    }
}
