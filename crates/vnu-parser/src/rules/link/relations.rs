use crate::context::ElementContext;
use crate::rules::media::srcset_sizes::parse_srcset;
use vnu_core::VNuMessage;

const PRELOAD_DESTINATIONS: &[&str] = &["fetch", "font", "image", "script", "style", "track"];
const MODULE_PRELOAD_DESTINATIONS: &[&str] = &[
    "audioworklet",
    "json",
    "paintworklet",
    "script",
    "serviceworker",
    "sharedworker",
    "style",
    "worker",
];

pub fn check_link_relations(
    tag: &ElementContext,
    has_rel: bool,
    rel_tokens: &[&str],
    msgs: &mut Vec<VNuMessage>,
) {
    if !has_rel && !tag.has_attr("itemprop") && !tag.has_attr("property") {
        msgs.push(VNuMessage::error(
            "Element “link” is missing one or more of the following attributes: “itemprop”, “property”, “rel”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("itemprop") && has_rel {
        msgs.push(VNuMessage::error(
            "Attribute “rel” not allowed on element “link” at this point.",
            tag.line,
            tag.col,
        ));
    }

    if !tag.has_attr("href") && !tag.has_attr("imagesrcset") && !tag.has_attr("resource") {
        msgs.push(VNuMessage::error(
            "A “link” element must have an “href” or “imagesrcset” attribute, or both.",
            tag.line,
            tag.col,
        ));
    }

    if rel_tokens.contains(&"preload") && !tag.has_attr("as") {
        msgs.push(VNuMessage::error(
            "A “link” element with a “rel” attribute that contains the value “preload” must have an “as” attribute.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("as")
        && (!has_rel
            || (!rel_tokens.contains(&"preload") && !rel_tokens.contains(&"modulepreload")))
    {
        msgs.push(VNuMessage::error(
            "A “link” element with an “as” attribute must have a “rel” attribute that contains the value “preload” or the value “modulepreload”.",
            tag.line,
            tag.col,
        ));
    }

    if let Some(as_val) = tag.get_attr("as") {
        if rel_tokens.contains(&"preload") && !PRELOAD_DESTINATIONS.contains(&as_val) {
            msgs.push(VNuMessage::error(
                format!(
                    "The value “{}” is not a valid value for the “as” attribute of a “link” element with “rel=preload”.",
                    as_val
                ),
                tag.line,
                tag.col,
            ));
        }
        if rel_tokens.contains(&"modulepreload") && !MODULE_PRELOAD_DESTINATIONS.contains(&as_val) {
            msgs.push(VNuMessage::error(
                format!(
                    "The value “{}” is not a valid value for the “as” attribute of a “link” element with “rel=modulepreload”.",
                    as_val
                ),
                tag.line,
                tag.col,
            ));
        }
    }

    if tag.has_attr("integrity")
        && (!has_rel
            || (!rel_tokens.contains(&"stylesheet")
                && !rel_tokens.contains(&"preload")
                && !rel_tokens.contains(&"modulepreload")))
    {
        msgs.push(VNuMessage::error(
            "A “link” element with an “integrity” attribute must have a “rel” attribute that contains the value “stylesheet” or the value “preload” or the value “modulepreload”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("disabled") && (!has_rel || !rel_tokens.contains(&"stylesheet")) {
        msgs.push(VNuMessage::error(
            "A “link” element with a “disabled” attribute must have a “rel” attribute that contains the value “stylesheet”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("sizes")
        && (!has_rel
            || (!rel_tokens.contains(&"icon")
                && !rel_tokens.contains(&"apple-touch-icon")
                && !rel_tokens.contains(&"apple-touch-icon-precomposed")))
    {
        msgs.push(VNuMessage::error(
            "A “link” element with a “sizes” attribute must have a “rel” attribute that contains the value “icon” or the value “apple-touch-icon” or the value “apple-touch-icon-precomposed”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("color") && (!has_rel || !rel_tokens.contains(&"mask-icon")) {
        msgs.push(VNuMessage::error(
            "A “link” element with a “color” attribute must have a “rel” attribute that contains the value “mask-icon”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("imagesrcset") && has_rel && !rel_tokens.contains(&"preload") {
        msgs.push(VNuMessage::error(
            "A “link” element with an “imagesrcset” attribute must have a “rel” attribute that contains the value “preload”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("imagesizes") && has_rel && !rel_tokens.contains(&"preload") {
        msgs.push(VNuMessage::error(
            "A “link” element with an “imagesizes” attribute must have a “rel” attribute that contains the value “preload”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("imagesrcset")
        && tag
            .get_attr("as")
            .is_none_or(|v| !v.eq_ignore_ascii_case("image"))
    {
        msgs.push(VNuMessage::error(
            "A “link” element with an “imagesrcset” attribute must have an “as” attribute with value “image”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("imagesizes")
        && tag
            .get_attr("as")
            .is_none_or(|v| !v.eq_ignore_ascii_case("image"))
    {
        msgs.push(VNuMessage::error(
            "A “link” element with an “imagesizes” attribute must have an “as” attribute with value “image”.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("imagesizes") && !tag.has_attr("imagesrcset") {
        msgs.push(VNuMessage::error(
            "The “imagesizes” attribute must only be specified if the “imagesrcset” attribute is also specified.",
            tag.line,
            tag.col,
        ));
    }

    if let Some(ss) = tag.get_attr("imagesrcset") {
        if let Some((has_w, _)) = parse_srcset(ss) {
            if has_w && !tag.has_attr("imagesizes") {
                msgs.push(VNuMessage::error(
                    "When the “imagesrcset” attribute has any image candidate string with a width descriptor, the “imagesizes” attribute must also be specified.",
                    tag.line,
                    tag.col,
                ));
            }
        }
    }

    if rel_tokens.contains(&"alternate")
        && rel_tokens.contains(&"stylesheet")
        && tag.get_attr("title").is_none_or(|t| t.is_empty())
    {
        msgs.push(VNuMessage::error(
            "A “link” element with a “rel” attribute that contains both the values “alternate” and “stylesheet” must have a “title” attribute with a non-empty value.",
            tag.line,
            tag.col,
        ));
    }

    if tag.has_attr("blocking") && (!has_rel || !rel_tokens.contains(&"stylesheet")) {
        msgs.push(VNuMessage::error(
            "A “link” element with a “blocking” attribute must have a “rel” attribute whose value is “stylesheet”.",
            tag.line,
            tag.col,
        ));
    }
}
