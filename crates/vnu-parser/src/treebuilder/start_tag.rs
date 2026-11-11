use super::names::groups::*;
use super::*;

/// A start tag while it moves through the insertion modes.
pub(super) struct StartTag {
    pub(super) en: ElemName,
    pub(super) attrs: Attrs,
    pub(super) self_closing: bool,
    pub(super) void_element: bool,
    pub(super) state: Option<TokenizerState>,
}

impl TreeBuilder {
    /// A start tag. Returns the tokenizer state to switch to, if any.
    pub fn start_tag(
        &mut self,
        name: &str,
        attrs: Attrs,
        self_closing: bool,
        line: usize,
        col: usize,
    ) -> Option<TokenizerState> {
        if self.halted {
            return None;
        }
        self.flush_characters();
        self.set_position(line, col);
        self.check_duplicate_id(&attrs, line, col);
        let mut t = StartTag {
            en: ElemName::lookup(name),
            attrs,
            self_closing,
            void_element: false,
            state: None,
        };
        self.need_to_drop_lf = false;
        'token: loop {
            if self.halted {
                return None;
            }
            if self.in_foreign() {
                match self.start_in_foreign(&mut t) {
                    Some(Step::Reprocess) => continue 'token,
                    Some(_) => break 'token,
                    None => {}
                }
            }
            let mut mode = self.mode;
            loop {
                let step = match mode {
                    Mode::InTemplate => self.start_in_template(&mut t),
                    Mode::InRow => self.start_in_row(&mut t),
                    Mode::InTableBody => self.start_in_table_body(&mut t),
                    Mode::InTable => self.start_in_table(&mut t),
                    Mode::InCaption => self.start_in_caption(&mut t),
                    Mode::InCell => self.start_in_cell(&mut t),
                    Mode::FramesetOk => self.start_frameset_ok(&mut t),
                    Mode::InBody => self.start_in_body(&mut t),
                    Mode::InHead => self.start_in_head(&mut t),
                    Mode::InHeadNoscript => self.start_in_head_noscript(&mut t),
                    other => self.start_other(other, &mut t),
                };
                match step {
                    Step::Done => break 'token,
                    Step::Reprocess => continue 'token,
                    Step::FallTo(next) => mode = next,
                }
            }
        }
        if t.self_closing {
            self.err("Self-closing syntax (“/>”) used on a non-void HTML element. Ignoring the slash and treating as a start tag.");
        } else if self_closing && t.void_element && !self.halted {
            self.messages.push(VNuMessage::info(
                "Trailing slash on void elements has no effect and interacts badly with unquoted attribute values.",
                self.line,
                self.col,
            ));
        }
        if self.halted { None } else { t.state }
    }

    /// ID uniqueness, checked by nu TreeBuilder before tree construction.
    fn check_duplicate_id(&mut self, attrs: &[(String, String)], line: usize, col: usize) {
        let Some(id) = attr(attrs, "id") else {
            return;
        };
        if self.is_template_contents() {
            return;
        }
        if let Some(&(l, c)) = self.id_locations.get(id) {
            self.err(format!("Duplicate ID “{id}”."));
            if !self.halted {
                self.messages.push(VNuMessage::warning(
                    format!("The first occurrence of ID “{id}” was here."),
                    l,
                    c,
                ));
            }
        } else {
            self.id_locations.insert(id.to_string(), (line, col));
        }
    }

    /// Start tags in SVG or MathML content. `None` when the mode handlers take over
    /// (integration points).
    fn start_in_foreign(&mut self, t: &mut StartTag) -> Option<Step> {
        let group = t.en.group();
        let cur = self.current()?.clone();
        let in_integration = cur.is_html_integration_point()
            || (cur.ns == Ns::MathMl
                && ((cur.group() == MI_MO_MN_MS_MTEXT && group != MGLYPH_OR_MALIGNMARK)
                    || (cur.group() == ANNOTATION_XML && group == SVG)));
        if in_integration {
            return None;
        }
        let breakout = matches!(
            group,
            B_ETC
                | DIV_ETC
                | BODY
                | BR
                | RUBY_ETC
                | DD_OR_DT
                | UL_OR_OL_OR_DL
                | EMBED
                | IMG
                | H1_H6
                | HEAD
                | HR
                | LI
                | META
                | NOBR
                | P
                | PRE_OR_LISTING
                | TABLE
                | FONT
        ) && !(group == FONT
            && !(attr(&t.attrs, "color").is_some()
                || attr(&t.attrs, "face").is_some()
                || attr(&t.attrs, "size").is_some()));
        if breakout {
            self.err_html_start_tag_in_foreign_context(&t.en.name);
            while self.current_ptr() > 0
                && !self
                    .current()
                    .is_some_and(|n| Self::is_special_parent_in_foreign(n))
            {
                self.pop();
            }
            return Some(Step::Reprocess);
        }
        if t.self_closing {
            self.append_void_foreign(&t.en, t.attrs.clone(), cur.ns);
            t.self_closing = false;
        } else {
            self.append_to_current_node_and_push_foreign(&t.en, t.attrs.clone(), cur.ns);
        }
        Some(Step::Done)
    }

    /// title, script, style, ... handled through IN_HEAD: push, enter TEXT mode.
    pub(super) fn start_tag_text(
        &mut self,
        en: &ElemName,
        attrs: Attrs,
        state: TokenizerState,
        may_foster: bool,
    ) -> TokenizerState {
        if may_foster {
            self.append_to_current_node_and_push_element_may_foster(en, attrs);
        } else {
            self.append_to_current_node_and_push_element(en, attrs);
        }
        self.original_mode = self.mode;
        self.mode = Mode::Text;
        state
    }

    pub(super) fn start_tag_template_in_head(&mut self, attrs: Attrs) {
        self.append_to_current_node_and_push_template_element(attrs);
        self.insert_marker();
        self.frameset_ok = false;
        self.original_mode = self.mode;
        self.mode = Mode::InTemplate;
        self.template_modes.push(Mode::InTemplate);
    }

    pub(super) fn start_in_template(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            COL => {
                self.template_modes.pop();
                self.template_modes.push(Mode::InColumnGroup);
                self.mode = Mode::InColumnGroup;
                Step::Reprocess
            }
            CAPTION | COLGROUP | TBODY_ETC => {
                self.template_modes.pop();
                self.template_modes.push(Mode::InTable);
                self.mode = Mode::InTable;
                Step::Reprocess
            }
            TR => {
                self.template_modes.pop();
                self.template_modes.push(Mode::InTableBody);
                self.mode = Mode::InTableBody;
                Step::Reprocess
            }
            TD_OR_TH => {
                self.template_modes.pop();
                self.template_modes.push(Mode::InRow);
                self.mode = Mode::InRow;
                Step::Reprocess
            }
            META | BASE | LINK_OR_BASEFONT_OR_BGSOUND => {
                self.append_void_element_to_current_may_foster(&t.en, t.attrs.clone());
                t.self_closing = false;
                t.void_element = true;
                Step::Done
            }
            TITLE => {
                t.state =
                    Some(self.start_tag_text(&t.en, t.attrs.clone(), TokenizerState::Rcdata, true));
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
            NOFRAMES | STYLE => {
                t.state = Some(self.start_tag_text(
                    &t.en,
                    t.attrs.clone(),
                    TokenizerState::Rawtext,
                    true,
                ));
                Step::Done
            }
            TEMPLATE => {
                self.start_tag_template_in_head(t.attrs.clone());
                Step::Done
            }
            _ => {
                self.template_modes.pop();
                self.template_modes.push(Mode::InBody);
                self.mode = Mode::InBody;
                Step::Reprocess
            }
        }
    }

    pub(super) fn start_in_row(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            TD_OR_TH => {
                let p = self.find_last_or_root_group(TR);
                self.clear_stack_back_to(p);
                self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                self.mode = Mode::InCell;
                self.insert_marker();
                Step::Done
            }
            CAPTION | COL | COLGROUP | TBODY_ETC | TR => {
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
            _ => Step::FallTo(Mode::InTableBody),
        }
    }

    pub(super) fn start_in_table_body(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            TR => {
                let p = self.find_last_in_table_scope_or_root_template_tbody();
                self.clear_stack_back_to(p);
                self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                self.mode = Mode::InRow;
                Step::Done
            }
            TD_OR_TH => {
                self.err(format!("“{}” start tag in table body.", t.en.name));
                let p = self.find_last_in_table_scope_or_root_template_tbody();
                self.clear_stack_back_to(p);
                self.append_to_current_node_and_push_element(&ElemName::lookup("tr"), no_attrs());
                self.mode = Mode::InRow;
                Step::Reprocess
            }
            CAPTION | COL | COLGROUP | TBODY_ETC => {
                let elt_pos = self.find_last_in_table_scope_or_root_template_tbody();
                if elt_pos == 0 || self.stack[elt_pos].group() == TEMPLATE {
                    self.err_stray_start_tag(&t.en.name);
                    return Step::Done;
                }
                self.clear_stack_back_to(elt_pos);
                self.pop();
                self.mode = Mode::InTable;
                Step::Reprocess
            }
            _ => Step::FallTo(Mode::InTable),
        }
    }

    pub(super) fn start_in_table(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        // `intableloop` breaks fall through to IN_BODY (or IN_HEAD for template).
        match group {
            CAPTION => {
                let p = self.find_last_or_root_group(TABLE);
                self.clear_stack_back_to(p);
                self.insert_marker();
                self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                self.mode = Mode::InCaption;
                Step::Done
            }
            COLGROUP => {
                let p = self.find_last_or_root_group(TABLE);
                self.clear_stack_back_to(p);
                self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                self.mode = Mode::InColumnGroup;
                Step::Done
            }
            COL => {
                let p = self.find_last_or_root_group(TABLE);
                self.clear_stack_back_to(p);
                self.append_to_current_node_and_push_element(
                    &ElemName::lookup("colgroup"),
                    no_attrs(),
                );
                self.mode = Mode::InColumnGroup;
                Step::Reprocess
            }
            TBODY_ETC => {
                let p = self.find_last_or_root_group(TABLE);
                self.clear_stack_back_to(p);
                self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                self.mode = Mode::InTableBody;
                Step::Done
            }
            TR | TD_OR_TH => {
                let p = self.find_last_or_root_group(TABLE);
                self.clear_stack_back_to(p);
                self.append_to_current_node_and_push_element(
                    &ElemName::lookup("tbody"),
                    no_attrs(),
                );
                self.mode = Mode::InTableBody;
                Step::Reprocess
            }
            TEMPLATE => Step::FallTo(Mode::InHead),
            TABLE => {
                self.err("Start tag for “table” seen but the previous “table” is still open.");
                let Some(elt_pos) = self.find_last_in_table_scope(&t.en.name) else {
                    return Step::Done;
                };
                self.generate_implied_end_tags();
                if !self.is_current("table") {
                    self.err("Unclosed elements on stack.");
                }
                self.pop_until(elt_pos);
                self.reset_the_insertion_mode();
                Step::Reprocess
            }
            SCRIPT => {
                self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                self.original_mode = self.mode;
                self.mode = Mode::Text;
                t.state = Some(TokenizerState::ScriptData);
                Step::Done
            }
            STYLE => {
                self.append_to_current_node_and_push_element(&t.en, t.attrs.clone());
                self.original_mode = self.mode;
                self.mode = Mode::Text;
                t.state = Some(TokenizerState::Rawtext);
                Step::Done
            }
            INPUT => {
                self.err(format!("Start tag “{}” seen in “table”.", t.en.name));
                if !equals_ci("hidden", attr(&t.attrs, "type")) {
                    Step::FallTo(Mode::InBody)
                } else {
                    self.push_pop_void(Ns::Html, Cow::Borrowed("input"), t.attrs.clone());
                    t.self_closing = false;
                    t.void_element = true;
                    Step::Done
                }
            }
            FORM => {
                if self.form_pointer || self.is_template_contents() {
                    self.err_form_when_form_open();
                    return Step::Done;
                }
                self.err(format!("Start tag “{}” seen in “table”.", t.en.name));
                self.form_pointer = true;
                self.push_pop_void(Ns::Html, Cow::Borrowed("form"), t.attrs.clone());
                Step::Done
            }
            _ => {
                self.err(format!("Start tag “{}” seen in “table”.", t.en.name));
                Step::FallTo(Mode::InBody)
            }
        }
    }

    pub(super) fn start_in_caption(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            CAPTION | COL | COLGROUP | TBODY_ETC | TR | TD_OR_TH => {
                let Some(elt_pos) = self.find_last_in_table_scope("caption") else {
                    self.err_stray_start_tag(&t.en.name);
                    return Step::Done;
                };
                self.generate_implied_end_tags();
                if self.current_ptr() != elt_pos {
                    self.err("Unclosed elements on stack.");
                }
                self.pop_until(elt_pos);
                self.clear_list_to_last_marker();
                self.mode = Mode::InTable;
                Step::Reprocess
            }
            _ => Step::FallTo(Mode::InCell),
        }
    }

    pub(super) fn start_in_cell(&mut self, t: &mut StartTag) -> Step {
        let group = t.en.group();
        match group {
            CAPTION | COL | COLGROUP | TBODY_ETC | TR | TD_OR_TH => {
                match self.find_last_in_table_scope_td_th() {
                    None => {
                        self.err("No cell to close.");
                        Step::Done
                    }
                    Some(p) => {
                        self.close_the_cell(p);
                        Step::Reprocess
                    }
                }
            }
            _ => Step::FallTo(Mode::FramesetOk),
        }
    }
}
