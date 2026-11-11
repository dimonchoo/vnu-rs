//! Jing's `SchemaBuilderImpl` with its `GrammarImpl`, `IncludeImpl`, `RootScope` and
//! `Override`: named definitions, `combine`, `include` overrides and `parent`
//! references. The compact-syntax parser ([`crate::compact_syntax`]) drives it.

use crate::compact_syntax::CompactSyntax;
use crate::datatype::DatatypeRef;
use crate::name::Name;
use crate::name_class::{NameClass, NameClassId};
use crate::pattern::{PatternId, RefId};
use crate::pattern_builder::ERROR;
use crate::schema_pattern_builder::*;
use std::collections::HashMap;

/// Reads the text of an included schema: `(href, base URI) -> (URI, text)`.
pub type ResourceLoader<'a> = dyn Fn(&str, &str) -> Result<(String, String), String> + 'a;

/// A scope for references (`Scope`): the root, or a grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Root,
    Grammar(usize),
}

/// Where definitions go (`GrammarSection`): a grammar (or a div in it), or the body
/// of an `include`, whose definitions override the included ones.
#[derive(Debug)]
pub enum Section {
    Grammar(usize),
    Include {
        grammar: usize,
        overrides: Vec<RefId>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combine {
    Choice,
    Interleave,
}

#[derive(Debug)]
struct GrammarImpl {
    defines: HashMap<String, RefId>,
    start_ref: RefId,
    parent: Scope,
}

pub struct SchemaBuilderImpl<'a> {
    pub spb: SchemaPatternBuilder,
    grammars: Vec<GrammarImpl>,
    /// `OpenIncludes`: URIs being parsed, outermost first.
    open_includes: Vec<String>,
    loader: &'a ResourceLoader<'a>,
    pub errors: Vec<String>,
}

impl<'a> SchemaBuilderImpl<'a> {
    pub fn new(loader: &'a ResourceLoader<'a>) -> Self {
        Self {
            spb: SchemaPatternBuilder::default(),
            grammars: Vec::new(),
            open_includes: Vec::new(),
            loader,
            errors: Vec::new(),
        }
    }

    pub fn error(&mut self, message: String) {
        self.errors.push(message);
    }

    /// `SchemaBuilderImpl.parse` followed by `expandPattern`.
    pub fn parse(&mut self, uri: &str) -> Result<PatternId, String> {
        let (uri, text) = (self.loader)(uri, "")?;
        self.open_includes.push(uri.clone());
        let p = CompactSyntax::new(self, &text, &uri, String::new()).input(Scope::Root);
        self.open_includes.pop();
        let p = p?;
        if !self.errors.is_empty() {
            return Err(self.errors.join("\n"));
        }
        Ok(self.spb.expand(p))
    }

    // ---------------------------------------------------------------- grammars

    pub fn make_grammar(&mut self, parent: Scope) -> usize {
        let start_ref = self.spb.make_ref(None);
        self.grammars.push(GrammarImpl {
            defines: HashMap::new(),
            start_ref,
            parent,
        });
        self.grammars.len() - 1
    }

    /// `GrammarImpl.endGrammar`.
    pub fn end_grammar(&mut self, g: usize) -> PatternId {
        let mut undefined: Vec<String> = Vec::new();
        for (name, &r) in &self.grammars[g].defines {
            if self.spb.refs[r.0 as usize].pattern.is_none() {
                undefined.push(name.clone());
                self.spb.refs[r.0 as usize].pattern = Some(ERROR);
            }
        }
        undefined.sort();
        for name in undefined {
            self.error(format!("Reference to undefined pattern “{name}”."));
        }
        let start_ref = self.grammars[g].start_ref;
        match self.spb.refs[start_ref.0 as usize].pattern {
            Some(start) => start,
            None => {
                self.error("Missing “start” element.".into());
                ERROR
            }
        }
    }

    fn lookup(&mut self, g: usize, name: Option<&str>) -> RefId {
        match name {
            None => self.grammars[g].start_ref,
            Some(name) => self.lookup1(g, name),
        }
    }

    fn lookup1(&mut self, g: usize, name: &str) -> RefId {
        if let Some(&r) = self.grammars[g].defines.get(name) {
            return r;
        }
        let r = self.spb.make_ref(Some(name.to_string()));
        self.grammars[g].defines.insert(name.to_string(), r);
        r
    }

    /// `GrammarSection.define` for `Section::Grammar` and `Section::Include`.
    pub fn define(
        &mut self,
        section: &mut Section,
        name: Option<&str>,
        combine: Option<Combine>,
        pattern: PatternId,
    ) {
        match section {
            Section::Grammar(g) => {
                let rp = self.lookup(*g, name);
                self.define_ref(rp, combine, pattern);
            }
            Section::Include { grammar, overrides } => {
                let rp = self.lookup(*grammar, name);
                overrides.push(rp);
                self.define_ref(rp, combine, pattern);
            }
        }
    }

    /// `GrammarImpl.define(RefPattern, ...)`.
    fn define_ref(&mut self, rp: RefId, combine: Option<Combine>, pattern: PatternId) {
        let r = rp.0 as usize;
        match self.spb.refs[r].replacement_status {
            REPLACEMENT_KEEP => {
                match combine {
                    None => {
                        if self.spb.refs[r].combine_implicit {
                            let msg = match &self.spb.refs[r].name {
                                None => {
                                    "Multiple definitions of start without “combine” attribute."
                                        .to_string()
                                }
                                Some(n) => format!(
                                    "Multiple definitions of “{n}” without “combine” attribute."
                                ),
                            };
                            self.error(msg);
                        } else {
                            self.spb.refs[r].combine_implicit = true;
                        }
                    }
                    Some(c) => {
                        let combine_type = match c {
                            Combine::Choice => COMBINE_CHOICE,
                            Combine::Interleave => COMBINE_INTERLEAVE,
                        };
                        let current = self.spb.refs[r].combine_type;
                        if current != COMBINE_NONE && current != combine_type {
                            let msg = match &self.spb.refs[r].name {
                                None => "Conflicting values of “combine” attribute for definition of start.".to_string(),
                                Some(n) => format!(
                                    "Conflicting values of “combine” attribute for definition of “{n}”."
                                ),
                            };
                            self.error(msg);
                        }
                        self.spb.refs[r].combine_type = combine_type;
                    }
                }
                let combined = match self.spb.refs[r].pattern {
                    None => pattern,
                    Some(existing) if self.spb.refs[r].combine_type == COMBINE_INTERLEAVE => {
                        self.spb.make_interleave(existing, pattern)
                    }
                    Some(existing) => self.spb.make_choice(existing, pattern),
                };
                self.spb.refs[r].pattern = Some(combined);
            }
            REPLACEMENT_REQUIRE => {
                self.spb.refs[r].replacement_status = REPLACEMENT_IGNORE;
            }
            _ => {}
        }
    }

    /// `Scope.makeRef`.
    pub fn make_ref(&mut self, scope: Scope, name: &str) -> PatternId {
        match scope {
            Scope::Root => {
                self.error("Reference to non-existent grammar.".into());
                ERROR
            }
            Scope::Grammar(g) => {
                let r = self.lookup1(g, name);
                self.spb.ref_pattern(r)
            }
        }
    }

    /// `Scope.makeParentRef`.
    pub fn make_parent_ref(&mut self, scope: Scope, name: &str) -> PatternId {
        let parent = match scope {
            Scope::Grammar(g) => self.grammars[g].parent,
            Scope::Root => Scope::Root,
        };
        if parent == Scope::Root {
            self.error("Reference to non-existent parent grammar.".into());
            return ERROR;
        }
        self.make_ref(parent, name)
    }

    pub fn grammar_of(section: &Section) -> usize {
        match section {
            Section::Grammar(g) | Section::Include { grammar: g, .. } => *g,
        }
    }

    /// `IncludeImpl.endInclude`.
    pub fn end_include(
        &mut self,
        section: Section,
        href: &str,
        base: &str,
        inherit_ns: String,
    ) -> Result<(), String> {
        let Section::Include { grammar, overrides } = section else {
            unreachable!("end_include on a grammar section");
        };
        let (uri, text) = (self.loader)(href, base)?;
        if self.open_includes.contains(&uri) {
            self.error(format!("Recursive inclusion of URL “{uri}”."));
            return Ok(());
        }
        let saved: Vec<u8> = overrides
            .iter()
            .map(|r| {
                let status = self.spb.refs[r.0 as usize].replacement_status;
                self.spb.refs[r.0 as usize].replacement_status = REPLACEMENT_REQUIRE;
                status
            })
            .collect();
        self.open_includes.push(uri.clone());
        let result = CompactSyntax::new(self, &text, &uri, inherit_ns).included_grammar(grammar);
        self.open_includes.pop();
        for &r in &overrides {
            if self.spb.refs[r.0 as usize].replacement_status == REPLACEMENT_REQUIRE {
                let msg = match &self.spb.refs[r.0 as usize].name {
                    None => "“start” in “include” does not override anything.".to_string(),
                    Some(n) => {
                        format!("Definition of “{n}” in “include” does not override anything.")
                    }
                };
                self.error(msg);
            }
        }
        for (r, status) in overrides.iter().zip(saved) {
            self.spb.refs[r.0 as usize].replacement_status = status;
        }
        result
    }

    // ------------------------------------------------------------ name classes

    pub fn make_name(&mut self, ns: &str, local: &str) -> NameClassId {
        let ns = self.spb.pb.names.namespace(ns);
        let name: Name = self.spb.pb.names.name(ns, local);
        self.spb.pb.name_classes.intern(NameClass::Simple(name))
    }

    pub fn make_ns_name(&mut self, ns: &str, except: Option<NameClassId>) -> NameClassId {
        let ns = self.spb.pb.names.namespace(ns);
        let nc = match except {
            None => NameClass::NsName(ns),
            Some(e) => NameClass::NsNameExcept(ns, e),
        };
        self.spb.pb.name_classes.intern(nc)
    }

    pub fn make_any_name(&mut self, except: Option<NameClassId>) -> NameClassId {
        let nc = match except {
            None => NameClass::AnyName,
            Some(e) => NameClass::AnyNameExcept(e),
        };
        self.spb.pb.name_classes.intern(nc)
    }

    pub fn make_name_class_choice(&mut self, ncs: &[NameClassId]) -> NameClassId {
        let mut result = ncs[0];
        for &nc in &ncs[1..] {
            result = self
                .spb
                .pb
                .name_classes
                .intern(NameClass::Choice(result, nc));
        }
        result
    }

    // ---------------------------------------------------------------- patterns

    pub fn make_choice(&mut self, ps: &[PatternId]) -> PatternId {
        let mut result = ps[0];
        for &p in &ps[1..] {
            result = self.spb.make_choice(result, p);
        }
        result
    }

    pub fn make_group(&mut self, ps: &[PatternId]) -> PatternId {
        let mut result = ps[0];
        for &p in &ps[1..] {
            result = self.spb.make_group(result, p);
        }
        result
    }

    pub fn make_interleave(&mut self, ps: &[PatternId]) -> PatternId {
        let mut result = ps[0];
        for &p in &ps[1..] {
            result = self.spb.make_interleave(result, p);
        }
        result
    }

    /// `makeValue`: the datatype is looked up later, by the validator.
    pub fn make_value(&mut self, library: &str, name: &str, value: &str) -> PatternId {
        let dt = self.spb.pb.datatype(DatatypeRef {
            library: library.to_string(),
            name: name.to_string(),
            params: Vec::new(),
        });
        let v = self.spb.pb.value(dt, value);
        self.spb.make_value(dt, v)
    }

    /// `DataPatternBuilderImpl.makePattern`.
    pub fn make_data(
        &mut self,
        library: &str,
        name: &str,
        params: Vec<(String, String)>,
        except: Option<PatternId>,
    ) -> PatternId {
        let dt = self.spb.pb.datatype(DatatypeRef {
            library: library.to_string(),
            name: name.to_string(),
            params,
        });
        match except {
            None => self.spb.make_data(dt),
            Some(e) => self.spb.make_data_except(dt, e),
        }
    }
}
