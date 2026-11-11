use crate::treebuilder::names::groups::*;
use crate::treebuilder::start_tag::StartTag;
use crate::treebuilder::*;

impl TreeBuilder {
    /// "in body" start tags of formatting elements: a, b, big, code, em, font, i, nobr, s, small, strike, strong, tt, u.
    pub(in crate::treebuilder) fn start_formatting_in_body(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            A => {
                let active_a = self
                    .find_in_list_after_last_marker("a")
                    .and_then(|pos| self.list[pos].clone());
                if let Some(active_a) = active_a {
                    self.err_foo_seen_when_foo_open(&t.en.name);
                    self.adoption_agency_end_tag("a");
                    self.remove_node_from_stack(&active_a);
                    if let Some(p) = self.find_in_list(&active_a) {
                        self.list.remove(p);
                    }
                }
                self.reconstruct_the_active_formatting_elements();
                self.append_to_current_node_and_push_formatting_element_may_foster(
                    &t.en,
                    t.attrs.clone(),
                );
                Step::Done
            }
            B_ETC | FONT => {
                self.reconstruct_the_active_formatting_elements();
                self.maybe_forget_earlier_duplicate_formatting_element(&t.en.name, &t.attrs);
                self.append_to_current_node_and_push_formatting_element_may_foster(
                    &t.en,
                    t.attrs.clone(),
                );
                Step::Done
            }
            NOBR => {
                self.reconstruct_the_active_formatting_elements();
                if self.find_last_in_scope("nobr").is_some() {
                    self.err_foo_seen_when_foo_open(&t.en.name);
                    self.adoption_agency_end_tag("nobr");
                    self.reconstruct_the_active_formatting_elements();
                }
                self.append_to_current_node_and_push_formatting_element_may_foster(
                    &t.en,
                    t.attrs.clone(),
                );
                Step::Done
            }
            _ => Step::Done,
        }
    }
}
