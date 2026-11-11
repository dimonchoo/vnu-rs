use crate::context::{DocumentMode, ElementContext, ValidationContext};

pub use crate::schema::is_void_element;

pub fn apply_implicit_closures(
    open_elements: &mut Vec<ElementContext>,
    tag_name: &str,
    is_start_tag: bool,
    line: usize,
    col: usize,
) -> Option<vnu_core::VNuMessage> {
    let mut msg = None;

    if is_start_tag {
        if tag_name.eq_ignore_ascii_case("li") {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("li") {
                    open_elements.pop();
                }
            }
        } else if tag_name.eq_ignore_ascii_case("option") {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("option") {
                    open_elements.pop();
                }
            }
        } else if tag_name.eq_ignore_ascii_case("optgroup") {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("option") {
                    open_elements.pop();
                }
            }
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("optgroup") {
                    open_elements.pop();
                }
            }
        } else if tag_name.eq_ignore_ascii_case("rb")
            || tag_name.eq_ignore_ascii_case("rt")
            || tag_name.eq_ignore_ascii_case("rp")
            || tag_name.eq_ignore_ascii_case("rtc")
        {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("rb")
                    || top.name.eq_ignore_ascii_case("rt")
                    || top.name.eq_ignore_ascii_case("rp")
                {
                    open_elements.pop();
                }
            }
            if tag_name.eq_ignore_ascii_case("rb") || tag_name.eq_ignore_ascii_case("rtc") {
                if let Some(top) = open_elements.last() {
                    if top.name.eq_ignore_ascii_case("rtc") {
                        open_elements.pop();
                    }
                }
            }
        } else if tag_name.eq_ignore_ascii_case("dt") || tag_name.eq_ignore_ascii_case("dd") {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("p") {
                    open_elements.pop();
                }
            }
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("dt") || top.name.eq_ignore_ascii_case("dd") {
                    open_elements.pop();
                }
            }
        } else if tag_name.eq_ignore_ascii_case("td") || tag_name.eq_ignore_ascii_case("th") {
            if let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("td") || top.name.eq_ignore_ascii_case("th") {
                    open_elements.pop();
                }
            }
        } else if tag_name.eq_ignore_ascii_case("tr") {
            while let Some(top) = open_elements.last() {
                if top.name.eq_ignore_ascii_case("td")
                    || top.name.eq_ignore_ascii_case("th")
                    || top.name.eq_ignore_ascii_case("tr")
                {
                    open_elements.pop();
                } else {
                    break;
                }
            }
        } else if tag_name.eq_ignore_ascii_case("p")
            || [
                "dl",
                "div",
                "ul",
                "ol",
                "table",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "pre",
                "blockquote",
                "form",
                "hr",
                "header",
                "footer",
                "nav",
                "section",
                "article",
            ]
            .iter()
            .any(|&e| e.eq_ignore_ascii_case(tag_name))
        {
            if let Some(pos) = open_elements
                .iter()
                .rposition(|e| e.name.eq_ignore_ascii_case("p"))
            {
                if pos < open_elements.len() - 1 {
                    msg = Some(vnu_core::VNuMessage::error(
                        "End tag “p” implied, but there were open elements.",
                        line,
                        col,
                    ));
                }
                open_elements.truncate(pos);
            }
        }
    } else if tag_name.eq_ignore_ascii_case("ul") || tag_name.eq_ignore_ascii_case("ol") {
        if let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("li") {
                open_elements.pop();
            }
        }
    } else if tag_name.eq_ignore_ascii_case("dl") || tag_name.eq_ignore_ascii_case("div") {
        if let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("dt") || top.name.eq_ignore_ascii_case("dd") {
                open_elements.pop();
            }
        }
    } else if tag_name.eq_ignore_ascii_case("tr") {
        if let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("td") || top.name.eq_ignore_ascii_case("th") {
                open_elements.pop();
            }
        }
    } else if tag_name.eq_ignore_ascii_case("datalist")
        || tag_name.eq_ignore_ascii_case("select")
        || tag_name.eq_ignore_ascii_case("optgroup")
    {
        if let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("option") {
                open_elements.pop();
            }
        }
    } else if ["table", "tbody", "thead", "tfoot"]
        .iter()
        .any(|&e| e.eq_ignore_ascii_case(tag_name))
    {
        while let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("td")
                || top.name.eq_ignore_ascii_case("th")
                || top.name.eq_ignore_ascii_case("tr")
            {
                open_elements.pop();
            } else {
                break;
            }
        }
    } else if tag_name.eq_ignore_ascii_case("ruby") {
        while let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("rb")
                || top.name.eq_ignore_ascii_case("rt")
                || top.name.eq_ignore_ascii_case("rp")
                || top.name.eq_ignore_ascii_case("rtc")
            {
                open_elements.pop();
            } else {
                break;
            }
        }
    } else if tag_name.eq_ignore_ascii_case("html") {
        if let Some(top) = open_elements.last() {
            if top.name.eq_ignore_ascii_case("body") {
                open_elements.pop();
            }
        }
    }

    msg
}

#[inline]
pub fn is_head_or_top_element(name: &str) -> bool {
    const HEAD_OR_TOP: &[&str] = &[
        "base", "body", "head", "html", "link", "meta", "noscript", "script", "style", "template",
        "title",
    ];
    HEAD_OR_TOP.iter().any(|&e| e.eq_ignore_ascii_case(name))
}

pub fn ensure_in_body_for_tag(
    ctx: &mut ValidationContext,
    tag_name: &str,
    line: usize,
    col: usize,
) -> Option<vnu_core::VNuMessage> {
    let mut msg = None;
    if ctx.mode == DocumentMode::Html
        && !ctx.body_seen
        && !is_head_or_top_element(tag_name)
        && !ctx.is_descendant_of("template")
    {
        if ctx
            .open_elements
            .iter()
            .any(|e| e.name.eq_ignore_ascii_case("noscript"))
        {
            msg = Some(vnu_core::VNuMessage::error(
                format!("Bad start tag in “{}” in “noscript” in “head”.", tag_name),
                line,
                col,
            ));
        }
        if let Some(pos) = ctx
            .open_elements
            .iter()
            .rposition(|e| e.name.eq_ignore_ascii_case("head"))
        {
            ctx.open_elements.truncate(pos);
        }
        ctx.body_seen = true;
        let body_elem = ElementContext {
            name: "body".to_string(),
            attrs: std::rc::Rc::from(Vec::new()),
            line,
            col,
            byte_offset: 0,
        };
        ctx.open_elements.push(body_elem);
    }
    msg
}
