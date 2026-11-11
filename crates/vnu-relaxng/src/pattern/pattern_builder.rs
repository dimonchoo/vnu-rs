//! Jing's `PatternBuilder` and `PatternInterner`: the pattern arena plus the
//! normalizing constructors shared by the schema builder and the validator.

use crate::datatype::{DatatypeId, DatatypeRef, ValueId};
use crate::name::NameTable;
use crate::name_class::NameClasses;
use crate::pattern::*;
use std::collections::HashMap;

pub const EMPTY: PatternId = PatternId(0);
pub const NOT_ALLOWED: PatternId = PatternId(1);
pub const TEXT: PatternId = PatternId(2);
pub const UNEXPANDED_NOT_ALLOWED: PatternId = PatternId(3);
pub const ERROR: PatternId = PatternId(4);

#[derive(Debug, Clone)]
pub struct PatternBuilder {
    pub(crate) nodes: Vec<PatternNode>,
    /// `PatternInterner` for the patterns this builder makes (choice, group,
    /// interleave, oneOrMore, after).
    interner: HashMap<Pattern, PatternId>,
    pub names: NameTable,
    pub name_classes: NameClasses,
    pub(crate) datatypes: Vec<DatatypeRef>,
    datatype_ids: HashMap<DatatypeRef, DatatypeId>,
    pub(crate) values: Vec<(DatatypeId, String)>,
    value_ids: HashMap<(DatatypeId, String), ValueId>,
}

impl Default for PatternBuilder {
    fn default() -> Self {
        let mut b = Self {
            nodes: Vec::new(),
            interner: HashMap::new(),
            names: NameTable::default(),
            name_classes: NameClasses::default(),
            datatypes: Vec::new(),
            datatype_ids: HashMap::new(),
            values: Vec::new(),
            value_ids: HashMap::new(),
        };
        for p in [
            Pattern::Empty,
            Pattern::NotAllowed,
            Pattern::Text,
            Pattern::UnexpandedNotAllowed,
            Pattern::Error,
        ] {
            b.alloc(p);
        }
        b
    }
}

impl PatternBuilder {
    #[inline]
    pub fn node(&self, p: PatternId) -> &PatternNode {
        &self.nodes[p.0 as usize]
    }

    #[inline]
    pub fn pattern(&self, p: PatternId) -> Pattern {
        self.nodes[p.0 as usize].pattern
    }

    #[inline]
    pub fn is_nullable(&self, p: PatternId) -> bool {
        self.nodes[p.0 as usize].nullable
    }

    #[inline]
    pub fn content_type(&self, p: PatternId) -> u8 {
        self.nodes[p.0 as usize].content_type
    }

    /// `Pattern.isNotAllowed()`: true for `notAllowed` and for an after pattern whose
    /// first operand is `notAllowed`.
    pub fn is_not_allowed(&self, p: PatternId) -> bool {
        match self.pattern(p) {
            Pattern::NotAllowed => true,
            Pattern::After(p1, _) => self.is_not_allowed(p1),
            _ => false,
        }
    }

    /// `Pattern.containsChoice(p)`.
    pub fn contains_choice(&self, this: PatternId, p: PatternId) -> bool {
        match self.pattern(this) {
            Pattern::Choice(p1, p2) => self.contains_choice(p1, p) || self.contains_choice(p2, p),
            _ => this == p,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub(crate) fn alloc(&mut self, pattern: Pattern) -> PatternId {
        let (nullable, content_type) = self.properties(pattern);
        let id = PatternId(self.nodes.len() as u32);
        self.nodes.push(PatternNode {
            pattern,
            nullable,
            content_type,
        });
        id
    }

    /// Nullability and content type, as the Java constructors compute them.
    fn properties(&self, pattern: Pattern) -> (bool, u8) {
        let ct = |p: PatternId| self.content_type(p);
        match pattern {
            Pattern::Empty => (true, EMPTY_CONTENT_TYPE),
            Pattern::Text => (true, MIXED_CONTENT_TYPE),
            Pattern::NotAllowed
            | Pattern::UnexpandedNotAllowed
            | Pattern::Error
            | Pattern::Ref(_) => (false, EMPTY_CONTENT_TYPE),
            Pattern::Choice(p1, p2) => (
                self.is_nullable(p1) || self.is_nullable(p2),
                ct(p1).max(ct(p2)),
            ),
            Pattern::Group(p1, p2) | Pattern::Interleave(p1, p2) => (
                self.is_nullable(p1) && self.is_nullable(p2),
                ct(p1).max(ct(p2)),
            ),
            Pattern::After(p1, p2) => (false, ct(p1).max(ct(p2))),
            Pattern::OneOrMore(p) => (self.is_nullable(p), ct(p)),
            Pattern::Element(..) => (false, ELEMENT_CONTENT_TYPE),
            Pattern::Attribute(..) => (false, EMPTY_CONTENT_TYPE),
            Pattern::Data(_) | Pattern::DataExcept(..) | Pattern::Value(..) | Pattern::List(_) => {
                (false, DATA_CONTENT_TYPE)
            }
        }
    }

    /// `PatternInterner.intern`.
    pub(crate) fn intern(&mut self, pattern: Pattern) -> PatternId {
        if let Some(&id) = self.interner.get(&pattern) {
            return id;
        }
        let id = self.alloc(pattern);
        self.interner.insert(pattern, id);
        id
    }

    /// Records `pattern` as interned at `id` (loading a compiled schema).
    pub(crate) fn register_interned(&mut self, pattern: Pattern, id: PatternId) {
        self.interner.insert(pattern, id);
    }

    /// Drops the patterns allocated after the first `len` ones.
    pub(crate) fn truncate(&mut self, len: usize) {
        for node in self.nodes.drain(len..) {
            if self
                .interner
                .get(&node.pattern)
                .is_some_and(|id| id.0 as usize >= len)
            {
                self.interner.remove(&node.pattern);
            }
        }
    }

    /// Mutates an element pattern when the schema is expanded (`ElementPattern.expand`).
    pub(crate) fn set_pattern(&mut self, id: PatternId, pattern: Pattern) {
        self.nodes[id.0 as usize].pattern = pattern;
    }

    pub fn datatype(&mut self, dt: DatatypeRef) -> DatatypeId {
        if let Some(&id) = self.datatype_ids.get(&dt) {
            return id;
        }
        let id = DatatypeId(self.datatypes.len() as u32);
        self.datatypes.push(dt.clone());
        self.datatype_ids.insert(dt, id);
        id
    }

    pub fn datatype_ref(&self, dt: DatatypeId) -> &DatatypeRef {
        &self.datatypes[dt.0 as usize]
    }

    pub fn value(&mut self, dt: DatatypeId, lexical: &str) -> ValueId {
        let key = (dt, lexical.to_string());
        if let Some(&id) = self.value_ids.get(&key) {
            return id;
        }
        let id = ValueId(self.values.len() as u32);
        self.values.push(key.clone());
        self.value_ids.insert(key, id);
        id
    }

    pub fn value_lexical(&self, v: ValueId) -> &str {
        &self.values[v.0 as usize].1
    }

    // ------------------------------------------------------------ constructors

    pub fn make_empty(&self) -> PatternId {
        EMPTY
    }

    pub fn make_not_allowed(&self) -> PatternId {
        NOT_ALLOWED
    }

    pub fn make_group(&mut self, p1: PatternId, p2: PatternId) -> PatternId {
        if p1 == EMPTY {
            return p2;
        }
        if p2 == EMPTY {
            return p1;
        }
        if p1 == NOT_ALLOWED || p2 == NOT_ALLOWED {
            return NOT_ALLOWED;
        }
        self.intern(Pattern::Group(p1, p2))
    }

    pub fn make_interleave(&mut self, p1: PatternId, p2: PatternId) -> PatternId {
        if p1 == EMPTY {
            return p2;
        }
        if p2 == EMPTY {
            return p1;
        }
        if p1 == NOT_ALLOWED || p2 == NOT_ALLOWED {
            return NOT_ALLOWED;
        }
        self.intern(Pattern::Interleave(p1, p2))
    }

    /// `PatternBuilder.makeChoice`, the base the subclasses call through `super`.
    pub fn make_choice_base(&mut self, p1: PatternId, p2: PatternId) -> PatternId {
        if p1 == EMPTY && self.is_nullable(p2) {
            return p2;
        }
        if p2 == EMPTY && self.is_nullable(p1) {
            return p1;
        }
        self.intern(Pattern::Choice(p1, p2))
    }

    pub fn make_one_or_more_base(&mut self, p: PatternId) -> PatternId {
        if p == EMPTY || p == NOT_ALLOWED || matches!(self.pattern(p), Pattern::OneOrMore(_)) {
            return p;
        }
        self.intern(Pattern::OneOrMore(p))
    }
}

impl PatternBuilder {
    /// `Pattern.accept(PatternVisitor)`.
    pub fn accept<V: PatternVisitor + ?Sized>(&self, p: PatternId, visitor: &mut V) {
        match self.pattern(p) {
            Pattern::Empty => visitor.visit_empty(),
            Pattern::NotAllowed | Pattern::UnexpandedNotAllowed => visitor.visit_not_allowed(),
            Pattern::Error => visitor.visit_error(),
            Pattern::Text => visitor.visit_text(),
            Pattern::Group(p1, p2) => visitor.visit_group(p1, p2),
            Pattern::Interleave(p1, p2) => visitor.visit_interleave(p1, p2),
            Pattern::Choice(p1, p2) => visitor.visit_choice(p1, p2),
            Pattern::OneOrMore(p1) => visitor.visit_one_or_more(p1),
            Pattern::Element(nc, content) => visitor.visit_element(nc, content),
            Pattern::Attribute(nc, value) => visitor.visit_attribute(nc, value),
            Pattern::Data(dt) => visitor.visit_data(dt),
            Pattern::DataExcept(dt, except) => visitor.visit_data_except(dt, except),
            Pattern::Value(dt, v) => visitor.visit_value(dt, v),
            Pattern::List(p1) => visitor.visit_list(p1),
            // AfterPattern.accept does nothing; references are expanded away.
            Pattern::After(..) | Pattern::Ref(_) => {}
        }
    }
}
