//! Jing's RELAX NG compact syntax parser (`rng-parse/.../compact/CompactSyntax.jj`).
//!
//! Like the JavaCC grammar it calls the schema builder while it parses. Annotations
//! and documentation comments are read and dropped, as `SchemaBuilderImpl` ignores
//! them.

use crate::datatype::XML_SCHEMA_DATATYPES;
use crate::name_class::NameClassId;
use crate::pattern::PatternId;
use crate::schema_builder_impl::{Combine, SchemaBuilderImpl, Scope, Section};
use std::collections::HashMap;

mod expr;
mod grammar;
mod lexer;
mod name_class;

use lexer::{Tok, tokenize, unescape};

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

/// A namespace as the compact syntax sees it: a URI, or `inherit`.
#[derive(Debug, Clone, PartialEq)]
enum NsRef {
    Inherit,
    Uri(String),
}

const IN_ELEMENT: u8 = 0;
const IN_ATTRIBUTE: u8 = 1;

pub struct CompactSyntax<'s, 'a> {
    sb: &'s mut SchemaBuilderImpl<'a>,
    tokens: Vec<Tok>,
    pos: usize,
    source_uri: String,
    /// `SchemaBuilderImpl.inheritNs` of the builder for this file.
    inherit_ns: String,
    default_namespace: NsRef,
    namespace_table: HashMap<String, NsRef>,
    datatypes_table: HashMap<String, String>,
}

type R<T> = Result<T, String>;

impl<'s, 'a> CompactSyntax<'s, 'a> {
    pub fn new(
        sb: &'s mut SchemaBuilderImpl<'a>,
        text: &str,
        source_uri: &str,
        inherit_ns: String,
    ) -> Self {
        let tokens = match tokenize(&unescape(text)) {
            Ok(t) => t,
            Err(e) => {
                sb.error(format!("{source_uri}: {e}"));
                vec![Tok::Eof]
            }
        };
        Self {
            sb,
            tokens,
            pos: 0,
            source_uri: source_uri.to_string(),
            inherit_ns,
            default_namespace: NsRef::Inherit,
            namespace_table: HashMap::new(),
            datatypes_table: HashMap::new(),
        }
    }

    // ------------------------------------------------------------------ tokens

    fn peek(&self) -> &Tok {
        &self.tokens[self.pos]
    }

    fn peek_at(&self, n: usize) -> &Tok {
        self.tokens
            .get(self.pos + n)
            .unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn next(&mut self) -> Tok {
        let t = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn is_punct(&self, p: &str) -> bool {
        matches!(self.peek(), Tok::Punct(q) if *q == p)
    }

    fn is_keyword(&self, k: &str) -> bool {
        matches!(self.peek(), Tok::Ident(s, true) if s == k)
    }

    fn expect_punct(&mut self, p: &str) -> R<()> {
        if self.is_punct(p) {
            self.next();
            Ok(())
        } else {
            Err(self.syntax_error(&format!("expected “{p}”")))
        }
    }

    fn expect_keyword(&mut self, k: &str) -> R<()> {
        if self.is_keyword(k) {
            self.next();
            Ok(())
        } else {
            Err(self.syntax_error(&format!("expected “{k}”")))
        }
    }

    fn syntax_error(&self, what: &str) -> String {
        format!(
            "{}: syntax error at token {}: {what}",
            self.source_uri, self.pos
        )
    }

    // ------------------------------------------------------------------ lexemes

    /// `Identifier`: an unescaped non-keyword or an escaped identifier.
    fn identifier(&mut self) -> R<String> {
        match self.next() {
            Tok::Ident(s, false) => Ok(s),
            _ => Err(self.syntax_error("expected an identifier")),
        }
    }

    /// `UnprefixedName`: an identifier or a keyword.
    fn unprefixed_name(&mut self) -> R<String> {
        match self.next() {
            Tok::Ident(s, _) => Ok(s),
            _ => Err(self.syntax_error("expected a name")),
        }
    }

    /// `Literal`: literal segments joined by `~`.
    fn literal(&mut self) -> R<String> {
        let mut s = match self.next() {
            Tok::Literal(s) => s,
            _ => return Err(self.syntax_error("expected a literal")),
        };
        while self.is_punct("~") {
            self.next();
            match self.next() {
                Tok::Literal(more) => s.push_str(&more),
                _ => return Err(self.syntax_error("expected a literal after “~”")),
            }
        }
        Ok(s)
    }
}
