use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use vnu_core::VNuMessage;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FigureState {
    Initial,
    FlowBeforeCaption,
    CaptionFirst,
    CaptionAfterFlow,
}

#[derive(Default)]
pub struct FigureRule {
    figure_stack: Vec<FigureState>,
}

impl Rule for FigureRule {
    fn name(&self) -> &'static str {
        "figure"
    }

    fn handles_character_tokens(&self) -> bool {
        true
    }

    fn interested_in_characters(&self, ctx: &ValidationContext) -> bool {
        ctx.open_elements
            .last()
            .is_some_and(|top| top.name.eq_ignore_ascii_case("figure"))
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if let Some(state) = self.figure_stack.last_mut() {
            let parent_is_figure = ctx
                .parent()
                .is_some_and(|p| p.name.eq_ignore_ascii_case("figure"));
            if parent_is_figure {
                if name_lower == "figcaption" {
                    match *state {
                        FigureState::Initial => {
                            *state = FigureState::CaptionFirst;
                        }
                        FigureState::FlowBeforeCaption => {
                            *state = FigureState::CaptionAfterFlow;
                        }
                        FigureState::CaptionFirst | FigureState::CaptionAfterFlow => {
                            msgs.push(VNuMessage::error(
                                "Element “figcaption” not allowed as child of “figure” in this context.",
                                tag.line,
                                tag.col,
                            ));
                        }
                    }
                } else {
                    match *state {
                        FigureState::Initial => {
                            *state = FigureState::FlowBeforeCaption;
                        }
                        FigureState::CaptionAfterFlow => {
                            msgs.push(VNuMessage::error(
                                format!("Element “{}” not allowed as child of “figure” in this context.", name_lower),
                                tag.line,
                                tag.col,
                            ));
                        }
                        FigureState::FlowBeforeCaption | FigureState::CaptionFirst => {}
                    }
                }
            }
        }

        if name_lower == "figure" {
            self.figure_stack.push(FigureState::Initial);
        }

        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        tag_name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        if tag_name.eq_ignore_ascii_case("figure") {
            self.figure_stack.pop();
        }
        vec![]
    }

    fn on_characters(
        &mut self,
        ctx: &ValidationContext,
        text: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if text.trim().is_empty() {
            return msgs;
        }

        if let Some(state) = self.figure_stack.last_mut() {
            let parent_is_figure = ctx
                .open_elements
                .last()
                .is_some_and(|p| p.name.eq_ignore_ascii_case("figure"));
            if parent_is_figure {
                match *state {
                    FigureState::Initial => {
                        *state = FigureState::FlowBeforeCaption;
                    }
                    FigureState::CaptionAfterFlow => {
                        msgs.push(VNuMessage::error(
                            "Text not allowed in “figure” in this context.",
                            line,
                            col,
                        ));
                    }
                    FigureState::FlowBeforeCaption | FigureState::CaptionFirst => {}
                }
            }
        }

        msgs
    }
}
