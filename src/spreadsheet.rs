use std::fs;
use std::io::{Error, ErrorKind};

use crate::formulas::{cell_to_token, Token};

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
//     pub fn iter_contents<'a>(&'a self) -> impl Iterator<Item = SpreadsheetRowIteratorItem> + 'a {
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

#[derive(Debug, Default, Clone)]
pub struct Spreadsheet {
    data: Vec<SpreadsheetRow>,
    pub col_widths: Vec<u16>,
    row_heights: Vec<u16>, // pub n_rows: usize,
                           // pub n_cols: usize,
}

impl Spreadsheet {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            col_widths: Vec::new(),
            row_heights: Vec::new(), // n_cols: 0,
                                     // n_rows: 0,
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

    pub fn set_cell(&mut self, cell: &SpreadsheetCell, value: &str) {
        self.resize_to_cell(cell);
        self.data[cell.row].contents[cell.col] = value.to_string();
    }

    pub fn resize_to_cell(&mut self, cell: &SpreadsheetCell) {
        if cell.row >= self.data.len() {
            self.data.resize(cell.row, SpreadsheetRow::default());
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
