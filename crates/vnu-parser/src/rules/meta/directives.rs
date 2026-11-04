use crate::context::ElementContext;
use vnu_core::VNuMessage;

pub fn validate_meta_refresh(tag: &ElementContext, content: &str, msgs: &mut Vec<VNuMessage>) {
    if !is_valid_refresh(content) {
        msgs.push(VNuMessage::error(
            format!(
                "Bad value “{}” for attribute “content” on element “meta”.",
                content
            ),
            tag.line,
            tag.col,
        ));
    }
}

fn is_valid_refresh(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }

    if i == 0 {
        return false;
    }

    if i == bytes.len() {
        return true;
    }

    if bytes[i] != b';' {
        return false;
    }
    i += 1;

    // Must be followed by space
    if i >= bytes.len() || !bytes[i].is_ascii_whitespace() {
        return false;
    }
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    // Must have URL= or url=
    let remaining = &s[i..];
    let lower = remaining.to_ascii_lowercase();
    if !lower.starts_with("url=") {
        return false;
    }

    let url_part = &remaining[4..];
    if url_part.is_empty() || url_part.starts_with('"') || url_part.starts_with('\'') {
        return false;
    }
    if url_part.ends_with(' ')
        || url_part.ends_with('\t')
        || url_part.ends_with('\n')
        || url_part.ends_with('\r')
    {
        return false;
    }

    true
}

const KNOWN_CSP_DIRECTIVES: &[&str] = &[
    "child-src",
    "connect-src",
    "default-src",
    "font-src",
    "frame-src",
    "img-src",
    "manifest-src",
    "media-src",
    "object-src",
    "prefetch-src",
    "script-src",
    "script-src-elem",
    "script-src-attr",
    "style-src",
    "style-src-elem",
    "style-src-attr",
    "worker-src",
    "base-uri",
    "sandbox",
    "form-action",
    "frame-ancestors",
    "report-uri",
    "report-to",
    "upgrade-insecure-requests",
    "block-all-mixed-content",
    "require-trusted-types-for",
    "trusted-types",
    "navigate-to",
    "allow",
    "options",
    "referrer",
];

pub fn validate_meta_csp(tag: &ElementContext, content: &str, msgs: &mut Vec<VNuMessage>) {
    if content.is_empty() {
        return;
    }

    if !content.is_ascii() || !is_valid_csp(content) {
        msgs.push(VNuMessage::error(
            format!(
                "Bad value “{}” for attribute “content” on element “meta”.",
                content
            ),
            tag.line,
            tag.col,
        ));
    }
}

fn is_valid_csp(s: &str) -> bool {
    let policies = s.split(',');
    for policy in policies {
        let trimmed_policy = policy.trim();
        if trimmed_policy.is_empty() {
            continue;
        }

        let directives = trimmed_policy.split(';');
        for dir in directives {
            let trimmed = dir.trim();
            if trimmed.is_empty() {
                continue;
            }

            let mut tokens = trimmed.split_whitespace();
            let Some(dir_name) = tokens.next() else {
                continue;
            };

            let dir_lower = dir_name.to_ascii_lowercase();
            if !KNOWN_CSP_DIRECTIVES.contains(&dir_lower.as_str()) {
                return false;
            }

            for tok in tokens {
                if tok.starts_with('\'') && tok.ends_with('\'') && tok.len() >= 2 {
                    let inside = &tok[1..tok.len() - 1];
                    let inside_lower = inside.to_ascii_lowercase();
                    let is_valid_keyword = matches!(
                        inside_lower.as_str(),
                        "self"
                            | "unsafe-inline"
                            | "unsafe-eval"
                            | "none"
                            | "strict-dynamic"
                            | "unsafe-hashes"
                            | "report-sample"
                            | "wasm-unsafe-eval"
                            | "allow-duplicates"
                            | "script"
                    ) || inside_lower.starts_with("nonce-")
                        || inside_lower.starts_with("sha256-")
                        || inside_lower.starts_with("sha384-")
                        || inside_lower.starts_with("sha512-");

                    if !is_valid_keyword {
                        return false;
                    }
                }
            }
        }
    }

    true
}
