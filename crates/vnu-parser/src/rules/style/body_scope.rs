use super::tokenizer::{Token, TokenType, Tokenizer};
use vnu_core::VNuMessage;

#[derive(Debug, PartialEq, Eq)]
pub enum ProblemKind {
    StyleRule,
    ScopeStart,
    AtRule,
}

#[derive(Debug)]
pub struct BodyStyleProblem {
    pub kind: ProblemKind,
    pub text: String,
    pub line_offset: usize,
    pub col_offset: usize,
}

const GROUPING_RULES: &[&str] = &[
    "media",
    "supports",
    "container",
    "starting-style",
    "layer",
    "when",
    "else",
    "navigation",
];

pub struct BodyStyleTracker;

impl BodyStyleTracker {
    pub fn check(css: &str) -> Vec<BodyStyleProblem> {
        let mut tokenizer = Tokenizer::new(css);
        tokenizer.run();
        let tokens = tokenizer.tokens;
        let mut problems = Vec::new();

        Self::check_block(css, &tokens, 0, tokens.len(), &mut problems);
        problems
    }

    fn check_block(
        css: &str,
        tokens: &[Token],
        from: usize,
        to: usize,
        problems: &mut Vec<BodyStyleProblem>,
    ) {
        let mut i = from;
        while i < to {
            let token = &tokens[i];
            let ttype = token.token_type;

            if matches!(
                ttype,
                TokenType::Whitespace
                    | TokenType::Semicolon
                    | TokenType::RightBrace
                    | TokenType::Cdo
                    | TokenType::Cdc
            ) {
                i += 1;
                continue;
            }

            let scan_start = if matches!(ttype, TokenType::AtKeyword) {
                i + 1
            } else {
                i
            };
            let stop = Self::scan(tokens, scan_start, to);
            let has_block = stop < to && tokens[stop].token_type == TokenType::LeftBrace;
            let next = if has_block {
                Self::find_close(tokens, stop, to) + 1
            } else {
                stop + 1
            };

            match ttype {
                TokenType::AtKeyword => {
                    let name = token.value.to_ascii_lowercase();
                    if name == "import" {
                        problems.push(Self::make_problem(
                            css,
                            ProblemKind::AtRule,
                            token.start,
                            token.end,
                        ));
                    } else if name == "scope" {
                        let mut open = i + 1;
                        while open < stop && tokens[open].token_type == TokenType::Whitespace {
                            open += 1;
                        }
                        if open < stop && tokens[open].token_type == TokenType::LeftParen {
                            let close =
                                Self::find_close(tokens, open, stop).min(stop.saturating_sub(1));
                            let end_pos = tokens[close].end;
                            problems.push(Self::make_problem(
                                css,
                                ProblemKind::ScopeStart,
                                tokens[open].start,
                                end_pos,
                            ));
                        }
                    } else if GROUPING_RULES.contains(&name.as_str()) && has_block {
                        let inner_end = next.saturating_sub(1);
                        if stop + 1 < inner_end {
                            Self::check_block(css, tokens, stop + 1, inner_end, problems);
                        }
                    }
                }
                _ => {
                    // Style rule
                    if has_block && stop > i {
                        let end_offset = Self::last_non_whitespace(tokens, i, stop);
                        problems.push(Self::make_problem(
                            css,
                            ProblemKind::StyleRule,
                            token.start,
                            end_offset,
                        ));
                    }
                }
            }

            i = next;
        }
    }

    fn scan(tokens: &[Token], from: usize, to: usize) -> usize {
        let mut depth: usize = 0;
        for (i, tok) in tokens.iter().enumerate().take(to).skip(from) {
            let ttype = tok.token_type;
            if depth == 0 && (ttype == TokenType::Semicolon || ttype == TokenType::LeftBrace) {
                return i;
            }
            match ttype {
                TokenType::LeftBracket | TokenType::LeftParen | TokenType::Function => {
                    depth += 1;
                }
                TokenType::RightBracket | TokenType::RightParen => {
                    depth = depth.saturating_sub(1);
                }
                TokenType::RightBrace if depth == 0 => {
                    return i;
                }
                _ => {}
            }
        }
        to
    }

    fn find_close(tokens: &[Token], open: usize, to: usize) -> usize {
        let mut depth: usize = 0;
        for (i, tok) in tokens.iter().enumerate().take(to).skip(open) {
            match tok.token_type {
                TokenType::LeftBrace
                | TokenType::LeftBracket
                | TokenType::LeftParen
                | TokenType::Function => {
                    depth += 1;
                }
                TokenType::RightBrace | TokenType::RightBracket | TokenType::RightParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        to.saturating_sub(1)
    }

    fn last_non_whitespace(tokens: &[Token], from: usize, to: usize) -> usize {
        let mut last = to.saturating_sub(1);
        while last > from && tokens[last].token_type == TokenType::Whitespace {
            last -= 1;
        }
        tokens[last].end
    }

    fn make_problem(css: &str, kind: ProblemKind, start: usize, end: usize) -> BodyStyleProblem {
        let safe_end = end.min(css.len());
        let safe_start = start.min(safe_end);
        let raw = &css[safe_start..safe_end];
        let text = raw.split_whitespace().collect::<Vec<_>>().join(" ");

        let mut line_offset = 0;
        let mut col_offset = 0;
        for (_i, c) in css[..safe_start].char_indices() {
            if c == '\n' {
                line_offset += 1;
                col_offset = 0;
            } else {
                col_offset += 1;
            }
        }

        BodyStyleProblem {
            kind,
            text,
            line_offset,
            col_offset,
        }
    }

    pub fn to_messages(
        problems: Vec<BodyStyleProblem>,
        style_line: usize,
        style_col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        for p in problems {
            let line = style_line + p.line_offset;
            let col = if p.line_offset == 0 {
                style_col + p.col_offset
            } else {
                p.col_offset + 1
            };

            let text = match p.kind {
                ProblemKind::StyleRule => format!(
                    "Style rule “{}” not allowed outside an “@scope” rule in a “style” element in “body”.",
                    p.text
                ),
                ProblemKind::ScopeStart => format!(
                    "Rule “@scope” with a scope start (“{}”) not allowed outside another “@scope” rule in a “style” element in “body”.",
                    p.text
                ),
                ProblemKind::AtRule => format!(
                    "Rule “{}” not allowed in a “style” element in “body”.",
                    p.text
                ),
            };

            msgs.push(VNuMessage::error(text, line, col));
        }
        msgs
    }
}
