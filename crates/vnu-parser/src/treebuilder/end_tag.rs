use super::names::groups::*;
use super::*;

impl TreeBuilder {
    pub fn end_tag(&mut self, name: &str, line: usize, col: usize) {
        if self.halted {
            return;
        }
        self.flush_characters();
        self.set_position(line, col);
        self.need_to_drop_lf = false;
        let en = ElemName::lookup(name);
        'token: loop {
            if self.halted {
                return;
            }
            if self.in_foreign() && self.end_in_foreign(&en) {
                return;
            }
            let mut mode = self.mode;
            loop {
                let step = match mode {
                    Mode::InTemplate if en.group() != TEMPLATE => {
                        self.err_stray_end_tag(&en.name);
                        Step::Done
                    }
                    Mode::InTemplate => Step::FallTo(Mode::InHead),
                    Mode::InRow => self.end_in_row(&en),
                    Mode::InTableBody => self.end_in_table_body(&en),
                    Mode::InTable => self.end_in_table(&en),
                    Mode::InCaption => self.end_in_caption(&en),
                    Mode::InCell => self.end_in_cell(&en),
                    Mode::FramesetOk | Mode::InBody => self.end_in_body(&en),
                    Mode::InHead => self.end_in_head(&en),
                    other => self.end_other(other, &en),
                };
                match step {
                    Step::Done => break 'token,
                    Step::Reprocess => continue 'token,
                    Step::FallTo(next) => mode = next,
                }
            }
        }
    }

    /// End tags while the current node is SVG or MathML. True when handled.
    fn end_in_foreign(&mut self, en: &ElemName) -> bool {
        let name: &str = &en.name;
        if let Some(cur) = self.current() {
            if cur.name != name {
                if self.current_ptr() == 0 {
                    self.err_stray_end_tag(name);
                } else {
                    let cur = cur.pop_name.clone();
                    self.err(format!(
                        "End tag “{name}” did not match the name of the current open element (“{cur}”)."
                    ));
                }
            }
        }
        let mut elt_pos = self.current_ptr();
        loop {
            if elt_pos == 0 {
                return true;
            }
            if self.stack[elt_pos].name == name {
                self.pop_until(elt_pos);
                return true;
            }
            elt_pos -= 1;
            if self.stack[elt_pos].ns == Ns::Html {
                return false;
            }
        }
    }

    pub(super) fn end_tag_template_in_head(&mut self) {
        let Some(elt_pos) = self.find_last("template") else {
            self.err_stray_end_tag("template");
            return;
        };
        self.generate_implied_end_tags_thoroughly();
        if !self.is_current("template") {
            self.err_unclosed_elements(elt_pos, "template");
        }
        self.pop_until(elt_pos);
        self.clear_list_to_last_marker();
        self.template_modes.pop();
        self.reset_the_insertion_mode();
    }

    pub(super) fn end_in_row(&mut self, en: &ElemName) -> Step {
        let group = en.group();
        let name: &str = &en.name;
        match group {
            TR => {
                let elt_pos = self.find_last_or_root_group(TR);
                if elt_pos == 0 {
                    self.err("No table row to close.");
                    return Step::Done;
                }
                self.clear_stack_back_to(elt_pos);
                self.pop();
                self.mode = Mode::InTableBody;
                Step::Done
            }
            TABLE => {
                let elt_pos = self.find_last_or_root_group(TR);
                if elt_pos == 0 {
                    self.err("No table row to close.");
                    return Step::Done;
                }
                self.clear_stack_back_to(elt_pos);
                self.pop();
                self.mode = Mode::InTableBody;
                Step::Reprocess
            }
            TBODY_ETC => {
                if self.find_last_in_table_scope(name).is_none() {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                }
                let elt_pos = self.find_last_or_root_group(TR);
                if elt_pos == 0 {
                    self.err("No table row to close.");
                    return Step::Done;
                }
                self.clear_stack_back_to(elt_pos);
                self.pop();
                self.mode = Mode::InTableBody;
                Step::Reprocess
            }
            BODY | CAPTION | COL | COLGROUP | HTML | TD_OR_TH => {
                self.err_stray_end_tag(name);
                Step::Done
            }
            _ => Step::FallTo(Mode::InTableBody),
        }
    }

    pub(super) fn end_in_table_body(&mut self, en: &ElemName) -> Step {
        let group = en.group();
        let name: &str = &en.name;
        match group {
            TBODY_ETC => {
                let elt_pos = self.find_last_or_root_name(name);
                if elt_pos == 0 {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                }
                self.clear_stack_back_to(elt_pos);
                self.pop();
                self.mode = Mode::InTable;
                Step::Done
            }
            TABLE => {
                let elt_pos = self.find_last_in_table_scope_or_root_template_tbody();
                if elt_pos == 0 || self.stack[elt_pos].group() == TEMPLATE {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                }
                self.clear_stack_back_to(elt_pos);
                self.pop();
                self.mode = Mode::InTable;
                Step::Reprocess
            }
            BODY | CAPTION | COL | COLGROUP | HTML | TD_OR_TH | TR => {
                self.err_stray_end_tag(name);
                Step::Done
            }
            _ => Step::FallTo(Mode::InTable),
        }
    }

    pub(super) fn end_in_table(&mut self, en: &ElemName) -> Step {
        let group = en.group();
        let name: &str = &en.name;
        match group {
            TABLE => {
                let Some(elt_pos) = self.find_last("table") else {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                };
                self.pop_until(elt_pos);
                self.reset_the_insertion_mode();
                Step::Done
            }
            BODY | CAPTION | COL | COLGROUP | HTML | TBODY_ETC | TD_OR_TH | TR => {
                self.err_stray_end_tag(name);
                Step::Done
            }
            TEMPLATE => Step::FallTo(Mode::InHead),
            _ => {
                self.err_stray_end_tag(name);
                Step::FallTo(Mode::InCaption)
            }
        }
    }

    pub(super) fn end_in_caption(&mut self, en: &ElemName) -> Step {
        let group = en.group();
        let name: &str = &en.name;
        match group {
            CAPTION => {
                let Some(elt_pos) = self.find_last_in_table_scope("caption") else {
                    return Step::Done;
                };
                self.generate_implied_end_tags();
                if self.current_ptr() != elt_pos {
                    self.err_unclosed_elements(elt_pos, name);
                }
                self.pop_until(elt_pos);
                self.clear_list_to_last_marker();
                self.mode = Mode::InTable;
                Step::Done
            }
            TABLE => {
                let Some(elt_pos) = self.find_last_in_table_scope("caption") else {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                };
                self.generate_implied_end_tags();
                if self.current_ptr() != elt_pos {
                    self.err_unclosed_elements(elt_pos, name);
                }
                self.pop_until(elt_pos);
                self.clear_list_to_last_marker();
                self.mode = Mode::InTable;
                Step::Reprocess
            }
            BODY | COL | COLGROUP | HTML | TBODY_ETC | TD_OR_TH | TR => {
                self.err_stray_end_tag(name);
                Step::Done
            }
            _ => Step::FallTo(Mode::InCell),
        }
    }

    pub(super) fn end_in_cell(&mut self, en: &ElemName) -> Step {
        let group = en.group();
        let name: &str = &en.name;
        match group {
            TD_OR_TH => {
                let Some(elt_pos) = self.find_last_in_table_scope(name) else {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                };
                self.generate_implied_end_tags();
                if !self.is_current(name) {
                    self.err_unclosed_elements(elt_pos, name);
                }
                self.pop_until(elt_pos);
                self.clear_list_to_last_marker();
                self.mode = Mode::InRow;
                Step::Done
            }
            TABLE | TBODY_ETC | TR => {
                if self.find_last_in_table_scope(name).is_none() {
                    self.err_stray_end_tag(name);
                    return Step::Done;
                }
                if let Some(p) = self.find_last_in_table_scope_td_th() {
                    self.close_the_cell(p);
                }
                Step::Reprocess
            }
            BODY | CAPTION | COL | COLGROUP | HTML => {
                self.err_stray_end_tag(name);
                Step::Done
            }
            _ => Step::FallTo(Mode::InBody),
        }
    }

    pub(super) fn end_in_head(&mut self, en: &ElemName) -> Step {
        let group = en.group();
        let name: &str = &en.name;
        match group {
            HEAD => {
                self.pop();
                self.mode = Mode::AfterHead;
                Step::Done
            }
            BR | HTML | BODY => {
                self.pop();
                self.mode = Mode::AfterHead;
                Step::Reprocess
            }
            TEMPLATE => {
                self.end_tag_template_in_head();
                Step::Done
            }
            _ => {
                self.err_stray_end_tag(name);
                Step::Done
            }
        }
    }

    pub(super) fn end_other(&mut self, m: Mode, en: &ElemName) -> Step {
        let group = en.group();
        let name: &str = &en.name;
        match m {
            Mode::InHeadNoscript => match group {
                NOSCRIPT => {
                    self.pop();
                    self.mode = Mode::InHead;
                    Step::Done
                }
                BR => {
                    self.err_stray_end_tag(name);
                    self.pop();
                    self.mode = Mode::InHead;
                    Step::Reprocess
                }
                _ => {
                    self.err_stray_end_tag(name);
                    Step::Done
                }
            },
            Mode::InColumnGroup => match group {
                COLGROUP => {
                    if self.current_ptr() == 0 || self.current_group() == TEMPLATE {
                        self.err("Garbage in “colgroup” fragment.");
                        return Step::Done;
                    }
                    self.pop();
                    self.mode = Mode::InTable;
                    Step::Done
                }
                COL => {
                    self.err_stray_end_tag(name);
                    Step::Done
                }
                TEMPLATE => {
                    self.end_tag_template_in_head();
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
                if group == HTML {
                    self.mode = Mode::AfterAfterBody;
                    return Step::Done;
                }
                self.err("Saw an end tag after “body” had been closed.");
                self.mode = self.body_mode();
                Step::Reprocess
            }
            Mode::InFrameset => {
                if group == FRAMESET {
                    if self.current_ptr() == 0 {
                        self.err_stray_end_tag(name);
                        return Step::Done;
                    }
                    self.pop();
                    if !self.is_current("frameset") {
                        self.mode = Mode::AfterFrameset;
                    }
                    return Step::Done;
                }
                self.err_stray_end_tag(name);
                Step::Done
            }
            Mode::AfterFrameset => {
                if group == HTML {
                    self.mode = Mode::AfterAfterFrameset;
                } else {
                    self.err_stray_end_tag(name);
                }
                Step::Done
            }
            Mode::Initial => {
                self.err(
                    "End tag seen without seeing a doctype first. Expected “<!DOCTYPE html>”.",
                );
                self.quirks = true;
                self.mode = Mode::BeforeHtml;
                Step::Reprocess
            }
            Mode::BeforeHtml => match group {
                HEAD | BR | HTML | BODY => {
                    self.append_html_element_to_document_and_push(no_attrs());
                    self.mode = Mode::BeforeHead;
                    Step::Reprocess
                }
                _ => {
                    self.err_stray_end_tag(name);
                    Step::Done
                }
            },
            Mode::BeforeHead => match group {
                HEAD | BR | HTML | BODY => {
                    self.append_to_current_node_and_push_head_element(no_attrs());
                    self.mode = Mode::InHead;
                    Step::Reprocess
                }
                _ => {
                    self.err_stray_end_tag(name);
                    Step::Done
                }
            },
            Mode::AfterHead => match group {
                TEMPLATE => {
                    self.end_tag_template_in_head();
                    Step::Done
                }
                HTML | BODY | BR => {
                    self.append_to_current_node_and_push_body_element(no_attrs());
                    self.mode = Mode::FramesetOk;
                    Step::Reprocess
                }
                _ => {
                    self.err_stray_end_tag(name);
                    Step::Done
                }
            },
            Mode::AfterAfterBody => {
                self.err_stray_end_tag(name);
                self.mode = self.body_mode();
                Step::Reprocess
            }
            Mode::AfterAfterFrameset => {
                self.err_stray_end_tag(name);
                Step::Done
            }
            Mode::Text => {
                self.pop();
                if self.original_mode == Mode::AfterHead {
                    self.silent_pop();
                }
                self.mode = self.original_mode;
                Step::Done
            }
            _ => Step::Done,
        }
    }
}
