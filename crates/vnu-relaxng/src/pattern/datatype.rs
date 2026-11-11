//! `org.relaxng.datatype`: datatypes, datatype libraries and their errors, plus Jing's
//! `BuiltinDatatypeLibrary` (`string` and `token` from the empty library URI).
//!
//! A compiled schema refers to datatypes by [`DatatypeRef`] (library URI, type name and
//! parameters); the validator asks a [`DatatypeLibraryFactory`] for the implementation.

pub const XML_SCHEMA_DATATYPES: &str = "http://www.w3.org/2001/XMLSchema-datatypes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DatatypeId(pub(crate) u32);

/// Index of a value literal (`ValuePattern`) in the compiled schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ValueId(pub(crate) u32);

/// A datatype as written in the schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatatypeRef {
    pub library: String,
    pub name: String,
    pub params: Vec<(String, String)>,
}

/// `org.relaxng.datatype.DatatypeException`. `warning` stands for v.Nu's
/// `Html5DatatypeException.isWarning()`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatatypeException {
    pub message: Option<String>,
    pub warning: bool,
}

impl DatatypeException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            warning: false,
        }
    }
}

/// `org.relaxng.datatype.Datatype` plus Jing's `Datatype2.alwaysValid()`. Values are
/// represented by a canonical string, so `sameValue` is string equality by default.
pub trait Datatype: Send + Sync {
    fn check_valid(&self, s: &str) -> Result<(), DatatypeException>;

    fn is_valid(&self, s: &str) -> bool {
        self.check_valid(s).is_ok()
    }

    /// `createValue`: the canonical value, or `None` when `s` is not a valid literal.
    fn create_value(&self, s: &str) -> Option<String>;

    fn same_value(&self, a: &str, b: &str) -> bool {
        a == b
    }

    fn always_valid(&self) -> bool {
        false
    }
}

/// Supplies datatype implementations to the validator.
pub trait DatatypeLibraryFactory {
    fn create_datatype(&self, dt: &DatatypeRef) -> Box<dyn Datatype>;
}

/// Jing's `BuiltinDatatypeLibrary.StringDatatype`.
pub struct StringDatatype;

impl Datatype for StringDatatype {
    fn check_valid(&self, _s: &str) -> Result<(), DatatypeException> {
        Ok(())
    }

    fn create_value(&self, s: &str) -> Option<String> {
        Some(s.to_string())
    }

    fn always_valid(&self) -> bool {
        true
    }
}

/// Jing's `BuiltinDatatypeLibrary.TokenDatatype`: whitespace is normalized.
pub struct TokenDatatype;

impl Datatype for TokenDatatype {
    fn check_valid(&self, _s: &str) -> Result<(), DatatypeException> {
        Ok(())
    }

    fn create_value(&self, s: &str) -> Option<String> {
        Some(normalize_whitespace(s))
    }

    fn always_valid(&self) -> bool {
        true
    }
}

/// A datatype that accepts every lexical form; values compare as strings.
pub struct AnyStringDatatype;

impl Datatype for AnyStringDatatype {
    fn check_valid(&self, _s: &str) -> Result<(), DatatypeException> {
        Ok(())
    }

    fn create_value(&self, s: &str) -> Option<String> {
        Some(s.to_string())
    }
}

/// XML whitespace collapsing, as `token` and most XSD types do.
pub fn normalize_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for token in s.split([' ', '\t', '\n', '\r']).filter(|t| !t.is_empty()) {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(token);
    }
    out
}

/// Datatypes of the empty library URI, then XSD, then [`AnyStringDatatype`] for
/// everything else. Callers with their own libraries wrap this.
pub struct BuiltinDatatypeLibraryFactory;

impl DatatypeLibraryFactory for BuiltinDatatypeLibraryFactory {
    fn create_datatype(&self, dt: &DatatypeRef) -> Box<dyn Datatype> {
        match (dt.library.as_str(), dt.name.as_str()) {
            ("", "string") => Box::new(StringDatatype),
            ("", "token") => Box::new(TokenDatatype),
            (XML_SCHEMA_DATATYPES, "string") => Box::new(AnyStringDatatype),
            (
                XML_SCHEMA_DATATYPES,
                "token" | "NMTOKEN" | "NCName" | "Name" | "language" | "anyURI" | "QName",
            ) => Box::new(XsdTokenDatatype),
            _ => Box::new(AnyStringDatatype),
        }
    }
}

/// XSD types whose values are whitespace-collapsed strings.
struct XsdTokenDatatype;

impl Datatype for XsdTokenDatatype {
    fn check_valid(&self, _s: &str) -> Result<(), DatatypeException> {
        Ok(())
    }

    fn create_value(&self, s: &str) -> Option<String> {
        Some(normalize_whitespace(s))
    }
}
