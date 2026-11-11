//! The tokens of the compact syntax (`CompactSyntaxTokenManager` and the escape
//! handling of `UCode_UCodeESC_CharStream`).

const KEYWORDS: &[&str] = &[
    "attribute",
    "default",
    "datatypes",
    "div",
    "element",
    "empty",
    "external",
    "grammar",
    "include",
    "inherit",
    "list",
    "mixed",
    "namespace",
    "notAllowed",
    "parent",
    "start",
    "string",
    "text",
    "token",
];

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Tok {
    /// An identifier; `true` when it is an unescaped keyword.
    Ident(String, bool),
    Prefixed(String, String),
    PrefixStar(String),
    Literal(String),
    Punct(&'static str),
    Eof,
}

/// Replaces `\x{N}` escapes, which the compact syntax allows anywhere.
pub(super) fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find('\\') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let xs = after.bytes().take_while(|&b| b == b'x').count();
        if xs > 0 && after[xs..].starts_with('{') {
            if let Some(close) = after[xs + 1..].find('}') {
                let hex = &after[xs + 1..xs + 1 + close];
                if let Some(c) = u32::from_str_radix(hex, 16).ok().and_then(char::from_u32) {
                    out.push(c);
                    rest = &after[xs + 1 + close + 1..];
                    continue;
                }
            }
        }
        out.push('\\');
        rest = after;
    }
    out.push_str(rest);
    out
}

fn is_name_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '\u{B7}')
}

pub(super) fn tokenize(text: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    let n = chars.len();
    let read_name = |i: &mut usize| -> String {
        let start = *i;
        while *i < n && is_name_char(chars[*i]) {
            *i += 1;
        }
        chars[start..*i].iter().collect()
    };
    while i < n {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '#' {
            while i < n && chars[i] != '\n' && chars[i] != '\r' {
                i += 1;
            }
            continue;
        }
        if c == '"' || c == '\'' {
            let triple = i + 2 < n && chars[i + 1] == c && chars[i + 2] == c;
            let mut s = String::new();
            if triple {
                i += 3;
                loop {
                    if i >= n {
                        return Err("unterminated literal".into());
                    }
                    if chars[i] == c && i + 2 < n && chars[i + 1] == c && chars[i + 2] == c {
                        i += 3;
                        break;
                    }
                    s.push(chars[i]);
                    i += 1;
                }
            } else {
                i += 1;
                // A newline can only come from a `\x{A}` escape here.
                while i < n && chars[i] != c {
                    s.push(chars[i]);
                    i += 1;
                }
                if i >= n {
                    return Err("unterminated literal".into());
                }
                i += 1;
            }
            tokens.push(Tok::Literal(s));
            continue;
        }
        if c == '\\' && i + 1 < n && is_name_start(chars[i + 1]) {
            i += 1;
            let name = read_name(&mut i);
            tokens.push(Tok::Ident(name, false));
            continue;
        }
        if is_name_start(c) {
            let name = read_name(&mut i);
            if i < n && chars[i] == ':' {
                if i + 1 < n && chars[i + 1] == '*' {
                    i += 2;
                    tokens.push(Tok::PrefixStar(name));
                    continue;
                }
                if i + 1 < n && is_name_start(chars[i + 1]) {
                    i += 1;
                    let local = read_name(&mut i);
                    tokens.push(Tok::Prefixed(name, local));
                    continue;
                }
            }
            let keyword = KEYWORDS.contains(&name.as_str());
            tokens.push(Tok::Ident(name, keyword));
            continue;
        }
        let two: String = chars[i..(i + 2).min(n)].iter().collect();
        let punct: &'static str = match two.as_str() {
            "|=" => "|=",
            "&=" => "&=",
            ">>" => ">>",
            _ => match c {
                '=' => "=",
                '{' => "{",
                '}' => "}",
                '(' => "(",
                ')' => ")",
                '[' => "[",
                ']' => "]",
                ',' => ",",
                '|' => "|",
                '&' => "&",
                '?' => "?",
                '*' => "*",
                '+' => "+",
                '-' => "-",
                '~' => "~",
                _ => return Err(format!("unexpected character “{c}”")),
            },
        };
        i += punct.len();
        tokens.push(Tok::Punct(punct));
    }
    tokens.push(Tok::Eof);
    Ok(tokens)
}
