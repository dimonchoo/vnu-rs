//! `Input`, `IncludedGrammar`, the preamble, annotations and grammar bodies.

use super::*;

impl CompactSyntax<'_, '_> {
    // ---------------------------------------------------------------- top level

    /// `Input`.
    pub fn input(mut self, scope: Scope) -> R<PatternId> {
        self.preamble()?;
        if self.top_level_lookahead() {
            let g = self.sb.make_grammar(scope);
            let mut section = Section::Grammar(g);
            self.grammar_body(&mut section, Scope::Grammar(g))?;
            self.expect_eof()?;
            Ok(self.sb.end_grammar(g))
        } else {
            let p = self.expr(true, scope)?;
            self.expect_eof()?;
            Ok(p)
        }
    }

    /// `IncludedGrammar`: definitions go straight into `grammar`.
    pub fn included_grammar(mut self, grammar: usize) -> R<()> {
        self.preamble()?;
        let mut section = Section::Grammar(grammar);
        let scope = Scope::Grammar(grammar);
        if self.top_level_lookahead() {
            self.grammar_body(&mut section, scope)?;
        } else {
            self.annotations()?;
            self.expect_keyword("grammar")?;
            self.expect_punct("{")?;
            self.grammar_body(&mut section, scope)?;
            self.expect_punct("}")?;
        }
        self.expect_eof()
    }

    pub(super) fn expect_eof(&mut self) -> R<()> {
        if *self.peek() == Tok::Eof {
            Ok(())
        } else {
            Err(self.syntax_error("expected end of file"))
        }
    }

    /// `TopLevelLookahead`.
    pub(super) fn top_level_lookahead(&self) -> bool {
        let mut i = 0;
        if matches!(self.peek_at(0), Tok::Prefixed(..))
            && matches!(self.peek_at(1), Tok::Punct("["))
        {
            return true;
        }
        if matches!(self.peek_at(0), Tok::Punct("[")) {
            i = self.skip_brackets_from(self.pos) - self.pos;
        }
        match self.peek_at(i) {
            Tok::Ident(s, true) if s == "start" || s == "div" || s == "include" => true,
            Tok::Ident(_, false) => {
                matches!(self.peek_at(i + 1), Tok::Punct("[" | "=" | "&=" | "|="))
            }
            _ => false,
        }
    }

    /// Index just past the balanced `[...]` that starts at `at`.
    pub(super) fn skip_brackets_from(&self, at: usize) -> usize {
        let mut depth = 0usize;
        let mut i = at;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Tok::Punct("[") => depth += 1,
                Tok::Punct("]") => {
                    depth -= 1;
                    if depth == 0 {
                        return i + 1;
                    }
                }
                Tok::Eof => return i,
                _ => {}
            }
            i += 1;
        }
        i
    }

    /// `Preamble`.
    pub(super) fn preamble(&mut self) -> R<()> {
        loop {
            if self.is_keyword("namespace")
                || (self.is_keyword("default")
                    && matches!(self.peek_at(1), Tok::Ident(s, true) if s == "namespace"))
            {
                self.namespace_decl()?;
            } else if self.is_keyword("datatypes") {
                self.next();
                let prefix = self.unprefixed_name()?;
                self.expect_punct("=")?;
                let uri = self.literal()?;
                self.datatypes_table.insert(prefix, uri);
            } else {
                break;
            }
        }
        self.namespace_table
            .insert("xml".into(), NsRef::Uri(XML_NS.into()));
        self.datatypes_table
            .entry("xsd".into())
            .or_insert_with(|| XML_SCHEMA_DATATYPES.into());
        Ok(())
    }

    /// `NamespaceDecl`.
    pub(super) fn namespace_decl(&mut self) -> R<()> {
        let mut is_default = false;
        let mut prefix = None;
        if self.is_keyword("default") {
            self.next();
            is_default = true;
            self.expect_keyword("namespace")?;
            if !self.is_punct("=") {
                prefix = Some(self.unprefixed_name()?);
            }
        } else {
            self.expect_keyword("namespace")?;
            prefix = Some(self.unprefixed_name()?);
        }
        self.expect_punct("=")?;
        let ns = if self.is_keyword("inherit") {
            self.next();
            NsRef::Inherit
        } else {
            NsRef::Uri(self.literal()?)
        };
        if is_default {
            self.default_namespace = ns.clone();
        }
        if let Some(prefix) = prefix {
            if prefix != "xmlns" && prefix != "xml" {
                self.namespace_table.insert(prefix, ns);
            }
        }
        Ok(())
    }

    pub(super) fn resolve_inherit(&self, ns: &NsRef) -> String {
        match ns {
            NsRef::Inherit => self.inherit_ns.clone(),
            NsRef::Uri(u) => u.clone(),
        }
    }

    pub(super) fn lookup_prefix(&mut self, prefix: &str) -> NsRef {
        match self.namespace_table.get(prefix) {
            Some(ns) => ns.clone(),
            None => {
                self.sb
                    .error(format!("{}: undeclared prefix “{prefix}”", self.source_uri));
                NsRef::Uri("#error".into())
            }
        }
    }

    pub(super) fn lookup_datatype(&mut self, prefix: &str) -> String {
        match self.datatypes_table.get(prefix) {
            Some(uri) => uri.clone(),
            None => {
                self.sb.error(format!(
                    "{}: undeclared datatype prefix “{prefix}”",
                    self.source_uri
                ));
                String::new()
            }
        }
    }

    // ------------------------------------------------------------- annotations

    /// `Annotations`: a leading `[...]`, dropped.
    pub(super) fn annotations(&mut self) -> R<()> {
        if self.is_punct("[") {
            self.pos = self.skip_brackets_from(self.pos);
        }
        Ok(())
    }

    /// `AnnotationElement`: a name followed by `[...]`, dropped.
    pub(super) fn annotation_element(&mut self) -> R<()> {
        match self.next() {
            Tok::Ident(..) | Tok::Prefixed(..) => {}
            _ => return Err(self.syntax_error("expected an annotation element")),
        }
        if !self.is_punct("[") {
            return Err(self.syntax_error("expected “[”"));
        }
        self.pos = self.skip_brackets_from(self.pos);
        Ok(())
    }

    /// Follow annotations (`>> element [...]`), dropped.
    pub(super) fn follow_annotations(&mut self) -> R<()> {
        while self.is_punct(">>") {
            self.next();
            self.annotation_element()?;
        }
        Ok(())
    }

    /// `AnnotationElementNotKeyword` lookahead inside grammar bodies.
    pub(super) fn at_annotation_element(&self) -> bool {
        matches!(self.peek_at(0), Tok::Prefixed(..) | Tok::Ident(_, false))
            && matches!(self.peek_at(1), Tok::Punct("["))
    }

    // ---------------------------------------------------------------- grammars

    /// `GrammarBody` / `IncludeBody`.
    pub(super) fn grammar_body(&mut self, section: &mut Section, scope: Scope) -> R<()> {
        loop {
            while self.at_annotation_element() {
                self.annotation_element()?;
            }
            if self.is_punct("}") || *self.peek() == Tok::Eof {
                return Ok(());
            }
            self.annotations()?;
            if self.is_keyword("div") {
                self.next();
                self.expect_punct("{")?;
                self.grammar_body(section, scope)?;
                self.expect_punct("}")?;
            } else if self.is_keyword("include") {
                if matches!(section, Section::Include { .. }) {
                    return Err(self.syntax_error("include inside include"));
                }
                self.include(section)?;
            } else if self.is_keyword("start") {
                self.next();
                let combine = self.assign_op()?;
                let p = self.expr(false, scope)?;
                self.sb.define(section, None, combine, p);
            } else {
                let name = self.identifier()?;
                let combine = self.assign_op()?;
                let p = self.expr(false, scope)?;
                self.sb.define(section, Some(&name), combine, p);
            }
        }
    }

    /// `AssignOp`.
    pub(super) fn assign_op(&mut self) -> R<Option<Combine>> {
        match self.next() {
            Tok::Punct("=") => Ok(None),
            Tok::Punct("|=") => Ok(Some(Combine::Choice)),
            Tok::Punct("&=") => Ok(Some(Combine::Interleave)),
            _ => Err(self.syntax_error("expected “=”, “|=” or “&=”")),
        }
    }

    /// `Include`.
    pub(super) fn include(&mut self, section: &Section) -> R<()> {
        self.expect_keyword("include")?;
        let href = self.literal()?;
        let ns = self.inherit()?;
        let grammar = SchemaBuilderImpl::grammar_of(section);
        let mut include = Section::Include {
            grammar,
            overrides: Vec::new(),
        };
        if self.is_punct("{") {
            self.next();
            self.grammar_body(&mut include, Scope::Grammar(grammar))?;
            self.expect_punct("}")?;
        }
        let inherit_ns = self.resolve_inherit(&ns);
        let base = self.source_uri.clone();
        self.sb.end_include(include, &href, &base, inherit_ns)
    }

    /// `Inherit`.
    pub(super) fn inherit(&mut self) -> R<NsRef> {
        if self.is_keyword("inherit") {
            self.next();
            self.expect_punct("=")?;
            let prefix = match self.next() {
                Tok::Ident(s, _) => s,
                _ => return Err(self.syntax_error("expected a prefix")),
            };
            Ok(self.lookup_prefix(&prefix))
        } else {
            Ok(self.default_namespace.clone())
        }
    }
}
