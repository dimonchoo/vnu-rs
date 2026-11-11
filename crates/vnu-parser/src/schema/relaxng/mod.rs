//! RELAX NG validation of HTML documents like Java v.Nu: the compiled
//! `http://s.validator.nu/html5.rnc` schema (built from `schema/` by `build.rs`), fed
//! with the elements and text the tree builder emits.
//!
//! Between the parser and Jing, v.Nu puts the same filters as here, in this order:
//! nu htmlparser's attribute naming (`AttributeName`, with `lang` mapped to `xml:lang`),
//! `AttributesPermutingXMLReaderWrapper`, then the schema wrappers set up in
//! `VerifierServletTransaction` (outermost first): `NamespaceChangingSchemaWrapper`,
//! `TemplateElementDroppingSchemaWrapper`, `RoleAttributeFilteringSchemaWrapper`,
//! `XmlLangAttributeDroppingSchemaWrapper`, `AriaAttributeDroppingSchemaWrapper` and
//! `DataAttributeDroppingSchemaWrapper`. The errors those wrappers report themselves
//! (bad `data-*` names, `xml:lang` without `lang`, unknown roles) come from the
//! hand-written rules.

mod attributes;
pub mod datatypes;
pub mod messages;
mod validator;

pub use validator::SchemaValidator;

use crate::treebuilder::Ns;
use std::sync::OnceLock;
use vnu_relaxng::Schema;

pub const XHTML_NS: &str = "http://www.w3.org/1999/xhtml";
pub const SVG_NS: &str = "http://www.w3.org/2000/svg";
pub const MATHML_NS: &str = "http://www.w3.org/1998/Math/MathML";
/// `NamespaceChangingContentHandlerWrapper.customElementNs`.
const CUSTOM_ELEMENTS_NS: &str = "http://n.validator.nu/custom-elements/";

static HTML5_SCHEMA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/html5.rngc"));

/// The compiled HTML schema, loaded once. `None` if the embedded bytes do not load,
/// which `build.rs` rules out; validation then goes on without the schema.
pub fn html5_schema() -> Option<&'static Schema> {
    static SCHEMA: OnceLock<Option<Schema>> = OnceLock::new();
    SCHEMA
        .get_or_init(|| Schema::from_bytes(HTML5_SCHEMA).ok())
        .as_ref()
}

fn ns_uri(ns: Ns) -> &'static str {
    match ns {
        Ns::Html => XHTML_NS,
        Ns::Svg => SVG_NS,
        Ns::MathMl => MATHML_NS,
    }
}
