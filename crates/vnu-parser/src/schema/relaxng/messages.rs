//! The text of RELAX NG errors, as v.Nu's `MessageEmitterAdapter.rngMessageText`
//! writes it for HTML documents (`html` is true, so names never get a namespace
//! label). Code segments become “…” as in the JSON output.

use std::borrow::Cow;
use vnu_relaxng::{DatatypeErrors, Name, PatternValidator, ValidationError};

/// `MessageEmitterAdapter.TRANSPARENT_ELEMENTS`.
const TRANSPARENT_ELEMENTS: &[&str] = &[
    "a", "ins", "del", "object", "video", "audio", "canvas", "map", "slot",
];

struct Text<'a> {
    out: String,
    v: &'a PatternValidator,
}

impl Text<'_> {
    /// `messageTextString(handler, chars, capitalize)`.
    fn s(&mut self, text: &str, capitalize: bool) {
        let mut chars = text.chars();
        match chars.next() {
            Some(c) if capitalize && c.is_ascii_lowercase() => {
                self.out.push(c.to_ascii_uppercase());
                self.out.push_str(chars.as_str());
            }
            _ => self.out.push_str(text),
        }
    }

    /// `codeString`.
    fn code(&mut self, text: &str) {
        self.out.push('“');
        self.out.push_str(text);
        self.out.push('”');
    }

    /// `element(handler, name, atSentenceStart)` with `html` set.
    fn element(&mut self, name: Name, at_sentence_start: bool) {
        self.s("element ", at_sentence_start);
        let local = self.v.local_name(name).to_string();
        self.code(&local);
    }

    /// `attribute(handler, attributeName, elementName, atSentenceStart)` with `html`
    /// set: the local name only.
    fn attribute(&mut self, name: Name, at_sentence_start: bool) {
        self.s("attribute ", at_sentence_start);
        let local = self.v.local_name(name).to_string();
        self.code(&local);
    }

    fn list(&mut self, names: impl IntoIterator<Item = impl AsRef<str>>) {
        let mut first = true;
        for name in names {
            if !first {
                self.s(", ", false);
            }
            first = false;
            self.code(name.as_ref());
        }
    }

    /// The "one or more of the following child elements" tail. Like Java, "frameset"
    /// is left out for HTML elements but the comma before the next name is not.
    fn one_of_child_elements(
        &mut self,
        current: Name,
        missing: &std::collections::BTreeSet<String>,
    ) {
        self.s(
            " is missing a required instance of one or more of the following child elements: ",
            false,
        );
        let html = self.v.namespace_uri(current) == "http://www.w3.org/1999/xhtml";
        let mut iter = missing.iter().peekable();
        while let Some(name) = iter.next() {
            if !(html && name == "frameset") {
                self.code(name);
                if iter.peek().is_some() {
                    self.s(", ", false);
                }
            }
        }
        self.s(".", false);
    }

    /// `emitDatatypeErrors`.
    fn datatype_errors(&mut self, errors: &DatatypeErrors) {
        if errors.is_empty() {
            self.s(".", false);
            return;
        }
        let typo_info = errors.iter().any(|(_, e)| {
            e.warning && e.message.as_deref().is_some_and(|m| m.contains("Typo for"))
        });
        if !typo_info {
            self.s(":", false);
        }
        for (_, e) in errors {
            if !typo_info {
                self.s(" ", false);
            }
            self.s(e.message.as_deref().unwrap_or(""), false);
        }
    }
}

/// True for `BadAttributeValueException`s that v.Nu reports as warnings
/// (`isRoleDirectoryWarning`).
pub fn is_role_directory_warning(v: &PatternValidator, e: &ValidationError) -> bool {
    matches!(e, ValidationError::BadAttributeValue { attribute, value, .. }
        if value == "directory" && v.local_name(*attribute) == "role")
}

/// True when the error is reported as a warning ("Potentially bad value").
pub fn is_warning(e: &ValidationError) -> bool {
    matches!(e, ValidationError::BadAttributeValue { errors, .. }
        if errors.iter().any(|(_, ex)| ex.warning))
}

/// `rngMessageText`.
pub fn message_text(v: &PatternValidator, e: &ValidationError) -> String {
    let mut t = Text {
        out: String::with_capacity(96),
        v,
    };
    match e {
        ValidationError::BadAttributeValue {
            current,
            attribute,
            value,
            errors,
            ..
        } => {
            if is_warning(e) {
                t.s("Potentially bad value ", false);
            } else {
                t.s("Bad value ", false);
            }
            if value.chars().count() < 200 {
                t.code(value);
            }
            t.s(" for ", false);
            t.attribute(*attribute, false);
            t.s(" on ", false);
            t.element(*current, false);
            t.datatype_errors(errors);
        }
        ValidationError::ImpossibleAttributeIgnored {
            current, attribute, ..
        } => {
            t.attribute(*attribute, true);
            t.s(" not allowed on ", false);
            t.element(*current, false);
            t.s(" at this point.", false);
        }
        ValidationError::OnlyTextNotAllowed { current, .. } => {
            t.element(*current, true);
            t.s(
                " is not allowed to have content that consists solely of text.",
                false,
            );
        }
        ValidationError::OutOfContextElement { current, parent } => {
            t.element(*current, true);
            t.s(" not allowed", false);
            let mut parent_name = None;
            if let Some(parent) = parent {
                parent_name = Some(v.local_name(*parent).to_string());
                t.s(" as child of ", false);
                t.element(*parent, false);
            }
            t.s(
                " in this context. (Suppressing further errors from this subtree.)",
                false,
            );
            if let Some(p) = parent_name.filter(|p| TRANSPARENT_ELEMENTS.contains(&p.as_str())) {
                t.s(
                    &format!(
                        " Note: The “{p}” element has a transparent content model; its allowed content is inherited from its parent element."
                    ),
                    false,
                );
            }
        }
        ValidationError::RequiredAttributesMissingOneOf {
            current,
            attributes,
            ..
        } => {
            t.element(*current, true);
            t.s(
                " is missing one or more of the following attributes: ",
                false,
            );
            t.list(attributes);
            t.s(".", false);
        }
        ValidationError::RequiredAttributesMissing {
            current, attribute, ..
        } => {
            t.element(*current, true);
            t.s(" is missing required attribute ", false);
            t.code(attribute);
            t.s(".", false);
        }
        ValidationError::RequiredElementsMissing {
            missing, parent, ..
        } => match parent {
            None => t.s("Required elements missing.", false),
            Some(parent) => {
                t.element(*parent, true);
                match missing {
                    None => t.s(" is missing a required child element", false),
                    Some(m) => {
                        t.s(" is missing a required instance of child element ", false);
                        t.code(m);
                    }
                }
                t.s(".", false);
            }
        },
        ValidationError::StringNotAllowed {
            current,
            value,
            errors,
            ..
        } => {
            let typo_info = errors.iter().any(|(_, ex)| {
                ex.warning
                    && ex
                        .message
                        .as_deref()
                        .is_some_and(|m| m.contains("Typo for"))
            });
            if !typo_info {
                t.s("Bad character content ", false);
                t.code(value);
                t.s(" for ", false);
                t.element(*current, false);
            }
            t.datatype_errors(errors);
        }
        ValidationError::TextNotAllowed { current } => {
            t.s("Text not allowed in ", false);
            if let Some(current) = current {
                t.element(*current, false);
            }
            t.s(" in this context.", false);
        }
        ValidationError::UnfinishedElement {
            current, missing, ..
        } => {
            t.element(*current, true);
            t.s(" is missing a required instance of child element ", false);
            t.code(missing);
            t.s(".", false);
        }
        ValidationError::UnfinishedElementOneOf {
            current, missing, ..
        } => {
            t.element(*current, true);
            t.one_of_child_elements(*current, missing);
        }
        ValidationError::RequiredElementsMissingOneOf {
            current,
            missing,
            parent: Some(parent),
        } => {
            t.element(*parent, true);
            t.one_of_child_elements(*current, missing);
        }
        ValidationError::RequiredElementsMissingOneOf { parent: None, .. } => {
            t.s("Required elements missing.", false);
        }
        ValidationError::UnknownElement { current, parent } => {
            t.s("Unknown ", false);
            t.element(*current, false);
            t.s(" not allowed", false);
            if let Some(parent) = parent {
                t.s(" as child of ", false);
                t.element(*parent, false);
            }
            t.s(".", false);
        }
        ValidationError::SchemaAllowsNothing => {
            t.s(
                "Schema does not allow anything: it is equivalent to “<notAllowed/>”.",
                false,
            );
        }
    }
    t.out
}

/// `MessageEmitterAdapter.getDisplayMessageForRng` wording, which upstream's
/// TestRunner compares, from the [`message_text`] wording. The two differ only for
/// out-of-context elements and text (the datatype details of bad values are not
/// produced before stage 4); every other message is returned as it is.
pub fn display_wording(message: &str) -> Cow<'_, str> {
    const SUPPRESSING: &str = " in this context. (Suppressing further errors from this subtree.)";
    if let Some(at) = message.find(SUPPRESSING) {
        let (head, rest) = message.split_at(at);
        let note = &rest[SUPPRESSING.len()..];
        if head.starts_with("Element “") {
            let head = head.replacen(
                "” not allowed as child of element “",
                "” not allowed as child of “",
                1,
            );
            return Cow::Owned(format!("{head} in this context.{note}"));
        }
    }
    if let Some(rest) = message.strip_prefix("Text not allowed in element “") {
        if rest.ends_with("” in this context.") {
            return Cow::Owned(format!("Text not allowed in “{rest}"));
        }
    }
    Cow::Borrowed(message)
}

#[cfg(test)]
mod tests {
    use super::display_wording;

    #[test]
    fn display_wording_matches_get_display_message() {
        assert_eq!(
            display_wording(
                "Element “div” not allowed as child of element “ul” in this context. (Suppressing further errors from this subtree.)"
            ),
            "Element “div” not allowed as child of “ul” in this context."
        );
        assert_eq!(
            display_wording(
                "Element “p” not allowed as child of element “a” in this context. (Suppressing further errors from this subtree.) Note: The “a” element has a transparent content model; its allowed content is inherited from its parent element."
            ),
            "Element “p” not allowed as child of “a” in this context. Note: The “a” element has a transparent content model; its allowed content is inherited from its parent element."
        );
        assert_eq!(
            display_wording("Text not allowed in element “ul” in this context."),
            "Text not allowed in “ul” in this context."
        );
        assert_eq!(display_wording("Stray end tag “p”."), "Stray end tag “p”.");
    }
}
