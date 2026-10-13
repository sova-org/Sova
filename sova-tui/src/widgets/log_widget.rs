use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    symbols::scrollbar,
    text::{Line, Text},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
};

use crate::{app::AppState, widgets::InteractiveWidget};

#[derive(Default)]
pub struct LogWidget {
    scroll_state: ScrollbarState,
    position: usize,
    view_len: usize,
    horizontal_scroll: u16,
    moved: bool
}

impl InteractiveWidget for LogWidget {
    fn on_key_event(&mut self, event: KeyEvent, state: &mut AppState) {
        match event.code {
            KeyCode::Up => {
                self.position = self.position.saturating_sub(1);
                self.scroll_state = self.scroll_state.position(self.position);
                self.moved = true;
            }
            KeyCode::Down => {
                self.position = std::cmp::min(
                    self.position.saturating_add(1),
                    state.client_state.logs.len().saturating_sub(self.view_len),
                );
                self.scroll_state = self.scroll_state.position(self.position);
                self.moved = true;
            }
            KeyCode::Left => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
                self.moved = true;
            }
            KeyCode::Right => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
                self.moved = true;
            }
            _ => (),
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        self.view_len = area.height as usize;
        if !self.moved && self.view_len > 0 {
            self.position = state.client_state.logs.len().saturating_sub(self.view_len);
            self.scroll_state = self
                .scroll_state
                .content_length(state.client_state.logs.len().saturating_sub(self.view_len))
                .position(self.position);
        }
        let lines: Vec<Line> = state.client_state.logs
            .iter()
            .map(|msg| {
                let color = match msg.level {
                    sova_core::Severity::Fatal => state.palette.error,
                    sova_core::Severity::Error => state.palette.error,
                    sova_core::Severity::Warn => state.palette.warning,
                    sova_core::Severity::Info => state.palette.foreground,
                    sova_core::Severity::Debug => state.palette.muted,
                };
                Line::from(msg.to_string()).style(color)
            })
            .collect();
        let paragraph = Paragraph::new(Text::from(lines))
            .scroll((self.position as u16, self.horizontal_scroll));
        paragraph.render(area, buf);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .render(
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                buf,
                &mut self.scroll_state,
            );
    }
}