use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap},
};

pub const NOTIFICATION_TIME_MS: u64 = 1000;

pub struct Notification {
    pub text: String,
    pub color: Color,
    pub triggered: Instant,
}

impl Notification {
    pub fn new() -> Self {
        Notification {
            text: Default::default(),
            color: Default::default(),
            triggered: Instant::now()
                .checked_sub(Duration::from_millis(NOTIFICATION_TIME_MS + 1))
                .unwrap(),
        }
    }

    pub fn show(&mut self, text: String, color: Color) {
        self.text = text;
        self.color = color;
        self.triggered = Instant::now();
    }

    pub fn info(&mut self, text: String) {
        self.show(text, Color::White);
    }

    pub fn positive(&mut self, text: String) {
        self.show(text, Color::LightGreen);
    }

    pub fn negative(&mut self, text: String) {
        self.show(text, Color::LightRed);
    }

    pub fn is_showing(&self) -> bool {
        self.triggered.elapsed().as_millis() < NOTIFICATION_TIME_MS as u128
    }
}

impl Widget for &Notification {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;
        if !self.is_showing() {
            return;
        }
        let paragraph = Paragraph::new(self.text.as_str())
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(self.color),
            );
        let width = 25 * area.width / 100;
        let len = 125 * (self.text.len() as u16) / 100;
        let lines = 2 + (len / width) + u16::from(len % width > 0);
        let horizontal = Layout::horizontal([Min(0), Length(width)]);
        let vertical = Layout::vertical([Length(lines)]);
        let [_, area] = horizontal.areas(area);
        let [area] = vertical.areas(area);
        Clear.render(area, buf);
        paragraph.render(area, buf);
    }
}
