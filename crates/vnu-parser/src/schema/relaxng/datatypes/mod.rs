//! The datatype libraries of the HTML schema, as v.Nu's
//! `Html5DatatypeLibraryFactory` provides them.
//!
//! Stage 3 of the parity plan: only `w:string` (`AsciiCaseInsensitiveString`) is real,
//! because enumerated attribute values depend on it. Every other datatype of the
//! `http://whattf.org/datatype-draft` library accepts any string, so attribute values
//! are still checked by the hand-written rules in `rules/datatypes`. The exception is
//! `w:mime-type` (validity only, no detail message yet), because `object` needs it to
//! pick the right branch, like Java does; the hand-written rules share its check.

pub mod mime;

use vnu_relaxng::{
    AnyStringDatatype, BuiltinDatatypeLibraryFactory, Datatype, DatatypeException,
    DatatypeLibraryFactory, DatatypeRef,
};

pub const HTML5_DATATYPES: &str = "http://whattf.org/datatype-draft";

pub struct Html5DatatypeLibraryFactory;

impl DatatypeLibraryFactory for Html5DatatypeLibraryFactory {
    fn create_datatype(&self, dt: &DatatypeRef) -> Box<dyn Datatype> {
        if dt.library != HTML5_DATATYPES {
            return BuiltinDatatypeLibraryFactory.create_datatype(dt);
        }
        match dt.name.as_str() {
            "string" => Box::new(AsciiCaseInsensitiveString),
            "mime-type" => Box::new(MimeType),
            _ => Box::new(AnyStringDatatype),
        }
    }
}

/// `nu.validator.datatype.AsciiCaseInsensitiveString`.
pub struct AsciiCaseInsensitiveString;

impl Datatype for AsciiCaseInsensitiveString {
    fn check_valid(&self, _s: &str) -> Result<(), DatatypeException> {
        Ok(())
    }

    fn create_value(&self, s: &str) -> Option<String> {
        Some(s.to_ascii_lowercase())
    }
}

/// `nu.validator.datatype.MimeType`, validity only: the detail messages come with
/// the datatype port (stage 4).
pub struct MimeType;

impl Datatype for MimeType {
    fn check_valid(&self, s: &str) -> Result<(), DatatypeException> {
        if mime::is_valid_mime_type(s) {
            Ok(())
        } else {
            Err(DatatypeException::default())
        }
    }

    fn create_value(&self, s: &str) -> Option<String> {
        self.is_valid(s).then(|| s.to_string())
    }
}
