use std::io::{stdout, Result, Stdout};

use copypasta::{ClipboardContext, ClipboardProvider};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{
            self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste,
            EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
            KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
        },
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout, Position, Rect},
    text::Line,
    widgets::Paragraph,
    Frame, Terminal,
};

use crate::{
    config::Config,
    formulas::{balance_parens, extract_references},
    spreadsheet::{Spreadsheet, SPREADSHEET_MAX_COLS, SPREADSHEET_MAX_ROWS},
    ui::{
        button::{Button, ButtonState},
        formula_suggestions::{FormulaSuggestions, FormulaSuggestionsState},
        infinite_table::{InfiniteTable, InfiniteTableState},
        text_input::{TextInput, TextInputState},
    },
    undo_stack,
};

pub type TUI = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<TUI> {
    execute!(stdout(), EnterAlternateScreen)?;
    execute!(stdout(), EnableMouseCapture)?;
    execute!(
        stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    )?;
    execute!(stdout(), EnableBracketedPaste)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    execute!(stdout(), DisableMouseCapture)?;
    execute!(stdout(), PopKeyboardEnhancementFlags)?;
    execute!(stdout(), DisableBracketedPaste)?;
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

#[derive(Debug)]
pub struct App {
    pub spreadsheet: Spreadsheet,
    pub focused_area: AppArea,

    pub formula_editor_state: TextInputState,
    pub infinite_table_state: InfiniteTableState,
    pub formula_suggestions_state: FormulaSuggestionsState,
    pub paste_button_state: ButtonState,

    pub config: Config,

    exit: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        App {
            spreadsheet: Spreadsheet::default(),
            focused_area: AppArea::default(),

            formula_editor_state: TextInputState::default(),
            infinite_table_state: InfiniteTableState::default(),
            formula_suggestions_state: FormulaSuggestionsState::default(),
            paste_button_state: ButtonState::default(),

            config,

            exit: false,
        }
    }

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
                x: self.formula_editor_state.cursor() as u16,
                y: 0,
            });
        } else {
            self.formula_editor_state.set_value(
                self.spreadsheet
                    .get_cell(&self.infinite_table_state.active_cell)
                    .to_string(),
            );
            // Needed so that cursor position doesn't persist and show text selection when unfocused.
            self.formula_editor_state.set_cursor(0);
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

        frame.render_stateful_widget(
            InfiniteTable {
                is_focused: self.focused_area == AppArea::Data,
                col_widths: self.spreadsheet.col_widths.clone(),
                col_space: 1,
                spreadsheet: &self.spreadsheet,
                highlights: if self.focused_area == AppArea::Editor
                    && self.formula_editor_state.value().starts_with("=")
                {
                    if let Ok(refs) = extract_references(&self.formula_editor_state.value()) {
                        vec![refs]
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }, // TODO: Add something that parses the active formula (if one) and then
                   // returns an array of [SpreadsheetCell; 2]
            },
            main_layout[1],
            &mut self.infinite_table_state,
        );
        frame.render_stateful_widget(
            TextInput::default(),
            main_layout[0],
            &mut self.formula_editor_state,
        );

        frame.render_widget(
            Paragraph::new(format!("Undo: {}", self.spreadsheet.undo_stack)),
            main_layout[2],
        );

        self.formula_suggestions_state.text_input_state = self.formula_editor_state.clone();
        frame.render_stateful_widget(
            FormulaSuggestions::default(),
            frame.area(),
            &mut self.formula_suggestions_state,
        );

        frame.render_stateful_widget(
            Button {
                text: String::from(if self.config.nerd_font {
                    "  "
                } else {
                    "txt"
                }),
            },
            Rect {
                x: 10,
                y: 10,
                width: 5,
                height: 3,
            },
            &mut ButtonState::default(),
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
        self.paste_button_state.handle_event(event);
        if self.paste_button_state.is_pressed {
            // TODO: self.
        }

        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    // Cell movement
                    KeyCode::Right => {
                        self.infinite_table_state.move_active_cell(
                            1,
                            0,
                            key_event.modifiers.contains(KeyModifiers::SHIFT),
                        );
                    }
                    KeyCode::Left => {
                        self.infinite_table_state.move_active_cell(
                            -1,
                            0,
                            key_event.modifiers.contains(KeyModifiers::SHIFT),
                        );
                    }
                    KeyCode::Down => {
                        self.infinite_table_state.move_active_cell(
                            0,
                            1,
                            key_event.modifiers.contains(KeyModifiers::SHIFT),
                        );
                    }
                    KeyCode::Up => {
                        self.infinite_table_state.move_active_cell(
                            0,
                            -1,
                            key_event.modifiers.contains(KeyModifiers::SHIFT),
                        );
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

                    // Undo/Redo
                    KeyCode::Char('z')
                        if key_event.modifiers.contains(KeyModifiers::SUPER)
                            && key_event.modifiers.contains(KeyModifiers::SHIFT) =>
                    {
                        if let Some([sel_start, sel_end]) = self.spreadsheet.redo() {
                            self.infinite_table_state.active_cell = sel_start;
                            self.infinite_table_state.selection_end = sel_end;
                            self.infinite_table_state.formula_cache.clear();
                        }
                    }
                    KeyCode::Char('z') if key_event.modifiers.contains(KeyModifiers::SUPER) => {
                        if let Some([sel_start, sel_end]) = self.spreadsheet.undo() {
                            self.infinite_table_state.active_cell = sel_start;
                            self.infinite_table_state.selection_end = sel_end;
                            self.infinite_table_state.formula_cache.clear();
                        }
                    }

                    // Copy/Paste
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        // TODO: Once selections are added, this needs multiple changes.
                        // TODO: Copying and pasting of formulas, not just their results.
                        let text = self
                            .spreadsheet
                            .select_matrix(
                                &self.infinite_table_state.active_cell,
                                &self.infinite_table_state.selection_end,
                            )
                            .iter()
                            .map(|r| r.join("\t"))
                            .collect::<Vec<String>>()
                            .join("\n");

                        let mut clipboard = ClipboardContext::new().unwrap();
                        clipboard.set_contents(text).unwrap();
                    }
                    KeyCode::Char('v') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        // TODO: Once selections are added, this needs multiple changes.
                        // TODO: Copying and pasting of formulas, not just their results.

                        let mut clipboard = ClipboardContext::new().unwrap();

                        if let Ok(text) = clipboard.get_contents() {
                            let mut mat: Vec<Vec<String>> = text
                                .to_string()
                                .split("\n")
                                .map(|r| r.split("\t").map(|c| c.to_string()).collect())
                                .collect();
                            let selection = self.infinite_table_state.selection();
                            if mat.len() == 1 && mat[0].len() == 1 {
                                // Handle the case where there is a single item in clipboard, where
                                // it must be pasted to every cell in the selection.
                                let rows = selection[1].row - selection[0].row + 1;
                                let cols = selection[1].col - selection[0].col + 1;
                                let value = mat[0][0].clone();
                                mat = vec![vec![value; cols]; rows];
                            }
                            self.spreadsheet.replace_matrix(&selection[0], mat);
                        }

                        self.infinite_table_state.formula_cache.clear()
                    }

                    // Editing
                    KeyCode::F(2) => {
                        self.focused_area = AppArea::Editor;
                        self.formula_editor_state
                            .set_cursor(self.formula_editor_state.value().len());
                    }
                    KeyCode::Char(c) => {
                        self.focused_area = AppArea::Editor;
                        self.formula_editor_state.set_value(c.to_string());
                        self.formula_editor_state
                            .set_cursor(self.formula_editor_state.value().len());
                    }
                    KeyCode::Backspace | KeyCode::Delete => {
                        let selection = self.infinite_table_state.selection();
                        let rows = selection[1].row - selection[0].row + 1;
                        let cols = selection[1].col - selection[0].col + 1;
                        let mat = vec![vec![String::new(); cols]; rows];
                        self.spreadsheet.replace_matrix(&selection[0], mat);

                        self.infinite_table_state.formula_cache.clear();
                    }

                    // Miscellanous
                    KeyCode::F(9) => {
                        self.infinite_table_state.formula_cache.clear();
                    }
                    _ => (),
                }
            }
            Event::Paste(text) => {
                if !text.is_empty() {
                    let mut mat: Vec<Vec<String>> = text
                        .to_string()
                        .split("\n")
                        .map(|r| r.split("\t").map(|c| c.to_string()).collect())
                        .collect();
                    let selection = self.infinite_table_state.selection();
                    if mat.len() == 1 && mat[0].len() == 1 {
                        // Handle the case where there is a single item in clipboard, where
                        // it must be pasted to every cell in the selection.
                        let rows = selection[1].row - selection[0].row + 1;
                        let cols = selection[1].col - selection[0].col + 1;
                        let value = mat[0][0].clone();
                        mat = vec![vec![value; cols]; rows];
                    }
                    self.spreadsheet.replace_matrix(&selection[0], mat);
                }
            }
            _ => (),
        }
    }

    fn handle_editor_event(&mut self, event: &Event) {
        self.formula_editor_state.handle_event(&event);
        self.formula_suggestions_state.handle_event(&event);
        self.formula_editor_state = self.formula_suggestions_state.text_input_state.clone();

        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Enter => {
                    // if self.formula_suggestions_state.visible {
                    //     return;
                    // }

                    self.focused_area = AppArea::Data;

                    let value = if self.formula_editor_state.value().starts_with("=") {
                        balance_parens(&self.formula_editor_state.value())
                    } else {
                        self.formula_editor_state.value()
                    }; // TODO: Add a popup to confirm auto-balancing

                    self.spreadsheet
                        .set_cell(&self.infinite_table_state.active_cell, &value);
                    self.infinite_table_state.formula_cache.clear();

                    if self
                        .spreadsheet
                        .get_col_width(&self.infinite_table_state.active_cell)
                        < self.formula_editor_state.value().len() as u16
                    {
                        self.spreadsheet.set_col_width(
                            &self.infinite_table_state.active_cell,
                            self.formula_editor_state.value().len() as u16,
                        );
                    }

                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.infinite_table_state.move_active_cell(0, -1, false);
                    } else {
                        self.infinite_table_state.move_active_cell(0, 1, false);
                    }
                }
                KeyCode::Esc => self.focused_area = AppArea::Data,
                _ => (),
            },

            _ => (),
        }
    }
}
