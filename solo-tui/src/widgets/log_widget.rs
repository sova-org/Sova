use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::{Paragraph, Widget},
};
use sova_core::LogMessage;

const MAX_LOGS: usize = 30;

#[derive(Default)]
pub struct LogWidget {
    logs: VecDeque<LogMessage>,
}

impl LogWidget {
    pub fn add_log(&mut self, msg: LogMessage) {
        if self.logs.len() == MAX_LOGS {
            self.logs.pop_front();
        }
        self.logs.push_back(msg);
    }
}

impl Widget for &LogWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines: Vec<Line> = self
            .logs
            .iter()
            .map(|msg| Line::from(msg.to_string()))
            .collect();
        let paragraph = Paragraph::new(Text::from(lines));
        paragraph.render(area, buf);
    }
}
