use std::io::{stdout, Result, Stdout};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Offset, Position},
    widgets::ScrollbarState,
    Frame, Terminal,
};

use crate::{
    spreadsheet::{Spreadsheet, SpreadsheetCell, SPREADSHEET_MAX_COLS, SPREADSHEET_MAX_ROWS},
    ui::{infinite_table::infinite_table, text_input::TextInput},
};

pub type TUI = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<TUI> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum AppArea {
    #[default]
    Data,
    Editor,
    Menu,
    CommandBar,
}

#[derive(Debug, Default, Clone)]
pub struct App {
    pub spreadsheet: Spreadsheet,
    pub active_cell: SpreadsheetCell,
    pub focused_area: AppArea,

    pub editor: TextInput,

    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub vertical_scroll_pos: usize,
    pub horizontal_scroll_pos: usize,

    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut TUI) -> Result<()> {
        while !self.exit {
            terminal.draw(|f| self.render_frame(f))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        if self.focused_area == AppArea::Editor {
            frame.set_cursor_position(Position {
                x: self.editor.cursor() as u16,
                y: 0,
            });
        } else {
            self.editor
                .set_value(self.spreadsheet.get_cell(&self.active_cell).to_string());
        }

        frame.render_widget(self.editor.render(), frame.area());
        frame.render_widget(
            infinite_table(&self.spreadsheet, &self.active_cell, &self.focused_area),
            frame.area().offset(Offset { x: 0, y: 1 }),
        );
    }

    fn handle_events(&mut self) -> Result<()> {
        // todo!()
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_keypress(key_event);
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_keypress(&mut self, key_event: KeyEvent) {
        self.handle_global_keypress(key_event);
        match self.focused_area {
            AppArea::Data => self.handle_data_keypress(key_event),
            AppArea::Editor => self.handle_editor_keypress(key_event),
            AppArea::Menu => (),
            AppArea::CommandBar => (),
        }
    }

    fn handle_global_keypress(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit = true
            }
            KeyCode::Delete => {
                print!(
                    "                              {:?}                ",
                    self.spreadsheet.col_widths
                );
            }
            _ => (),
        }
    }

    fn handle_data_keypress(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // Cell movement
            KeyCode::Right => {
                if self.active_cell.col < SPREADSHEET_MAX_COLS {
                    self.active_cell.col += 1
                }
            }
            KeyCode::Left => {
                if self.active_cell.col > 0 {
                    self.active_cell.col -= 1
                }
            }
            KeyCode::Down => {
                if self.active_cell.row < SPREADSHEET_MAX_ROWS {
                    self.active_cell.row += 1
                }
            }
            KeyCode::Up => {
                if self.active_cell.row > 0 {
                    self.active_cell.row -= 1
                }
            }

            // Movement (enter/tab)
            // TODO: Add the feature where tab and enter go to the start of the next thing, like excel
            KeyCode::Enter => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) && self.active_cell.row > 0 {
                    self.active_cell.row -= 1
                } else if self.active_cell.row < SPREADSHEET_MAX_ROWS {
                    self.active_cell.row += 1
                }
            }
            KeyCode::Tab => {
                if self.active_cell.col < SPREADSHEET_MAX_COLS {
                    self.active_cell.col += 1
                }
            }
            KeyCode::BackTab => {
                if self.active_cell.col > 0 {
                    self.active_cell.col -= 1
                }
            }

            // Resizing (temporary)
            KeyCode::Char('+') => {
                self.spreadsheet.set_col_width(
                    &self.active_cell,
                    self.spreadsheet.get_col_width(&self.active_cell) + 1,
                );
            }
            KeyCode::Char('-') => {
                self.spreadsheet.set_col_width(
                    &self.active_cell,
                    self.spreadsheet.get_col_width(&self.active_cell) - 1,
                );
            }

            // Editing
            KeyCode::F(2) => {
                self.focused_area = AppArea::Editor;
                self.editor.set_cursor(self.editor.value().len());
            }
            KeyCode::Char(c) => {
                self.focused_area = AppArea::Editor;
                self.editor.set_value(c.to_string());
                self.editor.set_cursor(self.editor.value().len());
            }
            _ => (),
        }
    }

    fn handle_editor_keypress(&mut self, key_event: KeyEvent) {
        self.editor.handle_event(&Event::Key(key_event));
        match key_event.code {
            KeyCode::Enter => {
                self.focused_area = AppArea::Data;
                self.spreadsheet
                    .set_cell(&self.active_cell, &self.editor.value());

                if self.spreadsheet.get_col_width(&self.active_cell)
                    < self.editor.value().len() as u16
                {
                    self.spreadsheet
                        .set_col_width(&self.active_cell, self.editor.value().len() as u16);
                }

                if key_event.modifiers.contains(KeyModifiers::SHIFT) && self.active_cell.row > 0 {
                    self.active_cell.row -= 1
                } else if self.active_cell.row < SPREADSHEET_MAX_ROWS {
                    self.active_cell.row += 1
                }
            }
            KeyCode::Esc => self.focused_area = AppArea::Data,
            _ => (),
        }
    }
}
