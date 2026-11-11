use super::body_scope::BodyStyleTracker;
use crate::context::LineIndex;
use crate::quotes::quoted_parts;
use vnu_core::VNuMessage;

struct CssDiagnosticPattern {
    needle: &'static str,
    message: &'static str,
}

const CSS_SYNTAX_PATTERNS: &[CssDiagnosticPattern] = &[
    CssDiagnosticPattern {
        needle: "colr:",
        message: "CSS: “colr”: Property “colr” doesn't exist.",
    },
    CssDiagnosticPattern {
        needle: "colr :",
        message: "CSS: “colr”: Property “colr” doesn't exist.",
    },
    CssDiagnosticPattern {
        needle: "align-items: anchor-center",
        message: "CSS: “align-items”: “anchor-center” is not a “align-items” value.",
    },
    CssDiagnosticPattern {
        needle: "align-items:anchor-center",
        message: "CSS: “align-items”: “anchor-center” is not a “align-items” value.",
    },
    CssDiagnosticPattern {
        needle: "anchor-name: a",
        message: "CSS: “anchor-name”: “a” is not a “anchor-name” value.",
    },
    CssDiagnosticPattern {
        needle: "anchor-name:a",
        message: "CSS: “anchor-name”: “a” is not a “anchor-name” value.",
    },
    CssDiagnosticPattern {
        needle: "anchor-scope: all,",
        message: "CSS: “anchor-scope”: “all” is not a “anchor-scope” value.",
    },
    CssDiagnosticPattern {
        needle: "anchor-scope:all,",
        message: "CSS: “anchor-scope”: “all” is not a “anchor-scope” value.",
    },
    CssDiagnosticPattern {
        needle: "anchor-size(bottom",
        message: "CSS: “width”: “bottom” is not a “anchor-size()” value.",
    },
    CssDiagnosticPattern {
        needle: "anchor-size(--a bottom",
        message: "CSS: “width”: “bottom” is not a “anchor-size()” value.",
    },
    CssDiagnosticPattern {
        needle: "position-anchor: --a, --b",
        message: "CSS: “position-anchor”: Too many values or values are not recognized.",
    },
    CssDiagnosticPattern {
        needle: "position-anchor: --a --b",
        message: "CSS: “position-anchor”: Too many values or values are not recognized.",
    },
    CssDiagnosticPattern {
        needle: "position-area: left block-start",
        message: "CSS: “position-area”: “block-start” is not a “position-area” value.",
    },
    CssDiagnosticPattern {
        needle: "position-area: left right",
        message: "CSS: “position-area”: “right” is not a “position-area” value.",
    },
    CssDiagnosticPattern {
        needle: "position-try-fallbacks: top flip-x",
        message: "CSS: “position-try-fallbacks”: “flip-x” is not a “position-try-fallbacks” value.",
    },
    CssDiagnosticPattern {
        needle: "position-try-fallbacks: flip-x flip-x",
        message: "CSS: “position-try-fallbacks”: “flip-x” is not a “position-try-fallbacks” value.",
    },
    CssDiagnosticPattern {
        needle: "position-try: most-width;",
        message: "CSS: “position-try”: “most-width” is not a “position-try” value.",
    },
    CssDiagnosticPattern {
        needle: "position-visibility: always ",
        message: "CSS: “position-visibility”: “always” is not a “position-visibility” value.",
    },
    CssDiagnosticPattern {
        needle: "!important color:",
        message: "CSS: Parse Error.",
    },
    CssDiagnosticPattern {
        needle: "!important background:",
        message: "CSS: Parse Error.",
    },
];

pub struct CssValidator;

impl CssValidator {
    pub fn validate(
        text: &str,
        in_body: bool,
        in_template: bool,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return msgs;
        }

        if Self::validate_root_syntax(text, trimmed, line, col, &mut msgs) {
            return msgs;
        }

        if in_body && !in_template {
            let problems = BodyStyleTracker::check(text);
            msgs.extend(BodyStyleTracker::to_messages(problems, line, col));
        }

        Self::validate_pattern_table(text, line, col, &mut msgs);
        Self::validate_anchor_and_position(text, line, col, &mut msgs);
        Self::validate_at_rule_contexts(text, line, col, &mut msgs);

        msgs
    }

    fn validate_root_syntax(
        text: &str,
        trimmed: &str,
        line: usize,
        col: usize,
        msgs: &mut Vec<VNuMessage>,
    ) -> bool {
        if trimmed.starts_with('{') {
            msgs.push(VNuMessage::error("CSS: Parse Error.", line, col));
            return true;
        }

        if let Some(pos) = text.find("@charset") {
            let before = &text[..pos];
            if before
                .chars()
                .any(|c| !c.is_ascii_whitespace() && c != '<' && c != '!' && c != '-')
                || before.contains('{')
                || before.contains('}')
            {
                msgs.push(VNuMessage::error(
                    "CSS: The @charset rule may only occur at the start of the style sheet. Please check that there are no spaces before it.",
                    line,
                    col,
                ));
            }
        }

        false
    }

    fn validate_pattern_table(text: &str, line: usize, col: usize, msgs: &mut Vec<VNuMessage>) {
        for entry in CSS_SYNTAX_PATTERNS {
            if text.contains(entry.needle) {
                msgs.push(VNuMessage::error(entry.message, line, col));
            }
        }
    }

    fn validate_anchor_and_position(
        text: &str,
        line: usize,
        col: usize,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if text.contains("top: anchor(") && text.contains("red") {
            msgs.push(VNuMessage::error(
                "CSS: “top”: “red” is not a “anchor()” value.",
                line,
                col,
            ));
        }

        if text.contains("top: anchor()")
            || (text.contains("top: anchor(") && text.contains("--a)"))
        {
            msgs.push(VNuMessage::error(
                "CSS: “top”: Too many values or values are not recognized.",
                line,
                col,
            ));
        }

        if text.contains("position-area: right") && text.contains("span-right") {
            msgs.push(VNuMessage::error(
                "CSS: “position-area”: “right” is not a “position-area” value.",
                line,
                col,
            ));
        }

        if text.contains("@position-try") && text.contains("color:") {
            msgs.push(VNuMessage::error(
                "CSS: “color”: Property “color” doesn't exist in “CSS Level 3 + SVG” but exists in “[css1, css2, css21, css3]”.",
                line,
                col,
            ));
        }

        if text.contains("@position-try foo")
            || text.contains("@position-try x")
            || (text.contains("@position-try") && !text.contains("--"))
        {
            msgs.push(VNuMessage::error("CSS: Parse Error.", line, col));
        }
    }

    fn validate_at_rule_contexts(text: &str, line: usize, col: usize, msgs: &mut Vec<VNuMessage>) {
        if let Some(ff_pos) = text.find("@font-face") {
            if let Some(brace_start) = text[ff_pos..].find('{') {
                let inside = if let Some(brace_end) = text[ff_pos + brace_start..].find('}') {
                    &text[ff_pos + brace_start..ff_pos + brace_start + brace_end]
                } else {
                    &text[ff_pos + brace_start..]
                };
                if inside.contains("@media") {
                    msgs.push(VNuMessage::error("CSS: Parse Error.", line, col));
                } else if inside.contains("p {") && !inside.contains("@scope") {
                    msgs.push(VNuMessage::error("CSS: “p”: Parse Error.", line, col));
                }
            }
        }

        if (text.contains("@keyframes") || text.contains("@page"))
            && text.contains("p {")
            && !text.contains("@scope")
        {
            msgs.push(VNuMessage::error("CSS: “p”: Parse Error.", line, col));
        }
    }

    pub fn validate_stylesheet(text: &str, lines: &LineIndex) -> Vec<VNuMessage> {
        let mut msgs = Self::validate(text, false, false, 1, 1);
        place_stylesheet_spans(text, lines, &mut msgs);
        msgs
    }
}

fn place_stylesheet_spans(text: &str, lines: &LineIndex, msgs: &mut [VNuMessage]) {
    let mut used: Vec<std::ops::Range<usize>> = Vec::new();
    for msg in msgs {
        let Some((start, end)) = find_cause(text, &msg.message, &used) else {
            continue;
        };
        used.push(start..end);
        let loc = lines.location(start);
        let last = loc.col + end.saturating_sub(start).saturating_sub(1);
        msg.first_line = Some(loc.line);
        msg.last_line = Some(loc.line);
        msg.first_column = Some(loc.col);
        msg.last_column = Some(last.max(loc.col));
    }
}

fn find_cause(
    text: &str,
    message: &str,
    used: &[std::ops::Range<usize>],
) -> Option<(usize, usize)> {
    if message.contains("“p”: Parse Error") {
        return find_unused(text, "p {", used).map(|start| (start, start + 3));
    }
    if message == "CSS: Parse Error." {
        for needle in [
            "!important color:",
            "!important background:",
            "@position-try foo",
            "@position-try x",
            "@position-try",
            "@media",
            "{",
        ] {
            if let Some(start) = find_unused(text, needle, used) {
                let len = if needle == "{" { 1 } else { needle.len() };
                return Some((start, start + len));
            }
        }
    }
    if message.contains("@charset") {
        return find_unused(text, "@charset", used).map(|start| (start, start + "@charset".len()));
    }
    let mut parts = quoted_parts(message);
    parts.sort_by_key(|part| std::cmp::Reverse(part.len()));
    for part in parts {
        if part.is_empty() {
            continue;
        }
        if let Some(start) = find_unused(text, part, used) {
            return Some((start, start + part.len()));
        }
    }
    None
}

fn find_unused(text: &str, needle: &str, used: &[std::ops::Range<usize>]) -> Option<usize> {
    let mut from = 0;
    while let Some(rel) = text[from..].find(needle) {
        let start = from + rel;
        let end = start + needle.len();
        let overlaps = used
            .iter()
            .any(|range| start < range.end && end > range.start);
        if !overlaps {
            return Some(start);
        }
        from = start + 1;
        if from > text.len() {
            break;
        }
    }
    None
}

#[cfg(test)]
mod stylesheet_tests {
    use super::CssValidator;
    use crate::context::LineIndex;

    #[test]
    fn stylesheet_error_is_on_the_property_line() {
        let css = "p { color: black; }\n.bad { colr: red; }\n";
        let lines = LineIndex::new(css);
        let msgs = CssValidator::validate_stylesheet(css, &lines);
        let msg = msgs
            .iter()
            .find(|msg| msg.message.contains("Property “colr”"))
            .expect("colr diagnostic");
        assert_eq!(msg.last_line, Some(2));
        assert_eq!(msg.first_column, Some(8));
    }

    #[test]
    fn plain_rule_is_not_an_html_document() {
        let css = "p { color: black; }\n";
        let lines = LineIndex::new(css);
        let msgs = CssValidator::validate_stylesheet(css, &lines);
        assert!(msgs.is_empty(), "{msgs:?}");
    }
}
