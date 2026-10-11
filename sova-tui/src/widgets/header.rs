use ratatui::{prelude::{Buffer, Rect}, style::Stylize, widgets::{Block, BorderType, Borders, Gauge, StatefulWidget, Widget}};

use crate::app::AppState;

pub struct Header;

impl StatefulWidget for Header {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Block::bordered()
            .bg(state.palette.surface)
            .border_type(BorderType::Plain)
            .borders(Borders::BOTTOM)
            .border_style(state.palette.foreground)
            .render(area, buf);

        let clock = &state.client_state.clock;
        let percent = if clock.quantum > 0.0 { (clock.phase() * 100.0 / clock.quantum) as u16 } else { 0 };

        let mut gauge = Gauge::default()
            .gauge_style(state.palette.surface)
            .percent(percent);
    }
}