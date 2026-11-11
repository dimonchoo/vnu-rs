//! Jing's `ValidatorPatternBuilder` and `PatternMemo`: the builder the validator
//! derives patterns with, and the per-pattern memo of derivatives.

use crate::data_deriv::DataDerivType;
use crate::datatype::{Datatype, DatatypeLibraryFactory};
use crate::name::Name;
use crate::pattern::*;
use crate::pattern_builder::*;
use crate::schema::Schema;
use std::collections::{HashMap, HashSet};

/// `PatternMemo`. Java keeps one memo object per pattern; here the memo of pattern
/// `p` is `memos[p]`, and the memoized derivatives are pattern ids.
/// The maps are boxed: most memos never get one, and there is a memo per pattern.
#[allow(clippy::box_collection)]
#[derive(Debug, Default)]
pub(crate) struct PatternMemo {
    pub end_attributes: Option<PatternId>,
    pub text_only: Option<PatternId>,
    pub end_tag_deriv: Option<PatternId>,
    pub mixed_text_deriv: Option<PatternId>,
    pub ignore_missing_attributes: Option<PatternId>,
    pub start_tag_open_deriv: Option<Box<HashMap<Name, PatternId>>>,
    pub start_tag_open_recover_deriv: Option<Box<HashMap<Name, PatternId>>>,
    pub start_attribute_deriv: Option<Box<HashMap<Name, PatternId>>>,
    pub data_deriv_type: Option<Box<DataDerivType>>,
}

pub struct ValidatorPatternBuilder {
    pub pb: PatternBuilder,
    pub(crate) memos: Vec<PatternMemo>,
    /// `choiceMap` of `NoteChoicesFunction` / `RemoveChoicesFunction`.
    choice_map: HashSet<PatternId>,
    pub(crate) datatypes: Vec<Box<dyn Datatype>>,
    /// `ValuePattern.getValue()`: the values of the schema's literals.
    pub(crate) values: Vec<Option<String>>,
    /// Patterns whose memo has a value table (see `clear_value_tables`).
    pub(crate) value_tables: Vec<PatternId>,
    base_len: usize,
    base_namespaces: usize,
    base_names: usize,
}

impl ValidatorPatternBuilder {
    pub fn new(schema: &Schema, factory: &dyn DatatypeLibraryFactory) -> Self {
        let pb = schema.pb.clone();
        let datatypes: Vec<Box<dyn Datatype>> = pb
            .datatypes
            .iter()
            .map(|dt| factory.create_datatype(dt))
            .collect();
        let values = pb
            .values
            .iter()
            .map(|(dt, lexical)| datatypes[dt.0 as usize].create_value(lexical))
            .collect();
        let base_len = pb.len();
        let base_namespaces = pb.names.namespaces().len();
        let base_names = pb.names.len();
        Self {
            pb,
            memos: Vec::new(),
            choice_map: HashSet::new(),
            datatypes,
            values,
            value_tables: Vec::new(),
            base_len,
            base_namespaces,
            base_names,
        }
    }

    /// Called between documents. Java builds a new builder per document; here the
    /// memos are kept as a cache, except the ones keyed by attribute values, and
    /// everything made while validating is dropped once there are more than
    /// `max_patterns` patterns or `max_names` names.
    pub fn reset_if_larger_than(&mut self, max_patterns: usize, max_names: usize) {
        self.clear_value_tables();
        if self.pb.len() > max_patterns || self.pb.names.len() > self.base_names + max_names {
            self.pb.truncate(self.base_len);
            self.pb
                .names
                .truncate(self.base_namespaces, self.base_names);
            self.memos.clear();
            self.value_tables.clear();
        }
    }

    #[inline]
    pub(crate) fn memo(&mut self, p: PatternId) -> &mut PatternMemo {
        let i = p.0 as usize;
        if self.memos.len() <= i {
            self.memos
                .resize_with(self.pb.len().max(i + 1), PatternMemo::default);
        }
        &mut self.memos[i]
    }

    pub fn make_after(&mut self, p1: PatternId, p2: PatternId) -> PatternId {
        self.pb.intern(Pattern::After(p1, p2))
    }

    pub fn make_group(&mut self, p1: PatternId, p2: PatternId) -> PatternId {
        self.pb.make_group(p1, p2)
    }

    pub fn make_interleave(&mut self, p1: PatternId, p2: PatternId) -> PatternId {
        self.pb.make_interleave(p1, p2)
    }

    pub fn make_one_or_more(&mut self, p: PatternId) -> PatternId {
        self.pb.make_one_or_more_base(p)
    }

    pub fn make_optional(&mut self, p: PatternId) -> PatternId {
        self.make_choice(p, EMPTY)
    }

    /// `ValidatorPatternBuilder.makeChoice`.
    pub fn make_choice(&mut self, p1: PatternId, mut p2: PatternId) -> PatternId {
        if p1 == p2 {
            return p1;
        }
        if p1 == NOT_ALLOWED {
            return p2;
        }
        if p2 == NOT_ALLOWED {
            return p1;
        }
        let p1_choice = matches!(self.pb.pattern(p1), Pattern::Choice(..));
        let p2_choice = matches!(self.pb.pattern(p2), Pattern::Choice(..));
        if !p1_choice {
            if self.pb.contains_choice(p2, p1) {
                return p2;
            }
        } else if !p2_choice {
            if self.pb.contains_choice(p1, p2) {
                return p1;
            }
        } else {
            self.note_choices(p1);
            p2 = self.remove_choices(p2);
            self.choice_map.clear();
        }
        if let (Pattern::After(a1, a2), Pattern::After(b1, b2)) =
            (self.pb.pattern(p1), self.pb.pattern(p2))
        {
            if a1 == b1 {
                let c = self.make_choice(a2, b2);
                return self.make_after(a1, c);
            }
            if a1 == NOT_ALLOWED {
                return p2;
            }
            if b1 == NOT_ALLOWED {
                return p1;
            }
            if a2 == b2 {
                let c = self.make_choice(a1, b1);
                return self.make_after(c, a2);
            }
        }
        self.pb.make_choice_base(p1, p2)
    }

    /// `NoteChoicesFunction`.
    fn note_choices(&mut self, p: PatternId) {
        match self.pb.pattern(p) {
            Pattern::Choice(p1, p2) => {
                self.note_choices(p1);
                self.note_choices(p2);
            }
            _ => {
                self.choice_map.insert(p);
            }
        }
    }

    /// `RemoveChoicesFunction`.
    fn remove_choices(&mut self, p: PatternId) -> PatternId {
        match self.pb.pattern(p) {
            Pattern::Choice(o1, o2) => {
                let p1 = self.remove_choices(o1);
                let p2 = self.remove_choices(o2);
                if p1 == o1 && p2 == o2 {
                    return p;
                }
                if p1 == NOT_ALLOWED {
                    return p2;
                }
                if p2 == NOT_ALLOWED {
                    return p1;
                }
                self.pb.intern(Pattern::Choice(p1, p2))
            }
            _ => {
                if self.choice_map.contains(&p) {
                    NOT_ALLOWED
                } else {
                    p
                }
            }
        }
    }

    // ------------------------------------------------------------------- memo

    /// `PatternMemo.endAttributes()`.
    pub fn end_attributes(&mut self, p: PatternId) -> PatternId {
        if let Some(r) = self.memo(p).end_attributes {
            return r;
        }
        let r = self.end_attributes_function(p);
        self.memo(p).end_attributes = Some(r);
        r
    }

    /// `PatternMemo.ignoreMissingAttributes()`.
    pub fn ignore_missing_attributes(&mut self, p: PatternId) -> PatternId {
        if let Some(r) = self.memo(p).ignore_missing_attributes {
            return r;
        }
        let r = self.ignore_missing_attributes_function(p);
        self.memo(p).ignore_missing_attributes = Some(r);
        r
    }

    /// `PatternMemo.textOnly()`.
    pub fn text_only(&mut self, p: PatternId) -> PatternId {
        if let Some(r) = self.memo(p).text_only {
            return r;
        }
        let r = self.text_only_function(p);
        self.memo(p).text_only = Some(r);
        r
    }

    /// `PatternMemo.endTagDeriv()`.
    pub fn end_tag_deriv(&mut self, p: PatternId) -> PatternId {
        if let Some(r) = self.memo(p).end_tag_deriv {
            return r;
        }
        let r = self.end_tag_deriv_function(p);
        self.memo(p).end_tag_deriv = Some(r);
        r
    }

    /// `PatternMemo.mixedTextDeriv()`.
    pub fn mixed_text_deriv(&mut self, p: PatternId) -> PatternId {
        if let Some(r) = self.memo(p).mixed_text_deriv {
            return r;
        }
        let r = self.mixed_text_deriv_function(p);
        self.memo(p).mixed_text_deriv = Some(r);
        r
    }

    /// `PatternMemo.startTagOpenDeriv(Name)`.
    pub fn start_tag_open_deriv(&mut self, p: PatternId, name: Name) -> PatternId {
        if let Some(&r) = self
            .memo(p)
            .start_tag_open_deriv
            .as_ref()
            .and_then(|m| m.get(&name))
        {
            return r;
        }
        let r = self.start_tag_open_deriv_function(p, name);
        self.memo(p)
            .start_tag_open_deriv
            .get_or_insert_with(Default::default)
            .insert(name, r);
        r
    }

    /// `PatternMemo.startTagOpenRecoverDeriv(Name)`.
    pub fn start_tag_open_recover_deriv(&mut self, p: PatternId, name: Name) -> PatternId {
        if let Some(&r) = self
            .memo(p)
            .start_tag_open_recover_deriv
            .as_ref()
            .and_then(|m| m.get(&name))
        {
            return r;
        }
        let r = self.start_tag_open_recover_deriv_function(p, name);
        self.memo(p)
            .start_tag_open_recover_deriv
            .get_or_insert_with(Default::default)
            .insert(name, r);
        r
    }

    /// `PatternMemo.startAttributeDeriv(Name)`.
    pub fn start_attribute_deriv(&mut self, p: PatternId, name: Name) -> PatternId {
        if let Some(&r) = self
            .memo(p)
            .start_attribute_deriv
            .as_ref()
            .and_then(|m| m.get(&name))
        {
            return r;
        }
        let r = self.start_attribute_deriv_function(p, name);
        self.memo(p)
            .start_attribute_deriv
            .get_or_insert_with(Default::default)
            .insert(name, r);
        r
    }

    /// `PatternMemo.recoverAfter()` (not memoized in Java either).
    pub fn recover_after(&mut self, p: PatternId) -> PatternId {
        self.recover_after_function(p)
    }
}
