//! A port of Jing's RELAX NG validator (`jing-trang/mod/rng-validate`, the v.Nu fork)
//! and of the compact-syntax schema reader it is fed with.
//!
//! Layout: `pattern` (patterns, names, name classes, datatypes and the base
//! `PatternBuilder`), `parse` (compact syntax, `SchemaBuilderImpl`,
//! `SchemaPatternBuilder`), `validate` (`ValidatorPatternBuilder`, the derivative
//! functions, `RequiredContent`, `PatternValidator`) and `schema` (the compiled form).

mod parse;
mod pattern;
mod schema;
mod validate;

use parse::{compact_syntax, schema_builder_impl, schema_pattern_builder};
use pattern::{datatype, name, name_class, pattern_builder};
use validate::{data_deriv, derivs, pattern_validator, validator_pattern_builder};

pub use data_deriv::DatatypeErrors;
pub use datatype::{
    AnyStringDatatype, BuiltinDatatypeLibraryFactory, Datatype, DatatypeException,
    DatatypeLibraryFactory, DatatypeRef,
};
pub use name::Name;
pub use pattern_validator::{Attribute, PatternValidator, ValidationError};
pub use schema::Schema;
