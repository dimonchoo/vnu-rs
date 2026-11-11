//! Jing's data derivatives: `DataDerivType` with `SingleDataDerivType`,
//! `BlankDataDerivType`, `DataDataDerivType`, `ValueDataDerivType` and
//! `InconsistentDataDerivType`; `DataDerivTypeFunction`; `DataDerivFunction`.

use crate::datatype::{DatatypeException, DatatypeId};
use crate::pattern::*;
use crate::pattern_builder::*;
use crate::validator_pattern_builder::ValidatorPatternBuilder;
use std::collections::HashMap;

/// Datatype errors collected while deriving (`PatternValidator.datatypeErrors`),
/// keyed by message like the Java map.
pub type DatatypeErrors = Vec<(String, DatatypeException)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataDerivKind {
    Single,
    Blank,
    Data(DatatypeId),
    Value(DatatypeId),
    Inconsistent,
}

/// A memo's `DataDerivType`: its kind and the derivatives it caches.
#[derive(Debug)]
pub(crate) struct DataDerivType {
    kind: DataDerivKind,
    /// Single: the derivative. Blank: for blank strings. Data: for valid strings.
    /// Value: for strings that are not a value of the datatype.
    first: Option<PatternId>,
    /// Blank: for non-blank strings. Data: for invalid strings.
    second: Option<PatternId>,
    /// Value: by value.
    value_table: Option<HashMap<String, PatternId>>,
}

impl DataDerivType {
    fn new(kind: DataDerivKind) -> Self {
        Self {
            kind,
            first: None,
            second: None,
            value_table: None,
        }
    }
}

impl DataDerivKind {
    /// `DataDerivType.combine`.
    fn combine(self, other: DataDerivKind) -> DataDerivKind {
        use DataDerivKind::*;
        match (self, other) {
            (Single, x) => x,
            (Blank, Blank | Single) => Blank,
            (Blank, _) => Inconsistent,
            (Data(a), Data(b)) => {
                if a == b {
                    self
                } else {
                    Inconsistent
                }
            }
            (Data(a), Value(b)) => {
                if a == b {
                    other
                } else {
                    Inconsistent
                }
            }
            (Data(_), x) => x.combine(self),
            (Value(a), Value(b)) => {
                if a == b {
                    self
                } else {
                    Inconsistent
                }
            }
            (Value(_), x) => x.combine(self),
            (Inconsistent, _) => Inconsistent,
        }
    }
}

/// `DataDerivFunction.isBlank`.
pub fn is_blank(s: &str) -> bool {
    s.bytes().all(|b| matches!(b, b'\r' | b'\n' | b' ' | b'\t'))
}

impl ValidatorPatternBuilder {
    /// `DataPattern.allowsAnyString()`.
    fn allows_any_string(&self, dt: DatatypeId) -> bool {
        self.datatypes[dt.0 as usize].always_valid()
    }

    /// `PatternMemo.dataDerivType()`.
    fn data_deriv_kind(&mut self, p: PatternId) -> DataDerivKind {
        if let Some(t) = &self.memo(p).data_deriv_type {
            return t.kind;
        }
        let kind = self.data_deriv_type_function(p);
        self.memo(p).data_deriv_type = Some(Box::new(DataDerivType::new(kind)));
        kind
    }

    /// `DataDerivTypeFunction`.
    fn data_deriv_type_function(&mut self, p: PatternId) -> DataDerivKind {
        match self.pb.pattern(p) {
            Pattern::After(p1, _) => {
                let ddt = self.data_deriv_kind(p1);
                if !self.pb.is_nullable(p1) {
                    ddt
                } else {
                    ddt.combine(DataDerivKind::Blank)
                }
            }
            Pattern::Choice(p1, p2) | Pattern::Group(p1, p2) | Pattern::Interleave(p1, p2) => {
                let a = self.data_deriv_kind(p1);
                let b = self.data_deriv_kind(p2);
                a.combine(b)
            }
            Pattern::OneOrMore(p1) => self.data_deriv_kind(p1),
            Pattern::List(_) => DataDerivKind::Inconsistent,
            Pattern::Value(dt, _) => DataDerivKind::Value(dt),
            Pattern::Data(dt) => {
                if self.allows_any_string(dt) {
                    DataDerivKind::Single
                } else {
                    DataDerivKind::Data(dt)
                }
            }
            Pattern::DataExcept(dt, except) => {
                if self.allows_any_string(dt) {
                    self.data_deriv_kind(except)
                } else {
                    let e = self.data_deriv_kind(except);
                    DataDerivKind::Data(dt).combine(e)
                }
            }
            _ => DataDerivKind::Single,
        }
    }

    /// `PatternMemo.dataDeriv(str, vc)`: the derivative types cache the result.
    pub fn data_deriv(&mut self, p: PatternId, s: &str, errors: &mut DatatypeErrors) -> PatternId {
        let kind = self.data_deriv_kind(p);
        let cached = {
            let t = self.ddt(p);
            match kind {
                DataDerivKind::Single => t.first,
                DataDerivKind::Blank => {
                    if is_blank(s) {
                        t.first
                    } else {
                        t.second
                    }
                }
                _ => None,
            }
        };
        if let Some(r) = cached {
            return r;
        }
        match kind {
            DataDerivKind::Single => {
                let r = self.data_deriv_function(p, s, errors);
                self.ddt(p).first = Some(r);
                r
            }
            DataDerivKind::Blank => {
                let r = self.data_deriv_function(p, s, errors);
                if is_blank(s) {
                    self.ddt(p).first = Some(r);
                } else {
                    self.ddt(p).second = Some(r);
                }
                r
            }
            DataDerivKind::Data(dt) => match self.datatypes[dt.0 as usize].check_valid(s) {
                Ok(()) => {
                    if let Some(r) = self.ddt(p).first {
                        return r;
                    }
                    let r = self.data_deriv_function(p, s, errors);
                    self.ddt(p).first = Some(r);
                    r
                }
                Err(e) => {
                    if let Some(msg) = e.message.clone().filter(|m| !m.is_empty()) {
                        if !errors.iter().any(|(m, _)| *m == msg) {
                            errors.push((msg, e));
                        }
                    }
                    if let Some(r) = self.ddt(p).second {
                        return r;
                    }
                    let r = self.data_deriv_function(p, s, errors);
                    self.ddt(p).second = Some(r);
                    r
                }
            },
            DataDerivKind::Value(dt) => match self.datatypes[dt.0 as usize].create_value(s) {
                None => {
                    if let Some(r) = self.ddt(p).first {
                        return r;
                    }
                    let r = self.data_deriv_function(p, s, errors);
                    self.ddt(p).first = Some(r);
                    r
                }
                Some(value) => {
                    if let Some(&r) = self.ddt(p).value_table.as_ref().and_then(|t| t.get(&value)) {
                        return r;
                    }
                    let r = self.data_deriv_function(p, s, errors);
                    let table = &mut self.ddt(p).value_table;
                    if table.is_none() {
                        self.value_tables.push(p);
                    }
                    self.ddt(p)
                        .value_table
                        .get_or_insert_with(HashMap::new)
                        .insert(value, r);
                    r
                }
            },
            DataDerivKind::Inconsistent => self.data_deriv_function(p, s, errors),
        }
    }

    /// Drops the derivatives memoized per attribute value: their keys come from the
    /// documents, so they would grow without bound across documents.
    pub(crate) fn clear_value_tables(&mut self) {
        for p in std::mem::take(&mut self.value_tables) {
            self.ddt(p).value_table = None;
        }
    }

    /// The memo's derivative type; [`Self::data_deriv_kind`] has set it already.
    fn ddt(&mut self, p: PatternId) -> &mut DataDerivType {
        self.memo(p)
            .data_deriv_type
            .get_or_insert_with(|| Box::new(DataDerivType::new(DataDerivKind::Inconsistent)))
    }

    /// `DataDerivFunction`.
    fn data_deriv_function(
        &mut self,
        p: PatternId,
        s: &str,
        errors: &mut DatatypeErrors,
    ) -> PatternId {
        match self.pb.pattern(p) {
            Pattern::Text => p,
            Pattern::List(op) => {
                let mut memo = op;
                for token in s.split(['\r', '\n', ' ', '\t']).filter(|t| !t.is_empty()) {
                    memo = self.data_deriv(memo, token, errors);
                }
                if self.pb.is_nullable(memo) {
                    EMPTY
                } else {
                    NOT_ALLOWED
                }
            }
            Pattern::Value(dt, v) => {
                let datatype = &self.datatypes[dt.0 as usize];
                match (datatype.create_value(s), &self.values[v.0 as usize]) {
                    (Some(value), Some(expected)) if datatype.same_value(expected, &value) => EMPTY,
                    _ => NOT_ALLOWED,
                }
            }
            Pattern::Data(dt) => self.data_case(dt, s),
            Pattern::DataExcept(dt, except) => {
                let tem = self.data_case(dt, s);
                if self.pb.is_nullable(tem) {
                    let e = self.data_deriv(except, s, errors);
                    if self.pb.is_nullable(e) {
                        return NOT_ALLOWED;
                    }
                }
                tem
            }
            Pattern::After(p1, p2) => {
                let d1 = self.data_deriv(p1, s, errors);
                if self.pb.is_nullable(d1) || (self.pb.is_nullable(p1) && is_blank(s)) {
                    p2
                } else {
                    NOT_ALLOWED
                }
            }
            Pattern::Choice(p1, p2) => {
                let a = self.data_deriv(p1, s, errors);
                let b = self.data_deriv(p2, s, errors);
                self.make_choice(a, b)
            }
            Pattern::Group(p1, p2) => {
                let d1 = self.data_deriv(p1, s, errors);
                let tem = self.make_group(d1, p2);
                if !self.pb.is_nullable(p1) {
                    return tem;
                }
                let d2 = self.data_deriv(p2, s, errors);
                self.make_choice(tem, d2)
            }
            Pattern::Interleave(p1, p2) => {
                let d1 = self.data_deriv(p1, s, errors);
                let a = self.make_interleave(d1, p2);
                let d2 = self.data_deriv(p2, s, errors);
                let b = self.make_interleave(p1, d2);
                self.make_choice(a, b)
            }
            Pattern::OneOrMore(op) => {
                let d = self.data_deriv(op, s, errors);
                let opt = self.make_optional(p);
                self.make_group(d, opt)
            }
            _ => NOT_ALLOWED,
        }
    }

    /// `DataDerivFunction.caseData`.
    fn data_case(&self, dt: DatatypeId, s: &str) -> PatternId {
        if self.allows_any_string(dt) || self.datatypes[dt.0 as usize].is_valid(s) {
            EMPTY
        } else {
            NOT_ALLOWED
        }
    }
}
