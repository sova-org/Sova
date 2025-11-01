use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::App,
    widgets::{footer::Footer, header::Header},
};

impl Widget for &mut App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;
        let block = Block::bordered()
            .title("sova-solo")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let layout = Layout::vertical([Length(3), Min(0), Length(5)]);
        let [header_area, content_area, footer_area] = layout.areas(area);

        let text = format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.",
        );

        let paragraph = Paragraph::new(text)
            .block(block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();

        Header::default().render(header_area, buf, &mut self.state);
        paragraph.render(content_area, buf);
        Footer::default().render(footer_area, buf, &mut self.state);
    }
}
