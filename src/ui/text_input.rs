use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Position, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::StatefulWidget,
};

const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);
const TRIPLE_CLICK_DURATION: Duration = Duration::from_millis(750);

// Simple, single-line text input box
#[derive(Debug, Default, Clone)]
pub struct TextInput {}

#[derive(Debug, Default, Clone)]
pub struct TextInputState {
    pub value: String,
    pub selection: [usize; 2],
    pub area: Rect,
    last_click: Option<Instant>,
}

impl StatefulWidget for TextInput {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let line: Line;
        if state.selection[0] == state.selection[1] {
            line = Line::from(state.value.clone()).reset_style();
        } else {
            let before_sel = Span::from(state.value[..state.sel_min()].to_string());
            let sel = Span::from(state.value[state.sel_min()..state.sel_max()].to_string())
                .on_dark_gray();
            let after_sel = Span::from(state.value[state.sel_max()..].to_string());
            line = Line::from(vec![before_sel, sel, after_sel]);
        }
        buf.set_line(area.top(), area.left(), &line, u16::MAX);
        state.area = area.clone();
    }
}

impl TextInputState {
    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Right => {
                        if self.selection[1] < self.value.len() {
                            self.selection[1] += 1;
                        }
                        if !key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            self.selection[0] = self.selection[1]
                        }
                    }
                    KeyCode::Left => {
                        if self.selection[1] > 0 {
                            self.selection[1] -= 1;
                        }
                        if !key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            self.selection[0] = self.selection[1]
                        }
                    }
                    KeyCode::Backspace => {
                        if self.selection[0] != self.selection[1] {
                            // Delete the selected text
                            self.value = self.value[..self.sel_min()].to_string()
                                + &self.value[self.sel_max()..];
                            self.set_cursor(self.sel_min());
                        } else {
                            // Delete the character before the cursor
                            self.value = self
                                .value
                                .chars()
                                .enumerate()
                                .filter(|(idx, _)| *idx + 1 != self.selection[1] as usize)
                                .map(|(_, c)| c)
                                .collect();

                            if self.selection[1] > 0 {
                                self.set_cursor(self.selection[1] - 1);
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if self.selection[0] != self.selection[1] {
                            self.value = self.value[..self.sel_min()].to_string()
                                + &self.value[self.sel_max()..];
                            self.set_cursor(self.sel_min());
                        }
                        self.value.insert(self.selection[1], c);
                        self.set_cursor(self.selection[1] + 1);
                    }
                    _ => (),
                }
            }
            Event::Mouse(mouse_event) => match mouse_event.kind {
                event::MouseEventKind::Down(_)
                    if self.area.contains(Position {
                        x: mouse_event.column,
                        y: mouse_event.row,
                    }) =>
                {
                    // TODO: Handle other mouse buttons, if they even do anything.

                    // Handle single clicks
                    let input_x = mouse_event.column - self.area.x;
                    if self.value.len() < input_x.into() {
                        self.set_cursor(self.value.len());
                    } else {
                        self.set_cursor(input_x.into());
                    }
                }
                event::MouseEventKind::Drag(_)
                    if self.area.contains(Position {
                        x: mouse_event.column,
                        y: mouse_event.row,
                    }) =>
                {
                    let input_x = mouse_event.column - self.area.x;
                    if self.value.len() < input_x.into() {
                        self.selection[1] = self.value.len()
                    } else {
                        self.selection[1] = input_x.into()
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }

    pub fn cursor(&self) -> usize {
        self.selection[1]
    }

    pub fn set_cursor(&mut self, x: usize) {
        self.selection[0] = x;
        self.selection[1] = x;
    }

    fn sel_min(&self) -> usize {
        *self.selection.iter().min().unwrap()
    }

    fn sel_max(&self) -> usize {
        *self.selection.iter().max().unwrap()
    }

    pub fn get_word_bounds(&self) -> Option<[usize; 2]> {
        let mut idx = self.cursor();
        let mut bounds: [usize; 2] = [0, 0];

        if self.value.len() == 0 {
            return None;
        }

        while let Some(char) = self.value.chars().nth(idx - 1) {
            if char.is_ascii_alphanumeric() {
                idx -= 1;
                bounds[0] = idx;
            } else {
                break;
            }
        }

        idx = self.cursor();
        while let Some(char) = self.value.chars().nth(idx - 1) {
            if char.is_ascii_alphanumeric() {
                bounds[1] = idx;
                idx += 1;
            } else {
                break;
            }
        }

        return Some(bounds);
    }

    pub fn get_word(&self) -> Option<String> {
        if let Some(bounds) = self.get_word_bounds() {
            Some(self.value[bounds[0]..bounds[1]].to_string())
        } else {
            None
        }
    }

    pub fn set_word(&mut self, word: &str) {
        println!("here");
        if let Some(bounds) = self.get_word_bounds() {
            println!("\n\n\n{:?}'{}'", bounds, self.value);
            self.value = self.value[..bounds[0]].to_string() + &self.value[bounds[1]..];
            self.value.insert_str(bounds[0], word);
            self.set_cursor(bounds[0] + word.len());
        } else {
            self.value.insert_str(self.cursor(), word);
            self.set_cursor(self.cursor() + word.len());
        }
    }
}
