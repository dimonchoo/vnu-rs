use crate::context::{ElementContext, ValidationContext};
use vnu_core::VNuMessage;

const DISALLOWED_ON_PICTURE: &[&str] = &[
    "align",
    "alt",
    "border",
    "crossorigin",
    "height",
    "hspace",
    "ismap",
    "longdesc",
    "lowsrc",
    "media",
    "name",
    "role",
    "sizes",
    "src",
    "vspace",
    "width",
];

const DISALLOWED_ON_SOURCE_IN_PICTURE: &[&str] = &[
    "align",
    "alt",
    "border",
    "crossorigin",
    "hspace",
    "ismap",
    "longdesc",
    "role",
    "vspace",
];

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub has_media: bool,
    pub has_type: bool,
    pub sizes_starts_with_auto: bool,
    pub has_w_without_sizes: bool,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PictureState {
    pub seen_img: bool,
    pub sources: Vec<SourceInfo>,
}

pub fn handle_picture_open(
    ctx: &ValidationContext,
    tag: &ElementContext,
    msgs: &mut Vec<VNuMessage>,
) -> PictureState {
    if ctx.is_descendant_of("picture") {
        msgs.push(VNuMessage::error(
            "Element “picture” not allowed as child of “picture” in this context.",
            tag.line,
            tag.col,
        ));
    }
    if let Some(parent) = ctx.open_elements.last() {
        if matches!(
            parent.name.to_ascii_lowercase().as_str(),
            "dl" | "ul" | "hgroup" | "rp"
        ) {
            msgs.push(VNuMessage::error(
                format!(
                    "Element “picture” not allowed as child of “{}” in this context.",
                    parent.name.to_ascii_lowercase()
                ),
                tag.line,
                tag.col,
            ));
        }
    }
    for attr in DISALLOWED_ON_PICTURE {
        if tag.has_attr(attr) {
            msgs.push(VNuMessage::error(
                format!(
                    "Attribute “{}” not allowed on element “picture” at this point.",
                    attr
                ),
                tag.line,
                tag.col,
            ));
        }
    }
    PictureState::default()
}

pub fn handle_picture_child(
    state: &mut PictureState,
    tag: &ElementContext,
    el: &str,
    msgs: &mut Vec<VNuMessage>,
) {
    if !matches!(el, "source" | "img" | "script" | "template") {
        msgs.push(VNuMessage::error(
            format!(
                "Element “{}” not allowed as child of “picture” in this context.",
                el
            ),
            tag.line,
            tag.col,
        ));
    } else if el == "source" {
        if !tag.has_attr("srcset") {
            msgs.push(VNuMessage::error(
                "Element “source” is missing required attribute “srcset”.",
                tag.line,
                tag.col,
            ));
        }
        if tag.has_attr("src") {
            msgs.push(VNuMessage::error(
                "Attribute “src” not allowed on element “source” at this point.",
                tag.line,
                tag.col,
            ));
        }
        for attr in DISALLOWED_ON_SOURCE_IN_PICTURE {
            if tag.has_attr(attr) {
                msgs.push(VNuMessage::error(
                    format!(
                        "Attribute “{}” not allowed on element “source” at this point.",
                        attr
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
        if state.seen_img {
            msgs.push(VNuMessage::error(
                "Element “source” not allowed as child of “picture” in this context.",
                tag.line,
                tag.col,
            ));
        }
        if let Some(media) = tag.get_attr("media") {
            let trimmed = media.trim();
            if trimmed.is_empty() {
                msgs.push(VNuMessage::error(
                    "Value of “media” attribute here must not be empty.",
                    tag.line,
                    tag.col,
                ));
            } else if trimmed.eq_ignore_ascii_case("all") {
                msgs.push(VNuMessage::error(
                    "Value of “media” attribute here must not be “all”.",
                    tag.line,
                    tag.col,
                ));
            } else if trimmed.contains(':') && trimmed.ends_with(":)") {
                msgs.push(VNuMessage::error(
                    format!(
                        "Bad value “{}” for attribute “media” on element “source”.",
                        media
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
        let sizes_auto = tag
            .get_attr("sizes")
            .map(|s| s.trim().to_ascii_lowercase().starts_with("auto"))
            .unwrap_or(false);
        let has_w = tag
            .get_attr("srcset")
            .and_then(super::srcset_sizes::parse_srcset)
            .is_some_and(|(w, _)| w);

        state.sources.push(SourceInfo {
            has_media: tag.has_attr("media"),
            has_type: tag.has_attr("type"),
            sizes_starts_with_auto: sizes_auto,
            has_w_without_sizes: has_w && !tag.has_attr("sizes"),
            line: tag.line,
            col: tag.col,
        });
    } else if el == "img" {
        if state.seen_img {
            msgs.push(VNuMessage::error(
                "Element “img” not allowed as child of “picture” in this context.",
                tag.line,
                tag.col,
            ));
        } else {
            state.seen_img = true;
            let img_has_lazy = tag.has_attr_val("loading", "lazy");
            let img_sizes_auto = tag
                .get_attr("sizes")
                .map(|s| s.trim().to_ascii_lowercase().starts_with("auto"))
                .unwrap_or(false);
            let img_has_srcset = tag.has_attr("srcset");
            let num_sources = state.sources.len();

            for (i, src) in state.sources.iter().enumerate() {
                if !src.has_media && !src.has_type && (i + 1 < num_sources || img_has_srcset) {
                    msgs.push(VNuMessage::error(
                        "A “source” element that has a following sibling “source” element or “img” element with a “srcset” attribute must have a “media” attribute and/or “type” attribute.",
                        src.line,
                        src.col,
                    ));
                }
                if src.has_w_without_sizes && (!img_has_lazy || !img_sizes_auto) {
                    msgs.push(VNuMessage::error(
                        "When the “srcset” attribute has any image candidate string with a width descriptor, the “sizes” attribute must also be specified — unless the following “img” element has a “loading” attribute set to “lazy” and a “sizes” attribute value starting with “auto”.",
                        src.line,
                        src.col,
                    ));
                }
                if src.sizes_starts_with_auto && (!img_has_lazy || !img_sizes_auto) {
                    msgs.push(VNuMessage::error(
                        "A “sizes” attribute value starting with “auto” on a “source” element requires the following “img” element to have a “loading” attribute set to “lazy” and a “sizes” attribute value starting with “auto”.",
                        src.line,
                        src.col,
                    ));
                }
            }
        }
    }
}
