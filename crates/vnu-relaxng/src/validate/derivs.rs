//! Jing's derivative pattern functions:
//! `StartTagOpenDerivFunction`, `StartTagOpenRecoverDerivFunction`,
//! `StartAttributeDerivFunction`, `ApplyAfterFunction`, `EndAttributesFunction`,
//! `IgnoreMissingAttributesFunction`, `TextOnlyFunction`, `MixedTextDerivFunction`,
//! `EndTagDerivFunction` and `RecoverAfterFunction`.
//!
//! The Java subclasses override a few cases of their base class; here each family is
//! one function with a `kind`, and `memo_apply` goes through the memo of that kind,
//! as each subclass's `apply(PatternMemo)` does.

use crate::name::Name;
use crate::pattern::*;
use crate::pattern_builder::*;
use crate::validator_pattern_builder::ValidatorPatternBuilder;

#[allow(clippy::enum_variant_names)] // Java class names
#[derive(Clone, Copy, PartialEq, Eq)]
enum StartKind {
    StartTagOpen,
    StartTagOpenRecover,
    StartAttribute,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EndAttributesKind {
    EndAttributes,
    IgnoreMissingAttributes,
    TextOnly,
    MixedTextDeriv,
}

/// The `apply(Pattern x)` bodies of the anonymous `ApplyAfterFunction` subclasses.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy)]
pub(crate) enum After {
    /// `makeGroup(x, p2)`
    GroupWith(PatternId),
    /// `makeGroup(p1, x)`
    GroupAfter(PatternId),
    /// `makeInterleave(x, p2)`
    InterleaveWith(PatternId),
    /// `makeInterleave(p1, x)`
    InterleaveAfter(PatternId),
    /// `makeAfter(x, p2)`
    AfterWith(PatternId),
    /// `makeGroup(x, makeOptional(p))`
    GroupOptional(PatternId),
    /// `makeEmpty()` (`PatternValidator.fixAfter`)
    Empty,
}

impl ValidatorPatternBuilder {
    /// `ApplyAfterFunction`.
    pub(crate) fn apply_after(&mut self, p: PatternId, f: After) -> PatternId {
        match self.pb.pattern(p) {
            Pattern::After(p1, p2) => {
                let x = self.apply_after_body(p2, f);
                self.make_after(p1, x)
            }
            Pattern::Choice(p1, p2) => {
                let a = self.apply_after(p1, f);
                let b = self.apply_after(p2, f);
                self.make_choice(a, b)
            }
            Pattern::NotAllowed => p,
            _ => {
                debug_assert!(false, "apply after botch");
                NOT_ALLOWED
            }
        }
    }

    fn apply_after_body(&mut self, x: PatternId, f: After) -> PatternId {
        match f {
            After::GroupWith(p2) => self.make_group(x, p2),
            After::GroupAfter(p1) => self.make_group(p1, x),
            After::InterleaveWith(p2) => self.make_interleave(x, p2),
            After::InterleaveAfter(p1) => self.make_interleave(p1, x),
            After::AfterWith(p2) => self.make_after(x, p2),
            After::GroupOptional(p) => {
                let opt = self.make_optional(p);
                self.make_group(x, opt)
            }
            After::Empty => EMPTY,
        }
    }

    // ------------------------------------------------- start tag / attribute

    pub(crate) fn start_tag_open_deriv_function(&mut self, p: PatternId, name: Name) -> PatternId {
        self.start_function(StartKind::StartTagOpen, p, name)
    }

    pub(crate) fn start_tag_open_recover_deriv_function(
        &mut self,
        p: PatternId,
        name: Name,
    ) -> PatternId {
        self.start_function(StartKind::StartTagOpenRecover, p, name)
    }

    pub(crate) fn start_attribute_deriv_function(&mut self, p: PatternId, name: Name) -> PatternId {
        self.start_function(StartKind::StartAttribute, p, name)
    }

    fn start_memo_apply(&mut self, kind: StartKind, p: PatternId, name: Name) -> PatternId {
        match kind {
            StartKind::StartTagOpen => self.start_tag_open_deriv(p, name),
            StartKind::StartTagOpenRecover => self.start_tag_open_recover_deriv(p, name),
            StartKind::StartAttribute => self.start_attribute_deriv(p, name),
        }
    }

    fn start_function(&mut self, kind: StartKind, p: PatternId, name: Name) -> PatternId {
        match self.pb.pattern(p) {
            Pattern::Choice(p1, p2) => {
                let a = self.start_memo_apply(kind, p1, name);
                let b = self.start_memo_apply(kind, p2, name);
                self.make_choice(a, b)
            }
            Pattern::Group(p1, p2) => {
                if kind == StartKind::StartAttribute {
                    let d1 = self.start_memo_apply(kind, p1, name);
                    let a = self.apply_after(d1, After::GroupWith(p2));
                    let d2 = self.start_memo_apply(kind, p2, name);
                    let b = self.apply_after(d2, After::GroupAfter(p1));
                    return self.make_choice(a, b);
                }
                let d1 = self.start_memo_apply(kind, p1, name);
                let tem = self.apply_after(d1, After::GroupWith(p2));
                let tem = if self.pb.is_nullable(p1) {
                    let d2 = self.start_memo_apply(kind, p2, name);
                    self.make_choice(tem, d2)
                } else {
                    tem
                };
                if kind == StartKind::StartTagOpenRecover && !self.pb.is_nullable(p1) {
                    let d2 = self.start_memo_apply(kind, p2, name);
                    return self.make_choice(tem, d2);
                }
                tem
            }
            Pattern::Interleave(p1, p2) => {
                let d1 = self.start_memo_apply(kind, p1, name);
                let a = self.apply_after(d1, After::InterleaveWith(p2));
                let d2 = self.start_memo_apply(kind, p2, name);
                let b = self.apply_after(d2, After::InterleaveAfter(p1));
                self.make_choice(a, b)
            }
            Pattern::After(p1, p2) => {
                let d1 = self.start_memo_apply(kind, p1, name);
                self.apply_after(d1, After::AfterWith(p2))
            }
            Pattern::OneOrMore(p1) => {
                let d1 = self.start_memo_apply(kind, p1, name);
                self.apply_after(d1, After::GroupOptional(p))
            }
            Pattern::Element(nc, content) => {
                if kind == StartKind::StartAttribute
                    || !self.pb.name_classes.contains(&self.pb.names, nc, name)
                {
                    return NOT_ALLOWED;
                }
                self.make_after(content, EMPTY)
            }
            Pattern::Attribute(nc, content) if kind == StartKind::StartAttribute => {
                if !self.pb.name_classes.contains(&self.pb.names, nc, name) {
                    return NOT_ALLOWED;
                }
                self.make_after(content, EMPTY)
            }
            _ => NOT_ALLOWED,
        }
    }

    // ---------------------------------------------------------- end attributes

    pub(crate) fn end_attributes_function(&mut self, p: PatternId) -> PatternId {
        self.end_attributes_family(EndAttributesKind::EndAttributes, p)
    }

    pub(crate) fn ignore_missing_attributes_function(&mut self, p: PatternId) -> PatternId {
        self.end_attributes_family(EndAttributesKind::IgnoreMissingAttributes, p)
    }

    pub(crate) fn text_only_function(&mut self, p: PatternId) -> PatternId {
        self.end_attributes_family(EndAttributesKind::TextOnly, p)
    }

    pub(crate) fn mixed_text_deriv_function(&mut self, p: PatternId) -> PatternId {
        self.end_attributes_family(EndAttributesKind::MixedTextDeriv, p)
    }

    fn end_memo_apply(&mut self, kind: EndAttributesKind, p: PatternId) -> PatternId {
        match kind {
            EndAttributesKind::EndAttributes => self.end_attributes(p),
            EndAttributesKind::IgnoreMissingAttributes => self.ignore_missing_attributes(p),
            EndAttributesKind::TextOnly => self.text_only(p),
            EndAttributesKind::MixedTextDeriv => self.mixed_text_deriv(p),
        }
    }

    fn end_attributes_family(&mut self, kind: EndAttributesKind, p: PatternId) -> PatternId {
        let mixed = kind == EndAttributesKind::MixedTextDeriv;
        match self.pb.pattern(p) {
            Pattern::Group(p1, p2) if mixed => {
                let q1 = self.end_memo_apply(kind, p1);
                let tem = if q1 == p1 { p } else { self.make_group(q1, p2) };
                if !self.pb.is_nullable(p1) {
                    return tem;
                }
                let q2 = self.end_memo_apply(kind, p2);
                self.make_choice(tem, q2)
            }
            Pattern::Interleave(p1, p2) if mixed => {
                let q1 = self.end_memo_apply(kind, p1);
                let q2 = self.end_memo_apply(kind, p2);
                let i1 = if q1 == p1 {
                    p
                } else {
                    self.make_interleave(q1, p2)
                };
                let i2 = if q2 == p2 {
                    p
                } else {
                    self.make_interleave(p1, q2)
                };
                self.make_choice(i1, i2)
            }
            Pattern::OneOrMore(p1) if mixed => {
                let q1 = self.end_memo_apply(kind, p1);
                let opt = self.make_optional(p);
                self.make_group(q1, opt)
            }
            Pattern::Text if mixed => p,
            Pattern::Group(p1, p2) => {
                let q1 = self.end_memo_apply(kind, p1);
                let q2 = self.end_memo_apply(kind, p2);
                if p1 == q1 && p2 == q2 {
                    return p;
                }
                self.make_group(q1, q2)
            }
            Pattern::Interleave(p1, p2) => {
                let q1 = self.end_memo_apply(kind, p1);
                let q2 = self.end_memo_apply(kind, p2);
                if p1 == q1 && p2 == q2 {
                    return p;
                }
                self.make_interleave(q1, q2)
            }
            Pattern::Choice(p1, p2) => {
                let q1 = self.end_memo_apply(kind, p1);
                let q2 = self.end_memo_apply(kind, p2);
                if p1 == q1 && p2 == q2 {
                    return p;
                }
                self.make_choice(q1, q2)
            }
            Pattern::OneOrMore(p1) => {
                let q1 = self.end_memo_apply(kind, p1);
                if p1 == q1 {
                    return p;
                }
                // Java passes the operand, not its derivative.
                self.make_one_or_more(p1)
            }
            Pattern::After(p1, p2) => {
                let q1 = self.end_memo_apply(kind, p1);
                if p1 == q1 {
                    return p;
                }
                self.make_after(q1, p2)
            }
            Pattern::Attribute(..) => match kind {
                EndAttributesKind::IgnoreMissingAttributes => EMPTY,
                EndAttributesKind::TextOnly => p,
                _ => NOT_ALLOWED,
            },
            Pattern::Element(..) if kind == EndAttributesKind::TextOnly => NOT_ALLOWED,
            _ if mixed => NOT_ALLOWED,
            _ => p,
        }
    }

    // ---------------------------------------------------------------- end tag

    pub(crate) fn end_tag_deriv_function(&mut self, p: PatternId) -> PatternId {
        match self.pb.pattern(p) {
            Pattern::Choice(p1, p2) => {
                let a = self.end_tag_deriv(p1);
                let b = self.end_tag_deriv(p2);
                self.make_choice(a, b)
            }
            Pattern::After(p1, p2) => {
                if self.pb.is_nullable(p1) {
                    p2
                } else {
                    NOT_ALLOWED
                }
            }
            _ => NOT_ALLOWED,
        }
    }

    pub(crate) fn recover_after_function(&mut self, p: PatternId) -> PatternId {
        match self.pb.pattern(p) {
            Pattern::Choice(p1, p2) => {
                let a = self.recover_after_function(p1);
                let b = self.recover_after_function(p2);
                self.make_choice(a, b)
            }
            Pattern::After(_, p2) => p2,
            Pattern::NotAllowed => p,
            _ => {
                debug_assert!(false, "recover after botch");
                NOT_ALLOWED
            }
        }
    }
}
