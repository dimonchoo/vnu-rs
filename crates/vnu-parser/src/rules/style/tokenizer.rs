#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Whitespace,
    Ident,
    Function,
    AtKeyword,
    Hash,
    StringLit,
    BadString,
    UrlLit,
    BadUrl,
    Delim(char),
    Number,
    Percentage,
    Dimension,
    Cdo,
    Cdc,
    Colon,
    Semicolon,
    Comma,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub start: usize,
    pub end: usize,
}

pub struct Tokenizer<'a> {
    bytes: &'a [u8],
    pos: usize,
    pub tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(css: &'a str) -> Self {
        Self {
            bytes: css.as_bytes(),
            pos: 0,
            tokens: Vec::new(),
        }
    }

    fn at(&self, offset: usize) -> char {
        self.bytes
            .get(offset)
            .copied()
            .map(|b| b as char)
            .unwrap_or('\0')
    }

    fn byte_pos(&self, offset: usize) -> usize {
        offset.min(self.bytes.len())
    }

    fn at_end(&self, offset: usize) -> bool {
        offset >= self.bytes.len()
    }

    fn is_ws(c: char) -> bool {
        c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\x0C'
    }

    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_name_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || c as u32 >= 0x80
    }

    fn is_name(c: char) -> bool {
        Self::is_name_start(c) || Self::is_digit(c) || c == '-'
    }

    fn is_valid_escape(&self, i: usize) -> bool {
        self.at(i) == '\\' && !self.at_end(i + 1) && self.at(i + 1) != '\n'
    }

    fn starts_ident(&self, i: usize) -> bool {
        let c = self.at(i);
        if c == '-' {
            return (!self.at_end(i + 1)
                && (Self::is_name_start(self.at(i + 1)) || self.at(i + 1) == '-'))
                || self.is_valid_escape(i + 1);
        }
        if Self::is_name_start(c) && !self.at_end(i) {
            return true;
        }
        self.is_valid_escape(i)
    }

    fn starts_number(&self, i: usize) -> bool {
        let c = self.at(i);
        if c == '+' || c == '-' {
            return Self::is_digit(self.at(i + 1))
                || (self.at(i + 1) == '.' && Self::is_digit(self.at(i + 2)));
        }
        if c == '.' {
            return Self::is_digit(self.at(i + 1));
        }
        Self::is_digit(c) && !self.at_end(i)
    }

    fn consume_name(&mut self) -> String {
        let mut s = String::new();
        while !self.at_end(self.pos) {
            let c = self.at(self.pos);
            if Self::is_name(c) {
                s.push(c);
                self.pos += 1;
            } else if self.is_valid_escape(self.pos) {
                self.pos += 1;
                if !self.at_end(self.pos) {
                    s.push(self.at(self.pos));
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
        s
    }

    fn add(&mut self, token_type: TokenType, value: String, start_idx: usize) {
        let start = self.byte_pos(start_idx);
        let end = self.byte_pos(self.pos);
        self.tokens.push(Token {
            token_type,
            value,
            start,
            end,
        });
    }

    pub fn run(&mut self) {
        while !self.at_end(self.pos) {
            let start = self.pos;
            let c = self.at(self.pos);

            // Skip comments /* ... */
            if c == '/' && self.at(self.pos + 1) == '*' {
                self.pos += 2;
                while !self.at_end(self.pos) {
                    if self.at(self.pos) == '*' && self.at(self.pos + 1) == '/' {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
                continue;
            }

            if Self::is_ws(c) {
                while !self.at_end(self.pos) && Self::is_ws(self.at(self.pos)) {
                    self.pos += 1;
                }
                self.add(TokenType::Whitespace, " ".to_string(), start);
            } else if c == '"' || c == '\'' {
                self.consume_string(c, start);
            } else if c == '#' {
                self.pos += 1;
                if self.starts_ident(self.pos) {
                    let name = self.consume_name();
                    self.add(TokenType::Hash, name, start);
                } else {
                    self.add(TokenType::Delim('#'), "#".to_string(), start);
                }
            } else if c == '(' {
                self.pos += 1;
                self.add(TokenType::LeftParen, "(".to_string(), start);
            } else if c == ')' {
                self.pos += 1;
                self.add(TokenType::RightParen, ")".to_string(), start);
            } else if c == '[' {
                self.pos += 1;
                self.add(TokenType::LeftBracket, "[".to_string(), start);
            } else if c == ']' {
                self.pos += 1;
                self.add(TokenType::RightBracket, "]".to_string(), start);
            } else if c == '{' {
                self.pos += 1;
                self.add(TokenType::LeftBrace, "{".to_string(), start);
            } else if c == '}' {
                self.pos += 1;
                self.add(TokenType::RightBrace, "}".to_string(), start);
            } else if c == ',' {
                self.pos += 1;
                self.add(TokenType::Comma, ",".to_string(), start);
            } else if c == ':' {
                self.pos += 1;
                self.add(TokenType::Colon, ":".to_string(), start);
            } else if c == ';' {
                self.pos += 1;
                self.add(TokenType::Semicolon, ";".to_string(), start);
            } else if (c == '+' || c == '.') && self.starts_number(self.pos) {
                self.consume_numeric(start);
            } else if c == '-' {
                if self.starts_number(self.pos) {
                    self.consume_numeric(start);
                } else if self.at(self.pos + 1) == '-' && self.at(self.pos + 2) == '>' {
                    self.pos += 3;
                    self.add(TokenType::Cdc, "-->".to_string(), start);
                } else if self.starts_ident(self.pos) {
                    self.consume_ident_like(start);
                } else {
                    self.pos += 1;
                    self.add(TokenType::Delim('-'), "-".to_string(), start);
                }
            } else if c == '<'
                && self.at(self.pos + 1) == '!'
                && self.at(self.pos + 2) == '-'
                && self.at(self.pos + 3) == '-'
            {
                self.pos += 4;
                self.add(TokenType::Cdo, "<!--".to_string(), start);
            } else if c == '@' {
                if self.starts_ident(self.pos + 1) {
                    self.pos += 1;
                    let name = self.consume_name();
                    self.add(TokenType::AtKeyword, name, start);
                } else {
                    self.pos += 1;
                    self.add(TokenType::Delim('@'), "@".to_string(), start);
                }
            } else if Self::is_digit(c) {
                self.consume_numeric(start);
            } else if Self::is_name_start(c) {
                self.consume_ident_like(start);
            } else {
                self.pos += 1;
                self.add(TokenType::Delim(c), c.to_string(), start);
            }
        }
    }

    fn consume_string(&mut self, quote: char, start: usize) {
        self.pos += 1;
        let mut s = String::new();
        while !self.at_end(self.pos) {
            let c = self.at(self.pos);
            if c == quote {
                self.pos += 1;
                self.add(TokenType::StringLit, s, start);
                return;
            }
            if c == '\n' {
                self.add(TokenType::BadString, s, start);
                return;
            }
            if c == '\\' {
                self.pos += 1;
                if !self.at_end(self.pos) {
                    s.push(self.at(self.pos));
                    self.pos += 1;
                }
                continue;
            }
            s.push(c);
            self.pos += 1;
        }
        self.add(TokenType::StringLit, s, start);
    }

    fn consume_numeric(&mut self, start: usize) {
        if self.at(self.pos) == '+' || self.at(self.pos) == '-' {
            self.pos += 1;
        }
        while Self::is_digit(self.at(self.pos)) && !self.at_end(self.pos) {
            self.pos += 1;
        }
        if self.at(self.pos) == '.' && Self::is_digit(self.at(self.pos + 1)) {
            self.pos += 2;
            while Self::is_digit(self.at(self.pos)) && !self.at_end(self.pos) {
                self.pos += 1;
            }
        }
        if self.starts_ident(self.pos) {
            self.consume_name();
            self.add(TokenType::Dimension, String::new(), start);
        } else if self.at(self.pos) == '%' && !self.at_end(self.pos) {
            self.pos += 1;
            self.add(TokenType::Percentage, String::new(), start);
        } else {
            self.add(TokenType::Number, String::new(), start);
        }
    }

    fn consume_ident_like(&mut self, start: usize) {
        let name = self.consume_name();
        if self.at(self.pos) == '(' && !self.at_end(self.pos) {
            self.pos += 1;
            self.add(TokenType::Function, name, start);
        } else {
            self.add(TokenType::Ident, name, start);
        }
    }
}
