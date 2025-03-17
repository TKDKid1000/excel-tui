use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event},
    layout::{Alignment, Position, Rect},
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};

#[derive(Default)]
pub struct Button {
    pub text: String,
}

#[derive(Debug, Default)]
pub struct ButtonState {
    area: Rect,
    pub is_pressed: bool,
}

impl StatefulWidget for Button {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);
        let content = Paragraph::new(self.text).block(block);
        content.render(area, buf);
        state.area = area.clone();
    }
}

impl ButtonState {
    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Mouse(mouse_event) => match mouse_event.kind {
                event::MouseEventKind::Down(_)
                    if self.area.contains(Position {
                        x: mouse_event.column,
                        y: mouse_event.row,
                    }) =>
                {
                    self.is_pressed = true;
                }
                event::MouseEventKind::Up(_)
                    if self.area.contains(Position {
                        x: mouse_event.column,
                        y: mouse_event.row,
                    }) =>
                {
                    self.is_pressed = false;
                }

                _ => (),
            },
            _ => (),
        }
    }
}
