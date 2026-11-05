use std::collections::{HashMap, HashSet};
use vnu_core::VNuMessage;

#[derive(Default)]
pub struct TableGridModel {
    pub table_counter: usize,
    pub table_stack: Vec<TableScope>,
    pub finished_table_th_ids: HashMap<usize, HashSet<String>>,
}

#[derive(Default)]
pub struct TableScope {
    pub table_index: usize,
    pub th_ids: HashSet<String>,
    pub columns_established_by_markup: Option<usize>,
    pub first_row_columns: Option<usize>,
    pub current_row_columns: usize,
    pub current_row_index: usize,
    pub current_row_has_cells: bool,
    pub active_row_group: Option<String>,
    pub row_group_row_count: usize,
    pub max_row_span_in_group: usize,
    pub columns_with_starting_cells: HashSet<usize>,
    pub max_column_reached: usize,
    pub grid: HashMap<(usize, usize), bool>,
    pub next_free_col_by_row: HashMap<usize, usize>,
    pub overlap_emitted: bool,
}

impl TableGridModel {
    pub fn reset(&mut self) {
        self.table_counter = 0;
        self.table_stack.clear();
        self.finished_table_th_ids.clear();
    }

    pub fn enter_table(&mut self) {
        self.table_counter += 1;
        self.table_stack.push(TableScope {
            table_index: self.table_counter,
            ..Default::default()
        });
    }

    pub fn enter_th_id(&mut self, id: &str) {
        if let Some(active) = self.table_stack.last_mut() {
            active.th_ids.insert(id.to_string());
        }
    }

    pub fn add_column_markup(&mut self, span_val: usize) {
        if let Some(active) = self.table_stack.last_mut() {
            *active.columns_established_by_markup.get_or_insert(0) += span_val;
        }
    }

    pub fn enter_row_group(&mut self, group_name: &str) {
        if let Some(active) = self.table_stack.last_mut() {
            active.active_row_group = Some(group_name.to_string());
            active.row_group_row_count = 0;
            active.max_row_span_in_group = 0;
        }
    }

    pub fn enter_tr(&mut self, auto_tbody: bool) {
        if let Some(active) = self.table_stack.last_mut() {
            if auto_tbody && active.active_row_group.is_none() {
                active.active_row_group = Some("tbody".to_string());
            }
            active.current_row_index += 1;
            active.row_group_row_count += 1;
            active.current_row_columns = 0;
            active.current_row_has_cells = false;
        }
    }

    pub fn place_cell(
        &mut self,
        colspan: usize,
        rowspan: usize,
        line: usize,
        col: usize,
        msgs: &mut Vec<VNuMessage>,
    ) {
        let Some(active) = self.table_stack.last_mut() else {
            return;
        };

        if colspan > 1000 || rowspan > 65534 {
            return;
        }

        active.current_row_has_cells = true;
        let r = active.current_row_index;

        let sim_colspan = colspan.min(64);
        let sim_rowspan = rowspan.min(64);

        let next_free = active.next_free_col_by_row.entry(r).or_insert(1);
        let mut c = *next_free;
        while active.grid.contains_key(&(r, c)) {
            c += 1;
        }
        *next_free = c + sim_colspan.max(1);

        if active.grid.contains_key(&(r, c)) && !active.overlap_emitted {
            msgs.push(VNuMessage::error(
                "Table cell is overlapped by later table cell.",
                line,
                col,
            ));
            active.overlap_emitted = true;
        }

        let span_area = sim_colspan.max(1).saturating_mul(sim_rowspan.max(1));
        if span_area <= 256 && active.grid.len() + span_area <= 4096 {
            for ro in 0..sim_rowspan.max(1) {
                if ro > 0 {
                    let row_free = active.next_free_col_by_row.entry(r + ro).or_insert(1);
                    if *row_free == c {
                        *row_free = c + sim_colspan.max(1);
                    }
                }
                for co in 0..sim_colspan.max(1) {
                    let cell_slot = (r + ro, c + co);
                    if active.grid.insert(cell_slot, true).is_some() && !active.overlap_emitted {
                        msgs.push(VNuMessage::error(
                            "Table cell is overlapped by later table cell.",
                            line,
                            col,
                        ));
                        active.overlap_emitted = true;
                    }
                }
            }
        } else {
            for ro in 0..sim_rowspan.max(1) {
                if ro > 0 {
                    let row_free = active.next_free_col_by_row.entry(r + ro).or_insert(1);
                    if *row_free == c {
                        *row_free = c + sim_colspan.max(1);
                    }
                }
            }
        }

        active.columns_with_starting_cells.insert(c);
        active.current_row_columns += colspan;
        active.max_column_reached = active.max_column_reached.max(c + colspan - 1);

        if rowspan > 1 {
            active.max_row_span_in_group = active
                .max_row_span_in_group
                .max(active.row_group_row_count + rowspan - 1);
        }
    }

    pub fn exit_tr(&mut self, line: usize, col: usize, msgs: &mut Vec<VNuMessage>) {
        let Some(active) = self.table_stack.last_mut() else {
            return;
        };

        if !active.current_row_has_cells {
            if let Some(ref group) = active.active_row_group {
                msgs.push(VNuMessage::error(
                    format!(
                        "Row {} of a row group established by a “{}” element has no cells beginning on it.",
                        active.row_group_row_count, group
                    ),
                    line,
                    col,
                ));
            } else {
                msgs.push(VNuMessage::error(
                    format!(
                        "Row {} of an implicit row group has no cells beginning on it.",
                        active.current_row_index
                    ),
                    line,
                    col,
                ));
            }
        }

        let w = active.current_row_columns;
        if let Some(markup) = active.columns_established_by_markup {
            if w > markup {
                msgs.push(VNuMessage::error(
                    format!(
                        "A table row was {} columns wide and exceeded the column count established using column markup ({}).",
                        w, markup
                    ),
                    line,
                    col,
                ));
            } else if w < markup {
                msgs.push(VNuMessage::error(
                    format!(
                        "A table row was {} columns wide, which is less than the column count established using column markup ({}).",
                        w, markup
                    ),
                    line,
                    col,
                ));
            }
        } else if let Some(first_w) = active.first_row_columns {
            if w > first_w {
                msgs.push(VNuMessage::warning(
                    format!(
                        "A table row was {} columns wide and exceeded the column count established by the first row ({}).",
                        w, first_w
                    ),
                    line,
                    col,
                ));
            } else if w < first_w {
                msgs.push(VNuMessage::warning(
                    format!(
                        "A table row was {} columns wide, which is less than the column count established by the first row ({}).",
                        w, first_w
                    ),
                    line,
                    col,
                ));
            }
        } else if w > 0 {
            active.first_row_columns = Some(w);
        }
    }

    pub fn exit_tbody(&mut self, line: usize, col: usize, msgs: &mut Vec<VNuMessage>) {
        if let Some(active) = self.table_stack.last() {
            if active.max_row_span_in_group > active.row_group_row_count {
                msgs.push(VNuMessage::error(
                    "Table cell spans past the end of its row group established by a “tbody” element; clipped to the end of the row group.",
                    line,
                    col,
                ));
            }
        }
    }

    pub fn exit_table(&mut self, line: usize, col: usize, msgs: &mut Vec<VNuMessage>) {
        let Some(finished) = self.table_stack.pop() else {
            return;
        };

        let max_c = finished.max_column_reached;
        let mut empty_start = None;
        for c in 1..=max_c {
            if !finished.columns_with_starting_cells.contains(&c) {
                if empty_start.is_none() {
                    empty_start = Some(c);
                }
            } else if let Some(start) = empty_start.take() {
                if start == c - 1 {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Table column {} established by element “td” has no cells beginning in it.",
                            start
                        ),
                        line,
                        col,
                    ));
                } else {
                    msgs.push(VNuMessage::error(
                        format!(
                            "Table columns in range {}…{} established by element “td” have no cells beginning in them.",
                            start,
                            c - 1
                        ),
                        line,
                        col,
                    ));
                }
            }
        }
        if let Some(start) = empty_start {
            if start == max_c {
                msgs.push(VNuMessage::error(
                    format!(
                        "Table column {} established by element “td” has no cells beginning in it.",
                        start
                    ),
                    line,
                    col,
                ));
            } else {
                msgs.push(VNuMessage::error(
                    format!(
                        "Table columns in range {}…{} established by element “td” have no cells beginning in them.",
                        start, max_c
                    ),
                    line,
                    col,
                ));
            }
        }

        self.finished_table_th_ids
            .insert(finished.table_index, finished.th_ids);
    }
}
