const ALLOWED_MEDIA_TYPES: &[&str] = &["all", "print", "screen"];
const DEPRECATED_MEDIA_TYPES: &[&str] = &[
    "aural",
    "braille",
    "embossed",
    "handheld",
    "projection",
    "speech",
    "tty",
    "tv",
];
const DEPRECATED_FEATURES: &[&str] = &[
    "device-aspect-ratio",
    "min-device-aspect-ratio",
    "max-device-aspect-ratio",
    "device-width",
    "min-device-width",
    "max-device-width",
    "device-height",
    "min-device-height",
    "max-device-height",
];
const LENGTH_UNITS: &[&str] = &[
    "px", "em", "rem", "ex", "ch", "vw", "vh", "vmin", "vmax", "cm", "mm", "in", "pt", "pc", "q",
];
const RESOLUTION_UNITS: &[&str] = &["dpi", "dpcm", "dppx", "x"];

pub fn is_valid_media_query_list(val: &str) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Must not have empty entries (e.g. ",screen", "screen,", "screen,,print")
    for part in val.split(',') {
        let part_trimmed = part.trim();
        if part_trimmed.is_empty() || !is_valid_single_media_query(part_trimmed) {
            return false;
        }
    }
    true
}

fn is_valid_single_media_query(mut q: &str) -> bool {
    if q.contains(';') {
        return false;
    }
    q = q.trim();

    // Check restrictors "not" or "only"
    if let Some(rest) = q
        .strip_prefix("not")
        .or_else(|| q.strip_prefix("NOT"))
        .or_else(|| q.strip_prefix("only"))
        .or_else(|| q.strip_prefix("ONLY"))
    {
        if rest.starts_with(|c: char| c.is_ascii_whitespace()) {
            q = rest.trim_start();
        } else if !rest.is_empty() && !rest.starts_with('(') {
            return false;
        }
    }

    // Media Condition vs Media Type
    if !q.starts_with('(') {
        let ident_end = q
            .find(|c: char| c.is_ascii_whitespace() || c == '(')
            .unwrap_or(q.len());
        let mtype = &q[..ident_end].to_ascii_lowercase();
        if DEPRECATED_MEDIA_TYPES.contains(&mtype.as_str())
            || !ALLOWED_MEDIA_TYPES.contains(&mtype.as_str())
        {
            return false;
        }
        q = q[ident_end..].trim_start();
    } else {
        let close_paren = match q.find(')') {
            Some(idx) => idx,
            None => return false,
        };
        let feature_expr = &q[1..close_paren];
        if !is_valid_feature_expression(feature_expr) {
            return false;
        }
        q = q[close_paren + 1..].trim_start();
    }

    // Process [ and <media-in-parens> ]*
    while !q.is_empty() {
        if q.starts_with("and(") || q.starts_with("AND(") {
            return false;
        }
        let after_and = if let Some(rest) = q.strip_prefix("and").or_else(|| q.strip_prefix("AND"))
        {
            if rest.starts_with(|c: char| c.is_ascii_whitespace()) {
                rest.trim_start()
            } else {
                return false;
            }
        } else {
            return false;
        };

        if !after_and.starts_with('(') {
            return false;
        }

        let close_paren = match after_and.find(')') {
            Some(idx) => idx,
            None => return false,
        };

        let feature_expr = &after_and[1..close_paren];
        if !is_valid_feature_expression(feature_expr) {
            return false;
        }

        q = after_and[close_paren + 1..].trim_start();
    }

    true
}

fn is_valid_feature_expression(expr: &str) -> bool {
    let expr = expr.trim();
    if expr.is_empty() {
        return false;
    }

    if let Some((name, val)) = expr.split_once(':') {
        let name = name.trim().to_ascii_lowercase();
        let val = val.trim();
        if val.is_empty() || DEPRECATED_FEATURES.contains(&name.as_str()) {
            return false;
        }

        // Length features
        if matches!(
            name.as_str(),
            "width" | "min-width" | "max-width" | "height" | "min-height" | "max-height"
        ) {
            if val == "0" || val == "0.0" || val == "00" || val == ".0" {
                return true;
            }
            for &unit in LENGTH_UNITS {
                if let Some(num_str) = val.strip_suffix(unit) {
                    if num_str.trim().parse::<f64>().is_ok() {
                        return true;
                    }
                }
            }
            return false;
        }

        // Integer features
        if matches!(
            name.as_str(),
            "color"
                | "min-color"
                | "max-color"
                | "color-index"
                | "min-color-index"
                | "max-color-index"
                | "monochrome"
                | "grid"
        ) {
            return val.chars().all(|c| c.is_ascii_digit());
        }

        // Resolution features
        if matches!(
            name.as_str(),
            "resolution" | "min-resolution" | "max-resolution"
        ) {
            for &unit in RESOLUTION_UNITS {
                if let Some(num_str) = val.strip_suffix(unit) {
                    if num_str.trim().parse::<f64>().is_ok() {
                        return true;
                    }
                }
            }
            return false;
        }

        true
    } else {
        let name = expr.to_ascii_lowercase();
        !DEPRECATED_FEATURES.contains(&name.as_str())
    }
}
