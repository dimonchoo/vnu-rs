use super::names::groups::*;
use super::*;

impl TreeBuilder {
    pub fn doctype(
        &mut self,
        name: Option<&str>,
        public_id: Option<&str>,
        system_id: Option<&str>,
        force_quirks: bool,
        line: usize,
        col: usize,
    ) {
        if self.halted {
            return;
        }
        self.set_position(line, col);
        self.need_to_drop_lf = false;
        if !self.in_foreign() && self.mode == Mode::Initial {
            if is_quirky(name, public_id, system_id, force_quirks) {
                self.err("Quirky doctype. Expected “<!DOCTYPE html>”.");
                self.quirks = true;
            } else if is_almost_standards(public_id, system_id) {
                self.err("Almost standards mode doctype. Expected “<!DOCTYPE html>”.");
                self.quirks = false;
            } else {
                let obsolete = (public_id == Some("-//W3C//DTD HTML 4.0//EN")
                    && (system_id.is_none()
                        || system_id == Some("http://www.w3.org/TR/REC-html40/strict.dtd")))
                    || (public_id == Some("-//W3C//DTD HTML 4.01//EN")
                        && (system_id.is_none()
                            || system_id == Some("http://www.w3.org/TR/html4/strict.dtd")))
                    || (public_id == Some("-//W3C//DTD XHTML 1.0 Strict//EN")
                        && system_id == Some("http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"))
                    || (public_id == Some("-//W3C//DTD XHTML 1.1//EN")
                        && system_id == Some("http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd"));
                if obsolete {
                    self.err("Obsolete doctype. Expected “<!DOCTYPE html>”.");
                } else if !((system_id.is_none() || system_id == Some("about:legacy-compat"))
                    && public_id.is_none())
                {
                    self.err("Legacy doctype. Expected “<!DOCTYPE html>”.");
                }
                self.quirks = false;
            }
            self.mode = Mode::BeforeHtml;
            return;
        }
        self.err("Stray doctype.");
    }

    pub fn comment(&mut self) {
        if self.halted {
            return;
        }
        self.need_to_drop_lf = false;
        self.flush_characters();
    }

    /// A character token run. `line`/`col` locate its first character.
    pub fn characters(&mut self, text: &str, line: usize, col: usize) {
        if self.halted || text.is_empty() {
            return;
        }
        self.set_position(line, col);
        let mut text = text;
        if self.need_to_drop_lf {
            self.need_to_drop_lf = false;
            if let Some(rest) = text.strip_prefix('\n') {
                text = rest;
                if text.is_empty() {
                    return;
                }
            }
        }
        match self.mode {
            Mode::InBody | Mode::InCell | Mode::InCaption => {
                if !self.is_in_foreign_but_not_integration_point() {
                    self.reconstruct_the_active_formatting_elements();
                }
                return;
            }
            Mode::Text => return,
            Mode::InTable | Mode::InTableBody | Mode::InRow => {
                self.accumulate_table_chars(text);
                return;
            }
            _ => {}
        }
        // `rest` starts at the character being (re)processed, like Java's index `i`.
        let mut rest = text;
        while let Some(c) = rest.chars().next() {
            if self.halted {
                return;
            }
            let next = &rest[c.len_utf8()..];
            if is_space(c) {
                match self.mode {
                    Mode::Initial | Mode::BeforeHtml | Mode::BeforeHead => {
                        rest = next;
                        continue;
                    }
                    Mode::InHead
                    | Mode::InHeadNoscript
                    | Mode::AfterHead
                    | Mode::InColumnGroup
                    | Mode::InFrameset
                    | Mode::AfterFrameset => {
                        rest = next;
                        continue;
                    }
                    Mode::FramesetOk
                    | Mode::InTemplate
                    | Mode::InBody
                    | Mode::InCell
                    | Mode::InCaption => {
                        if !self.is_in_foreign_but_not_integration_point() {
                            self.reconstruct_the_active_formatting_elements();
                        }
                        return;
                    }
                    Mode::InTable | Mode::InTableBody | Mode::InRow => {
                        self.accumulate_table_chars(&rest[..c.len_utf8()]);
                        rest = next;
                        continue;
                    }
                    Mode::AfterBody | Mode::AfterAfterBody | Mode::AfterAfterFrameset => {
                        self.reconstruct_the_active_formatting_elements();
                        rest = next;
                        continue;
                    }
                    Mode::Text => return,
                }
            }
            match self.mode {
                Mode::Initial => {
                    self.err("Non-space characters found without seeing a doctype first. Expected “<!DOCTYPE html>”.");
                    self.quirks = true;
                    self.mode = Mode::BeforeHtml;
                }
                Mode::BeforeHtml => {
                    self.append_html_element_to_document_and_push(no_attrs());
                    self.mode = Mode::BeforeHead;
                }
                Mode::BeforeHead => {
                    self.append_to_current_node_and_push_head_element(no_attrs());
                    self.mode = Mode::InHead;
                }
                Mode::InHead => {
                    self.pop();
                    self.mode = Mode::AfterHead;
                }
                Mode::InHeadNoscript => {
                    self.err("Non-space character inside “noscript” inside “head”.");
                    self.pop();
                    self.mode = Mode::InHead;
                }
                Mode::AfterHead => {
                    self.append_to_current_node_and_push_body_element(no_attrs());
                    self.mode = Mode::FramesetOk;
                }
                Mode::FramesetOk => {
                    self.frameset_ok = false;
                    self.mode = Mode::InBody;
                }
                Mode::InTemplate | Mode::InBody | Mode::InCell | Mode::InCaption => {
                    if !self.is_in_foreign_but_not_integration_point() {
                        self.reconstruct_the_active_formatting_elements();
                    }
                    return;
                }
                Mode::InTable | Mode::InTableBody | Mode::InRow => {
                    self.accumulate_table_chars(rest);
                    return;
                }
                Mode::InColumnGroup => {
                    if self.current_ptr() == 0 || self.current_group() == TEMPLATE {
                        self.err("Non-space in “colgroup” when parsing fragment.");
                        rest = next;
                        continue;
                    }
                    self.pop();
                    self.mode = Mode::InTable;
                }
                Mode::AfterBody => {
                    self.err("Non-space character after body.");
                    self.fatal();
                    self.mode = self.body_mode();
                }
                Mode::InFrameset => {
                    self.err("Non-space in “frameset”.");
                    rest = next;
                }
                Mode::AfterFrameset => {
                    self.err("Non-space after “frameset”.");
                    rest = next;
                }
                Mode::AfterAfterBody => {
                    self.err("Non-space character in page trailer.");
                    self.mode = self.body_mode();
                }
                Mode::AfterAfterFrameset => {
                    self.err("Non-space character in page trailer.");
                    rest = next;
                }
                Mode::Text => return,
            }
        }
    }

    /// Java buffers table text and inspects it on flush; only its start and whether it
    /// holds a non-space character matter here.
    pub(super) fn accumulate_table_chars(&mut self, s: &str) {
        if !self.table_chars {
            self.table_chars = true;
            self.table_chars_loc = (self.line, self.col);
        }
        self.table_non_space |= s.chars().any(|c| !is_space(c));
    }

    pub(super) fn flush_characters(&mut self) {
        if !self.table_chars {
            return;
        }
        let non_space = self.table_non_space;
        self.table_chars = false;
        self.table_non_space = false;
        if matches!(self.mode, Mode::InTable | Mode::InTableBody | Mode::InRow) && non_space {
            let (line, col) = (self.line, self.col);
            self.set_position(self.table_chars_loc.0, self.table_chars_loc.1);
            self.err("Misplaced non-space characters inside a table.");
            self.reconstruct_the_active_formatting_elements();
            if self.current_fosters() {
                let table_pos = self.find_last_or_root_group(TABLE);
                let template_pos = self.find_last_or_root_group(TEMPLATE);
                if template_pos < table_pos {
                    self.fatal();
                }
            }
            self.set_position(line, col);
        }
    }

    pub fn eof(&mut self, line: usize, col: usize) {
        if self.halted {
            return;
        }
        self.flush_characters();
        self.set_position(line, col);
        loop {
            if self.halted {
                return;
            }
            match self.mode {
                Mode::Initial => {
                    self.err("End of file seen without seeing a doctype first. Expected “<!DOCTYPE html>”.");
                    self.quirks = true;
                    self.mode = Mode::BeforeHtml;
                }
                Mode::BeforeHtml => {
                    self.append_html_element_to_document_and_push(no_attrs());
                    self.mode = Mode::BeforeHead;
                }
                Mode::BeforeHead => {
                    self.append_to_current_node_and_push_head_element(no_attrs());
                    self.mode = Mode::InHead;
                }
                Mode::InHead => {
                    if self.current_ptr() > 1 {
                        self.err_eof_with_unclosed_elements();
                    }
                    while self.current_ptr() > 0 {
                        self.pop();
                    }
                    self.mode = Mode::AfterHead;
                }
                Mode::InHeadNoscript => {
                    self.err_eof_with_unclosed_elements();
                    while self.current_ptr() > 1 {
                        self.pop();
                    }
                    self.mode = Mode::InHead;
                }
                Mode::AfterHead => {
                    self.append_to_current_node_and_push_body_element(no_attrs());
                    self.mode = Mode::InBody;
                }
                Mode::InTableBody
                | Mode::InRow
                | Mode::InTable
                | Mode::InColumnGroup
                | Mode::FramesetOk
                | Mode::InCaption
                | Mode::InCell
                | Mode::InBody
                | Mode::InTemplate => {
                    if self.mode != Mode::InTemplate {
                        for i in (1..self.stack.len()).rev() {
                            match self.stack[i].group() {
                                DD_OR_DT | LI | P | TBODY_ETC | TD_OR_TH | BODY | HTML => {}
                                _ => {
                                    self.err_eof_with_unclosed_elements();
                                    break;
                                }
                            }
                        }
                        if self.template_modes.is_empty() {
                            break;
                        }
                    }
                    let Some(elt_pos) = self.find_last("template") else {
                        break;
                    };
                    self.err_list_all_unclosed();
                    self.pop_until(elt_pos);
                    self.clear_list_to_last_marker();
                    self.template_modes.pop();
                    self.reset_the_insertion_mode();
                }
                Mode::Text => {
                    self.err("End of file seen when expecting text or an end tag.");
                    self.err_list_all_unclosed();
                    if self.original_mode == Mode::AfterHead {
                        self.pop();
                    }
                    self.pop();
                    self.mode = self.original_mode;
                }
                Mode::InFrameset => {
                    if self.current_ptr() > 0 {
                        self.err_eof_with_unclosed_elements();
                    }
                    break;
                }
                Mode::AfterBody
                | Mode::AfterFrameset
                | Mode::AfterAfterBody
                | Mode::AfterAfterFrameset => break,
            }
        }
        while !self.stack.is_empty() {
            self.pop();
        }
    }
}

fn is_quirky(
    name: Option<&str>,
    public_id: Option<&str>,
    system_id: Option<&str>,
    force_quirks: bool,
) -> bool {
    if force_quirks || name != Some("html") {
        return true;
    }
    if public_id.is_some() {
        if QUIRKY_PUBLIC_IDS
            .iter()
            .any(|prefix| starts_with_ci(prefix, public_id))
        {
            return true;
        }
        if equals_ci("-//w3o//dtd w3 html strict 3.0//en//", public_id)
            || equals_ci("-/w3c/dtd html 4.0 transitional/en", public_id)
            || equals_ci("html", public_id)
        {
            return true;
        }
    }
    if system_id.is_none() {
        starts_with_ci("-//w3c//dtd html 4.01 transitional//", public_id)
            || starts_with_ci("-//w3c//dtd html 4.01 frameset//", public_id)
    } else {
        equals_ci(
            "http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd",
            system_id,
        )
    }
}

fn is_almost_standards(public_id: Option<&str>, system_id: Option<&str>) -> bool {
    starts_with_ci("-//w3c//dtd xhtml 1.0 transitional//", public_id)
        || starts_with_ci("-//w3c//dtd xhtml 1.0 frameset//", public_id)
        || (system_id.is_some()
            && (starts_with_ci("-//w3c//dtd html 4.01 transitional//", public_id)
                || starts_with_ci("-//w3c//dtd html 4.01 frameset//", public_id)))
}

const QUIRKY_PUBLIC_IDS: &[&str] = &[
    "+//silmaril//dtd html pro v0r11 19970101//",
    "-//advasoft ltd//dtd html 3.0 aswedit + extensions//",
    "-//as//dtd html 3.0 aswedit + extensions//",
    "-//ietf//dtd html 2.0 level 1//",
    "-//ietf//dtd html 2.0 level 2//",
    "-//ietf//dtd html 2.0 strict level 1//",
    "-//ietf//dtd html 2.0 strict level 2//",
    "-//ietf//dtd html 2.0 strict//",
    "-//ietf//dtd html 2.0//",
    "-//ietf//dtd html 2.1e//",
    "-//ietf//dtd html 3.0//",
    "-//ietf//dtd html 3.2 final//",
    "-//ietf//dtd html 3.2//",
    "-//ietf//dtd html 3//",
    "-//ietf//dtd html level 0//",
    "-//ietf//dtd html level 1//",
    "-//ietf//dtd html level 2//",
    "-//ietf//dtd html level 3//",
    "-//ietf//dtd html strict level 0//",
    "-//ietf//dtd html strict level 1//",
    "-//ietf//dtd html strict level 2//",
    "-//ietf//dtd html strict level 3//",
    "-//ietf//dtd html strict//",
    "-//ietf//dtd html//",
    "-//metrius//dtd metrius presentational//",
    "-//microsoft//dtd internet explorer 2.0 html strict//",
    "-//microsoft//dtd internet explorer 2.0 html//",
    "-//microsoft//dtd internet explorer 2.0 tables//",
    "-//microsoft//dtd internet explorer 3.0 html strict//",
    "-//microsoft//dtd internet explorer 3.0 html//",
    "-//microsoft//dtd internet explorer 3.0 tables//",
    "-//netscape comm. corp.//dtd html//",
    "-//netscape comm. corp.//dtd strict html//",
    "-//o'reilly and associates//dtd html 2.0//",
    "-//o'reilly and associates//dtd html extended 1.0//",
    "-//o'reilly and associates//dtd html extended relaxed 1.0//",
    "-//softquad software//dtd hotmetal pro 6.0::19990601::extensions to html 4.0//",
    "-//softquad//dtd hotmetal pro 4.0::19971010::extensions to html 4.0//",
    "-//spyglass//dtd html 2.0 extended//",
    "-//sq//dtd html 2.0 hotmetal + extensions//",
    "-//sun microsystems corp.//dtd hotjava html//",
    "-//sun microsystems corp.//dtd hotjava strict html//",
    "-//w3c//dtd html 3 1995-03-24//",
    "-//w3c//dtd html 3.2 draft//",
    "-//w3c//dtd html 3.2 final//",
    "-//w3c//dtd html 3.2//",
    "-//w3c//dtd html 3.2s draft//",
    "-//w3c//dtd html 4.0 frameset//",
    "-//w3c//dtd html 4.0 transitional//",
    "-//w3c//dtd html experimental 19960712//",
    "-//w3c//dtd html experimental 970421//",
    "-//w3c//dtd w3 html//",
    "-//w3o//dtd w3 html 3.0//",
    "-//webtechs//dtd mozilla html 2.0//",
    "-//webtechs//dtd mozilla html//",
];
