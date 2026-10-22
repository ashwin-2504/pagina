use super::types::ImportedBlock;

#[derive(Default)]
pub struct TableParserState {
    pub in_tbl: bool,
    pub indent: u32,
    pub has_borders: bool,
    pub table_rows: Vec<Vec<Vec<String>>>,
    pub current_row: Vec<Vec<String>>,
    pub current_cell_paragraphs: Vec<String>,
}

impl TableParserState {
    pub fn start_table(&mut self) {
        self.in_tbl = true;
        self.table_rows.clear();
        self.indent = 0;
        self.has_borders = false;
    }

    pub fn start_row(&mut self) {
        self.current_row.clear();
    }

    pub fn start_cell(&mut self) {
        self.current_cell_paragraphs.clear();
    }

    pub fn push_cell_paragraph(&mut self, p: String) {
        self.current_cell_paragraphs.push(p);
    }

    pub fn end_cell(&mut self) {
        self.current_row
            .push(self.current_cell_paragraphs.clone());
        self.current_cell_paragraphs.clear();
    }

    pub fn end_row(&mut self) {
        self.table_rows.push(self.current_row.clone());
        self.current_row.clear();
    }

    pub fn check_border(&mut self, val: Option<&str>) {
        if self.in_tbl {
            if let Some(v) = val {
                let lower = v.to_lowercase();
                if lower != "none" && lower != "nil" && lower != "0" && !lower.is_empty() {
                    self.has_borders = true;
                }
            }
        }
    }

    pub fn end_table(&mut self) -> Option<ImportedBlock> {
        self.in_tbl = false;
        let is_borderless = !self.has_borders;
        let is_align_right = self.indent >= 4000;
        let is_single_row = self.table_rows.len() == 1;
        let non_empty_cells: Vec<Vec<String>> = if is_single_row {
            self.table_rows[0]
                .iter()
                .filter(|cell_ps| !cell_ps.is_empty())
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        if is_single_row
            && !is_borderless
            && non_empty_cells.len() >= 2
            && non_empty_cells.len() <= 3
        {
            Some(ImportedBlock::Grid(non_empty_cells))
        } else if !self.table_rows.is_empty() {
            Some(ImportedBlock::Table {
                rows: self.table_rows.clone(),
                is_borderless,
                is_align_right,
            })
        } else {
            None
        }
    }
}
