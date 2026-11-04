pub const SANDBOX_KEYWORDS: &[&str] = &[
    "allow-downloads",
    "allow-forms",
    "allow-modals",
    "allow-orientation-lock",
    "allow-pointer-lock",
    "allow-popups",
    "allow-popups-to-escape-sandbox",
    "allow-presentation",
    "allow-same-origin",
    "allow-scripts",
    "allow-top-navigation",
    "allow-top-navigation-by-user-activation",
    "allow-top-navigation-to-custom-protocols",
];

// Browsing context name or keyword
pub fn is_valid_browsing_context(val: &str) -> bool {
    if val.is_empty() {
        return false;
    }
    if let Some(rest) = val.strip_prefix('_') {
        matches!(
            rest.to_ascii_lowercase().as_str(),
            "blank" | "self" | "parent" | "top"
        )
    } else {
        true
    }
}

// Simple color: #rrggbb (case-insensitive hex, length 7)
pub fn is_valid_simple_color(val: &str) -> bool {
    if val.len() != 7 || !val.starts_with('#') {
        return false;
    }
    val[1..].chars().all(|c| c.is_ascii_hexdigit())
}

// Positive integer (no sign, digits only, > 0)
pub fn is_valid_positive_integer(val: &str) -> bool {
    if val.is_empty() || !val.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    match val.parse::<u64>() {
        Ok(n) => n > 0,
        _ => false,
    }
}

// Positive float (> 0)
pub fn is_valid_positive_float(val: &str) -> bool {
    if val.is_empty() || val.starts_with('-') || val.starts_with('+') {
        return false;
    }
    match val.parse::<f64>() {
        Ok(n) => n > 0.0 && n.is_finite(),
        _ => false,
    }
}

// Custom element name
pub fn is_valid_custom_element_name(val: &str) -> bool {
    if val.is_empty() {
        return false;
    }
    let first = val.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        return false;
    }
    if !val.contains('-') || val.chars().any(|c| c.is_ascii_uppercase() || c.is_ascii_whitespace()) {
        return false;
    }
    const RESERVED_NAMES: &[&str] = &[
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-format",
        "font-face-name",
        "font-face-src",
        "font-face-uri",
        "missing-glyph",
    ];
    !RESERVED_NAMES.contains(&val)
}
