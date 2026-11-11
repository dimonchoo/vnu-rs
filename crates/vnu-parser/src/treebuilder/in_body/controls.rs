use crate::treebuilder::names::groups::*;
use crate::treebuilder::start_tag::StartTag;
use crate::treebuilder::*;

impl TreeBuilder {
    /// "in body" start tags of form controls: form, button, input, select, option, optgroup.
    pub(in crate::treebuilder) fn start_form_control_in_body(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            FORM => {
                if self.form_pointer && !self.is_template_contents() {
                    self.err_form_when_form_open();
                    return Step::Done;
                }
                self.implicitly_close_p();
                self.append_to_current_node_and_push_form_element_may_foster(t.attrs.clone());
                Step::Done
            }
            BUTTON => {
                if let Some(elt_pos) = self.find_last_in_scope(&t.en.name) {
                    self.err_foo_seen_when_foo_open(&t.en.name);
                    self.generate_implied_end_tags();
                    if !self.is_current(&t.en.name) {
                        self.err_unclosed_elements_implied(elt_pos, &t.en.name);
                    }
                    self.pop_until(elt_pos);
                    return Step::Reprocess;
                }
                self.reconstruct_the_active_formatting_elements();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
            INPUT => {
                if let Some(elt_pos) = self.find_last_in_scope("select") {
                    self.err(format!("“{}” start tag with “select” open.", t.en.name));
                    self.pop_until(elt_pos);
                    return Step::Reprocess;
                }
                self.reconstruct_the_active_formatting_elements();
                self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                t.self_closing = false;
                t.void_element = true;
                Step::Done
            }
            SELECT => {
                if let Some(elt_pos) = self.find_last_in_scope(&t.en.name) {
                    self.err("“select” start tag where end tag expected.");
                    self.pop_until(elt_pos);
                    return Step::Done;
                }
                self.reconstruct_the_active_formatting_elements();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                self.frameset_ok = false;
                Step::Done
            }
            OPTION => {
                if self.find_last_in_scope("select").is_some() {
                    self.generate_implied_end_tags_except_for("optgroup");
                    if let Some(p) = self.find_last_in_scope("option") {
                        self.err_unclosed_elements_implied(p, &t.en.name);
                    }
                } else if self.is_current("option") {
                    self.pop();
                }
                self.reconstruct_the_active_formatting_elements();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
            OPTGROUP => {
                if self.find_last_in_scope("select").is_some() {
                    self.generate_implied_end_tags();
                    if let Some(p) = self.find_last_in_scope("option") {
                        self.err_unclosed_elements_implied(p, &t.en.name);
                    } else if let Some(p) = self.find_last_in_scope("optgroup") {
                        self.err_unclosed_elements_implied(p, &t.en.name);
                    }
                } else if self.is_current("option") {
                    self.pop();
                }
                self.reconstruct_the_active_formatting_elements();
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
            _ => Step::Done,
        }
    }

    /// "in body" start tags of ruby annotations: rb, rtc, rt, rp.
    pub(in crate::treebuilder) fn start_ruby_in_body(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            RB_OR_RTC | RT_OR_RP => {
                let elt_pos = self.find_last_in_scope("ruby");
                if elt_pos.is_some() {
                    if group == RB_OR_RTC {
                        self.generate_implied_end_tags();
                    } else {
                        self.generate_implied_end_tags_except_for("rtc");
                    }
                }
                if elt_pos != Some(self.current_ptr())
                    && (group == RB_OR_RTC || !self.is_current("rtc"))
                {
                    match elt_pos {
                        None => self.err(format!(
                            "Start tag “{}” seen without a “ruby” element being open.",
                            t.en.name
                        )),
                        Some(_) => self.err("Unclosed children in “ruby”."),
                    }
                }
                self.append_to_current_node_and_push_element_may_foster(&t.en, t.attrs.clone());
                Step::Done
            }
            _ => Step::Done,
        }
    }
}
