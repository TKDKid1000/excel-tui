use std::{cmp::min, collections::HashMap};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, MouseEventKind},
    layout::{Position, Rect},
    style::{Color, Style},
    widgets::StatefulWidget,
};

use crate::{
    references::Reference,
    spreadsheet::{Spreadsheet, SpreadsheetCell, SPREADSHEET_MAX_COLS, SPREADSHEET_MAX_ROWS},
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

pub struct InfiniteTable<'a> {
    pub is_focused: bool,
    pub col_widths: Vec<u16>,
    pub col_space: u16,
    pub spreadsheet: &'a Spreadsheet,
}

#[derive(Debug, Default, Clone)]
pub struct InfiniteTableState {
    pub active_cell: SpreadsheetCell,
    vertical_scroll: u32,
    horizontal_scroll: u16,
    pub formula_cache: HashMap<SpreadsheetCell, String>,

    visible_rows: [u32; 2],
    visible_cols: [u16; 2],
    cells: HashMap<SpreadsheetCell, Rect>,

    col_edges: [u16; 2],

    area: Rect,
}

impl<'a> InfiniteTable<'a> {
    fn render_headers(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut <InfiniteTable as StatefulWidget>::State,
        row_header_width: u16,
        row_header_gap: u16,
    ) where
        Self: Sized,
    {
        // NOTE TO SELF: There is very likely an issue where this will render into other cells that it shouldn't.
        // This will be addressed eventually.

        let mut render_x = 0;
        for col in 0..area.width {
            let col_width = self.col_widths[col as usize] as i16;
            // Max renderable cols is the terminal width
            let start_x = render_x as i16 - state.horizontal_scroll as i16;

            let text = Reference::index_to_alpha(col as u32 + 1)
                .to_string()
                .center(col_width as usize, ' ');

            if start_x >= -(text.len() as i16) {
                // Eventually, trim the text to fit it when it's only partially visible.
                if start_x > 0 {
                    buf.set_string(
                        start_x as u16 + row_header_width + row_header_gap + area.x,
                        area.y,
                        text,
                        Style::new(),
                    );
                } else {
                    let sliced_text = text[start_x.unsigned_abs() as usize..].to_string();
                    buf.set_string(
                        row_header_width + row_header_gap + area.x,
                        area.y,
                        sliced_text,
                        Style::new(),
                    );
                }
            }
            render_x += col_width + self.col_space as i16;
        }

        // TODO: Row height, once implemented
        for row in 1..area.height {
            buf.set_string(
                area.x,
                area.y + row,
                (row as u32 + state.vertical_scroll)
                    .to_string()
                    .center(row_header_width as usize, ' '),
                Style::new(),
            );
        }
    }

    fn render_data(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut <InfiniteTable as StatefulWidget>::State,
    ) where
        Self: Sized,
    {
        state.visible_rows = [
            state.vertical_scroll as u32,
            state.vertical_scroll + area.height as u32,
        ];
        state.visible_cols = [0, 0];
        state.cells.clear();

        // NOTE TO SELF: There is very likely an issue where this will render into other cells that it shouldn't.
        // This will be addressed eventually.

        // TODO: Row height, once implemented
        for row in 0..area.height {
            let mut render_x = 0;
            for col in 0..area.width {
                let col_width = self.col_widths[col as usize] as i16;
                // Max renderable cols is the terminal width
                let start_x = render_x as i16 - state.horizontal_scroll as i16;

                let cell = SpreadsheetCell {
                    row: (row as u32 + state.vertical_scroll) as usize,
                    col: col.into(),
                };
                let text = render_cell(
                    &cell,
                    col_width as usize,
                    2,
                    &self.spreadsheet,
                    &mut state.formula_cache,
                );

                let mut cell_style = Style::new();
                if state.active_cell == cell {
                    cell_style = cell_style.bg(Color::White).fg(Color::Black);
                    if !self.is_focused {
                        cell_style = cell_style.bg(Color::Gray);
                    }
                }

                if start_x > area.width as i16 {
                    state.visible_cols[1] = col - 1;
                    break;
                }

                if start_x as i16 >= -(text.len() as i16) {
                    // Eventually, trim the text to fit it when it's only partially visible.
                    if start_x as i16 > 0 {
                        buf.set_string(start_x as u16 + area.x, area.y + row, text, cell_style);
                        state.cells.insert(
                            cell,
                            Rect {
                                x: start_x as u16 + area.x,
                                y: area.y + row,
                                width: col_width as u16,
                                height: 1, // TODO: Row heights, once again
                            },
                        );
                    } else {
                        state.visible_cols[0] = col;
                        let sliced_text = text[start_x.unsigned_abs() as usize..].to_string();
                        buf.set_string(area.x, area.y + row, sliced_text, cell_style);
                        state.cells.insert(
                            cell,
                            Rect {
                                x: area.x,
                                y: area.y + row,
                                width: col_width as u16,
                                height: 1, // TODO: Row heights, once again
                            },
                        );
                    }
                }
                render_x += col_width + self.col_space as i16;
            }
        }

        state.col_edges = [
            if state.visible_cols[0] == 0 {
                0
            } else {
                self.col_widths[1..state.visible_cols[0] as usize]
                    .iter()
                    .map(|c| c + self.col_space)
                    .sum::<u16>()
            },
            self.col_widths[..=state.visible_cols[1] as usize + 1]
                .iter()
                .map(|c| c + self.col_space)
                .sum::<u16>()
                - area.width
                - self.col_space,
        ];

        state.area = area;
    }
}

impl<'a> StatefulWidget for InfiniteTable<'a> {
    type State = InfiniteTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let row_header_width = 3;
        let row_header_gap = 1;

        self.render_data(
            Rect {
                x: area.x + row_header_width + row_header_gap,
                y: area.y + 1, // Constant, height of col header
                width: area.width - row_header_width - row_header_gap,
                height: area.height - 1, // Constant, height of col header
            },
            buf,
            state,
        );
        self.render_headers(area, buf, state, row_header_width, row_header_gap);
    }
}

impl InfiniteTableState {
    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Mouse(mouse_event)
                if self.area.contains(Position {
                    x: mouse_event.column,
                    y: mouse_event.row,
                }) =>
            {
                match mouse_event.kind {
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
                    MouseEventKind::Down(_) => {
                        // TODO: Handle other mouse buttons (certainly needed here)

                        for (cell, rect) in self.cells.iter() {
                            if rect.contains(Position {
                                x: mouse_event.column,
                                y: mouse_event.row,
                            }) {
                                self.active_cell = cell.clone();
                            }
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    pub fn move_active_cell(&mut self, x: i32, y: i32) {
        let mut dx = x;
        while dx > 0 && self.active_cell.col < SPREADSHEET_MAX_COLS {
            self.active_cell.col += 1;
            dx -= 1;
            if self.visible_cols[1] < self.active_cell.col as u16 {
                self.horizontal_scroll = self.col_edges[1];
            }
        }
        while dx < 0 && self.active_cell.col > 0 {
            self.active_cell.col -= 1;
            dx += 1;
            if self.visible_cols[0] > self.active_cell.col as u16 {
                self.horizontal_scroll = self.col_edges[0];
            }
        }

        let mut dy = y;
        while dy > 0 && self.active_cell.row < SPREADSHEET_MAX_ROWS {
            self.active_cell.row += 1;
            dy -= 1;
            if self.visible_rows[1] <= self.active_cell.row as u32 {
                // TODO: Scroll by row height, once implemented.
                self.vertical_scroll += 1;
            }
        }
        while dy < 0 && self.active_cell.row > 0 {
            self.active_cell.row -= 1;
            dy += 1;
            if self.visible_rows[0] > self.active_cell.row as u32 {
                // TODO: Scroll by row height, once implemented.
                self.vertical_scroll -= 1;
            }
        }
    }
}
