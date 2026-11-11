mod controls;
mod end;
mod formatting;

use super::names::groups::*;
use super::start_tag::StartTag;
use super::*;

/// The "in body" insertion mode (and "frameset-ok", which falls into it).
impl TreeBuilder {
    pub(in crate::treebuilder) fn start_frameset_ok(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            FRAMESET => {
                if self.mode == Mode::FramesetOk {
                    if self.current_ptr() == 0 || self.stack[1].group() != BODY {
                        self.err_stray_start_tag(&t.en.name);
                        return Step::Done;
                    }
                    self.err("“frameset” start tag seen.");
                    // detachFromParent(body)
                    self.fatal();
                    return Step::Done;
                }
                self.err_stray_start_tag(&t.en.name);
                return Step::Done;
            }
            PRE_OR_LISTING | LI | DD_OR_DT | BUTTON | MARQUEE_OR_APPLET | OBJECT | TABLE
            | AREA_OR_WBR | KEYGEN | BR | EMBED | IMG | INPUT | HR | TEXTAREA | XMP | IFRAME
            | SELECT => {
                if self.mode == Mode::FramesetOk
                    && !(group == INPUT && equals_ci("hidden", attr(&t.attrs, "type")))
                {
                    self.frameset_ok = false;
                    self.mode = Mode::InBody;
                }
            }
            _ => {}
        }
        Step::FallTo(Mode::InBody)
    }

    pub(in crate::treebuilder) fn start_in_body(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            HTML => {
                self.err_stray_start_tag(&t.en.name);
                if !self.is_template_contents() {
                    self.add_attributes_to_html(&t.attrs);
                }
                Step::Done
            }
            BASE | LINK_OR_BASEFONT_OR_BGSOUND | META | STYLE | SCRIPT | TITLE | TEMPLATE => {
                Step::FallTo(Mode::InHead)
            }
            BODY => {
                if self.current_ptr() == 0
                    || self.stack[1].group() != BODY
                    || self.is_template_contents()
                {
                    self.err_stray_start_tag(&t.en.name);
                    return Step::Done;
                }
                self.err_foo_seen_when_foo_open(&t.en.name);
                self.frameset_ok = false;
                if self.mode == Mode::FramesetOk {
                    self.mode = Mode::InBody;
                }
                self.add_attributes_to_body(&t.attrs);
                Step::Done
            }
            P | DIV_ETC | UL_OR_OL_OR_DL | ADDRESS_ETC => {
                self.implicitly_close_p();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
            H1_H6 => {
                self.implicitly_close_p();
                if self.current_group() == H1_H6 {
                    self.err("Heading cannot be a child of another heading.");
                    self.pop();
                }
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
            FIELDSET => {
                self.implicitly_close_p();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
            PRE_OR_LISTING => {
                self.implicitly_close_p();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                self.need_to_drop_lf = true;
                Step::Done
            }
            LI | DD_OR_DT => {
                let mut elt_pos = self.current_ptr();
                loop {
                    let node = self.stack[elt_pos].clone();
                    if node.group() == group {
                        self.generate_implied_end_tags_except_for(&node.name);
                        if elt_pos != self.current_ptr() {
                            self.err_unclosed_elements_implied(elt_pos, &t.en.name);
                        }
                        self.pop_until(elt_pos);
                        break;
                    } else if elt_pos == 0
                        || (node.is_special()
                            && (node.ns != Ns::Html
                                || (node.name != "p"
                                    && node.name != "address"
                                    && node.name != "div")))
                    {
                        break;
                    }
                    elt_pos -= 1;
                }
                self.implicitly_close_p();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
            PLAINTEXT => {
                self.implicitly_close_p();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                t.state = Some(TokenizerState::Plaintext);
                Step::Done
            }
            OBJECT | MARQUEE_OR_APPLET => {
                self.reconstruct_the_active_formatting_elements();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                self.insert_marker();
                Step::Done
            }
            TABLE => {
                if !self.quirks {
                    self.implicitly_close_p();
                }
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                self.mode = Mode::InTable;
                Step::Done
            }
            BR | EMBED | AREA_OR_WBR | KEYGEN | PARAM_OR_SOURCE_OR_TRACK => {
                if group != PARAM_OR_SOURCE_OR_TRACK {
                    self.reconstruct_the_active_formatting_elements();
                }
                self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                t.self_closing = false;
                t.void_element = true;
                Step::Done
            }
            HR => {
                self.implicitly_close_p();
                if self.find_last_in_scope("select").is_some() {
                    self.generate_implied_end_tags();
                    let option = self.find_last_in_scope("option");
                    let optgroup = self.find_last_in_scope("optgroup");
                    if let Some(p) = option.or(optgroup) {
                        self.err_unclosed_elements_implied(p, &t.en.name);
                    }
                }
                self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                t.self_closing = false;
                t.void_element = true;
                Step::Done
            }
            IMAGE => {
                self.err("Saw a start tag “image”.");
                t.en = ElemName::lookup("img");
                Step::Reprocess
            }
            IMG => {
                self.reconstruct_the_active_formatting_elements();
                self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                t.self_closing = false;
                t.void_element = true;
                Step::Done
            }
            TEXTAREA => {
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                t.state = Some(TokenizerState::Rcdata);
                self.original_mode = self.mode;
                self.mode = Mode::Text;
                self.need_to_drop_lf = true;
                Step::Done
            }
            XMP => {
                self.implicitly_close_p();
                self.reconstruct_the_active_formatting_elements();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                self.original_mode = self.mode;
                self.mode = Mode::Text;
                t.state = Some(TokenizerState::Rawtext);
                Step::Done
            }
            NOSCRIPT => {
                // Scripting is disabled in the validator.
                self.reconstruct_the_active_formatting_elements();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
            NOFRAMES | IFRAME | NOEMBED => {
                t.state = Some(self.start_tag_text(
                    &t.en,
                    t.attrs.clone(),
                    TokenizerState::Rawtext,
                    true,
                ));
                Step::Done
            }
            MATH | SVG => {
                self.reconstruct_the_active_formatting_elements();
                let ns = if group == MATH { Ns::MathMl } else { Ns::Svg };
                if t.self_closing {
                    self.append_void_foreign(&t.en, t.attrs.clone(), ns);
                    t.self_closing = false;
                } else {
                    self.append_to_current_node_and_push_foreign(&t.en, t.attrs.clone(), ns);
                }
                Step::Done
            }
            CAPTION | TBODY_ETC | TR | TD_OR_TH | COL | COLGROUP | FRAME | FRAMESET | HEAD => {
                self.err_stray_start_tag(&t.en.name);
                Step::Done
            }
            A | B_ETC | FONT | NOBR => self.start_formatting_in_body(t),
            FORM | BUTTON | INPUT | SELECT | OPTION | OPTGROUP => {
                self.start_form_control_in_body(t)
            }
            RB_OR_RTC | RT_OR_RP => self.start_ruby_in_body(t),
            _ => {
                self.reconstruct_the_active_formatting_elements();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
        }
    }
}
