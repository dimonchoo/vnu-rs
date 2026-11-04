use crate::context::ElementContext;
use vnu_core::VNuMessage;

pub fn check_meta_attributes(tag: &ElementContext, msgs: &mut Vec<VNuMessage>) {
    if tag.has_attr("charset") {
        if tag.has_attr("content") {
            msgs.push(VNuMessage::error(
                "Attribute “content” not allowed on element “meta” at this point.",
                tag.line,
                tag.col,
            ));
        }

        if let Some(charset) = tag.get_attr("charset") {
            if !charset.eq_ignore_ascii_case("utf-8") {
                msgs.push(VNuMessage::error(
                    format!(
                        "Internal encoding declaration “{}” disagrees with the actual encoding of the document (“utf-8”).",
                        charset
                    ),
                    tag.line,
                    tag.col,
                ));
                msgs.push(VNuMessage::error(
                    "The only allowed value for the “charset” attribute for the “meta” element is “utf-8”.",
                    tag.line,
                    tag.col,
                ));
            }
        }
    }

    if tag.has_attr("name") {
        if tag.has_attr("itemprop") {
            msgs.push(VNuMessage::error(
                "Attribute “itemprop” not allowed on element “meta” at this point.",
                tag.line,
                tag.col,
            ));
        }

        if tag.has_attr_val("name", "viewport") {
            if let Some(content) = tag.get_attr("content") {
                let lower = content.to_ascii_lowercase();
                if lower.contains("user-scalable=no")
                    || lower.contains("user-scalable=0")
                    || lower.contains("maximum-scale=1.0")
                {
                    msgs.push(VNuMessage::warning(
                        "Consider avoiding viewport values that prevent users from resizing documents.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }
    }

    if tag.has_attr("media")
        && tag
            .get_attr("name")
            .is_none_or(|n| !n.eq_ignore_ascii_case("theme-color"))
    {
        msgs.push(VNuMessage::error(
            "A “meta” element with a “media” attribute must have a “name” attribute whose value is “theme-color”.",
            tag.line,
            tag.col,
        ));
    }

    if let Some(equiv) = tag.get_attr("http-equiv") {
        if equiv.eq_ignore_ascii_case("content-language") {
            msgs.push(VNuMessage::error(
                "Using the “meta” element to specify the document-wide default language is obsolete. Consider specifying the language on the root element instead.",
                tag.line,
                tag.col,
            ));
        } else if equiv.eq_ignore_ascii_case("x-ua-compatible")
            && tag
                .get_attr("content")
                .is_none_or(|c| !c.eq_ignore_ascii_case("ie=edge"))
        {
            msgs.push(VNuMessage::error(
                "A “meta” element with an “http-equiv” attribute whose value is “X-UA-Compatible” must have a “content” attribute with the value “IE=edge”.",
                tag.line,
                tag.col,
            ));
        }
    }
}
