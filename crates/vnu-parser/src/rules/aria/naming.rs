use crate::context::ElementContext;

pub const PROHIBITED_NAMED_ELEMENTS: &[&str] = &[
    "abbr",
    "b",
    "bdi",
    "bdo",
    "body",
    "caption",
    "cite",
    "code",
    "data",
    "del",
    "div",
    "em",
    "figcaption",
    "i",
    "ins",
    "kbd",
    "legend",
    "mark",
    "p",
    "pre",
    "q",
    "rp",
    "rt",
    "s",
    "samp",
    "small",
    "span",
    "strong",
    "sub",
    "sup",
    "time",
    "u",
    "var",
];

pub const ROLES_WHICH_CANNOT_BE_NAMED: &[&str] = &[
    "caption",
    "code",
    "deletion",
    "emphasis",
    "generic",
    "insertion",
    "paragraph",
    "presentation",
    "strong",
    "subscript",
    "superscript",
];

pub fn is_prohibited_from_being_named(elem_name: &str, tag: &ElementContext) -> bool {
    let matches_elem = elem_name.contains('-')
        || (elem_name.eq_ignore_ascii_case("a") && !tag.has_attr("href"))
        || (elem_name.eq_ignore_ascii_case("area") && !tag.has_attr("href"))
        || PROHIBITED_NAMED_ELEMENTS
            .iter()
            .any(|&e| e.eq_ignore_ascii_case(elem_name));

    if !matches_elem {
        return false;
    }

    if let Some(role_attr) = tag.get_attr("role") {
        let mut has_role = false;
        for r in role_attr.split_whitespace() {
            has_role = true;
            if ROLES_WHICH_CANNOT_BE_NAMED
                .iter()
                .any(|&unnameable| unnameable.eq_ignore_ascii_case(r))
            {
                return true;
            }
        }
        if has_role {
            return false;
        }
    }
    true
}
