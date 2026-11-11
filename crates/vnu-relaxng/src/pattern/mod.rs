//! Jing's `Pattern` class hierarchy (`EmptyPattern`, `NotAllowedPattern`, `ChoicePattern`,
//! `GroupPattern`, `InterleavePattern`, `OneOrMorePattern`, `AfterPattern`,
//! `ElementPattern`, `AttributePattern`, `DataPattern`, `DataExceptPattern`,
//! `ValuePattern`, `ListPattern`, `TextPattern`, `ErrorPattern`, `RefPattern`,
//! `UnexpandedNotAllowedPattern`).
//!
//! Patterns live in an arena owned by a [`crate::PatternBuilder`] and are referred to
//! by [`PatternId`]. Java compares patterns by identity; here that is id equality.

pub mod datatype;
pub mod name;
pub mod name_class;
pub mod pattern_builder;

use crate::datatype::{DatatypeId, ValueId};
use crate::name_class::NameClassId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PatternId(pub(crate) u32);

/// Index of a `RefPattern` (named definition) in the schema builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RefId(pub(crate) u32);

pub const EMPTY_CONTENT_TYPE: u8 = 0;
pub const ELEMENT_CONTENT_TYPE: u8 = 1;
pub const MIXED_CONTENT_TYPE: u8 = 2;
pub const DATA_CONTENT_TYPE: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pattern {
    Empty,
    NotAllowed,
    /// `UnexpandedNotAllowedPattern`: `notAllowed` written in the schema, which turns
    /// into [`Pattern::NotAllowed`] when the schema is expanded.
    UnexpandedNotAllowed,
    Error,
    Text,
    Choice(PatternId, PatternId),
    Group(PatternId, PatternId),
    Interleave(PatternId, PatternId),
    After(PatternId, PatternId),
    OneOrMore(PatternId),
    /// Name class and content. Both change once, when the schema is expanded.
    Element(NameClassId, PatternId),
    Attribute(NameClassId, PatternId),
    Data(DatatypeId),
    DataExcept(DatatypeId, PatternId),
    Value(DatatypeId, ValueId),
    List(PatternId),
    Ref(RefId),
}

#[derive(Debug, Clone, Copy)]
pub struct PatternNode {
    pub pattern: Pattern,
    pub nullable: bool,
    pub content_type: u8,
}

/// Callbacks of Jing's `PatternVisitor`.
pub trait PatternVisitor {
    fn visit_empty(&mut self) {}
    fn visit_not_allowed(&mut self) {}
    fn visit_error(&mut self) {}
    fn visit_group(&mut self, p1: PatternId, p2: PatternId);
    fn visit_interleave(&mut self, p1: PatternId, p2: PatternId);
    fn visit_choice(&mut self, p1: PatternId, p2: PatternId);
    fn visit_one_or_more(&mut self, p: PatternId);
    fn visit_element(&mut self, nc: NameClassId, content: PatternId);
    fn visit_attribute(&mut self, nc: NameClassId, value: PatternId);
    fn visit_data(&mut self, _dt: DatatypeId) {}
    fn visit_data_except(&mut self, _dt: DatatypeId, _except: PatternId) {}
    fn visit_value(&mut self, _dt: DatatypeId, _value: ValueId) {}
    fn visit_text(&mut self) {}
    fn visit_list(&mut self, p: PatternId);
}

impl Pattern {
    /// `BinaryPattern.getOperand1()/getOperand2()`.
    pub fn operands(self) -> Option<(PatternId, PatternId)> {
        match self {
            Pattern::Choice(p1, p2)
            | Pattern::Group(p1, p2)
            | Pattern::Interleave(p1, p2)
            | Pattern::After(p1, p2) => Some((p1, p2)),
            _ => None,
        }
    }
}
