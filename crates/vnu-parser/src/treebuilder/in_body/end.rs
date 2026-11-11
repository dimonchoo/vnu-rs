use crate::treebuilder::names::groups::*;
use crate::treebuilder::*;

impl TreeBuilder {
    pub(in crate::treebuilder) fn end_in_body(&mut self, en: &ElemName) -> Step {
        let group = en.group();
        let name: &str = &en.name;
        match group {
            BODY => {
                if !self.is_second_on_stack_body() {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                }
                for i in 2..self.stack.len() {
                    match self.stack[i].group() {
                        DD_OR_DT | LI | OPTGROUP | OPTION | P | RB_OR_RTC | RT_OR_RP | TD_OR_TH
                        | TBODY_ETC => {}
                        _ => {
                            self.err_end_with_unclosed_elements(name);
                            break;
                        }
                    }
                }
                self.mode = Mode::AfterBody;
                Step::Done
            }
            HTML => {
                if !self.is_second_on_stack_body() {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                }
                for i in 0..self.stack.len() {
                    match self.stack[i].group() {
                        DD_OR_DT | LI | P | RB_OR_RTC | RT_OR_RP | TBODY_ETC | TD_OR_TH | BODY
                        | HTML => {}
                        _ => {
                            self.err_end_with_unclosed_elements(name);
                            break;
                        }
                    }
                }
                self.mode = Mode::AfterBody;
                Step::Reprocess
            }
            DIV_ETC | UL_OR_OL_OR_DL | PRE_OR_LISTING | FIELDSET | BUTTON | SELECT
            | ADDRESS_ETC => {
                match self.find_last_in_scope(name) {
                    None => self.err_stray_end_tag(name),
                    Some(elt_pos) => {
                        self.generate_implied_end_tags();
                        if !self.is_current(name) {
                            self.err_unclosed_elements(elt_pos, name);
                        }
                        self.pop_until(elt_pos);
                    }
                }
                Step::Done
            }
            FORM => {
                if !self.is_template_contents() {
                    if !self.form_pointer {
                        self.err_stray_end_tag(name);
                        return Step::Done;
                    }
                    self.form_pointer = false;
                    let Some(elt_pos) = self.find_last_in_scope(name) else {
                        self.err_stray_end_tag(name);
                        return Step::Done;
                    };
                    self.generate_implied_end_tags();
                    if !self.is_current(name) {
                        self.err_unclosed_elements(elt_pos, name);
                    }
                    self.remove_from_stack(elt_pos);
                    return Step::Done;
                }
                let Some(elt_pos) = self.find_last_in_scope(name) else {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                };
                self.generate_implied_end_tags();
                if !self.is_current(name) {
                    self.err_unclosed_elements(elt_pos, name);
                }
                self.pop_until(elt_pos);
                Step::Done
            }
            P => {
                let Some(elt_pos) = self.find_last_in_button_scope("p") else {
                    self.err_no_element_to_close("p");
                    if self.in_foreign() {
                        self.err_html_start_tag_in_foreign_context(name);
                        while self.current_ptr() > 0
                            && self.current().is_some_and(|n| n.ns != Ns::Html)
                        {
                            self.pop();
                        }
                    }
                    self.append_void_element_to_current_may_foster(en, no_attrs());
                    return Step::Done;
                };
                self.generate_implied_end_tags_except_for("p");
                if elt_pos != self.current_ptr() {
                    self.err_unclosed_elements(elt_pos, name);
                }
                self.pop_until(elt_pos);
                Step::Done
            }
            LI | DD_OR_DT => {
                let found = if group == LI {
                    self.find_last_in_list_scope(name)
                } else {
                    self.find_last_in_scope(name)
                };
                match found {
                    None => self.err_no_element_to_close(name),
                    Some(elt_pos) => {
                        self.generate_implied_end_tags_except_for(name);
                        if elt_pos != self.current_ptr() {
                            self.err_unclosed_elements(elt_pos, name);
                        }
                        self.pop_until(elt_pos);
                    }
                }
                Step::Done
            }
            H1_H6 => {
                match self.find_last_in_scope_hn() {
                    None => self.err_stray_end_tag(name),
                    Some(elt_pos) => {
                        self.generate_implied_end_tags();
                        if !self.is_current(name) {
                            self.err_unclosed_elements(elt_pos, name);
                        }
                        self.pop_until(elt_pos);
                    }
                }
                Step::Done
            }
            OBJECT | MARQUEE_OR_APPLET => {
                match self.find_last_in_scope(name) {
                    None => self.err_stray_end_tag(name),
                    Some(elt_pos) => {
                        self.generate_implied_end_tags();
                        if !self.is_current(name) {
                            self.err_unclosed_elements(elt_pos, name);
                        }
                        self.pop_until(elt_pos);
                        self.clear_list_to_last_marker();
                    }
                }
                Step::Done
            }
            BR => {
                self.err("End tag “br”.");
                if self.in_foreign() {
                    self.err_html_start_tag_in_foreign_context(name);
                    while self.current_ptr() > 0 && self.current().is_some_and(|n| n.ns != Ns::Html)
                    {
                        self.pop();
                    }
                }
                self.reconstruct_the_active_formatting_elements();
                self.append_void_element_to_current_may_foster(en, no_attrs());
                Step::Done
            }
            TEMPLATE => Step::FallTo(Mode::InHead),
            AREA_OR_WBR
            | KEYGEN
            | PARAM_OR_SOURCE_OR_TRACK
            | EMBED
            | IMG
            | IMAGE
            | INPUT
            | HR
            | IFRAME
            | NOEMBED
            | NOFRAMES
            | TABLE
            | TEXTAREA => {
                self.err_stray_end_tag(name);
                Step::Done
            }
            _ => {
                if matches!(group, NOSCRIPT | A | B_ETC | FONT | NOBR)
                    && self.adoption_agency_end_tag(name)
                {
                    return Step::Done;
                }
                if self.is_current(name) {
                    self.pop();
                    return Step::Done;
                }
                let mut elt_pos = self.current_ptr();
                loop {
                    let node = self.stack[elt_pos].clone();
                    if node.is_html(name) {
                        self.generate_implied_end_tags_except_for(name);
                        if !self.is_current(name) {
                            self.err_unclosed_elements(elt_pos, name);
                        }
                        self.pop_until(elt_pos);
                        return Step::Done;
                    } else if elt_pos == 0 || node.is_special() {
                        self.err_stray_end_tag(name);
                        return Step::Done;
                    }
                    elt_pos -= 1;
                }
            }
        }
    }
}
