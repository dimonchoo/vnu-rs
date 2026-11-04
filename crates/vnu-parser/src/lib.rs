pub mod context;
pub mod engine;
pub mod rule;
pub mod rules;

pub use context::{ElementContext, LineIndex, SourceLocation, ValidationContext};
pub use engine::ValidatorEngine;
pub use rule::Rule;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_html() {
        let mut engine = ValidatorEngine::default();
        let html = "<!DOCTYPE html><html lang=\"en\"><head><title>Test</title></head><body><p>Hello</p></body></html>";
        let report = engine.validate(html);
        assert!(report.is_valid(), "Report had errors: {:?}", report.messages);
        assert_eq!(report.error_count(), 0);
        assert_eq!(report.warning_count(), 0);
    }

    #[test]
    fn test_missing_doctype_and_alt() {
        let mut engine = ValidatorEngine::default();
        let html = "<html><body><img src=\"test.png\"></body></html>";
        let report = engine.validate(html);
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|m| m.message.contains("Expected “<!DOCTYPE html>”")));
        assert!(report.messages.iter().any(|m| m.message.contains("must have an “alt” attribute")));
    }

    #[test]
    fn test_duplicate_id_and_nesting() {
        let mut engine = ValidatorEngine::default();
        let html = "<!DOCTYPE html><html lang=\"en\"><head><title>T</title></head><body><div id=\"foo\"></div><div id=\"foo\"></div><a href=\"#\"><a>nested</a></a></body></html>";
        let report = engine.validate(html);
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|m| m.message.contains("Duplicate ID “foo”")));
        assert!(report.messages.iter().any(|m| m.message.contains("Start tag “a” seen but an element of the same type was already open")));
    }
}
