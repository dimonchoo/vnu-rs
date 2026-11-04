pub mod context;
pub mod engine;
pub mod rule;
pub mod rules;
pub mod syntax;

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

    #[test]
    fn test_forms_validation() {
        let mut engine = ValidatorEngine::default();
        let html = "<!DOCTYPE html><html lang=\"en\"><head><title>T</title></head><body><form><form></form></form><button type=\"invalid\">b</button><input type=\"bogus\"></body></html>";
        let report = engine.validate(html);
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|m| m.message.contains("Nested forms are not allowed")));
        assert!(report.messages.iter().any(|m| m.message.contains("Bad value “invalid” for attribute “type” on element “button”")));
        assert!(report.messages.iter().any(|m| m.message.contains("Bad value “bogus” for attribute “type” on element “input”")));
    }

    #[test]
    fn test_obsolete_and_tables() {
        let mut engine = ValidatorEngine::default();
        let html = "<!DOCTYPE html><html lang=\"en\"><head><title>T</title></head><body><center>old</center><font>f</font><div align=\"center\">d</div><table><td>stray</td></table></body></html>";
        let report = engine.validate(html);
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|m| m.message.contains("The “center” element is obsolete")));
        assert!(report.messages.iter().any(|m| m.message.contains("The “font” element is obsolete")));
        assert!(report.messages.iter().any(|m| m.message.contains("The “align” attribute on the “div” element is obsolete")));
        assert!(report.messages.iter().any(|m| m.message.contains("“td” start tag in table body")));
    }

    #[test]
    fn test_aria_and_content_model() {
        let mut engine = ValidatorEngine::default();
        let html = "<!DOCTYPE html><html lang=\"en\"><head><title>T</title></head><body><div role=\"invalid_role\">div</div><div aria-fake=\"test\">div</div><li>stray li</li><source src=\"s.mp4\"></body></html>";
        let report = engine.validate(html);
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|m| m.message.contains("Bad value “invalid_role” for attribute “role” on element “div”")));
        assert!(report.messages.iter().any(|m| m.message.contains("Attribute “aria-fake” not allowed on element “div” at this point")));
        assert!(report.messages.iter().any(|m| m.message.contains("Element “li” not allowed as child of element “body”")));
        assert!(report.messages.iter().any(|m| m.message.contains("Element “source” not allowed as child of element “body”")));
    }

    #[test]
    fn test_metadata_and_tag_mismatch() {
        let mut engine = ValidatorEngine::default();
        let html = "<!DOCTYPE html><html lang=\"en\"><head><title>T</title><base href=\"/\"><base href=\"/2\"><meta charset=\"latin1\"><link rel=\"stylesheet\"></head><body><svg><path d=\"M0 0\"></svg></body></html>";
        let report = engine.validate(html);
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|m| m.message.contains("Element “base” not allowed as child of element “head”")));
        assert!(report.messages.iter().any(|m| m.message.contains("The only allowed value for the “charset” attribute for the “meta” element is “utf-8”")));
        assert!(report.messages.iter().any(|m| m.message.contains("A “link” element must have an “href” or “imagesrcset” attribute")));
        assert!(report.messages.iter().any(|m| m.message.contains("End tag “svg” did not match the name of the current open element (“path”)")));
    }

    #[test]
    fn test_svg_standalone_validation() {
        let mut engine = ValidatorEngine::default();
        let valid_svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\"><circle cx=\"50\" cy=\"50\" r=\"40\"/></svg>";
        let valid_report = engine.validate(valid_svg);
        assert!(valid_report.is_valid());
        assert!(valid_report.messages.iter().any(|m| m.message.contains("Using the preset for SVG")));

        let missing_ns_svg = "<svg viewBox=\"0 0 100 100\"><circle cx=\"50\" cy=\"50\" r=\"40\"/></svg>";
        let err_report = engine.validate(missing_ns_svg);
        assert!(!err_report.is_valid());
        assert!(err_report.messages.iter().any(|m| m.message.contains("XML document with no namespace")));
    }

    #[test]
    fn test_link_relations_and_body_context() {
        let mut engine = ValidatorEngine::default();
        let invalid = "<!DOCTYPE html><html lang=\"en\"><head><title>T</title><link rel=\"alternate stylesheet\" href=\"s.css\"><link rel=\"preload\" href=\"f.woff2\" as=\"font\"><link rel=\"preload\" href=\"data.json\" as=\"json\"></head><body><link rel=\"canonical\" href=\"https://example.com/\"></body></html>";
        let report = engine.validate(invalid);
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|m| m.message.contains("must have a “title” attribute with a non-empty value")));
        assert!(report.messages.iter().any(|m| m.message.contains("The value “json” is not a valid value for the “as” attribute of a “link” element with “rel=preload”")));
        assert!(report.messages.iter().any(|m| m.message.contains("must not appear as a descendant of a “body” element")));
    }

    #[test]
    fn test_meta_uniqueness_and_directives() {
        let mut engine = ValidatorEngine::default();
        let invalid = "<!DOCTYPE html><html lang=\"en\"><head><title>T</title><meta charset=\"utf-8\"><meta charset=\"utf-8\"><meta http-equiv=\"refresh\" content=\"invalid\"><meta name=\"viewport\" content=\"width=device-width, user-scalable=no\"></head><body></body></html>";
        let report = engine.validate(invalid);
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|m| m.message.contains("A document must not include more than one “meta” element with a “charset” attribute.")));
        assert!(report.messages.iter().any(|m| m.message.contains("Bad value “invalid” for attribute “content” on element “meta”.")));
        assert!(report.messages.iter().any(|m| m.message.contains("Consider avoiding viewport values that prevent users from resizing documents.")));
    }

    #[test]
    fn test_select_and_option_rules() {
        let mut engine = ValidatorEngine::default();
        let invalid = "<!DOCTYPE html><html lang=\"en\"><head><title>T</title></head><body><select multiple><button>B</button><option>1</option></select><select required><option value=\"val\">not-empty</option></select></body></html>";
        let report = engine.validate(invalid);
        assert!(!report.is_valid());
        assert!(report.messages.iter().any(|m| m.message.contains("A “button” element is only allowed as a child of a “select” element that is a drop-down box")));
        assert!(report.messages.iter().any(|m| m.message.contains("must have either an empty “value” attribute, or must have no text content")));
    }
}
