use std::cmp::{max, min};
use std::fmt::Display;
use std::io::{Error, ErrorKind};
use std::ops::Index;
use std::{cell, fs};

use strum::Display;

use crate::formulas::{cell_to_token, Token};
use crate::undo_stack::UndoStack;

#[derive(Debug)]
pub struct SpreadsheetRowIteratorItem {
    pub idx: usize,
    pub value: String,
}

#[derive(Debug, Default, Clone)]
pub struct SpreadsheetRow {
    pub row_idx: usize,
    pub contents: Vec<String>,
}

// Unused, but keeping it for future reference.
// impl SpreadsheetRow {
//     pub fn iter_contents(&'a self) -> impl Iterator<Item = SpreadsheetRowIteratorItem> + 'a {
//         self.contents.iter().enumerate().map(|(idx, value)| {
//             return SpreadsheetRowIteratorItem {
//                 idx,
//                 value: value.clone(),
//             };
//         })
//     }
// }

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct SpreadsheetCell {
    pub row: usize,
    pub col: usize,
}

pub const SPREADSHEET_MAX_ROWS: usize = 2usize.pow(20);
pub const SPREADSHEET_MAX_COLS: usize = 2usize.pow(14);
pub const DEFAULT_COL_WIDTH: u16 = 10;

#[derive(Debug, Clone)]
pub struct SpreadsheetEdit {
    cell: SpreadsheetCell,
    before: String,
    after: String,
}

impl PartialEq for SpreadsheetEdit {
    fn eq(&self, other: &Self) -> bool {
        self.after == other.after && self.cell == other.cell
    }
}

impl Display for SpreadsheetEdit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {} -> {}", self.cell, self.before, self.after)
    }
}

#[derive(Debug, Default)]
pub struct Spreadsheet {
    data: Vec<SpreadsheetRow>,
    pub col_widths: Vec<u16>,
    row_heights: Vec<u16>,
    pub undo_stack: UndoStack<Vec<SpreadsheetEdit>>,
}

impl Spreadsheet {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            col_widths: vec![DEFAULT_COL_WIDTH; SPREADSHEET_MAX_COLS],
            row_heights: Vec::new(),
            undo_stack: UndoStack::default(),
        }
    }

    // pub fn load_rows(&mut self, lower: i32, upper: i32) {}

    pub fn from_csv(path: &str) -> Result<Spreadsheet, Error> {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Err(Error::new(ErrorKind::NotFound, "File not found")),
        };
        let parsed: Vec<SpreadsheetRow> = contents
            .lines()
            .map(parse_csv_line)
            .enumerate()
            .map(|(idx, line)| SpreadsheetRow {
                row_idx: idx,
                contents: line.clone(),
            })
            .collect();

        let max_cols = parsed.iter().map(|r| r.contents.len()).max().unwrap_or(10);

        return Ok(Spreadsheet {
            data: parsed,
            col_widths: vec![DEFAULT_COL_WIDTH; SPREADSHEET_MAX_COLS],
            row_heights: Vec::new(),
            undo_stack: UndoStack::default(),
        });
    }

    // fn from_xls(path: &str) {
    //     todo!()
    // }

    // TODO: Give this a range parameter.
    pub fn iter_rows(&self) -> std::slice::Iter<'_, SpreadsheetRow> {
        self.data.iter().clone()
    }

    fn in_spreadsheet(&self, cell: &SpreadsheetCell) -> bool {
        cell.row < self.data.len()
            && cell.col < self.data[cell.row].contents.len()
            && self.col_widths.len() > cell.col
    }

    pub fn get_cell(&self, cell: &SpreadsheetCell) -> &str {
        if !self.in_spreadsheet(cell) {
            return "";
        }
        return self.data[cell.row].contents[cell.col].as_str();
    }

    fn internal_set_cell(&mut self, cell: &SpreadsheetCell, value: &str) {
        self.resize_to_cell(cell);
        self.data[cell.row].contents[cell.col] = value.to_string();
    }

    pub fn set_cell(&mut self, cell: &SpreadsheetCell, value: &str) {
        self.undo_stack.edit(vec![SpreadsheetEdit {
            cell: cell.clone(),
            before: if self.in_spreadsheet(cell) {
                self.data[cell.row].contents[cell.col].clone()
            } else {
                String::new()
            },
            after: value.to_string(),
        }]);
        self.internal_set_cell(cell, value);
    }

    pub fn undo(&mut self) -> Option<[SpreadsheetCell; 2]> {
        if let Some(edits) = self.undo_stack.undo() {
            let min_row = edits.iter().min_by_key(|c| c.cell.row).unwrap().cell.row;
            let max_row = edits.iter().max_by_key(|c| c.cell.row).unwrap().cell.row;
            let min_col = edits.iter().min_by_key(|c| c.cell.col).unwrap().cell.col;
            let max_col = edits.iter().max_by_key(|c| c.cell.col).unwrap().cell.col;
            for edit in edits.iter() {
                self.internal_set_cell(&edit.cell, &edit.before);
            }
            return Some([
                SpreadsheetCell {
                    row: min_row,
                    col: min_col,
                },
                SpreadsheetCell {
                    row: max_row,
                    col: max_col,
                },
            ]);
        }
        None
    }

    pub fn redo(&mut self) -> Option<[SpreadsheetCell; 2]> {
        if let Some(edits) = self.undo_stack.redo() {
            let min_row = edits.iter().min_by_key(|c| c.cell.row).unwrap().cell.row;
            let max_row = edits.iter().max_by_key(|c| c.cell.row).unwrap().cell.row;
            let min_col = edits.iter().min_by_key(|c| c.cell.col).unwrap().cell.col;
            let max_col = edits.iter().max_by_key(|c| c.cell.col).unwrap().cell.col;

            for edit in edits.iter() {
                self.internal_set_cell(&edit.cell, &edit.after);
            }

            return Some([
                SpreadsheetCell {
                    row: min_row,
                    col: min_col,
                },
                SpreadsheetCell {
                    row: max_row,
                    col: max_col,
                },
            ]);
        }
        None
    }
    pub fn resize_to_cell(&mut self, cell: &SpreadsheetCell) {
        if cell.row >= self.data.len() {
            self.data.resize(cell.row + 1, SpreadsheetRow::default());
        }
        if cell.col >= self.data[cell.row].contents.len() {
            self.data[cell.row]
                .contents
                .resize(cell.col + 1, String::new());
        }
        if self.col_widths.len() <= cell.col {
            self.col_widths.resize(cell.col + 1, DEFAULT_COL_WIDTH);
        }
    }

    pub fn get_col_width(&self, cell: &SpreadsheetCell) -> u16 {
        if let Some(width) = self.col_widths.get(cell.col) {
            return *width;
        }
        return DEFAULT_COL_WIDTH;
    }

    pub fn set_col_width(&mut self, cell: &SpreadsheetCell, width: u16) {
        if self.col_widths.len() > cell.col {
            self.col_widths[cell.col] = width;
        }
    }

    // TODO: Make it a Vec<Token> once functions with multiple outputs are implemented
    pub fn get_cell_value(&self, cell: &SpreadsheetCell) -> Result<Token, ()> {
        return cell_to_token(self.get_cell(cell), self);
    }

    pub fn select_matrix(&self, a: &SpreadsheetCell, b: &SpreadsheetCell) -> Vec<Vec<String>> {
        let min_row = min(a.row, b.row);
        let min_col = min(a.col, b.col);
        let max_row = max(a.row, b.row);
        let max_col = max(a.col, b.col);

        let mut mat: Vec<Vec<String>> = Vec::new();

        for row in min_row..=max_row {
            let mut row_items: Vec<String> = Vec::new();
            for col in min_col..=max_col {
                row_items.push(
                    self.get_cell_value(&SpreadsheetCell { row, col })
                        .unwrap()
                        .content,
                );
            }
            mat.push(row_items);
        }
        mat
    }

    pub fn replace_matrix(&mut self, start: &SpreadsheetCell, mat: Vec<Vec<String>>) {
        let mut changes: Vec<SpreadsheetEdit> = Vec::new();
        for row in 0..mat.len() {
            for col in 0..mat[row].len() {
                let cell = SpreadsheetCell {
                    row: start.row + row,
                    col: start.col + col,
                };
                let value = mat[row][col].clone();
                changes.push(SpreadsheetEdit {
                    cell: cell.clone(),
                    before: self.get_cell(&cell).to_string(),
                    after: value.clone(),
                });
                self.internal_set_cell(&cell, &value);
            }
        }
        self.undo_stack.edit(changes);
    }
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut inside_quote = false;
    line.split(|c| {
        if c == '"' {
            inside_quote = !inside_quote;
        }
        c == ',' && !inside_quote
    })
    .map(|c| c.to_string())
    .collect()
}
