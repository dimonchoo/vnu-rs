//! `com.thaiopensource.xml.util.Name`: a namespace URI plus a local name.
//!
//! Names are interned in a [`NameTable`] so that comparing two names (the hot path of
//! name-class matching and the derivative memo maps) is an integer comparison.

use std::collections::HashMap;

/// An interned namespace URI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Namespace(pub(crate) u32);

/// An interned (namespace URI, local name) pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(pub(crate) u32);

#[derive(Debug, Clone, Default)]
pub struct NameTable {
    namespaces: Vec<Box<str>>,
    namespace_ids: HashMap<Box<str>, Namespace>,
    /// Per namespace: local name to interned name.
    by_namespace: Vec<HashMap<Box<str>, Name>>,
    names: Vec<(Namespace, Box<str>)>,
}

impl NameTable {
    pub fn namespace(&mut self, uri: &str) -> Namespace {
        if let Some(ns) = self.find_namespace(uri) {
            return ns;
        }
        let ns = Namespace(self.namespaces.len() as u32);
        self.namespaces.push(uri.into());
        self.namespace_ids.insert(uri.into(), ns);
        self.by_namespace.push(HashMap::new());
        ns
    }

    pub fn find_namespace(&self, uri: &str) -> Option<Namespace> {
        self.namespace_ids.get(uri).copied()
    }

    pub fn name(&mut self, ns: Namespace, local: &str) -> Name {
        if let Some(&name) = self.by_namespace[ns.0 as usize].get(local) {
            return name;
        }
        let name = Name(self.names.len() as u32);
        self.names.push((ns, local.into()));
        self.by_namespace[ns.0 as usize].insert(local.into(), name);
        name
    }

    pub fn find_name(&self, ns: Namespace, local: &str) -> Option<Name> {
        self.by_namespace[ns.0 as usize].get(local).copied()
    }

    pub fn namespace_of(&self, name: Name) -> Namespace {
        self.names[name.0 as usize].0
    }

    pub fn local_name(&self, name: Name) -> &str {
        &self.names[name.0 as usize].1
    }

    pub fn namespace_uri(&self, ns: Namespace) -> &str {
        &self.namespaces[ns.0 as usize]
    }

    pub(crate) fn namespaces(&self) -> &[Box<str>] {
        &self.namespaces
    }

    pub(crate) fn names(&self) -> &[(Namespace, Box<str>)] {
        &self.names
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Drops names interned after the first `names` names and `namespaces` namespaces.
    pub(crate) fn truncate(&mut self, namespaces: usize, names: usize) {
        for (ns, local) in self.names.drain(names.min(self.names.len())..) {
            self.by_namespace[ns.0 as usize].remove(&local);
        }
        for uri in self
            .namespaces
            .drain(namespaces.min(self.namespaces.len())..)
        {
            self.namespace_ids.remove(&uri);
        }
        self.by_namespace.truncate(namespaces);
    }
}
