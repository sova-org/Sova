use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    symbols::scrollbar,
    text::{Line, Text},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
};
use sova_core::LogMessage;

const MAX_LOGS: usize = 60;

#[derive(Default)]
pub struct LogWidget {
    logs: VecDeque<LogMessage>,
    scroll_state: ScrollbarState,
    position: usize,
    view_len: usize,
    horizontal_scroll: u16,
}

impl LogWidget {
    pub fn add_log(&mut self, msg: LogMessage) {
        if self.logs.len() == MAX_LOGS {
            self.logs.pop_front();
        }
        self.logs.push_back(msg);
        if self.view_len > 0 {
            self.position = self.logs.len().saturating_sub(self.view_len);
            self.scroll_state = self
                .scroll_state
                .content_length(self.logs.len().saturating_sub(self.view_len))
                .position(self.position);
        }
    }

    pub fn get_help() -> &'static str {
        ""
    }

    pub fn process_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Up => {
                self.position = self.position.saturating_sub(1);
                self.scroll_state = self.scroll_state.position(self.position);
            }
            KeyCode::Down => {
                self.position = std::cmp::min(
                    self.position.saturating_add(1),
                    self.logs.len().saturating_sub(self.view_len),
                );
                self.scroll_state = self.scroll_state.position(self.position);
            }
            KeyCode::Left => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
            }
            KeyCode::Right => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
            }
            _ => (),
        }
    }
}

impl Widget for &mut LogWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.view_len = area.height as usize;
        let lines: Vec<Line> = self
            .logs
            .iter()
            .map(|msg| Line::from(msg.to_string()))
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
