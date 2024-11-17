use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Position},
    style::Style,
    text::{Line, Text},
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
    pub fn new(value: String, selection: [usize; 2]) -> Self {
        Self { value, selection }
    }

    pub fn render(&self) -> Paragraph {
        return Paragraph::new(self.value.clone());
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Right => {
                        if self.selection[1] < self.value.len() {
                            self.selection[1] += 1;
                        }
                        self.selection[0] = self.selection[1]
                    }
                    KeyCode::Left => {
                        if self.selection[1] > 0 {
                            self.selection[1] -= 1;
                        }
                        self.selection[0] = self.selection[1]
                    }
                    KeyCode::Backspace => {
                        if self.selection[0] != self.selection[1] {
                            // Delete the selected text
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
                            todo!()
                        } else {
                            self.value.insert(self.selection[1], c);
                            self.set_cursor(self.selection[1] + 1);
                        }
                    }
                    _ => (),
                }
            }
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
}
