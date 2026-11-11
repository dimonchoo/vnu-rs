use super::names::groups::*;
use super::*;

impl TreeBuilder {
    pub(super) fn current(&self) -> Option<&Node> {
        self.stack.last()
    }

    /// Java's `currentPtr`; 0 on an empty stack (Java's -1 is never compared there).
    pub(super) fn current_ptr(&self) -> usize {
        self.stack.len().saturating_sub(1)
    }

    pub(super) fn current_group(&self) -> u32 {
        self.stack.last().map_or(OTHER, |n| n.group())
    }

    pub(super) fn current_fosters(&self) -> bool {
        self.stack.last().is_some_and(|n| n.is_foster_parenting())
    }

    pub(super) fn is_current(&self, name: &str) -> bool {
        self.stack.last().is_some_and(|n| n.is_html(name))
    }

    pub(super) fn new_loc(&self) -> Rc<Loc> {
        Rc::new(Loc {
            line: self.line,
            col: self.col,
            tainted: Cell::new(false),
        })
    }

    pub(super) fn push(&mut self, node: Node) {
        if self.record_events && !self.halted {
            self.events.push(TreeEvent::Push {
                ns: node.ns,
                name: node.pop_name.clone(),
                attrs: node.attrs.clone(),
                line: node.loc.line,
                col: node.loc.col,
            });
        }
        self.stack.push(node);
    }

    pub(super) fn silent_push(&mut self, node: Node) {
        self.stack.push(node);
    }

    pub(super) fn pop(&mut self) {
        let Some(node) = self.stack.pop() else {
            return;
        };
        if self.record_events && !self.halted {
            self.events.push(TreeEvent::Pop {
                ns: node.ns,
                name: node.pop_name.clone(),
            });
        }
    }

    pub(super) fn silent_pop(&mut self) {
        self.stack.pop();
    }

    /// Push then pop, as `appendVoidElementToCurrent*` does.
    pub(super) fn push_pop_void(&mut self, ns: Ns, name: Cow<'static, str>, attrs: Attrs) {
        if !self.record_events || self.halted {
            return;
        }
        self.events.push(TreeEvent::Push {
            ns,
            name: name.clone(),
            attrs,
            line: self.line,
            col: self.col,
        });
        self.events.push(TreeEvent::Pop { ns, name });
    }

    pub(super) fn html_node(&self, en: &ElemName, attrs: Attrs) -> Node {
        Rc::new(StackNode {
            flags: en.flags,
            name: en.name.clone(),
            pop_name: en.name.clone(),
            ns: Ns::Html,
            attrs,
            loc: self.new_loc(),
        })
    }

    pub(super) fn remove_from_stack(&mut self, pos: usize) {
        if pos == self.current_ptr() {
            self.pop();
        } else {
            self.fatal();
            self.stack.remove(pos);
        }
    }

    pub(super) fn remove_node_from_stack(&mut self, node: &Node) {
        if self.current().is_some_and(|c| Rc::ptr_eq(c, node)) {
            self.pop();
            return;
        }
        if let Some(pos) = self.stack.iter().rposition(|n| Rc::ptr_eq(n, node)) {
            self.fatal();
            self.stack.remove(pos);
        }
    }

    pub(super) fn is_in_stack(&self, node: &Node) -> bool {
        self.stack.iter().any(|n| Rc::ptr_eq(n, node))
    }

    pub(super) fn is_template_contents(&self) -> bool {
        self.find_last("template").is_some()
    }

    pub(super) fn is_in_foreign_but_not_integration_point(&self) -> bool {
        self.stack
            .last()
            .is_some_and(|n| !Self::is_special_parent_in_foreign(n))
    }

    pub(super) fn is_special_parent_in_foreign(node: &StackNode) -> bool {
        node.ns == Ns::Html
            || node.is_html_integration_point()
            || (node.ns == Ns::MathMl && node.group() == MI_MO_MN_MS_MTEXT)
    }

    pub(super) fn is_second_on_stack_body(&self) -> bool {
        self.stack.len() >= 2 && self.stack[1].group() == BODY
    }

    // Java uses NOT_FOUND_ON_STACK; here `None`. Index 0 is never returned by the
    // scope searches, as in Java (`i > 0`).

    pub(super) fn find_last(&self, name: &str) -> Option<usize> {
        (1..self.stack.len())
            .rev()
            .find(|&i| self.stack[i].is_html(name))
    }

    pub(super) fn find_last_in_table_scope(&self, name: &str) -> Option<usize> {
        for i in (1..self.stack.len()).rev() {
            let n = &self.stack[i];
            if n.ns == Ns::Html {
                if n.name == name {
                    return Some(i);
                } else if n.name == "table" || n.name == "template" {
                    return None;
                }
            }
        }
        None
    }

    pub(super) fn find_last_in_button_scope(&self, name: &str) -> Option<usize> {
        for i in (1..self.stack.len()).rev() {
            let n = &self.stack[i];
            if n.ns == Ns::Html {
                if n.name == name {
                    return Some(i);
                } else if n.name == "button" {
                    return None;
                }
            }
            if n.is_scoping() {
                return None;
            }
        }
        None
    }

    pub(super) fn find_last_in_scope(&self, name: &str) -> Option<usize> {
        for i in (1..self.stack.len()).rev() {
            let n = &self.stack[i];
            if n.is_html(name) {
                return Some(i);
            } else if n.is_scoping() {
                return None;
            }
        }
        None
    }

    pub(super) fn find_last_in_list_scope(&self, name: &str) -> Option<usize> {
        for i in (1..self.stack.len()).rev() {
            let n = &self.stack[i];
            if n.ns == Ns::Html {
                if n.name == name {
                    return Some(i);
                } else if n.name == "ul" || n.name == "ol" {
                    return None;
                }
            }
            if n.is_scoping() {
                return None;
            }
        }
        None
    }

    pub(super) fn find_last_in_scope_hn(&self) -> Option<usize> {
        for i in (1..self.stack.len()).rev() {
            let n = &self.stack[i];
            if n.group() == H1_H6 {
                return Some(i);
            } else if n.is_scoping() {
                return None;
            }
        }
        None
    }

    pub(super) fn find_last_in_table_scope_or_root_template_tbody(&self) -> usize {
        (1..self.stack.len())
            .rev()
            .find(|&i| {
                let n = &self.stack[i];
                n.ns == Ns::Html && (n.group() == TBODY_ETC || n.group() == TEMPLATE)
            })
            .unwrap_or(0)
    }

    pub(super) fn find_last_in_table_scope_td_th(&self) -> Option<usize> {
        for i in (1..self.stack.len()).rev() {
            let n = &self.stack[i];
            if n.ns == Ns::Html {
                if n.name == "td" || n.name == "th" {
                    return Some(i);
                } else if n.name == "table" || n.name == "template" {
                    return None;
                }
            }
        }
        None
    }

    pub(super) fn find_last_or_root_name(&self, name: &str) -> usize {
        self.find_last(name).unwrap_or(0)
    }

    pub(super) fn find_last_or_root_group(&self, group: u32) -> usize {
        (1..self.stack.len())
            .rev()
            .find(|&i| self.stack[i].ns == Ns::Html && self.stack[i].group() == group)
            .unwrap_or(0)
    }

    pub(super) fn generate_implied_end_tags_except_for(&mut self, name: &str) {
        while let Some(node) = self.current() {
            match node.group() {
                P | LI | DD_OR_DT | OPTION | OPTGROUP | RB_OR_RTC | RT_OR_RP => {
                    if node.is_html(name) {
                        return;
                    }
                    self.pop();
                }
                _ => return,
            }
        }
    }

    pub(super) fn generate_implied_end_tags(&mut self) {
        while matches!(
            self.current_group(),
            P | LI | DD_OR_DT | OPTION | OPTGROUP | RB_OR_RTC | RT_OR_RP
        ) {
            self.pop();
        }
    }

    pub(super) fn generate_implied_end_tags_thoroughly(&mut self) {
        while matches!(
            self.current_group(),
            CAPTION
                | COLGROUP
                | DD_OR_DT
                | LI
                | OPTGROUP
                | OPTION
                | P
                | RB_OR_RTC
                | RT_OR_RP
                | TBODY_ETC
                | TD_OR_TH
                | TR
        ) {
            self.pop();
        }
    }

    pub(super) fn pop_until(&mut self, elt_pos: usize) {
        while self.stack.len() > elt_pos {
            self.pop();
        }
    }

    pub(super) fn close_the_cell(&mut self, elt_pos: usize) {
        self.generate_implied_end_tags();
        if elt_pos != self.current_ptr() {
            self.err_unclosed_elements_cell(elt_pos);
        }
        self.pop_until(elt_pos);
        self.clear_list_to_last_marker();
        self.mode = Mode::InRow;
    }

    pub(super) fn clear_stack_back_to(&mut self, elt_pos: usize) {
        let elt_group = self.stack[elt_pos].group();
        while self.current_ptr() > elt_pos {
            let Some(cur) = self.current() else {
                return;
            };
            if cur.ns == Ns::Html
                && cur.group() == TEMPLATE
                && (elt_group == TABLE || elt_group == TBODY_ETC || elt_group == TR || elt_pos == 0)
            {
                return;
            }
            self.pop();
        }
    }

    pub(super) fn body_mode(&self) -> Mode {
        if self.frameset_ok {
            Mode::FramesetOk
        } else {
            Mode::InBody
        }
    }

    pub(super) fn reset_the_insertion_mode(&mut self) {
        for i in (0..self.stack.len()).rev() {
            let node = &self.stack[i];
            let name = node.name.as_ref();
            if name == "td" || name == "th" {
                self.mode = Mode::InCell;
                return;
            } else if name == "tr" {
                self.mode = Mode::InRow;
                return;
            } else if name == "tbody" || name == "thead" || name == "tfoot" {
                self.mode = Mode::InTableBody;
                return;
            } else if name == "caption" {
                self.mode = Mode::InCaption;
                return;
            } else if name == "colgroup" {
                self.mode = Mode::InColumnGroup;
                return;
            } else if name == "table" {
                self.mode = Mode::InTable;
                return;
            } else if node.ns != Ns::Html {
                self.mode = self.body_mode();
                return;
            } else if name == "template" {
                self.mode = *self.template_modes.last().unwrap_or(&Mode::InBody);
                return;
            } else if name == "head" {
                self.mode = Mode::InHead;
                return;
            } else if name == "body" {
                self.mode = self.body_mode();
                return;
            } else if name == "frameset" {
                self.mode = Mode::InFrameset;
                return;
            } else if name == "html" {
                self.mode = if self.head_pointer {
                    Mode::AfterHead
                } else {
                    Mode::BeforeHead
                };
                return;
            } else if i == 0 {
                self.mode = self.body_mode();
                return;
            }
        }
    }

    pub(super) fn implicitly_close_p(&mut self) {
        let Some(elt_pos) = self.find_last_in_button_scope("p") else {
            return;
        };
        self.generate_implied_end_tags_except_for("p");
        if elt_pos != self.current_ptr() {
            self.err_unclosed_elements_implied(elt_pos, "p");
        }
        self.pop_until(elt_pos);
    }
}
