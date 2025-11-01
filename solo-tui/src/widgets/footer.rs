use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Padding, Paragraph, StatefulWidget, Widget},
};

use crate::{app::AppState, page::Page};

#[derive(Default)]
pub struct Footer;

fn map_style(state: &AppState, page: Page) -> Style {
    if state.page == page {
        Style::default().bold().fg(Color::Magenta)
    } else {
        Style::default()
    }
}

impl StatefulWidget for Footer {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use Constraint::*;

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(1));

        let inner = block.inner(area);
        block.render(area, buf);

        let lines = vec![
            Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::from(" "),
                Span::styled("C", map_style(state, Page::Configure)),
                Span::from(" "),
                Span::styled(" ", Style::default()),
            ]),
            Line::from(vec![
                Span::styled("D", map_style(state, Page::Devices)),
                Span::from(" "),
                Span::styled("S", map_style(state, Page::Scene)),
                Span::from(" "),
                Span::styled("E", map_style(state, Page::Edit)),
            ]),
            Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::from(" "),
                Span::styled("L", map_style(state, Page::Logs)),
                Span::from(" "),
                Span::styled("V", map_style(state, Page::Vars)),
            ]),
        ];
        let map = Paragraph::new(Text::from(lines));

        let [left, middle, right] = Layout::horizontal([Length(5), Min(0), Length(5)]).areas(inner);

        map.render(right, buf);
    }
}
