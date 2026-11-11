//! Name classes: `NameClass`, `NameClassAlternatives`, `BasicNameClass`.

use super::*;

impl CompactSyntax<'_, '_> {
    // ------------------------------------------------------------ name classes

    /// `NameClass`.
    pub(super) fn name_class(&mut self, context: u8) -> R<NameClassId> {
        self.annotations()?;
        match self.peek().clone() {
            Tok::Punct("*") => {
                self.next();
                if self.is_punct("-") {
                    self.next();
                    let except = self.basic_name_class(context)?;
                    let nc = self.sb.make_any_name(Some(except));
                    self.follow_annotations()?;
                    Ok(nc)
                } else {
                    let nc = self.sb.make_any_name(None);
                    self.follow_annotations()?;
                    self.name_class_alternatives(context, nc)
                }
            }
            Tok::PrefixStar(prefix) => {
                self.next();
                let ns = self.lookup_prefix(&prefix);
                let ns = self.resolve_inherit(&ns);
                if self.is_punct("-") {
                    self.next();
                    let except = self.basic_name_class(context)?;
                    let nc = self.sb.make_ns_name(&ns, Some(except));
                    self.follow_annotations()?;
                    Ok(nc)
                } else {
                    let nc = self.sb.make_ns_name(&ns, None);
                    self.follow_annotations()?;
                    self.name_class_alternatives(context, nc)
                }
            }
            _ => {
                let nc = self.primary_name_class(context)?;
                self.follow_annotations()?;
                self.name_class_alternatives(context, nc)
            }
        }
    }

    /// `NameClassAlternatives`.
    pub(super) fn name_class_alternatives(
        &mut self,
        context: u8,
        first: NameClassId,
    ) -> R<NameClassId> {
        if !self.is_punct("|") {
            return Ok(first);
        }
        let mut ncs = vec![first];
        while self.is_punct("|") {
            self.next();
            ncs.push(self.basic_name_class(context)?);
        }
        Ok(self.sb.make_name_class_choice(&ncs))
    }

    /// `BasicNameClass`: a primary or open name class without an except.
    pub(super) fn basic_name_class(&mut self, context: u8) -> R<NameClassId> {
        self.annotations()?;
        let nc = match self.peek().clone() {
            Tok::Punct("*") => {
                self.next();
                self.sb.make_any_name(None)
            }
            Tok::PrefixStar(prefix) => {
                self.next();
                let ns = self.lookup_prefix(&prefix);
                let ns = self.resolve_inherit(&ns);
                self.sb.make_ns_name(&ns, None)
            }
            _ => self.primary_name_class(context)?,
        };
        self.follow_annotations()?;
        Ok(nc)
    }

    /// `PrimaryNameClass`: a name or a parenthesized name class.
    pub(super) fn primary_name_class(&mut self, context: u8) -> R<NameClassId> {
        match self.next() {
            Tok::Ident(local, _) => {
                let ns = if context == IN_ATTRIBUTE {
                    String::new()
                } else {
                    let d = self.default_namespace.clone();
                    self.resolve_inherit(&d)
                };
                Ok(self.sb.make_name(&ns, &local))
            }
            Tok::Prefixed(prefix, local) => {
                let ns = self.lookup_prefix(&prefix);
                let ns = self.resolve_inherit(&ns);
                Ok(self.sb.make_name(&ns, &local))
            }
            Tok::Punct("(") => {
                let nc = self.name_class(context)?;
                self.expect_punct(")")?;
                Ok(nc)
            }
            _ => Err(self.syntax_error("expected a name class")),
        }
    }
}
