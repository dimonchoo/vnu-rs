//! Patterns: `Expr`, `UnaryExpr`, `PrimaryExpr`, `DataExpr`.

use super::*;

impl CompactSyntax<'_, '_> {
    // ---------------------------------------------------------------- patterns

    /// `Expr`.
    pub(super) fn expr(&mut self, top_level: bool, scope: Scope) -> R<PatternId> {
        let first = self.unary_expr(top_level, scope)?;
        let op = match self.peek() {
            Tok::Punct(p @ ("|" | "&" | ",")) => *p,
            _ => return Ok(first),
        };
        let mut patterns = vec![first];
        while self.is_punct(op) {
            self.next();
            patterns.push(self.unary_expr(top_level, scope)?);
        }
        Ok(match op {
            "|" => self.sb.make_choice(&patterns),
            "&" => self.sb.make_interleave(&patterns),
            _ => self.sb.make_group(&patterns),
        })
    }

    /// `UnaryExpr`.
    pub(super) fn unary_expr(&mut self, top_level: bool, scope: Scope) -> R<PatternId> {
        let mut p = self.annotated_primary_expr(top_level, scope)?;
        let occur = match self.peek() {
            Tok::Punct(o @ ("+" | "?" | "*")) => Some(*o),
            _ => None,
        };
        if let Some(o) = occur {
            self.next();
            p = match o {
                "+" => self.sb.spb.make_one_or_more(p),
                "?" => self.sb.spb.make_optional(p),
                _ => self.sb.spb.make_zero_or_more(p),
            };
            self.follow_annotations()?;
        }
        Ok(p)
    }

    /// `AnnotatedPrimaryExpr`.
    pub(super) fn annotated_primary_expr(&mut self, top_level: bool, scope: Scope) -> R<PatternId> {
        self.annotations()?;
        let p = self.primary_expr(top_level, scope)?;
        self.follow_annotations()?;
        Ok(p)
    }

    /// `PrimaryExpr`.
    pub(super) fn primary_expr(&mut self, _top_level: bool, scope: Scope) -> R<PatternId> {
        match self.peek().clone() {
            Tok::Ident(k, true) => match k.as_str() {
                "element" => {
                    self.next();
                    let nc = self.name_class(IN_ELEMENT)?;
                    self.expect_punct("{")?;
                    let p = self.expr(false, scope)?;
                    self.expect_punct("}")?;
                    Ok(self.sb.spb.make_element(nc, p))
                }
                "attribute" => {
                    self.next();
                    let nc = self.name_class(IN_ATTRIBUTE)?;
                    self.expect_punct("{")?;
                    let p = self.expr(false, scope)?;
                    self.expect_punct("}")?;
                    Ok(self.sb.spb.make_attribute(nc, p))
                }
                "list" => {
                    self.next();
                    self.expect_punct("{")?;
                    let p = self.expr(false, scope)?;
                    self.expect_punct("}")?;
                    Ok(self.sb.spb.make_list(p))
                }
                "mixed" => {
                    self.next();
                    self.expect_punct("{")?;
                    let p = self.expr(false, scope)?;
                    self.expect_punct("}")?;
                    Ok(self.sb.spb.make_mixed(p))
                }
                "empty" => {
                    self.next();
                    Ok(self.sb.spb.make_empty())
                }
                "text" => {
                    self.next();
                    Ok(self.sb.spb.make_text())
                }
                "notAllowed" => {
                    self.next();
                    Ok(self.sb.spb.make_unexpanded_not_allowed())
                }
                "parent" => {
                    self.next();
                    let name = self.identifier()?;
                    Ok(self.sb.make_parent_ref(scope, &name))
                }
                "grammar" => {
                    self.next();
                    self.expect_punct("{")?;
                    let g = self.sb.make_grammar(scope);
                    let mut section = Section::Grammar(g);
                    self.grammar_body(&mut section, Scope::Grammar(g))?;
                    self.expect_punct("}")?;
                    Ok(self.sb.end_grammar(g))
                }
                "external" => Err(self.syntax_error("“external” is not supported")),
                "string" | "token" => self.data_expr(scope),
                _ => Err(self.syntax_error("unexpected keyword")),
            },
            Tok::Ident(_, false) => {
                let name = self.identifier()?;
                Ok(self.sb.make_ref(scope, &name))
            }
            Tok::Prefixed(..) => self.data_expr(scope),
            Tok::Literal(_) => {
                let value = self.literal()?;
                Ok(self.sb.make_value("", "token", &value))
            }
            Tok::Punct("(") => {
                self.next();
                let p = self.expr(false, scope)?;
                self.expect_punct(")")?;
                Ok(p)
            }
            _ => Err(self.syntax_error("expected a pattern")),
        }
    }

    /// `DataExpr`.
    pub(super) fn data_expr(&mut self, scope: Scope) -> R<PatternId> {
        let (library, name) = match self.next() {
            Tok::Ident(n, true) => (String::new(), n),
            Tok::Prefixed(prefix, local) => (self.lookup_datatype(&prefix), local),
            _ => return Err(self.syntax_error("expected a datatype name")),
        };
        if matches!(self.peek(), Tok::Literal(_)) {
            let value = self.literal()?;
            return Ok(self.sb.make_value(&library, &name, &value));
        }
        let mut params = Vec::new();
        if self.is_punct("{") {
            self.next();
            while !self.is_punct("}") {
                self.annotations()?;
                let pname = self.unprefixed_name()?;
                self.expect_punct("=")?;
                let pvalue = self.literal()?;
                params.push((pname, pvalue));
            }
            self.next();
        }
        let except = if self.is_punct("-") {
            self.next();
            self.annotations()?;
            Some(self.primary_expr(false, scope)?)
        } else {
            None
        };
        Ok(self.sb.make_data(&library, &name, params, except))
    }
}
