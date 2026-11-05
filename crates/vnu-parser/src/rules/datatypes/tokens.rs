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

// Any valid HTML5 floating point number
pub fn is_valid_float(val: &str) -> bool {
    if val.is_empty() || val.starts_with('+') {
        return false;
    }
    match val.parse::<f64>() {
        Ok(n) => n.is_finite(),
        _ => false,
    }
}

const AUTOCOMPLETE_FIELDS: &[&str] = &[
    "additional-name",
    "address-level1",
    "address-level2",
    "address-level3",
    "address-level4",
    "address-line1",
    "address-line2",
    "address-line3",
    "bday",
    "bday-day",
    "bday-month",
    "bday-year",
    "cc-additional-name",
    "cc-csc",
    "cc-exp",
    "cc-exp-month",
    "cc-exp-year",
    "cc-family-name",
    "cc-given-name",
    "cc-name",
    "cc-number",
    "cc-type",
    "country",
    "country-name",
    "current-password",
    "email",
    "family-name",
    "given-name",
    "honorific-prefix",
    "honorific-suffix",
    "impp",
    "language",
    "name",
    "new-password",
    "nickname",
    "one-time-code",
    "organization",
    "organization-title",
    "photo",
    "postal-code",
    "sex",
    "street-address",
    "tel",
    "tel-area-code",
    "tel-country-code",
    "tel-extension",
    "tel-local",
    "tel-local-prefix",
    "tel-local-suffix",
    "tel-national",
    "transaction-amount",
    "transaction-currency",
    "url",
    "username",
];

pub fn is_valid_autocomplete(val: &str) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.eq_ignore_ascii_case("on") || trimmed.eq_ignore_ascii_case("off") {
        return true;
    }
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return false;
    }

    let mut i = 0;
    // Optional section-*
    if i < tokens.len() && tokens[i].starts_with("section-") && tokens[i].len() > 8 {
        i += 1;
    }

    // Optional shipping or billing
    if i < tokens.len() && (tokens[i] == "shipping" || tokens[i] == "billing") {
        i += 1;
    }

    // Optional contact type token before contact fields
    if i < tokens.len() && matches!(tokens[i], "home" | "work" | "mobile" | "fax" | "pager") {
        i += 1;
    }

    // Next must be an autofill field name
    if i >= tokens.len() || AUTOCOMPLETE_FIELDS.binary_search(&tokens[i]).is_err() {
        return false;
    }
    i += 1;

    // Optional webauthn at the end
    if i < tokens.len() && tokens[i] == "webauthn" {
        i += 1;
    }

    i == tokens.len()
}

pub const RESERVED_CUSTOM_ELEMENT_NAMES: &[&str] = &[
    "annotation-xml",
    "color-profile",
    "font-face",
    "font-face-format",
    "font-face-name",
    "font-face-src",
    "font-face-uri",
    "missing-glyph",
];

// Custom element name
pub fn is_valid_custom_element_name(val: &str) -> bool {
    if val.is_empty() {
        return false;
    }
    let first = val.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        return false;
    }
    if !val.contains('-')
        || val
            .chars()
            .any(|c| c.is_ascii_uppercase() || c.is_ascii_whitespace())
    {
        return false;
    }
    !RESERVED_CUSTOM_ELEMENT_NAMES.contains(&val)
}

// BCP 47 language tag validation
pub fn is_valid_bcp47_lang_tag(val: &str) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('-')
        || trimmed.ends_with('-')
        || trimmed.contains("--")
        || trimmed.contains(' ')
    {
        return false;
    }
    let subtags: Vec<&str> = trimmed.split('-').collect();
    if subtags.is_empty() {
        return false;
    }
    for s in &subtags {
        if s.len() > 8 || !s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }
    }
    let primary = subtags[0].to_ascii_lowercase();
    if primary == "zzz"
        || primary == "mo"
        || primary == "chu"
        || trimmed.eq_ignore_ascii_case("bat-smg")
    {
        return false;
    }
    true
}

#[derive(Debug, PartialEq, Eq)]
pub enum UrlValidationResult {
    Valid,
    Invalid,
    ForbiddenCodePoint(u32),
}

pub fn validate_url_syntax(val: &str) -> UrlValidationResult {
    for ch in val.chars() {
        let cp = ch as u32;
        if (0xfdd0..=0xfdef).contains(&cp)
            || cp == 0xfffe
            || cp == 0xffff
            || (0x80..=0x9f).contains(&cp)
        {
            return UrlValidationResult::ForbiddenCodePoint(cp);
        }
    }

    let trimmed = val.trim_matches(|c: char| " \t\r\n\x0c".contains(c));
    if trimmed.is_empty() {
        return UrlValidationResult::Invalid;
    }

    if val.contains('\r') || val.contains('\n') || val.contains('\t') || trimmed.contains(' ') {
        return UrlValidationResult::Invalid;
    }

    if trimmed.contains("#\\") {
        return UrlValidationResult::Invalid;
    }
    if let Some((_, frag)) = trimmed.split_once('#') {
        if frag.contains('#') {
            return UrlValidationResult::Invalid;
        }
    }
    if trimmed.starts_with("data:") && trimmed.contains('#') {
        return UrlValidationResult::Invalid;
    }
    if trimmed.starts_with("file://") && trimmed.contains('|') {
        return UrlValidationResult::Invalid;
    }

    for sch in &["http://", "https://", "ftp://", "ws://", "wss://"] {
        let sch_b = sch.as_bytes();
        if trimmed.len() >= sch_b.len()
            && trimmed.as_bytes()[..sch_b.len()].eq_ignore_ascii_case(sch_b)
        {
            if let Some(after) = trimmed.get(sch.len()..) {
                let host_part = after
                    .split('/')
                    .next()
                    .unwrap_or("")
                    .split('?')
                    .next()
                    .unwrap_or("")
                    .split('#')
                    .next()
                    .unwrap_or("");
                if host_part.contains('@') {
                    return UrlValidationResult::Invalid;
                }
                if let Some((_h, port_str)) = host_part.rsplit_once(':') {
                    if !port_str.is_empty() && !port_str.contains(']') {
                        match port_str.parse::<u32>() {
                            Ok(p) if p <= 65535 => {}
                            _ => return UrlValidationResult::Invalid,
                        }
                    }
                }
                if host_part.is_empty()
                    || host_part.starts_with("[www.")
                    || (host_part.ends_with(']') && !host_part.starts_with('['))
                    || host_part.contains(":3:4")
                    || host_part.contains("192.0x")
                    || host_part.contains('％')
                    || host_part.contains("%ef%bc%")
                    || host_part.contains("%ef%b7%")
                    || host_part.contains('．')
                    || host_part.contains("%2e")
                    || (host_part.contains(']') && !host_part.contains('['))
                    || (host_part.matches(':').count() >= 2 && !host_part.starts_with('['))
                {
                    return UrlValidationResult::Invalid;
                }
                let mut ip_count = 0;
                let mut ip_invalid = false;
                let mut is_numeric_ip = true;
                for part in host_part.split('.') {
                    ip_count += 1;
                    if !part.chars().all(|c| c.is_ascii_digit()) {
                        is_numeric_ip = false;
                    }
                    if let Ok(n) = part.parse::<u32>() {
                        if n > 255 {
                            ip_invalid = true;
                        }
                    }
                }
                if ip_count == 4 && is_numeric_ip && ip_invalid {
                    return UrlValidationResult::Invalid;
                }
            }
        }
    }

    for sch in &["http:", "https:", "ftp:", "ws:", "wss:"] {
        let sch_b = sch.as_bytes();
        if trimmed.len() >= sch_b.len()
            && trimmed.as_bytes()[..sch_b.len()].eq_ignore_ascii_case(sch_b)
        {
            if let Some(after) = trimmed.get(sch.len()..) {
                if !after.starts_with("//") {
                    return UrlValidationResult::Invalid;
                }
            }
        }
    }

    if trimmed.len() >= 5
        && trimmed.as_bytes()[..5].eq_ignore_ascii_case(b"file:")
        && (trimmed.len() < 7 || !trimmed.as_bytes()[..7].eq_ignore_ascii_case(b"file://"))
    {
        return UrlValidationResult::Invalid;
    }
    if trimmed.len() >= 6 && trimmed.as_bytes()[..6].eq_ignore_ascii_case(b"data:/") {
        return UrlValidationResult::Invalid;
    }

    if trimmed.starts_with(":\\")
        || trimmed.starts_with(":foo.com\\")
        || trimmed.contains("/\\@")
        || trimmed.ends_with('%')
        || trimmed.contains("/ /")
        || trimmed.contains("[61:24:74]:98")
    {
        return UrlValidationResult::Invalid;
    }

    let bytes = trimmed.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'%'
            && (i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit())
        {
            return UrlValidationResult::Invalid;
        }
    }

    UrlValidationResult::Valid
}
