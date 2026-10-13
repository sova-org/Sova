use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Stylize,
    widgets::{
        Block, BorderType, Clear,
        StatefulWidget, Widget,
    },
};

use crate::{app::AppState, widgets::InteractiveWidget};

#[derive(Default)]
pub struct Floating {
    pub title: String, 
    pub inner: Option<Box<dyn InteractiveWidget>>
}

impl Floating {
    pub fn open<T: InteractiveWidget + 'static, S: ToString>(
        &mut self,
        title: S,
        content: T,
    ) {
        self.title = title.to_string();
        self.inner = Some(Box::new(content));
    }

    pub fn hide(&mut self) {
        self.inner = None;
    }

    pub fn is_showing(&self) -> bool {
        self.inner.is_some()
    }

    pub fn process_event(&mut self, state: &mut AppState, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => self.hide(),
            _ => match &mut self.inner {
                Some(i) => i.on_key_event(event, state),
                None => ()
            }
        }
    }

    fn floating_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let [area] = horizontal.areas(area);
        let [area] = vertical.areas(area);
        area
    }
}

impl StatefulWidget for &mut Floating {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let Some(inner) = &mut self.inner else {
            return;
        };
        let area = Floating::floating_area(area, 80, 70);
        Clear.render(area, buf);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(self.title.as_str())
            .bg(state.palette.surface)
            .fg(state.palette.foreground);
        let inner_area = block.inner(area);
        block.render(area, buf);
        inner.render(inner_area, buf, state);
    }
}
