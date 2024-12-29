use std::{cmp::min, collections::HashMap};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, MouseEventKind},
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Cell, Row, StatefulWidget, Table},
};

use crate::{
    app::AppArea,
    references::Reference,
    spreadsheet::{Spreadsheet, SpreadsheetCell, SPREADSHEET_MAX_COLS},
    utils::StringPadding,
};

fn render_cell(
    cell: &SpreadsheetCell,
    max_length: usize,
    decimals: u32,
    spreadsheet: &Spreadsheet,
    formula_cache: &mut HashMap<SpreadsheetCell, String>,
) -> String {
    let mut cell_text = spreadsheet.get_cell(cell).to_string();
    let mut rendered: String;
    if cell_text.starts_with("=") {
        if let Some(cached_value) = formula_cache.get(cell) {
            cell_text = cached_value.clone();
        } else if let Ok(cell_value) = spreadsheet.get_cell_value(cell) {
            cell_text = cell_value.content;
            formula_cache.insert(cell.clone(), cell_text.clone());
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

        rendered = rendered.left_pad(max_length, ' ');
    } else {
        rendered = cell_text.to_string();
    }

    // Shouldn't ever fail, but if it does, just return an empty string
    rendered
        .get(0..min(max_length, rendered.len()))
        .unwrap_or("")
        .to_string()
        .right_pad(max_length, ' ')
}

pub struct InfiniteTable {
    pub is_focused: bool,
    pub col_widths: Vec<u16>,
    pub col_space: u16,
}

#[derive(Debug, Default, Clone)]
pub struct InfiniteTableState {
    pub active_cell: SpreadsheetCell,
    vertical_scroll: u16,
    horizontal_scroll: u16,
}

impl StatefulWidget for InfiniteTable {
    type State = InfiniteTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        // NOTE TO SELF: There is very likely an issue where this will render into other cells that it shouldn't.
        // This will be addressed eventually.

        let row_header_width = 3;

        let mut render_x = 0;
        for col in 1..area.width {
            let col_width = self.col_widths[col as usize] as i16;
            // Max renderable cols is the terminal width
            let start_x = area.x as i16 + render_x as i16 - state.horizontal_scroll as i16;

            let text = Reference::index_to_alpha(col as u32)
                .to_string()
                .center(col_width as usize, ' ');

            if start_x >= -(text.len() as i16) {
                // Eventually, trim the text to fit it when it's only partially visible.
                if start_x > 0 {
                    buf.set_string(
                        start_x as u16 + row_header_width,
                        area.y,
                        text,
                        Style::new(),
                    );
                } else {
                    let sliced_text = text[start_x.unsigned_abs() as usize..].to_string();
                    buf.set_string(row_header_width, area.y, sliced_text, Style::new());
                }
            }
            render_x += col_width + self.col_space as i16;
        }

        // TODO: Row height, once implemented
        for row in 1..area.height {
            buf.set_string(
                area.x,
                area.y + row,
                (row + state.vertical_scroll)
                    .to_string()
                    .center(row_header_width as usize, ' '),
                Style::new(),
            );
        }

        buf.set_string(
            area.x + 3,
            area.y + 1,
            format!(
                "Widths: {}, V Scroll: {}, H Scroll: {}",
                self.col_widths.len(),
                state.vertical_scroll,
                state.horizontal_scroll
            ),
            Style::new(),
        );
    }
}

impl InfiniteTableState {
    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::ScrollDown => {
                    self.vertical_scroll += 1;
                }
                MouseEventKind::ScrollUp => {
                    if self.vertical_scroll >= 1 {
                        self.vertical_scroll -= 1;
                    }
                }
                MouseEventKind::ScrollLeft => {
                    self.horizontal_scroll += 1;
                }
                MouseEventKind::ScrollRight => {
                    if self.horizontal_scroll >= 1 {
                        self.horizontal_scroll -= 1;
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

pub fn infinite_table<'a>(
    spreadsheet: &'a mut Spreadsheet,
    active_cell: &SpreadsheetCell,
    focused_area: &AppArea,
    formula_cache: &'a mut HashMap<SpreadsheetCell, String>,
) -> Table<'a> {
    spreadsheet.resize_to_cell(active_cell); // TODO: Remove this once selecting and quick cell jumping added
    let mut rows: Vec<Row> = spreadsheet
        .iter_rows()
        .enumerate()
        .map(|(y, r)| {
            let mut c: Vec<Cell> = r
                .contents
                .iter()
                .enumerate()
                .map(|(idx, _)| {
                    let cell = SpreadsheetCell { col: idx, row: y };
                    let rendered: String;
                    rendered = render_cell(
                        &cell,
                        spreadsheet.get_col_width(&cell) as usize,
                        2,
                        &spreadsheet,
                        formula_cache,
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

            // TODO: Same fixed value that needs to change
            let row_marker = (y + 1).to_string().center(3, ' ');
            c.insert(0, Cell::new(row_marker.black().on_white()));
            Row::new(c)
        })
        .collect();

    // Still me saving memory. Again, scrolling, yada yada, all that jazz.
    rows.insert(
        0,
        Row::new(vec![0; 5].iter().enumerate().map(|(idx, _)| {
            if idx == 0 {
                String::new().reset()
            } else {
                let length = spreadsheet.get_col_width(&SpreadsheetCell {
                    col: idx - 1,
                    row: 0,
                }) as usize;
                // Reference::index_to_alpha(idx as u32)
                length.to_string().center(length, ' ').black().on_white()
            }
        })),
    );

    // TODO: Once scrolling is implemented, filter this to only return those in the range...
    // also do that to the above statement.
    let mut widths: Vec<Constraint> = vec![0; /*SPREADSHEET_MAX_COLS*/ 20 /* (me saving memory) */]
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            Constraint::Length(spreadsheet.get_col_width(&SpreadsheetCell { col: idx, row: 0 }))
        })
        .collect();
    widths.insert(0, Constraint::Length(3)); // TODO: Dynamically update this depending on what's visible

    return Table::new(rows, widths)
        .column_spacing(1)
        .style(Style::new().on_black().white());
}
