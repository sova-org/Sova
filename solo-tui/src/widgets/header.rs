use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Gauge, StatefulWidget, Widget},
};

use crate::app::AppState;

#[derive(Default)]
pub struct Header;

impl StatefulWidget for Header {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let beat = state.clock.beat();
        let quantum = state.clock.quantum();
        let quantumized = beat % quantum;
        let progress = quantumized / quantum;

        let label = Span::styled(
            format!(
                "{:.1} ({} / {})",
                beat,
                quantumized as i8 + 1,
                quantum as i8
            ),
            Style::new().bold().fg(Color::White),
        );

        let play = if state.playing { "▶" } else { "■" };

        let title = format!("| Sova - {:.0} BPM - {play} |", state.clock.tempo());

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(ratatui::text::Line::from(title).centered());

        Gauge::default()
            .block(block)
            .gauge_style(Color::Magenta)
            .ratio(progress)
            .label(label)
            .render(area, buf);
    }
}
