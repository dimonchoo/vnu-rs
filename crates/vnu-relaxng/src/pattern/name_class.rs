//! Jing's name classes (`NameClass`, `SimpleNameClass`, `AnyNameClass`,
//! `AnyNameExceptNameClass`, `NsNameClass`, `NsNameExceptNameClass`, `ChoiceNameClass`,
//! `NullNameClass`).
//!
//! Name classes are interned in a [`NameClasses`] table, so structural equality
//! (Java's `equals`) is id equality.

use crate::name::{Name, NameTable, Namespace};
use std::collections::HashMap;

pub const SPECIFICITY_NONE: i32 = -1;
pub const SPECIFICITY_ANY_NAME: i32 = 0;
pub const SPECIFICITY_NS_NAME: i32 = 1;
pub const SPECIFICITY_NAME: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NameClassId(pub(crate) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameClass {
    Simple(Name),
    AnyName,
    AnyNameExcept(NameClassId),
    NsName(Namespace),
    NsNameExcept(Namespace, NameClassId),
    Choice(NameClassId, NameClassId),
    Null,
}

/// Callbacks of Jing's `NameClassVisitor`.
pub trait NameClassVisitor {
    fn visit_choice(&mut self, classes: &NameClasses, nc1: NameClassId, nc2: NameClassId);
    fn visit_ns_name(&mut self, _ns: Namespace) {}
    fn visit_ns_name_except(&mut self, classes: &NameClasses, _ns: Namespace, nc: NameClassId) {
        classes.accept(nc, self);
    }
    fn visit_any_name(&mut self) {}
    fn visit_any_name_except(&mut self, classes: &NameClasses, nc: NameClassId) {
        classes.accept(nc, self);
    }
    fn visit_name(&mut self, name: Name);
    fn visit_null(&mut self) {}
}

#[derive(Debug, Clone, Default)]
pub struct NameClasses {
    classes: Vec<NameClass>,
    interner: HashMap<NameClass, NameClassId>,
}

impl NameClasses {
    pub fn intern(&mut self, nc: NameClass) -> NameClassId {
        if let Some(&id) = self.interner.get(&nc) {
            return id;
        }
        let id = NameClassId(self.classes.len() as u32);
        self.classes.push(nc);
        self.interner.insert(nc, id);
        id
    }

    pub fn get(&self, id: NameClassId) -> NameClass {
        self.classes[id.0 as usize]
    }

    pub(crate) fn all(&self) -> &[NameClass] {
        &self.classes
    }

    pub fn contains(&self, names: &NameTable, id: NameClassId, name: Name) -> bool {
        match self.get(id) {
            NameClass::Simple(n) => n == name,
            NameClass::AnyName => true,
            NameClass::AnyNameExcept(nc) => !self.contains(names, nc, name),
            NameClass::NsName(ns) => names.namespace_of(name) == ns,
            NameClass::NsNameExcept(ns, nc) => {
                names.namespace_of(name) == ns && !self.contains(names, nc, name)
            }
            NameClass::Choice(nc1, nc2) => {
                self.contains(names, nc1, name) || self.contains(names, nc2, name)
            }
            NameClass::Null => false,
        }
    }

    pub fn contains_specificity(&self, names: &NameTable, id: NameClassId, name: Name) -> i32 {
        let specificity = |contained: bool, s: i32| {
            if contained { s } else { SPECIFICITY_NONE }
        };
        match self.get(id) {
            NameClass::Simple(_) => specificity(self.contains(names, id, name), SPECIFICITY_NAME),
            NameClass::AnyName => SPECIFICITY_ANY_NAME,
            NameClass::AnyNameExcept(_) => {
                specificity(self.contains(names, id, name), SPECIFICITY_ANY_NAME)
            }
            NameClass::NsName(_) | NameClass::NsNameExcept(..) => {
                specificity(self.contains(names, id, name), SPECIFICITY_NS_NAME)
            }
            NameClass::Choice(nc1, nc2) => self
                .contains_specificity(names, nc1, name)
                .max(self.contains_specificity(names, nc2, name)),
            NameClass::Null => SPECIFICITY_NONE,
        }
    }

    pub fn is_open(&self, id: NameClassId) -> bool {
        match self.get(id) {
            NameClass::Simple(_) | NameClass::Null => false,
            NameClass::AnyName
            | NameClass::AnyNameExcept(_)
            | NameClass::NsName(_)
            | NameClass::NsNameExcept(..) => true,
            NameClass::Choice(nc1, nc2) => self.is_open(nc1) || self.is_open(nc2),
        }
    }

    pub fn accept<V: NameClassVisitor + ?Sized>(&self, id: NameClassId, visitor: &mut V) {
        match self.get(id) {
            NameClass::Simple(name) => visitor.visit_name(name),
            NameClass::AnyName => visitor.visit_any_name(),
            NameClass::AnyNameExcept(nc) => visitor.visit_any_name_except(self, nc),
            NameClass::NsName(ns) => visitor.visit_ns_name(ns),
            NameClass::NsNameExcept(ns, nc) => visitor.visit_ns_name_except(self, ns, nc),
            NameClass::Choice(nc1, nc2) => visitor.visit_choice(self, nc1, nc2),
            NameClass::Null => visitor.visit_null(),
        }
    }
}
