pub mod grid;

use crate::context::{ElementContext, ValidationContext};
use crate::rule::Rule;
use grid::TableGridModel;
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct TablesRule {
    model: TableGridModel,
    pending_headers: Vec<(String, usize, usize, usize)>,
}

impl Rule for TablesRule {
    fn name(&self) -> &'static str {
        "tables"
    }

    fn reset(&mut self) {
        self.model.reset();
        self.pending_headers.clear();
    }

    fn interested_in_start_tag(&self, tag: &ElementContext) -> bool {
        matches!(
            tag.name.as_str(),
            "table"
                | "TABLE"
                | "th"
                | "TH"
                | "td"
                | "TD"
                | "tr"
                | "TR"
                | "col"
                | "COL"
                | "colgroup"
                | "COLGROUP"
                | "thead"
                | "THEAD"
                | "tbody"
                | "TBODY"
                | "tfoot"
                | "TFOOT"
                | "input"
                | "INPUT"
        )
    }

    fn on_start_tag(&mut self, ctx: &ValidationContext, tag: &ElementContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        let name_lower = tag.name.to_ascii_lowercase();

        if name_lower == "table" {
            self.model.enter_table();
        } else if name_lower == "th" {
            if let Some(id) = tag.get_attr("id") {
                self.model.enter_th_id(id);
            }
        } else if name_lower == "input"
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

        if name_lower == "col" || name_lower == "colgroup" {
            self.validate_col_and_colgroup(ctx, tag, &name_lower, &mut msgs);
        } else if matches!(name_lower.as_str(), "thead" | "tbody" | "tfoot") {
            self.model.enter_row_group(&name_lower);
        } else if name_lower == "tr" {
            self.validate_tr_start(ctx, tag, &mut msgs);
        } else if name_lower == "td" || name_lower == "th" {
            self.validate_cell_start(ctx, tag, &name_lower, &mut msgs);
        }

        msgs
    }

    fn on_end_tag(
        &mut self,
        _ctx: &ValidationContext,
        tag_name: &str,
        line: usize,
        col: usize,
    ) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        if tag_name.eq_ignore_ascii_case("tr") {
            self.model.exit_tr(line, col, &mut msgs);
        } else if tag_name.eq_ignore_ascii_case("tbody") {
            self.model.exit_tbody(line, col, &mut msgs);
        } else if tag_name.eq_ignore_ascii_case("table") {
            self.model.exit_table(line, col, &mut msgs);
        }
        msgs
    }

    fn on_finish(&mut self, ctx: &ValidationContext) -> Vec<VNuMessage> {
        let mut msgs = Vec::new();
        for (id, t_idx, line, col) in &self.pending_headers {
            let th_set = self
                .model
                .table_stack
                .iter()
                .find(|s| s.table_index == *t_idx)
                .map(|s| &s.th_ids)
                .or_else(|| self.model.finished_table_th_ids.get(t_idx));
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

impl TablesRule {
    fn validate_col_and_colgroup(
        &mut self,
        ctx: &ValidationContext,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
        if name_lower == "col"
            && ctx
                .parent()
                .is_some_and(|p| p.name.eq_ignore_ascii_case("table"))
        {
            msgs.push(VNuMessage::error(
                "Element “col” not allowed as child of “table” in this context.",
                tag.line,
                tag.col,
            ));
        }

        let span_val = tag
            .get_attr("span")
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(1);

        if name_lower == "col" || (name_lower == "colgroup" && tag.has_attr("span")) {
            self.model.add_column_markup(span_val);
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

    fn validate_tr_start(
        &mut self,
        ctx: &ValidationContext,
        tag: &ElementContext,
        msgs: &mut Vec<VNuMessage>,
    ) {
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
        } else {
            let auto_tbody = ctx.mode == crate::context::DocumentMode::Html && !ctx.is_xhtml;
            self.model.enter_tr(auto_tbody);
        }
    }

    fn validate_cell_start(
        &mut self,
        ctx: &ValidationContext,
        tag: &ElementContext,
        name_lower: &str,
        msgs: &mut Vec<VNuMessage>,
    ) {
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

        let colspan = tag
            .get_attr("colspan")
            .and_then(|cs| cs.trim().parse::<usize>().ok())
            .unwrap_or(1);
        let rowspan = tag
            .get_attr("rowspan")
            .and_then(|rs| rs.trim().parse::<usize>().ok())
            .unwrap_or(1);

        let mut has_span_error = false;
        if let Some(cs) = tag.get_attr("colspan") {
            if let Ok(v) = cs.trim().parse::<u64>() {
                if v > 1000 {
                    msgs.push(VNuMessage::error(
                        "The value of the “colspan” attribute must be less than or equal to 1000.",
                        tag.line,
                        tag.col,
                    ));
                    has_span_error = true;
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
                    has_span_error = true;
                }
            }
        }

        if !has_span_error {
            self.model
                .place_cell(colspan, rowspan, tag.line, tag.col, msgs);
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
                    }
                    break;
                }
            }
        }

        if name_lower == "td" {
            if let Some(headers) = tag.get_attr("headers") {
                if let Some(active) = self.model.table_stack.last() {
                    let t_idx = active.table_index;
                    for id in headers.split_whitespace() {
                        self.pending_headers
                            .push((id.to_string(), t_idx, tag.line, tag.col));
                    }
                }
            }
        }
    }
}
