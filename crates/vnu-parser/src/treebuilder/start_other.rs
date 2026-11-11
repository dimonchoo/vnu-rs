use super::names::groups::*;
use super::*;

use super::start_tag::StartTag;

/// Start tags in the head modes and the modes before and after the body.
impl TreeBuilder {
    pub(super) fn start_in_head(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            HTML => {
                self.err_stray_start_tag(&t.en.name);
                if !self.is_template_contents() {
                    self.add_attributes_to_html(&t.attrs);
                }
                Step::Done
            }
            BASE | LINK_OR_BASEFONT_OR_BGSOUND => {
                self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                t.self_closing = false;
                t.void_element = true;
                Step::Done
            }
            META => Step::FallTo(Mode::InHeadNoscript),
            TITLE => {
                t.state =
                    Some(self.start_tag_text(&t.en, t.attrs.clone(), TokenizerState::Rcdata, true));
                Step::Done
            }
            NOSCRIPT => {
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                self.mode = Mode::InHeadNoscript;
                Step::Done
            }
            SCRIPT => {
                t.state = Some(self.start_tag_text(
                    &t.en,
                    t.attrs.clone(),
                    TokenizerState::ScriptData,
                    true,
                ));
                Step::Done
            }
            STYLE | NOFRAMES => {
                t.state = Some(self.start_tag_text(
                    &t.en,
                    t.attrs.clone(),
                    TokenizerState::Rawtext,
                    true,
                ));
                Step::Done
            }
            HEAD => {
                self.err_foo_seen_when_foo_open(&t.en.name);
                Step::Done
            }
            TEMPLATE => {
                self.start_tag_template_in_head(t.attrs.clone());
                Step::Done
            }
            _ => {
                self.pop();
                self.mode = Mode::AfterHead;
                Step::Reprocess
            }
        }
    }

    pub(super) fn start_in_head_noscript(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            HTML => {
                self.err_stray_start_tag(&t.en.name);
                if !self.is_template_contents() {
                    self.add_attributes_to_html(&t.attrs);
                }
                Step::Done
            }
            LINK_OR_BASEFONT_OR_BGSOUND | META => {
                self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                t.self_closing = false;
                t.void_element = true;
                Step::Done
            }
            STYLE | NOFRAMES => {
                self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                self.original_mode = self.mode;
                self.mode = Mode::Text;
                t.state = Some(TokenizerState::Rawtext);
                Step::Done
            }
            HEAD | NOSCRIPT => {
                self.err_foo_seen_when_foo_open(&t.en.name);
                Step::Done
            }
            _ => {
                self.err(format!(
                    "Bad start tag in “{}” in “noscript” in “head”.",
                    t.en.name
                ));
                self.pop();
                self.mode = Mode::InHead;
                Step::Reprocess
            }
        }
    }

    pub(super) fn start_other(&mut self, m: Mode, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match m {
            Mode::InColumnGroup => match group {
                HTML => {
                    self.err_stray_start_tag(&t.en.name);
                    if !self.is_template_contents() {
                        self.add_attributes_to_html(&t.attrs);
                    }
                    Step::Done
                }
                COL => {
                    self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                    t.self_closing = false;
                    t.void_element = true;
                    Step::Done
                }
                TEMPLATE => {
                    self.start_tag_template_in_head(t.attrs.clone());
                    Step::Done
                }
                _ => {
                    if self.current_ptr() == 0 || self.current_group() == TEMPLATE {
                        self.err("Garbage in “colgroup” fragment.");
                        return Step::Done;
                    }
                    self.pop();
                    self.mode = Mode::InTable;
                    Step::Reprocess
                }
            },
            Mode::AfterBody => {
                self.err_stray_start_tag(&t.en.name);
                if group == HTML {
                    if !self.is_template_contents() {
                        self.add_attributes_to_html(&t.attrs);
                    }
                    return Step::Done;
                }
                self.mode = self.body_mode();
                Step::Reprocess
            }
            Mode::InFrameset | Mode::AfterFrameset => {
                if m == Mode::InFrameset {
                    match group {
                        FRAMESET => {
                            self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                            return Step::Done;
                        }
                        FRAME => {
                            self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                            t.self_closing = false;
                            t.void_element = true;
                            return Step::Done;
                        }
                        _ => {}
                    }
                }
                match group {
                    HTML => {
                        self.err_stray_start_tag(&t.en.name);
                        if !self.is_template_contents() {
                            self.add_attributes_to_html(&t.attrs);
                        }
                    }
                    NOFRAMES => {
                        self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                        self.original_mode = self.mode;
                        self.mode = Mode::Text;
                        t.state = Some(TokenizerState::Rawtext);
                    }
                    _ => self.err_stray_start_tag(&t.en.name),
                }
                Step::Done
            }
            Mode::Initial => {
                self.err(
                    "Start tag seen without seeing a doctype first. Expected “<!DOCTYPE html>”.",
                );
                self.quirks = true;
                self.mode = Mode::BeforeHtml;
                Step::Reprocess
            }
            Mode::BeforeHtml => {
                if group == HTML {
                    self.append_html_element_to_document_and_push(t.attrs.clone());
                    self.mode = Mode::BeforeHead;
                    return Step::Done;
                }
                self.append_html_element_to_document_and_push(no_attrs());
                self.mode = Mode::BeforeHead;
                Step::Reprocess
            }
            Mode::BeforeHead => match group {
                HTML => {
                    self.err_stray_start_tag(&t.en.name);
                    if !self.is_template_contents() {
                        self.add_attributes_to_html(&t.attrs);
                    }
                    Step::Done
                }
                HEAD => {
                    self.append_to_current_node_and_push_head_element(t.attrs.clone());
                    self.mode = Mode::InHead;
                    Step::Done
                }
                _ => {
                    self.append_to_current_node_and_push_head_element(no_attrs());
                    self.mode = Mode::InHead;
                    Step::Reprocess
                }
            },
            Mode::AfterHead => match group {
                HTML => {
                    self.err_stray_start_tag(&t.en.name);
                    if !self.is_template_contents() {
                        self.add_attributes_to_html(&t.attrs);
                    }
                    Step::Done
                }
                BODY => {
                    self.append_to_current_node_and_push_body_element(t.attrs.clone());
                    self.frameset_ok = false;
                    self.mode = Mode::InBody;
                    Step::Done
                }
                FRAMESET => {
                    self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                    self.mode = Mode::InFrameset;
                    Step::Done
                }
                TEMPLATE => {
                    self.err(format!(
                        "“{}” element between “head” and “body”.",
                        t.en.name
                    ));
                    self.push_head_pointer_onto_stack();
                    let head_on_stack = self.current().cloned();
                    self.start_tag_template_in_head(t.attrs.clone());
                    if let Some(head) = head_on_stack {
                        self.remove_node_from_stack(&head);
                    }
                    Step::Done
                }
                BASE | LINK_OR_BASEFONT_OR_BGSOUND | META => {
                    self.err(format!(
                        "“{}” element between “head” and “body”.",
                        t.en.name
                    ));
                    self.push_head_pointer_onto_stack();
                    self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                    t.self_closing = false;
                    t.void_element = true;
                    self.pop();
                    Step::Done
                }
                SCRIPT | STYLE | NOFRAMES | TITLE => {
                    self.err(format!(
                        "“{}” element between “head” and “body”.",
                        t.en.name
                    ));
                    self.push_head_pointer_onto_stack();
                    self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                    self.original_mode = self.mode;
                    self.mode = Mode::Text;
                    t.state = Some(match group {
                        SCRIPT => TokenizerState::ScriptData,
                        TITLE => TokenizerState::Rcdata,
                        _ => TokenizerState::Rawtext,
                    });
                    Step::Done
                }
                HEAD => {
                    self.err_stray_start_tag(&t.en.name);
                    Step::Done
                }
                _ => {
                    self.append_to_current_node_and_push_body_element(no_attrs());
                    self.mode = Mode::FramesetOk;
                    Step::Reprocess
                }
            },
            Mode::AfterAfterBody => {
                self.err_stray_start_tag(&t.en.name);
                if group == HTML {
                    if !self.is_template_contents() {
                        self.add_attributes_to_html(&t.attrs);
                    }
                    return Step::Done;
                }
                self.fatal();
                self.mode = self.body_mode();
                Step::Reprocess
            }
            Mode::AfterAfterFrameset => {
                match group {
                    HTML => {
                        self.err_stray_start_tag(&t.en.name);
                        if !self.is_template_contents() {
                            self.add_attributes_to_html(&t.attrs);
                        }
                    }
                    NOFRAMES => {
                        t.state = Some(self.start_tag_text(
                            &t.en,
                            t.attrs.clone(),
                            TokenizerState::Rawtext,
                            true,
                        ));
                    }
                    _ => self.err_stray_start_tag(&t.en.name),
                }
                Step::Done
            }
            _ => Step::Done,
        }
    }
}
