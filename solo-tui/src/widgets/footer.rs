use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Padding, Paragraph, StatefulWidget, Widget},
};

use crate::app::AppState;

#[derive(Default)]
pub struct Footer;

impl StatefulWidget for Footer {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use Constraint::*;

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(1));

        let inner = block.inner(area);
        block.render(area, buf);

        let text = format!("  C  \nD S E\n  L V");

        let [left, middle, right] = Layout::horizontal([Length(5), Min(0), Length(5)]).areas(inner);

        let paragraph = Paragraph::new(text);

        paragraph.render(right, buf);
    }
}
