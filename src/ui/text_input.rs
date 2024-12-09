use std::cmp::min;

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Alignment, Position},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget},
    Frame,
};

// Simple, single-line text input box
#[derive(Debug, Default, Clone)]
pub struct TextInput {
    value: String,
    selection: [usize; 2],
}

impl TextInput {
    pub fn render(&self) -> Line {
        if self.selection[0] == self.selection[1] {
            return Line::from(self.value.clone()).reset_style();
        }

        let before_sel = Span::from(self.value[..self.sel_min()].to_string());
        let sel =
            Span::from(self.value[self.sel_min()..self.sel_max()].to_string()).on_light_blue();
        let after_sel = Span::from(self.value[self.sel_max()..].to_string());

        return Line::from(vec![before_sel, sel, after_sel]);
    }

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
                event::MouseEventKind::Down(_) => {
                    // TODO: Handle other mouse buttons, if they even do anything.
                    
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
}
