#[derive(Copy, Clone, PartialEq, Eq)]
enum MimeState {
    AtStart,
    InSupertype,
    AtSubtypeStart,
    InSubtype,
    SemicolonSeen,
    WsBeforeSemicolon,
    InParamName,
    EqualsSeen,
    InQuotedString,
    InUnquotedString,
    InQuotedPair,
    CloseQuoteSeen,
}

#[inline]
fn is_token_char(c: u8) -> bool {
    matches!(c, 33..=126)
        && !matches!(
            c,
            b'(' | b')'
                | b'<'
                | b'>'
                | b'@'
                | b','
                | b';'
                | b':'
                | b'\\'
                | b'"'
                | b'/'
                | b'['
                | b']'
                | b'?'
                | b'='
                | b'{'
                | b'}'
        )
}

#[inline]
fn is_qdtext_char(c: u8) -> bool {
    matches!(c, 32..=126 | b'\t' | b'\r' | b'\n')
}

#[inline]
fn is_mime_whitespace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\r' | b'\n')
}

pub fn is_valid_mime_type(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }

    let mut state = MimeState::AtStart;

    for &b in bytes {
        match state {
            MimeState::AtStart => {
                if is_token_char(b) {
                    state = MimeState::InSupertype;
                } else {
                    return false;
                }
            }
            MimeState::InSupertype => {
                if is_token_char(b) {
                    continue;
                } else if b == b'/' {
                    state = MimeState::AtSubtypeStart;
                } else {
                    return false;
                }
            }
            MimeState::AtSubtypeStart => {
                if is_token_char(b) {
                    state = MimeState::InSubtype;
                } else {
                    return false;
                }
            }
            MimeState::InSubtype => {
                if is_token_char(b) {
                    continue;
                } else if b == b';' {
                    state = MimeState::SemicolonSeen;
                } else if is_mime_whitespace(b) {
                    state = MimeState::WsBeforeSemicolon;
                } else {
                    return false;
                }
            }
            MimeState::WsBeforeSemicolon => {
                if is_mime_whitespace(b) {
                    continue;
                } else if b == b';' {
                    state = MimeState::SemicolonSeen;
                } else {
                    return false;
                }
            }
            MimeState::SemicolonSeen => {
                if is_mime_whitespace(b) {
                    continue;
                } else if is_token_char(b) {
                    state = MimeState::InParamName;
                } else {
                    return false;
                }
            }
            MimeState::InParamName => {
                if is_token_char(b) {
                    continue;
                } else if b == b'=' {
                    state = MimeState::EqualsSeen;
                } else {
                    return false;
                }
            }
            MimeState::EqualsSeen => {
                if b == b'"' {
                    state = MimeState::InQuotedString;
                } else if is_token_char(b) {
                    state = MimeState::InUnquotedString;
                } else {
                    return false;
                }
            }
            MimeState::InQuotedString => {
                if b == b'\\' {
                    state = MimeState::InQuotedPair;
                } else if b == b'"' {
                    state = MimeState::CloseQuoteSeen;
                } else if is_qdtext_char(b) {
                    continue;
                } else {
                    return false;
                }
            }
            MimeState::InQuotedPair => {
                if b <= 127 {
                    state = MimeState::InQuotedString;
                } else {
                    return false;
                }
            }
            MimeState::CloseQuoteSeen => {
                if b == b';' {
                    state = MimeState::SemicolonSeen;
                } else if is_mime_whitespace(b) {
                    state = MimeState::WsBeforeSemicolon;
                } else {
                    return false;
                }
            }
            MimeState::InUnquotedString => {
                if is_token_char(b) {
                    continue;
                } else if b == b';' {
                    state = MimeState::SemicolonSeen;
                } else if is_mime_whitespace(b) {
                    state = MimeState::WsBeforeSemicolon;
                } else {
                    return false;
                }
            }
        }
    }

    matches!(
        state,
        MimeState::InSubtype | MimeState::InUnquotedString | MimeState::CloseQuoteSeen
    )
}
