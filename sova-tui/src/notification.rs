use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer, layout::{Constraint, Layout, Rect}, style::{Color, Stylize}, widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap},
};
use ratatui_themes::Style;

use crate::theme::Palette;

pub const NOTIFICATION_TIME_MS: u64 = 1000;

pub struct Notification {
    pub text: String,
    pub color: Color,
    pub text_style: Style,
    pub bg: Color,
    pub triggered: Instant,
}

impl Notification {
    pub fn new() -> Self {
        Notification {
            text: Default::default(),
            color: Default::default(),
            text_style: Default::default(),
            bg: Default::default(),
            triggered: Instant::now()
                .checked_sub(Duration::from_millis(NOTIFICATION_TIME_MS + 1))
                .unwrap(),
        }
    }

    pub fn show(&mut self, text: String, color: Color, text_style: Style, bg: Color) {
        self.text = text;
        self.color = color;
        self.triggered = Instant::now();
        self.text_style = text_style;
        self.bg = bg;
    }

    pub fn info<T : ToString>(&mut self, text: T, palette: &Palette) {
        self.show(text.to_string(), palette.info, palette.text(), palette.surface);
    }

    pub fn positive<T : ToString>(&mut self, text: T, palette: &Palette) {
        self.show(text.to_string(), palette.success, palette.text(), palette.surface);
    }

    pub fn negative<T : ToString>(&mut self, text: T, palette: &Palette) {
        self.show(text.to_string(), palette.error, palette.text(), palette.surface);
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
            .style(self.text_style)
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .bg(self.bg)
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
