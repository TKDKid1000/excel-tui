use std::{
    collections::HashMap,
    io::{stdout, Result, Stdout},
};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
            KeyModifiers,
        },
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout, Position},
    widgets::ScrollbarState,
    Frame, Terminal,
};

use crate::{
    spreadsheet::{Spreadsheet, SpreadsheetCell, SPREADSHEET_MAX_COLS, SPREADSHEET_MAX_ROWS},
    ui::{
        infinite_table::{infinite_table, InfiniteTable, InfiniteTableState},
        text_input::{TextInput, TextInputState},
    },
};

pub type TUI = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<TUI> {
    execute!(stdout(), EnterAlternateScreen)?;
    execute!(stdout(), EnableMouseCapture)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    execute!(stdout(), DisableMouseCapture)?;
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
    pub focused_area: AppArea,
    pub formula_cache: HashMap<SpreadsheetCell, String>,

    pub editor_state: TextInputState,
    pub infinite_table_state: InfiniteTableState,

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
                x: self.editor_state.cursor() as u16,
                y: 0,
            });
        } else {
            self.editor_state.set_value(
                self.spreadsheet
                    .get_cell(&self.infinite_table_state.active_cell)
                    .to_string(),
            );
            // Needed so that cursor position doesn't persist and show text selection when unfocused.
            self.editor_state.set_cursor(0);
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Fill(1)])
            .split(frame.area());

        frame.render_stateful_widget(TextInput::default(), main_layout[0], &mut self.editor_state);
        // frame.render_widget(
        //     infinite_table(
        //         &mut self.spreadsheet,
        //         &self.infinite_table_state.active_cell,
        //         &self.focused_area,
        //         &mut self.formula_cache,
        //     ),
        //     main_layout[1],
        // );
        frame.render_stateful_widget(
            InfiniteTable {
                is_focused: self.focused_area == AppArea::Data,
                col_widths: self.spreadsheet.col_widths.clone(),
                col_space: 1
            },
            main_layout[1],
            &mut self.infinite_table_state,
        );
    }

    fn handle_events(&mut self) -> Result<()> {
        let event = event::read()?;
        self.handle_global_event(&event);
        match self.focused_area {
            AppArea::Data => self.handle_data_event(&event),
            AppArea::Editor => self.handle_editor_event(&event),
            AppArea::Menu => (),
            AppArea::CommandBar => (),
        }
        Ok(())
    }

    fn handle_global_event(&mut self, event: &Event) {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.exit = true
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn handle_data_event(&mut self, event: &Event) {
        self.infinite_table_state.handle_event(event);
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    // Cell movement
                    KeyCode::Right => {
                        if self.infinite_table_state.active_cell.col < SPREADSHEET_MAX_COLS {
                            self.infinite_table_state.active_cell.col += 1
                        }
                    }
                    KeyCode::Left => {
                        if self.infinite_table_state.active_cell.col > 0 {
                            self.infinite_table_state.active_cell.col -= 1
                        }
                    }
                    KeyCode::Down => {
                        if self.infinite_table_state.active_cell.row < SPREADSHEET_MAX_ROWS {
                            self.infinite_table_state.active_cell.row += 1
                        }
                    }
                    KeyCode::Up => {
                        if self.infinite_table_state.active_cell.row > 0 {
                            self.infinite_table_state.active_cell.row -= 1
                        }
                    }

                    // Movement (enter/tab)
                    // TODO: Add the feature where tab and enter go to the start of the next thing, like excel
                    KeyCode::Enter => {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT)
                            && self.infinite_table_state.active_cell.row > 0
                        {
                            self.infinite_table_state.active_cell.row -= 1
                        } else if self.infinite_table_state.active_cell.row < SPREADSHEET_MAX_ROWS {
                            self.infinite_table_state.active_cell.row += 1
                        }
                    }
                    KeyCode::Tab => {
                        if self.infinite_table_state.active_cell.col < SPREADSHEET_MAX_COLS {
                            self.infinite_table_state.active_cell.col += 1
                        }
                    }
                    KeyCode::BackTab => {
                        if self.infinite_table_state.active_cell.col > 0 {
                            self.infinite_table_state.active_cell.col -= 1
                        }
                    }

                    // Resizing (temporary)
                    KeyCode::Char('+') => {
                        self.spreadsheet.set_col_width(
                            &self.infinite_table_state.active_cell,
                            self.spreadsheet
                                .get_col_width(&self.infinite_table_state.active_cell)
                                + 1,
                        );
                    }
                    KeyCode::Char('-') => {
                        self.spreadsheet.set_col_width(
                            &self.infinite_table_state.active_cell,
                            self.spreadsheet
                                .get_col_width(&self.infinite_table_state.active_cell)
                                - 1,
                        );
                    }

                    // Editing
                    KeyCode::F(2) => {
                        self.focused_area = AppArea::Editor;
                        self.editor_state
                            .set_cursor(self.editor_state.value().len());
                    }
                    KeyCode::Char(c) => {
                        self.focused_area = AppArea::Editor;
                        self.editor_state.set_value(c.to_string());
                        self.editor_state
                            .set_cursor(self.editor_state.value().len());
                    }
                    KeyCode::Backspace | KeyCode::Delete => {
                        self.spreadsheet
                            .set_cell(&self.infinite_table_state.active_cell, "");
                        self.formula_cache.clear();
                    }

                    // Miscellanous
                    KeyCode::F(9) => {
                        self.formula_cache.clear();
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    fn handle_editor_event(&mut self, event: &Event) {
        self.editor_state.handle_event(&event);
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Enter => {
                    self.focused_area = AppArea::Data;
                    self.spreadsheet.set_cell(
                        &self.infinite_table_state.active_cell,
                        &self.editor_state.value(),
                    );
                    self.formula_cache.clear();

                    if self
                        .spreadsheet
                        .get_col_width(&self.infinite_table_state.active_cell)
                        < self.editor_state.value().len() as u16
                    {
                        self.spreadsheet.set_col_width(
                            &self.infinite_table_state.active_cell,
                            self.editor_state.value().len() as u16,
                        );
                    }

                    if key_event.modifiers.contains(KeyModifiers::SHIFT)
                        && self.infinite_table_state.active_cell.row > 0
                    {
                        self.infinite_table_state.active_cell.row -= 1
                    } else if self.infinite_table_state.active_cell.row < SPREADSHEET_MAX_ROWS {
                        self.infinite_table_state.active_cell.row += 1
                    }
                }
                KeyCode::Esc => self.focused_area = AppArea::Data,
                _ => (),
            },

            _ => (),
        }
    }
}
