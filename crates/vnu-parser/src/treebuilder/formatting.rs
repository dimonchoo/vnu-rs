use super::*;

impl TreeBuilder {
    pub(super) fn insert_marker(&mut self) {
        self.list.push(None);
    }

    pub(super) fn clear_list_to_last_marker(&mut self) {
        while let Some(entry) = self.list.pop() {
            if entry.is_none() {
                return;
            }
        }
    }

    pub(super) fn find_in_list(&self, node: &Node) -> Option<usize> {
        self.list
            .iter()
            .rposition(|e| e.as_ref().is_some_and(|n| Rc::ptr_eq(n, node)))
    }

    pub(super) fn find_in_list_after_last_marker(&self, name: &str) -> Option<usize> {
        for i in (0..self.list.len()).rev() {
            match &self.list[i] {
                None => return None,
                Some(n) if n.name == name => return Some(i),
                _ => {}
            }
        }
        None
    }

    pub(super) fn maybe_forget_earlier_duplicate_formatting_element(
        &mut self,
        name: &str,
        attrs: &[(String, String)],
    ) {
        let mut candidate = None;
        let mut count = 0;
        for i in (0..self.list.len()).rev() {
            let Some(node) = &self.list[i] else {
                break;
            };
            if node.name == name && same_attrs(&node.attrs, attrs) {
                candidate = Some(i);
                count += 1;
            }
        }
        if count >= 3 {
            if let Some(c) = candidate {
                self.list.remove(c);
            }
        }
    }

    pub(super) fn reconstruct_the_active_formatting_elements(&mut self) {
        let Some(Some(most_recent)) = self.list.last() else {
            return;
        };
        if self.is_in_stack(most_recent) {
            return;
        }
        let last = self.list.len() - 1;
        let mut entry_pos = last as isize;
        loop {
            entry_pos -= 1;
            if entry_pos == -1 {
                break;
            }
            match &self.list[entry_pos as usize] {
                None => break,
                Some(n) if self.is_in_stack(n) => break,
                _ => {}
            }
        }
        for pos in (entry_pos + 1) as usize..=last {
            // Entries after the reopen point are elements, never markers.
            let Some(entry) = self.list[pos].clone() else {
                continue;
            };
            if self.current_fosters() {
                self.fatal();
            }
            let clone = entry.reopen();
            self.push(clone.clone());
            self.list[pos] = Some(clone);
        }
    }

    /// Returns false when the end tag must be handled like "any other end tag".
    pub(super) fn adoption_agency_end_tag(&mut self, name: &str) -> bool {
        if let Some(cur) = self.current() {
            if cur.is_html(name) && self.find_in_list(cur).is_none() {
                self.pop();
                return true;
            }
        }
        // Java runs this up to eight times; every case except plain popping ends in
        // fatal() or returns, so one round is enough.
        let Some(formatting_elt_list_pos) = self.find_in_list_after_last_marker(name) else {
            return false;
        };
        let Some(formatting_elt) = self.list[formatting_elt_list_pos].clone() else {
            return false;
        };
        let mut formatting_elt_stack_pos = None;
        let mut in_scope = true;
        for i in (0..self.stack.len()).rev() {
            let node = &self.stack[i];
            if Rc::ptr_eq(node, &formatting_elt) {
                formatting_elt_stack_pos = Some(i);
                break;
            } else if node.is_scoping() {
                in_scope = false;
            }
        }
        let Some(formatting_elt_stack_pos) = formatting_elt_stack_pos else {
            self.err_no_element_to_close(name);
            self.list.remove(formatting_elt_list_pos);
            return true;
        };
        if !in_scope {
            self.err_no_element_to_close(name);
            return true;
        }
        if formatting_elt_stack_pos != self.current_ptr() {
            self.err(format!("End tag “{name}” violates nesting rules."));
        }
        let mut furthest_block_pos = formatting_elt_stack_pos + 1;
        while furthest_block_pos <= self.current_ptr() {
            if self.stack[furthest_block_pos].is_special() {
                break;
            }
            furthest_block_pos += 1;
        }
        if furthest_block_pos > self.current_ptr() {
            self.pop_until(formatting_elt_stack_pos);
            self.list.remove(formatting_elt_list_pos);
            return true;
        }
        // A furthest block exists: the algorithm now moves nodes (removeFromStack
        // of a non-current node, detachFromParent or a foster-parent insertion),
        // which SAXStreamer cannot do.
        self.fatal();
        true
    }
}

fn same_attrs(a: &[(String, String)], b: &[(String, String)]) -> bool {
    a.len() == b.len()
        && a.iter()
            .all(|(k, v)| b.iter().any(|(k2, v2)| k == k2 && v == v2))
}
