//! Jing's `SchemaPatternBuilder` and the `Pattern.expand` methods: building patterns
//! while the schema is read, then replacing references by what they refer to.

use crate::datatype::{DatatypeId, ValueId};
use crate::name_class::{NameClass, NameClassId};
use crate::pattern::*;
use crate::pattern_builder::*;
use std::collections::HashMap;

/// A named definition (`RefPattern`).
#[derive(Debug, Clone)]
pub struct RefPattern {
    pub pattern: Option<PatternId>,
    pub name: Option<String>,
    pub combine_implicit: bool,
    pub combine_type: u8,
    pub replacement_status: u8,
    expanded: Option<PatternId>,
}

pub const REPLACEMENT_KEEP: u8 = 0;
pub const REPLACEMENT_REQUIRE: u8 = 1;
pub const REPLACEMENT_IGNORE: u8 = 2;

pub const COMBINE_NONE: u8 = 0;
pub const COMBINE_CHOICE: u8 = 1;
pub const COMBINE_INTERLEAVE: u8 = 2;

#[derive(Debug, Default)]
pub struct SchemaPatternBuilder {
    pub pb: PatternBuilder,
    /// `schemaInterner`: element, attribute, data, value, list and error patterns.
    schema_interner: HashMap<Pattern, PatternId>,
    pub refs: Vec<RefPattern>,
    expanded_elements: Vec<bool>,
}

impl SchemaPatternBuilder {
    fn schema_intern(&mut self, p: Pattern) -> PatternId {
        if let Some(&id) = self.schema_interner.get(&p) {
            return id;
        }
        let id = self.pb.alloc(p);
        self.schema_interner.insert(p, id);
        id
    }

    pub fn make_ref(&mut self, name: Option<String>) -> RefId {
        let id = RefId(self.refs.len() as u32);
        self.refs.push(RefPattern {
            pattern: None,
            name,
            combine_implicit: false,
            combine_type: COMBINE_NONE,
            replacement_status: REPLACEMENT_KEEP,
            expanded: None,
        });
        id
    }

    pub fn ref_pattern(&mut self, r: RefId) -> PatternId {
        self.schema_intern(Pattern::Ref(r))
    }

    pub fn make_element(&mut self, nc: NameClassId, content: PatternId) -> PatternId {
        self.schema_intern(Pattern::Element(nc, content))
    }

    pub fn make_attribute(&mut self, nc: NameClassId, value: PatternId) -> PatternId {
        if value == NOT_ALLOWED {
            return value;
        }
        self.schema_intern(Pattern::Attribute(nc, value))
    }

    pub fn make_data(&mut self, dt: DatatypeId) -> PatternId {
        self.schema_intern(Pattern::Data(dt))
    }

    pub fn make_data_except(&mut self, dt: DatatypeId, except: PatternId) -> PatternId {
        self.schema_intern(Pattern::DataExcept(dt, except))
    }

    pub fn make_value(&mut self, dt: DatatypeId, value: ValueId) -> PatternId {
        self.schema_intern(Pattern::Value(dt, value))
    }

    pub fn make_text(&self) -> PatternId {
        TEXT
    }

    pub fn make_unexpanded_not_allowed(&self) -> PatternId {
        UNEXPANDED_NOT_ALLOWED
    }

    pub fn make_empty(&self) -> PatternId {
        EMPTY
    }

    pub fn make_group(&mut self, p1: PatternId, p2: PatternId) -> PatternId {
        self.pb.make_group(p1, p2)
    }

    pub fn make_interleave(&mut self, p1: PatternId, p2: PatternId) -> PatternId {
        self.pb.make_interleave(p1, p2)
    }

    pub fn make_choice(&mut self, p1: PatternId, p2: PatternId) -> PatternId {
        if p1 == NOT_ALLOWED || p1 == p2 {
            return p2;
        }
        if p2 == NOT_ALLOWED {
            return p1;
        }
        self.pb.make_choice_base(p1, p2)
    }

    pub fn make_one_or_more(&mut self, p: PatternId) -> PatternId {
        if p == TEXT {
            return p;
        }
        self.pb.make_one_or_more_base(p)
    }

    pub fn make_optional(&mut self, p: PatternId) -> PatternId {
        self.make_choice(p, EMPTY)
    }

    pub fn make_zero_or_more(&mut self, p: PatternId) -> PatternId {
        let p = self.make_one_or_more(p);
        self.make_optional(p)
    }

    pub fn make_list(&mut self, p: PatternId) -> PatternId {
        if p == NOT_ALLOWED {
            return p;
        }
        self.schema_intern(Pattern::List(p))
    }

    pub fn make_mixed(&mut self, p: PatternId) -> PatternId {
        self.make_interleave(TEXT, p)
    }

    pub fn null_name_class(&mut self) -> NameClassId {
        self.pb.name_classes.intern(NameClass::Null)
    }

    // ------------------------------------------------------------------ expand

    /// `Pattern.expand(SchemaPatternBuilder)`.
    pub fn expand(&mut self, p: PatternId) -> PatternId {
        match self.pb.pattern(p) {
            Pattern::UnexpandedNotAllowed => NOT_ALLOWED,
            Pattern::Choice(p1, p2) => {
                let (ep1, ep2) = (self.expand(p1), self.expand(p2));
                if ep1 != p1 || ep2 != p2 {
                    self.make_choice(ep1, ep2)
                } else {
                    p
                }
            }
            Pattern::Group(p1, p2) => {
                let (ep1, ep2) = (self.expand(p1), self.expand(p2));
                if ep1 != p1 || ep2 != p2 {
                    self.make_group(ep1, ep2)
                } else {
                    p
                }
            }
            Pattern::Interleave(p1, p2) => {
                let (ep1, ep2) = (self.expand(p1), self.expand(p2));
                if ep1 != p1 || ep2 != p2 {
                    self.make_interleave(ep1, ep2)
                } else {
                    p
                }
            }
            Pattern::OneOrMore(p1) => {
                let ep = self.expand(p1);
                if ep != p1 {
                    self.make_one_or_more(ep)
                } else {
                    p
                }
            }
            Pattern::Element(nc, content) => {
                let index = p.0 as usize;
                if self.expanded_elements.len() <= index {
                    self.expanded_elements.resize(index + 1, false);
                }
                if !self.expanded_elements[index] {
                    self.expanded_elements[index] = true;
                    let ec = self.expand(content);
                    let nc = if self.pb.is_not_allowed(ec) {
                        self.null_name_class()
                    } else {
                        nc
                    };
                    self.pb.set_pattern(p, Pattern::Element(nc, ec));
                }
                p
            }
            Pattern::Attribute(nc, value) => {
                let ep = self.expand(value);
                if ep != value {
                    self.make_attribute(nc, ep)
                } else {
                    p
                }
            }
            Pattern::List(p1) => {
                let ep = self.expand(p1);
                if ep != p1 { self.make_list(ep) } else { p }
            }
            Pattern::Ref(r) => {
                if let Some(e) = self.refs[r.0 as usize].expanded {
                    return e;
                }
                // Like Java, the flag is set after expanding: recursion always goes
                // through an element, whose own flag stops it.
                let target = self.refs[r.0 as usize].pattern.unwrap_or(ERROR);
                let e = self.expand(target);
                self.refs[r.0 as usize].expanded = Some(e);
                e
            }
            _ => p,
        }
    }
}
