use std::cmp::{max, min};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::formula_functions::get_funcs;
use crate::utils::FuzzySearch;

use super::text_input::TextInputState;

#[derive(Default)]
pub struct FormulaSuggestions {}

#[derive(Debug, Default)]
pub struct FormulaSuggestionsState {
    pub text_input_state: TextInputState,
    pub visible: bool,
    list_state: ListState,
}

impl FormulaSuggestions {
    fn render_suggestions(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut <FormulaSuggestions as StatefulWidget>::State,
    ) {
        if !state.visible {
            return;
        }

        let suggestions = state.get_suggestions();
        if suggestions.len() == 0 {
            return;
        }

        if state.list_state.selected() == None {
            // Excel has the first option selected by default
            state.list_state.select(Some(0));
        }

        let cursor = state.text_input_state.cursor();
        let suggestions_area = Rect::new(
            cursor as u16 + state.text_input_state.area.x,
            state.text_input_state.area.y + state.text_input_state.area.height,
            min(
                area.width - state.text_input_state.area.x - cursor as u16,
                max(
                    suggestions.iter().max_by_key(|s| s.len()).unwrap().len(),
                    "Functions".len(),
                ) as u16
                    + 2,
            ),
            min(
                area.height - state.text_input_state.area.y - state.text_input_state.area.height,
                suggestions.len() as u16 + 2,
            ),
        );

        Clear.render(suggestions_area, buf);
        let block = Block::new().title("Functions").borders(Borders::ALL);

        let list = List::new(suggestions)
            // .wrap(Wrap { trim: false })
            // .style(Style::new().black())
            .highlight_style(Style::new().bg(Color::White).fg(Color::Black))
            .block(block);
        StatefulWidget::render(list, suggestions_area, buf, &mut state.list_state);
    }
}

impl StatefulWidget for FormulaSuggestions {
    type State = FormulaSuggestionsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        self.render_suggestions(area, buf, state);
    }
}

impl FormulaSuggestionsState {
    pub fn handle_event(&mut self, event: &Event) {
        self.text_input_state.handle_event(event);
        // Copy and then trim the input state to everything before the cursor
        // If it's text, attempt to match it to functions, showing only those that start with the
        // current text. If it's currently text, search from the current text to the nearest
        // non-alphanumeric character. Otherwise, search from all functions. Only enable this when
        // tab is pressed and disable it on escape or enter (selecting a function).
        if !self.text_input_state.value().starts_with("=") {
            self.visible = false;
            return;
        }

        let cursor = self.text_input_state.cursor();
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char(c) => {
                        self.visible = c.is_ascii_alphanumeric();

                        // Clear selection on hide
                        if !self.visible {
                            self.list_state.select(None);
                        }
                    }
                    KeyCode::Backspace => {
                        self.visible = self
                            .text_input_state
                            .value
                            .chars()
                            .nth(cursor - 1)
                            .is_some_and(|c| c.is_ascii_alphanumeric());

                        // Clear selection on hide
                        if !self.visible {
                            self.list_state.select(None);
                        }
                    }
                    KeyCode::Up => {
                        self.list_state.select_previous();
                    }
                    KeyCode::Down => {
                        self.list_state.select_next();
                    }
                    KeyCode::Tab => {
                        let suggestions = self.get_suggestions();
                        if suggestions.len() == 0 {
                            return;
                        }
                        if self.list_state.selected().is_none() {
                            self.list_state.select(Some(0));
                        }
                        let word = suggestions[self.list_state.selected().unwrap()].clone() + "(";
                        self.text_input_state.set_word(word.as_str());
                    }
                    _ => {
                        self.visible = false;
                    }
                }
            }
            _ => (),
        }
    }

    pub fn get_suggestions(&self) -> Vec<String> {
        // println!("\n{:?}", self.text_input_state.get_word());
        if let Some(current_word) = self.text_input_state.get_word() {
            if current_word.len() == 0 {
                return Vec::new();
            }
            let funcs = get_funcs()
                .keys()
                .map(|k| k.to_string())
                .collect::<Vec<String>>();
            funcs.fuzzy_search(current_word.to_ascii_uppercase().as_str(), 2)
        } else {
            Vec::new()
        }
        // Vec::new()
    }
}
