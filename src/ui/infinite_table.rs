use std::cmp::min;

use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    widgets::{Cell, Row, Table},
};

use crate::{
    app::{App, AppArea},
    formulas,
    spreadsheet::{self, Spreadsheet, SpreadsheetCell, SPREADSHEET_MAX_COLS},
};

fn left_pad(string: String, length: usize, pad_char: char) -> String {
    if string.len() >= length {
        return string;
    }
    let mut working = string.clone();
    while working.len() < length {
        working.insert(0, pad_char);
    }
    working
}

fn right_pad(string: String, length: usize, pad_char: char) -> String {
    if string.len() >= length {
        return string;
    }
    let mut working = string.clone();
    while working.len() < length {
        working.push(pad_char);
    }
    working
}

fn render_cell(
    cell: &SpreadsheetCell,
    max_length: usize,
    decimals: u32,
    spreadsheet: &Spreadsheet,
) -> String {
    let mut cell_text = spreadsheet.get_cell(cell).to_string();
    let mut rendered: String;
    if cell_text.starts_with("=") {
        if let Ok(cell_value) = spreadsheet.get_cell_value(cell) {
            cell_text = cell_value.content
        }
    }

    if let Ok(number) = cell_text.parse::<f32>() {
        let rounding_scalar = f32::powf(10f32, (decimals) as f32);
        rendered = ((number * rounding_scalar).round() / rounding_scalar).to_string();

        if let Some(rounded_decimals) = rendered.split_once(".") {
            for _ in 0..(decimals as usize - rounded_decimals.1.len()) {
                rendered.push('0');
            }
        } else {
            rendered.push('.');
            for _ in 0..decimals {
                rendered.push('0');
            }
        }

        rendered = left_pad(rendered, max_length, ' ');
    } else {
        rendered = cell_text.to_string();
    }

    // Shouldn't ever fail, but if it does, just return an empty string
    right_pad(
        rendered
            .get(0..min(max_length, rendered.len()))
            .unwrap_or("")
            .to_string(),
        max_length,
        ' ',
    )
}

pub fn infinite_table<'a>(
    spreadsheet: &'a mut Spreadsheet,
    active_cell: &SpreadsheetCell,
    focused_area: &AppArea,
) -> Table<'a> {
    spreadsheet.resize_to_cell(active_cell); // TODO: Remove this once selecting and quick cell jumping added
    let rows_map = &spreadsheet.iter_rows().enumerate().map(|(y, r)| {
        let c: Vec<Cell> = r
            .contents
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                let rendered = render_cell(
                    &SpreadsheetCell { col: idx, row: y },
                    spreadsheet.get_col_width(&SpreadsheetCell { col: idx, row: y }) as usize,
                    2,
                    &spreadsheet,
                );
                if idx == active_cell.col && y == active_cell.row {
                    if *focused_area == AppArea::Data {
                        return Cell::new(rendered.black().on_gray());
                    } else {
                        return Cell::new(rendered.on_dark_gray());
                    }
                }
                Cell::new(rendered)
            })
            .collect();

        Row::new(c)
    });
    let rows: Vec<Row> = rows_map.clone().collect();

    // TODO: Once scrolling is implemented, filter this to only return those in the range...
    // also do that to the above statement.
    let widths: Vec<Constraint> = vec![0; /*SPREADSHEET_MAX_COLS*/ 10 /* (me saving memory) */]
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            Constraint::Length(spreadsheet.get_col_width(&SpreadsheetCell { col: idx, row: 0 }))
        })
        .collect();
    // println!("{:?}", widths);

    return Table::new(rows, widths)
        .column_spacing(1)
        .style(Style::new().on_black().white());
}
