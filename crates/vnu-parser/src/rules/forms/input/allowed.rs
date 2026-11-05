const AUTOCOMPLETE_TYPES: &[&str] = &[
    "color",
    "date",
    "datetime-local",
    "email",
    "hidden",
    "month",
    "number",
    "password",
    "range",
    "search",
    "tel",
    "text",
    "time",
    "url",
    "week",
];
const LIST_TYPES: &[&str] = &[
    "color",
    "date",
    "datetime-local",
    "email",
    "month",
    "number",
    "range",
    "search",
    "tel",
    "text",
    "time",
    "url",
    "week",
];
const MAXLENGTH_TYPES: &[&str] = &["email", "password", "search", "tel", "text", "url"];
const MINLENGTH_TYPES: &[&str] = &["email", "password", "search", "tel", "text", "url"];
const PATTERN_TYPES: &[&str] = &["email", "password", "search", "tel", "text", "url"];
const PLACEHOLDER_TYPES: &[&str] = &[
    "email", "number", "password", "search", "tel", "text", "url",
];
const READONLY_TYPES: &[&str] = &[
    "date",
    "datetime-local",
    "email",
    "month",
    "number",
    "password",
    "search",
    "tel",
    "text",
    "time",
    "url",
    "week",
];
const REQUIRED_TYPES: &[&str] = &[
    "checkbox",
    "date",
    "datetime-local",
    "email",
    "file",
    "month",
    "number",
    "password",
    "radio",
    "search",
    "tel",
    "text",
    "time",
    "url",
    "week",
];
#[allow(dead_code)]
const SIZE_TYPES: &[&str] = &["email", "password", "search", "tel", "text", "url"];

pub fn get_input_attribute_allowed(attr: &str) -> Option<&'static [&'static str]> {
    match attr {
        "autocomplete" => Some(AUTOCOMPLETE_TYPES),
        "list" => Some(LIST_TYPES),
        "maxlength" => Some(MAXLENGTH_TYPES),
        "minlength" => Some(MINLENGTH_TYPES),
        "pattern" => Some(PATTERN_TYPES),
        "placeholder" => Some(PLACEHOLDER_TYPES),
        "readonly" => Some(READONLY_TYPES),
        "required" => Some(REQUIRED_TYPES),
        // Note: in upstream Assertions.java, "size" is present in INPUT_ATTRIBUTES,
        // but line 2856 has `else if (OBSOLETE_STYLE_ATTRS.containsKey(attLocal))` which matches "size"
        // (for hr) and shadows the input size check, so upstream Java never enforces input type for size.
        _ => None,
    }
}

pub fn is_attr_allowed_for_type(attr: &str, input_type: &str) -> bool {
    match attr {
        "accept" | "capture" => input_type == "file",
        "alt" | "src" | "height" | "width" => input_type == "image",
        "checked" => matches!(input_type, "checkbox" | "radio"),
        "dirname" => matches!(
            input_type,
            "hidden" | "text" | "search" | "tel" | "url" | "email" | "password" | "submit"
        ),
        "formaction" | "formenctype" | "formmethod" | "formnovalidate" | "formtarget" => {
            matches!(input_type, "submit" | "image")
        }
        "min" | "max" | "step" => matches!(
            input_type,
            "date" | "month" | "week" | "time" | "datetime-local" | "number" | "range"
        ),
        "multiple" => matches!(input_type, "email" | "file"),
        "value" => input_type != "file",
        "alpha" | "colorspace" => input_type == "color",
        "popovertarget" | "popovertargetaction" => {
            matches!(input_type, "button" | "submit" | "reset" | "image")
        }
        "srcset" => false,
        _ => true,
    }
}

pub fn is_global_or_common_attr(attr: &str) -> bool {
    if attr.starts_with("data-")
        || attr.starts_with("aria-")
        || attr.starts_with("on")
        || attr.starts_with("xmlns")
        || attr.starts_with("xml:")
    {
        return true;
    }
    matches!(
        attr,
        "type"
            | "name"
            | "disabled"
            | "form"
            | "autofocus"
            | "id"
            | "class"
            | "style"
            | "title"
            | "lang"
            | "dir"
            | "hidden"
            | "tabindex"
            | "accesskey"
            | "contenteditable"
            | "spellcheck"
            | "translate"
            | "inputmode"
            | "enterkeyhint"
            | "inert"
            | "popover"
            | "slot"
            | "is"
            | "role"
            | "itemid"
            | "itemprop"
            | "itemref"
            | "itemscope"
            | "itemtype"
            | "about"
            | "content"
            | "datatype"
            | "inlist"
            | "prefix"
            | "property"
            | "resource"
            | "typeof"
            | "vocab"
    )
}

pub fn render_type_list(types: &[&str]) -> String {
    let mut s = String::new();
    let len = types.len();
    for (i, t) in types.iter().enumerate() {
        if i > 0 {
            if len == 2 {
                s.push_str(" or ");
            } else if i == len - 1 {
                s.push_str(", or ");
            } else {
                s.push_str(", ");
            }
        }
        s.push('“');
        s.push_str(t);
        s.push('”');
    }
    s
}
