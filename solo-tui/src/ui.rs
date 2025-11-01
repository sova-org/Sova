use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    widgets::{Block, BorderType, StatefulWidget, Widget},
};

use crate::{
    app::App,
    page::Page,
    widgets::{footer::Footer, header::Header},
};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;

        let title = match self.state.page {
            Page::Scene => "scene",
            Page::Devices => "devices",
            Page::Edit => "edit",
            Page::Configure => "configure",
            Page::Logs => "logs",
            Page::Vars => "vars",
        };

        let block = Block::bordered()
            .title(title)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let layout = Layout::vertical([Length(3), Min(0), Length(5)]);
        let [header_area, content_area, footer_area] = layout.areas(area);

        Header::default().render(header_area, buf, &mut self.state);
        block.render(content_area, buf);
        Footer::default().render(footer_area, buf, &mut self.state);
    }
}
