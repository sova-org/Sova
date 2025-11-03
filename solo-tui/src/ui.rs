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

        let block = Block::bordered()
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let layout = Layout::vertical([Length(3), Min(0), Length(5)]);
        let [header_area, middle_area, footer_area] = layout.areas(area);
        let content_area = block.inner(middle_area);

        let title = match self.state.page {
            Page::Scene => {
                self.scene_widget
                    .render(block.inner(content_area), buf, &mut self.state);
                "scene"
            }
            Page::Devices => "devices",
            Page::Edit => "edit",
            Page::Configure => "configure",
            Page::Time => "time",
            Page::Logs => {
                self.log_widget.render(content_area, buf);
                "logs"
            }
            Page::Vars => "vars",
        };

        Header::default().render(header_area, buf, &mut self.state);
        block.title(title).render(middle_area, buf);
        Footer::default().render(footer_area, buf, &mut self.state);

        self.popup.render(area, buf);
    }
}
