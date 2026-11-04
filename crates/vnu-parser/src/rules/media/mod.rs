pub mod picture_context;
pub mod srcset_sizes;

use vnu_core::VNuMessage;
use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use self::picture_context::{handle_picture_child, handle_picture_open, PictureState};
use self::srcset_sizes::validate_media_attributes;

#[derive(Default)]
pub struct MediaRule {
    picture_stack: Vec<PictureState>,
}

impl Rule for MediaRule {
    fn name(&self) -> &'static str {
        "media"
    }

    fn reset(&mut self) {
        self.picture_stack.clear();
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, ctx: &ValidationContext) -> bool {
        ctx.open_elements.last().is_some_and(|top| top.name.eq_ignore_ascii_case("picture"))
    }

    fn on_characters(&mut self, _ctx: &ValidationContext, text: &str, line: usize, col: usize) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if !text.trim().is_empty() {
            msgs.push(VNuMessage::error(
                "Text not allowed in “picture” in this context.",
                line,
                col,
            ));
        }
        msgs
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let el = tag.name.to_ascii_lowercase();

        let is_in_picture = ctx
            .open_elements
            .last()
            .map(|e| e.name.eq_ignore_ascii_case("picture"))
            .unwrap_or(false);

        let is_in_video_or_audio = ctx
            .open_elements
            .last()
            .map(|e| e.name.eq_ignore_ascii_case("video") || e.name.eq_ignore_ascii_case("audio"))
            .unwrap_or(false);

        validate_media_attributes(tag, &el, is_in_picture, is_in_video_or_audio, &mut msgs);

        // Img constraints
        if el == "img" {
            if tag.has_attr("type") {
                msgs.push(VNuMessage::error(
                    "Attribute “type” not allowed on element “img” at this point.",
                    tag.line,
                    tag.col,
                ));
            }
            if !tag.has_attr("src") && !tag.has_attr("srcset") {
                msgs.push(VNuMessage::error(
                    "Element “img” is missing one or more of the following attributes: “src”, “srcset”.",
                    tag.line,
                    tag.col,
                ));
            }
        }

        // Picture element opening
        if el == "picture" {
            let state = handle_picture_open(ctx, tag, &mut msgs);
            self.picture_stack.push(state);
            return msgs;
        }

        // Children of picture
        if is_in_picture {
            if let Some(state) = self.picture_stack.last_mut() {
                handle_picture_child(state, tag, &el, &mut msgs);
            }
        }

        msgs
    }

    fn on_end_tag(&mut self, _ctx: &ValidationContext, name: &str, line: usize, col: usize) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if name.eq_ignore_ascii_case("picture") {
            if let Some(state) = self.picture_stack.pop() {
                if !state.seen_img {
                    msgs.push(VNuMessage::error(
                        "Element “picture” is missing a required instance of child element “img”.",
                        line,
                        col,
                    ));
                }
            }
        }
        msgs
    }
}
