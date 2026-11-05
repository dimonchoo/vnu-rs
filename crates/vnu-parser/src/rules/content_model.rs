use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

fn allowed_parents(tag_name: &str) -> Option<&'static [&'static str]> {
    match tag_name {
        "li" => Some(&["ul", "ol", "menu", "template"]),
        "dt" | "dd" => Some(&["dl", "div", "template"]),
        "source" => Some(&["audio", "video", "picture", "template"]),
        "track" => Some(&["audio", "video", "template"]),
        "area" => Some(&["map", "template"]),
        "param" => Some(&["object", "template"]),
        "optgroup" => Some(&["select", "template"]),
        "option" => Some(&["select", "optgroup", "datalist", "template"]),
        _ => None,
    }
}

const TRANSPARENT_ELEMENTS: &[&str] = &["a", "audio", "video", "canvas", "del", "ins"];

#[derive(Default)]
pub struct ContentModelRule {
    details_stack: Vec<(usize, bool)>, // (child_count, has_valid_summary)
    ruby_stack: Vec<(usize, usize, usize)>, // (rp_count, rt_count, rtc_count)
    media_default_track_seen: Vec<bool>,
    fieldset_has_non_legend: Vec<bool>,
    video_has_non_source: Vec<bool>,
}

impl ContentModelRule {
    fn check_media_and_track(
        &mut self,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if name_lower == "audio" || name_lower == "video" {
            self.media_default_track_seen.push(false);
            if name_lower == "video" {
                self.video_has_non_source.push(false);
            }
        } else if name_lower == "track" && tag.has_attr("default") {
            if let Some(seen) = self.media_default_track_seen.last_mut() {
                if *seen {
                    msgs.push(VNuMessage::error(
                        "The “default” attribute must not occur on more than one “track” element within the same “audio” or “video” element.",
                        tag.line,
                        tag.col,
                    ));
                }
                *seen = true;
            }
        }
    }

    fn check_details_and_summary(
        &mut self,
        ctx: &ValidationContext,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if name_lower == "details" {
            self.details_stack.push((0, false));
        }

        if let Some(parent) = ctx.parent() {
            if parent.name.eq_ignore_ascii_case("details") {
                if let Some((child_count, has_valid_summary)) = self.details_stack.last_mut() {
                    if name_lower == "summary" {
                        if *child_count == 0 {
                            *has_valid_summary = true;
                        } else {
                            msgs.push(VNuMessage::error(
                                "Element “summary” not allowed as child of “details” in this context.",
                                tag.line,
                                tag.col,
                            ));
                        }
                    }
                    *child_count += 1;
                }
            }
        }
    }

    fn check_fieldset_and_legend(
        &mut self,
        ctx: &ValidationContext,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if name_lower == "fieldset" {
            self.fieldset_has_non_legend.push(false);
        }

        if let Some(parent) = ctx.parent() {
            if parent.name.eq_ignore_ascii_case("fieldset") {
                if name_lower == "legend" {
                    if let Some(true) = self.fieldset_has_non_legend.last() {
                        msgs.push(VNuMessage::error(
                            "Element “legend” not allowed as child of “fieldset” in this context.",
                            tag.line,
                            tag.col,
                        ));
                    }
                } else if let Some(flag) = self.fieldset_has_non_legend.last_mut() {
                    *flag = true;
                }
            }
        }
    }

    fn check_video_and_source(
        &mut self,
        ctx: &ValidationContext,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if let Some(parent) = ctx.parent() {
            if parent.name.eq_ignore_ascii_case("video") {
                if name_lower == "source" {
                    if let Some(true) = self.video_has_non_source.last() {
                        msgs.push(VNuMessage::error(
                            "Element “source” not allowed as child of “video” in this context. Note: The “video” element has a transparent content model; its allowed content is inherited from its parent element.",
                            tag.line,
                            tag.col,
                        ));
                    }
                } else if let Some(flag) = self.video_has_non_source.last_mut() {
                    *flag = true;
                }
            }
        }
    }

    fn check_ruby_and_annotations(
        &mut self,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if name_lower == "rb" {
            msgs.push(VNuMessage::info(
                "Not all browsers position items appropriately when \"tabular markup\" is used with the “rb” element. See https://www.w3.org/International/articles/ruby/markup.en.html#visual for more guidance.",
                tag.line,
                tag.col,
            ));
        } else if name_lower == "rtc" {
            msgs.push(VNuMessage::info(
                "Not all browsers position items appropriately when the “rtc” element is used. See https://www.w3.org/International/articles/ruby/markup.en.html#visual for more guidance.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "ruby" {
            self.ruby_stack.push((0, 0, 0));
        } else if let Some((rp, rt, rtc)) = self.ruby_stack.last_mut() {
            if name_lower == "rp" {
                *rp += 1;
            } else if name_lower == "rt" {
                *rt += 1;
            } else if name_lower == "rtc" {
                *rtc += 1;
            }
        }
    }

    fn check_colgroup_col(
        ctx: &ValidationContext,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if name_lower == "col" {
            if let Some(parent) = ctx.parent() {
                if parent.name.eq_ignore_ascii_case("colgroup") && parent.has_attr("span") {
                    msgs.push(VNuMessage::error(
                        "Element “col” not allowed as child of “colgroup” in this context.",
                        tag.line,
                        tag.col,
                    ));
                }
            }
        }
    }

    fn check_object_data(tag: &ElementContext, name_lower: &str, msgs: &mut Vec<VNuMessage>) {
        if name_lower == "object" && !tag.has_attr("data") {
            msgs.push(VNuMessage::error(
                "Element “object” is missing required attribute “data”.",
                tag.line,
                tag.col,
            ));
        }
    }

    fn check_transparent_content_model(
        ctx: &ValidationContext,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if let Some(parent) = ctx.parent() {
            let parent_name = &parent.name;
            if TRANSPARENT_ELEMENTS
                .iter()
                .any(|&t| t.eq_ignore_ascii_case(parent_name))
            {
                let non_transparent_ancestor = ctx.open_elements.iter().rev().skip(1).find(|e| {
                    !TRANSPARENT_ELEMENTS
                        .iter()
                        .any(|&t| t.eq_ignore_ascii_case(&e.name))
                });

                if let Some(anc) = non_transparent_ancestor {
                    let is_phrasing = matches!(
                        anc.name.to_ascii_lowercase().as_str(),
                        "p" | "span" | "em" | "strong" | "small" | "b" | "i" | "u" | "q" | "label"
                    );
                    if (is_phrasing && (name_lower == "p" || name_lower == "div"))
                        || (anc.name.eq_ignore_ascii_case("text") && name_lower == "tspan")
                    {
                        msgs.push(VNuMessage::error(
                            format!(
                                "Element “{}” not allowed as child of “{}” in this context. Note: The “{}” element has a transparent content model; its allowed content is inherited from its parent element.",
                                name_lower, parent_name, parent_name
                            ),
                            tag.line,
                            tag.col,
                        ));
                    }
                }
            }
        }
    }

    fn check_allowed_parents(
        ctx: &ValidationContext,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if let Some(expected_parents) = allowed_parents(name_lower) {
            let parent_opt = ctx.parent();
            let mut parent_matches = parent_opt
                .map(|p| {
                    expected_parents
                        .iter()
                        .any(|&exp| exp.eq_ignore_ascii_case(&p.name))
                })
                .unwrap_or(false);

            if !parent_matches && name_lower == "area" && ctx.is_descendant_of("map") {
                parent_matches = true;
            }

            if !parent_matches
                && (name_lower == "option" || name_lower == "optgroup")
                && parent_opt.is_some_and(|p| p.name.eq_ignore_ascii_case("div"))
                && ctx.is_descendant_of("select")
            {
                parent_matches = true;
            }

            if !parent_matches {
                let parent_name = parent_opt.map(|p| p.name.as_str()).unwrap_or("body");
                msgs.push(VNuMessage::error(
                    format!(
                        "Element “{}” not allowed as child of “{}” in this context.",
                        name_lower, parent_name
                    ),
                    tag.line,
                    tag.col,
                ));
            }
        }
    }
}

impl Rule for ContentModelRule {
    fn name(&self) -> &'static str {
        "content-model"
    }

    fn reset(&mut self) {
        self.details_stack.clear();
        self.ruby_stack.clear();
        self.media_default_track_seen.clear();
        self.fieldset_has_non_legend.clear();
        self.video_has_non_source.clear();
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, ctx: &ValidationContext) -> bool {
        ctx.parent()
            .is_some_and(|p| p.name.eq_ignore_ascii_case("iframe"))
    }

    fn on_characters(
        &mut self,
        ctx: &ValidationContext,
        text: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if let Some(parent) = ctx.parent() {
            if parent.name.eq_ignore_ascii_case("iframe") && !text.trim().is_empty() {
                msgs.push(VNuMessage::error(
                    "Text not allowed in “iframe” in this context.",
                    line,
                    col,
                ));
            }
        }
        msgs
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        self.check_media_and_track(tag, &name_lower, &mut msgs);
        self.check_details_and_summary(ctx, tag, &name_lower, &mut msgs);
        self.check_fieldset_and_legend(ctx, tag, &name_lower, &mut msgs);
        self.check_video_and_source(ctx, tag, &name_lower, &mut msgs);
        self.check_ruby_and_annotations(tag, &name_lower, &mut msgs);
        Self::check_colgroup_col(ctx, tag, &name_lower, &mut msgs);
        Self::check_object_data(tag, &name_lower, &mut msgs);
        Self::check_transparent_content_model(ctx, tag, &name_lower, &mut msgs);
        Self::check_allowed_parents(ctx, tag, &name_lower, &mut msgs);

        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        name: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if name.eq_ignore_ascii_case("details") {
            if let Some((_, has_valid_summary)) = self.details_stack.pop() {
                if !has_valid_summary {
                    msgs.push(VNuMessage::error(
                        "Element “details” is missing a required instance of child element “summary”.",
                        line,
                        col,
                    ));
                }
            }
        } else if name.eq_ignore_ascii_case("fieldset") {
            self.fieldset_has_non_legend.pop();
        } else if name.eq_ignore_ascii_case("video") {
            self.video_has_non_source.pop();
            self.media_default_track_seen.pop();
        } else if name.eq_ignore_ascii_case("audio") {
            self.media_default_track_seen.pop();
        } else if name.eq_ignore_ascii_case("ruby") {
            if let Some((rp, rt, rtc)) = self.ruby_stack.pop() {
                if rt == 0 && rtc == 0 && rp == 0 {
                    msgs.push(VNuMessage::error(
                        "Element “ruby” is missing a required instance of one or more of the following child elements: “rp”, “rt”, “rtc”.",
                        line,
                        col,
                    ));
                } else if rp >= 2 && rt == 1 {
                    msgs.push(VNuMessage::error(
                        "Element “ruby” is missing a required instance of child element “rt”.",
                        line,
                        col,
                    ));
                }
            }
        }
        msgs
    }
}
