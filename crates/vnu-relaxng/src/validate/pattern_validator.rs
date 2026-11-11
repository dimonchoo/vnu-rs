//! Jing's `PatternValidator` (v.Nu fork) with its `NameRecordingVisitor`, and the
//! validation exceptions of `com.thaiopensource.relaxng.exceptions`.

use crate::data_deriv::DatatypeErrors;
use crate::datatype::DatatypeLibraryFactory;
use crate::derivs::After;
use crate::name::Name;
use crate::name_class::{NameClassId, NameClassVisitor, NameClasses};
use crate::pattern::*;
use crate::pattern_builder::*;
use crate::schema::Schema;
use crate::validator_pattern_builder::ValidatorPatternBuilder;
use std::collections::{BTreeSet, HashMap};

/// The exceptions `PatternValidator` reports. `current` is the element the error is
/// about and `parent` its parent (`peek()`), as in `AbstractValidationException`.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    RequiredElementsMissingOneOf {
        current: Name,
        missing: BTreeSet<String>,
        parent: Option<Name>,
    },
    RequiredElementsMissing {
        current: Name,
        missing: Option<String>,
        parent: Option<Name>,
    },
    UnknownElement {
        current: Name,
        parent: Option<Name>,
    },
    OutOfContextElement {
        current: Name,
        parent: Option<Name>,
    },
    ImpossibleAttributeIgnored {
        current: Name,
        parent: Option<Name>,
        attribute: Name,
    },
    BadAttributeValue {
        current: Name,
        parent: Option<Name>,
        attribute: Name,
        value: String,
        errors: DatatypeErrors,
    },
    RequiredAttributesMissingOneOf {
        current: Name,
        attributes: BTreeSet<String>,
        parent: Option<Name>,
    },
    RequiredAttributesMissing {
        current: Name,
        attribute: String,
        parent: Option<Name>,
    },
    OnlyTextNotAllowed {
        current: Name,
        parent: Option<Name>,
    },
    StringNotAllowed {
        current: Name,
        parent: Option<Name>,
        value: String,
        errors: DatatypeErrors,
    },
    UnfinishedElementOneOf {
        current: Name,
        missing: BTreeSet<String>,
        parent: Option<Name>,
    },
    UnfinishedElement {
        current: Name,
        missing: String,
        parent: Option<Name>,
    },
    TextNotAllowed {
        current: Option<Name>,
    },
    SchemaAllowsNothing,
}

/// An attribute as SAX reports it.
#[derive(Debug, Clone, Copy)]
pub struct Attribute<'a> {
    pub namespace_uri: &'a str,
    pub local_name: &'a str,
    pub value: &'a str,
}

/// Patterns and names made while validating beyond these are dropped between
/// documents.
const MAX_PATTERNS: usize = 1 << 20;
const MAX_NEW_NAMES: usize = 1 << 16;

pub struct PatternValidator {
    builder: ValidatorPatternBuilder,
    start: PatternId,
    recover_pattern_table: HashMap<Name, PatternId>,
    memo: PatternId,
    had_error: bool,
    collecting_characters: bool,
    char_buf: String,
    datatype_errors: DatatypeErrors,
    stack: Vec<Name>,
    suppress_depth: usize,
    errors: Vec<ValidationError>,
}

impl std::fmt::Debug for PatternValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PatternValidator")
            .field("memo", &self.memo)
            .field("stack", &self.stack)
            .field("suppress_depth", &self.suppress_depth)
            .field("patterns", &self.builder.pb.len())
            .finish_non_exhaustive()
    }
}

impl PatternValidator {
    pub fn new(schema: &Schema, factory: &dyn DatatypeLibraryFactory) -> Self {
        let builder = ValidatorPatternBuilder::new(schema, factory);
        let mut v = Self {
            builder,
            start: schema.start,
            recover_pattern_table: HashMap::new(),
            memo: schema.start,
            had_error: false,
            collecting_characters: false,
            char_buf: String::new(),
            datatype_errors: Vec::new(),
            stack: Vec::with_capacity(48),
            suppress_depth: 0,
            errors: Vec::new(),
        };
        v.reset();
        v
    }

    /// `reset()`.
    pub fn reset(&mut self) {
        self.builder
            .reset_if_larger_than(MAX_PATTERNS, MAX_NEW_NAMES);
        if self.builder.memos.is_empty() {
            self.recover_pattern_table.clear();
        }
        self.had_error = false;
        self.collecting_characters = false;
        self.memo = self.start;
        self.char_buf.clear();
        self.datatype_errors.clear();
        self.stack.clear();
        self.errors.clear();
    }

    pub fn local_name(&self, name: Name) -> &str {
        self.builder.pb.names.local_name(name)
    }

    pub fn namespace_uri(&self, name: Name) -> &str {
        let ns = self.builder.pb.names.namespace_of(name);
        self.builder.pb.names.namespace_uri(ns)
    }

    /// Moves the errors reported so far into `out`.
    pub fn take_errors(&mut self, out: &mut Vec<ValidationError>) {
        out.append(&mut self.errors);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn name(&mut self, namespace_uri: &str, local_name: &str) -> Name {
        let names = &mut self.builder.pb.names;
        let ns = names.namespace(namespace_uri);
        names.name(ns, local_name)
    }

    fn peek(&self) -> Option<Name> {
        self.stack.last().copied()
    }

    fn is_not_allowed(&self, p: PatternId) -> bool {
        self.builder.pb.is_not_allowed(p)
    }

    /// `setMemo`: false if `m` is notAllowed.
    fn set_memo(&mut self, m: PatternId) -> bool {
        if self.is_not_allowed(m) {
            false
        } else {
            self.memo = m;
            true
        }
    }

    fn error(&mut self, e: ValidationError) {
        if self.suppress_depth > 0 || (self.had_error && self.is_not_allowed(self.memo)) {
            return;
        }
        self.had_error = true;
        self.errors.push(e);
    }

    /// `startDocument()`.
    pub fn start_document(&mut self) {
        self.stack.clear();
        self.suppress_depth = 0;
        if self.is_not_allowed(self.memo) {
            self.error(ValidationError::SchemaAllowsNothing);
        }
    }

    fn start_collecting_characters(&mut self) {
        if !self.collecting_characters {
            self.collecting_characters = true;
            self.char_buf.clear();
        }
    }

    fn flush_characters(&mut self) {
        self.collecting_characters = false;
        if !is_xml_whitespace(&self.char_buf) {
            self.text();
        }
    }

    fn record_names(&self, p: PatternId) -> RecordedNames {
        let mut nrv = NameRecordingVisitor::new(&self.builder.pb);
        self.builder.pb.accept(p, &mut nrv);
        RecordedNames {
            attributes: nrv.attributes,
            elements: nrv.elements,
            visited_choice: nrv.visited_choice,
        }
    }

    /// `startElement`.
    pub fn start_element(
        &mut self,
        namespace_uri: &str,
        local_name: &str,
        attributes: &[Attribute<'_>],
    ) {
        if self.collecting_characters {
            self.flush_characters();
        }
        if self.suppress_depth > 0 {
            self.suppress_depth += 1;
        }
        let name = self.name(namespace_uri, local_name);
        self.start_tag_open(name);
        for att in attributes {
            self.attribute(name, att);
        }
        self.end_attributes(name);
        if self.builder.pb.content_type(self.memo) == DATA_CONTENT_TYPE {
            self.start_collecting_characters();
        }
        self.stack.push(name);
    }

    /// The start-tag part of `startElement`, with its recovery.
    fn start_tag_open(&mut self, name: Name) {
        let deriv = self.builder.start_tag_open_deriv(self.memo, name);
        if self.set_memo(deriv) {
            return;
        }
        let mut next = self.builder.start_tag_open_recover_deriv(self.memo, name);
        if !self.is_not_allowed(next) {
            let p = self.builder.get_required_frontier_content(self.memo);
            let nrv = self.record_names(p);
            let parent = self.peek();
            if nrv.visited_choice {
                self.error(ValidationError::RequiredElementsMissingOneOf {
                    current: name,
                    missing: nrv.elements,
                    parent,
                });
            } else if !nrv.elements.is_empty() {
                for element in nrv.elements {
                    self.error(ValidationError::RequiredElementsMissing {
                        current: name,
                        missing: Some(element),
                        parent,
                    });
                }
            } else {
                self.error(ValidationError::RequiredElementsMissing {
                    current: name,
                    missing: None,
                    parent,
                });
            }
        } else {
            let element = self.find_element(name);
            next = self.builder.make_after(element, self.memo);
            let parent = self.peek();
            if self.is_not_allowed(next) {
                self.error(ValidationError::UnknownElement {
                    current: name,
                    parent,
                });
            } else {
                self.error(ValidationError::OutOfContextElement {
                    current: name,
                    parent,
                });
            }
            if self.suppress_depth == 0 {
                self.suppress_depth = 1;
            }
        }
        self.memo = next;
    }

    /// One attribute of `startElement`.
    fn attribute(&mut self, name: Name, att: &Attribute<'_>) {
        let att_name = self.name(att.namespace_uri, att.local_name);
        self.datatype_errors.clear();
        let deriv = self.builder.start_attribute_deriv(self.memo, att_name);
        if !self.set_memo(deriv) {
            let parent = self.peek();
            self.error(ValidationError::ImpossibleAttributeIgnored {
                current: name,
                parent,
                attribute: att_name,
            });
            return;
        }
        let mut errors = std::mem::take(&mut self.datatype_errors);
        let deriv = self.builder.data_deriv(self.memo, att.value, &mut errors);
        self.datatype_errors = errors;
        if !self.set_memo(deriv) {
            let parent = self.peek();
            self.error(ValidationError::BadAttributeValue {
                current: name,
                parent,
                attribute: att_name,
                value: att.value.to_string(),
                errors: self.datatype_errors.clone(),
            });
            self.memo = self.builder.recover_after(self.memo);
        }
    }

    /// The end of the attributes in `startElement`.
    fn end_attributes(&mut self, name: Name) {
        let deriv = self.builder.end_attributes(self.memo);
        if self.set_memo(deriv) {
            return;
        }
        let p = self.builder.get_required_attributes(self.memo);
        let nrv = self.record_names(p);
        let parent = self.peek();
        if nrv.visited_choice {
            self.error(ValidationError::RequiredAttributesMissingOneOf {
                current: name,
                attributes: nrv.attributes,
                parent,
            });
        } else {
            for attribute in nrv.attributes {
                self.error(ValidationError::RequiredAttributesMissing {
                    current: name,
                    attribute,
                    parent,
                });
            }
        }
        self.memo = self.builder.ignore_missing_attributes(self.memo);
    }

    /// `fixAfter`.
    fn fix_after(&mut self, p: PatternId) -> PatternId {
        self.builder.apply_after(p, After::Empty)
    }

    /// `endElement`.
    pub fn end_element(&mut self) {
        let Some(name) = self.stack.pop() else {
            return;
        };
        // The derivative may be notAllowed simply because the parent is notAllowed;
        // no error is given in that case.
        if self.collecting_characters {
            self.collecting_characters = false;
            let deriv = self.builder.text_only(self.memo);
            if !self.set_memo(deriv) {
                let parent = self.peek();
                self.error(ValidationError::OnlyTextNotAllowed {
                    current: name,
                    parent,
                });
                self.memo = self.builder.recover_after(self.memo);
                return;
            }
            let data = std::mem::take(&mut self.char_buf);
            let mut errors = std::mem::take(&mut self.datatype_errors);
            let deriv = self.builder.data_deriv(self.memo, &data, &mut errors);
            if !self.set_memo(deriv) {
                let next = self.builder.recover_after(self.memo);
                errors.clear();
                if !self.is_not_allowed(self.memo) {
                    let report = !self.is_not_allowed(next) || {
                        let fixed = self.fix_after(self.memo);
                        let d = self.builder.data_deriv(fixed, &data, &mut errors);
                        self.is_not_allowed(d)
                    };
                    if report {
                        let parent = self.peek();
                        self.error(ValidationError::StringNotAllowed {
                            current: name,
                            parent,
                            value: data.clone(),
                            errors: errors.clone(),
                        });
                    }
                }
                self.memo = next;
            }
            self.datatype_errors = errors;
            self.char_buf = data;
        } else {
            let deriv = self.builder.end_tag_deriv(self.memo);
            if !self.set_memo(deriv) {
                let next = self.builder.recover_after(self.memo);
                if !self.is_not_allowed(self.memo) {
                    let report = !self.is_not_allowed(next) || {
                        let fixed = self.fix_after(self.memo);
                        let d = self.builder.end_tag_deriv(fixed);
                        self.is_not_allowed(d)
                    };
                    if report {
                        let p = self.builder.get_required_content(self.memo);
                        let nrv = self.record_names(p);
                        let parent = self.peek();
                        if nrv.visited_choice {
                            self.error(ValidationError::UnfinishedElementOneOf {
                                current: name,
                                missing: nrv.elements,
                                parent,
                            });
                        } else {
                            for element in nrv.elements {
                                self.error(ValidationError::UnfinishedElement {
                                    current: name,
                                    missing: element,
                                    parent,
                                });
                            }
                        }
                    }
                }
                self.memo = next;
            }
        }
        if self.suppress_depth > 0 {
            self.suppress_depth -= 1;
        }
    }

    /// `characters`.
    pub fn characters(&mut self, s: &str) {
        if self.collecting_characters {
            self.char_buf.push_str(s);
            return;
        }
        if !is_xml_whitespace(s) {
            self.text();
        }
    }

    fn text(&mut self) {
        let deriv = self.builder.mixed_text_deriv(self.memo);
        if !self.set_memo(deriv) {
            let current = self.peek();
            self.error(ValidationError::TextNotAllowed { current });
        }
    }

    /// `findElement`, memoized per name.
    fn find_element(&mut self, name: Name) -> PatternId {
        if let Some(&p) = self.recover_pattern_table.get(&name) {
            return p;
        }
        let p = self.builder.find_element(name, self.start);
        self.recover_pattern_table.insert(name, p);
        p
    }
}

fn is_xml_whitespace(s: &str) -> bool {
    s.bytes().all(|b| matches!(b, b' ' | b'\r' | b'\t' | b'\n'))
}

/// What a [`NameRecordingVisitor`] found.
struct RecordedNames {
    attributes: BTreeSet<String>,
    elements: BTreeSet<String>,
    visited_choice: bool,
}

/// `NameRecordingVisitor`: the element and attribute names of a pattern.
struct NameRecordingVisitor<'a> {
    pb: &'a PatternBuilder,
    name_record: Option<String>,
    attributes: BTreeSet<String>,
    elements: BTreeSet<String>,
    visited_choice: bool,
}

impl<'a> NameRecordingVisitor<'a> {
    fn new(pb: &'a PatternBuilder) -> Self {
        Self {
            pb,
            name_record: None,
            attributes: BTreeSet::new(),
            elements: BTreeSet::new(),
            visited_choice: false,
        }
    }

    fn is_empty(&self, p: PatternId) -> bool {
        p == EMPTY
    }

    fn visit_pair(&mut self, p1: PatternId, p2: PatternId) {
        if self.is_empty(p1) && self.is_empty(p2) {
            return;
        }
        if self.is_empty(p1) {
            self.pb.accept(p2, self);
            return;
        }
        if self.is_empty(p2) {
            self.pb.accept(p1, self);
            return;
        }
        self.pb.accept(p1, self);
        self.pb.accept(p2, self);
    }
}

impl PatternVisitor for NameRecordingVisitor<'_> {
    fn visit_group(&mut self, p1: PatternId, p2: PatternId) {
        self.visit_pair(p1, p2);
    }

    fn visit_interleave(&mut self, p1: PatternId, p2: PatternId) {
        self.visit_pair(p1, p2);
    }

    fn visit_choice(&mut self, p1: PatternId, p2: PatternId) {
        if self.is_empty(p1) && self.is_empty(p2) {
            return;
        }
        self.pb.accept(p1, self);
        self.pb.accept(p2, self);
        self.visited_choice = true;
    }

    fn visit_one_or_more(&mut self, p: PatternId) {
        if !self.is_empty(p) {
            self.pb.accept(p, self);
        }
    }

    fn visit_element(&mut self, nc: NameClassId, _content: PatternId) {
        // Just the element name class, not the content.
        self.pb.name_classes.accept(nc, self);
        // Java's TreeSet would throw on a null record; skip it instead.
        if let Some(name) = &self.name_record {
            self.elements.insert(name.clone());
        }
    }

    fn visit_attribute(&mut self, nc: NameClassId, _value: PatternId) {
        self.pb.name_classes.accept(nc, self);
        if let Some(name) = &self.name_record {
            self.attributes.insert(name.clone());
        }
    }

    fn visit_list(&mut self, p: PatternId) {
        self.pb.accept(p, self);
    }
}

impl NameClassVisitor for NameRecordingVisitor<'_> {
    fn visit_choice(&mut self, classes: &NameClasses, nc1: NameClassId, nc2: NameClassId) {
        classes.accept(nc1, self);
        classes.accept(nc2, self);
    }

    fn visit_name(&mut self, name: Name) {
        self.name_record = Some(self.pb.names.local_name(name).to_string());
    }
}
