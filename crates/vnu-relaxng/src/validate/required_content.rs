//! Jing's `RequiredContent` (v.Nu fork), `FindElementFunction` and
//! `Pattern.accept(PatternVisitor)`.

use crate::name::Name;
use crate::name_class::SPECIFICITY_NONE;
use crate::pattern::*;
use crate::pattern_builder::*;
use crate::validator_pattern_builder::ValidatorPatternBuilder;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Required {
    Content,
    FrontierContent,
    Attributes,
}

impl ValidatorPatternBuilder {
    /// `RequiredContent.getRequiredContent`.
    pub fn get_required_content(&mut self, p: PatternId) -> PatternId {
        self.required(Required::Content, p)
    }

    /// `RequiredContent.getRequiredFrontierContent`.
    pub fn get_required_frontier_content(&mut self, p: PatternId) -> PatternId {
        self.required(Required::FrontierContent, p)
    }

    /// `RequiredContent.getRequiredAttributes`.
    pub fn get_required_attributes(&mut self, p: PatternId) -> PatternId {
        self.required(Required::Attributes, p)
    }

    fn required(&mut self, kind: Required, p: PatternId) -> PatternId {
        let nullable = self.pb.is_nullable(p);
        match self.pb.pattern(p) {
            Pattern::After(p1, _) => {
                if self.pb.is_nullable(p1) {
                    EMPTY
                } else {
                    self.required(kind, p1)
                }
            }
            Pattern::Group(p1, p2) if kind == Required::FrontierContent => {
                if nullable {
                    EMPTY
                } else if self.pb.is_nullable(p1) {
                    self.required(kind, p2)
                } else {
                    self.required(kind, p1)
                }
            }
            pattern @ (Pattern::Choice(p1, p2)
            | Pattern::Group(p1, p2)
            | Pattern::Interleave(p1, p2)) => {
                if nullable {
                    EMPTY
                } else if self.pb.is_nullable(p1) {
                    self.required(kind, p2)
                } else if self.pb.is_nullable(p2) {
                    self.required(kind, p1)
                } else {
                    let a = self.required(kind, p1);
                    let b = self.required(kind, p2);
                    match pattern {
                        Pattern::Choice(..) => self.make_choice(a, b),
                        Pattern::Group(..) => self.make_group(a, b),
                        _ => self.make_interleave(a, b),
                    }
                }
            }
            Pattern::OneOrMore(p1) => {
                if nullable {
                    EMPTY
                } else {
                    let r = self.required(kind, p1);
                    self.make_one_or_more(r)
                }
            }
            Pattern::Element(..) if kind == Required::Attributes => EMPTY,
            _ => {
                if nullable {
                    EMPTY
                } else {
                    p
                }
            }
        }
    }

    /// `FindElementFunction.findElement`.
    pub fn find_element(&mut self, name: Name, start: PatternId) -> PatternId {
        let mut f = FindElement {
            processed: HashSet::new(),
            specificity: SPECIFICITY_NONE,
            pattern: None,
        };
        self.find_element_apply(&mut f, name, start);
        f.pattern.unwrap_or(NOT_ALLOWED)
    }

    fn find_element_apply(&mut self, f: &mut FindElement, name: Name, p: PatternId) {
        match self.pb.pattern(p) {
            Pattern::Group(p1, p2) | Pattern::Interleave(p1, p2) | Pattern::Choice(p1, p2) => {
                if f.processed.insert(p) {
                    self.find_element_apply(f, name, p1);
                    self.find_element_apply(f, name, p2);
                }
            }
            Pattern::OneOrMore(p1) => {
                if f.processed.insert(p) {
                    self.find_element_apply(f, name, p1);
                }
            }
            Pattern::Element(nc, content) => {
                if f.processed.insert(p) {
                    let s = self
                        .pb
                        .name_classes
                        .contains_specificity(&self.pb.names, nc, name);
                    if s > f.specificity {
                        f.specificity = s;
                        f.pattern = Some(content);
                    } else if s == f.specificity && s != SPECIFICITY_NONE {
                        let prev = f.pattern.unwrap_or(NOT_ALLOWED);
                        f.pattern = Some(self.make_choice(prev, content));
                    }
                    self.find_element_apply(f, name, content);
                }
            }
            _ => {}
        }
    }
}

struct FindElement {
    processed: HashSet<PatternId>,
    specificity: i32,
    pattern: Option<PatternId>,
}
