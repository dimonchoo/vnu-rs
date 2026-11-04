use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use std::collections::{HashMap, HashSet};
use vnu_core::VNuMessage;

#[derive(Default)]
struct TableScope {
    table_index: usize,
    th_ids: HashSet<String>,
}

#[derive(Default)]
pub struct TablesRule {
    table_counter: usize,
    table_stack: Vec<TableScope>,
    finished_table_th_ids: HashMap<usize, HashSet<String>>,
    pending_headers: Vec<(String, usize, usize, usize)>,
}

impl Rule for TablesRule {
    fn name(&self) -> &'static str {
        "tables"
    }

    fn reset(&mut self) {
        self.table_counter = 0;
        self.table_stack.clear();
        self.finished_table_th_ids.clear();
        self.pending_headers.clear();
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "table" {
            self.table_counter += 1;
            self.table_stack.push(TableScope {
                table_index: self.table_counter,
                th_ids: HashSet::new(),
            });
        }

        if name_lower == "th" {
            if let Some(id) = tag.get_attr("id") {
                if let Some(active) = self.table_stack.last_mut() {
                    active.th_ids.insert(id.to_string());
                }
            }
        }

        if name_lower == "input"
            && ctx
                .open_elements
                .last()
                .is_some_and(|top| top.name.eq_ignore_ascii_case("table"))
        {
            msgs.push(VNuMessage::error(
                "Start tag “input” seen in “table”.",
                tag.line,
                tag.col,
            ));
        }

        if name_lower == "col" {
            if ctx
                .parent()
                .is_some_and(|p| p.name.eq_ignore_ascii_case("table"))
            {
                msgs.push(VNuMessage::error(
                    "Element “col” not allowed as child of “table” in this context.",
                    tag.line,
                    tag.col,
                ));
            }
            if let Some(span) = tag.get_attr("span") {
                if let Ok(v) = span.trim().parse::<u64>() {
                    if v > 1000 {
                        msgs.push(VNuMessage::error(
                            "The value of the “span” attribute must be less than or equal to 1000.",
                            tag.line,
                            tag.col,
                        ));
                    }
                }
            }
        }

        if name_lower == "tr" {
            let parent_is_table_section = ctx
                .parent()
                .map(|p| {
                    let p_name = p.name.to_ascii_lowercase();
                    p_name == "table" || p_name == "thead" || p_name == "tbody" || p_name == "tfoot"
                })
                .unwrap_or(false);

            if !parent_is_table_section {
                msgs.push(VNuMessage::error(
                    "Stray start tag “tr”.",
                    tag.line,
                    tag.col,
                ));
            }
        } else if name_lower == "td" || name_lower == "th" {
            let parent_is_tr = ctx
                .parent()
                .map(|p| p.name.eq_ignore_ascii_case("tr"))
                .unwrap_or(false);
            if !parent_is_tr {
                if ctx.is_descendant_of("table") {
                    msgs.push(VNuMessage::error(
                        format!("“{}” start tag in table body.", name_lower),
                        tag.line,
                        tag.col,
                    ));
                } else {
                    msgs.push(VNuMessage::error(
                        format!("Stray start tag “{}”.", name_lower),
                        tag.line,
                        tag.col,
                    ));
                }
            }

            if let Some(cs) = tag.get_attr("colspan") {
                if let Ok(v) = cs.trim().parse::<u64>() {
                    if v > 1000 {
                        msgs.push(VNuMessage::error(
                            "The value of the “colspan” attribute must be less than or equal to 1000.",
                            tag.line,
                            tag.col,
                        ));
                    }
                }
            }
            if let Some(rs) = tag.get_attr("rowspan") {
                if let Ok(v) = rs.trim().parse::<u64>() {
                    if v > 65534 {
                        msgs.push(VNuMessage::error(
                            "The value of the “rowspan” attribute must be less than or equal to 65534.",
                            tag.line,
                            tag.col,
                        ));
                    }
                }
            }

            if name_lower == "td" && tag.has_attr("role") {
                for ancestor in ctx.open_elements.iter().rev() {
                    if ancestor.name.eq_ignore_ascii_case("table") {
                        let r = ancestor.get_attr("role").unwrap_or("");
                        if r.is_empty() || matches!(r, "table" | "grid" | "treegrid") {
                            msgs.push(VNuMessage::error(
                                "The “role” attribute must not be used on a “td” element which has a “table” ancestor with no “role” attribute, or with a “role” attribute whose value is “table”, “grid”, or “treegrid”.",
                                tag.line,
                                tag.col,
                            ));
                            break;
                        }
                    }
                }
            }

            if name_lower == "td" {
                if let Some(headers) = tag.get_attr("headers") {
                    if let Some(active) = self.table_stack.last() {
                        let t_idx = active.table_index;
                        for id in headers.split_whitespace() {
                            self.pending_headers
                                .push((id.to_string(), t_idx, tag.line, tag.col));
                        }
                    }
                }
            }
        } else if matches!(
            name_lower.as_str(),
            "thead" | "tbody" | "tfoot" | "caption" | "colgroup"
        ) {
            let parent_is_table = ctx
                .parent()
                .map(|p| p.name.eq_ignore_ascii_case("table"))
                .unwrap_or(false);
            if !parent_is_table {
                msgs.push(VNuMessage::error(
                    format!("Stray start tag “{}”.", name_lower),
                    tag.line,
                    tag.col,
                ));
            }
        }

        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        tag_name: &str,
        _line: usize,
        _col: usize,
    ) -> Vec<VNuMessage> {
        if tag_name.eq_ignore_ascii_case("table") {
            if let Some(finished) = self.table_stack.pop() {
                self.finished_table_th_ids
                    .insert(finished.table_index, finished.th_ids);
            }
        }
        vec![]
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();

        for (id, t_idx, line, col) in &self.pending_headers {
            let th_set = self
                .table_stack
                .iter()
                .find(|s| s.table_index == *t_idx)
                .map(|s| &s.th_ids)
                .or_else(|| self.finished_table_th_ids.get(t_idx));
            let id_in_table = th_set.is_some_and(|set| set.contains(id));
            if !id_in_table || ctx.id_to_tag.get(id).map(|t| t.as_str()) != Some("th") {
                msgs.push(VNuMessage::error(
                    format!(
                        "The “headers” attribute on the element “td” refers to the ID “{}”, but there is no “th” element with that ID in the same table.",
                        id
                    ),
                    *line,
                    *col,
                ));
            }
        }

        msgs
    }
}
