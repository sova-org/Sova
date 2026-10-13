use ratatui::{layout::{Constraint, Flex, Layout, Margin}, prelude::{Buffer, Rect}, style::Stylize, text::{Line, Span}, widgets::{Block, BorderType, Borders, Gauge, StatefulWidget, Widget}};
use ratatui_themes::Style;
use sova_core::schedule::playback::PlaybackState;

use crate::app::AppState;

pub struct Header;

impl StatefulWidget for Header {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let b = Block::bordered()
            .bg(state.palette.surface)
            .border_type(BorderType::Plain)
            .borders(Borders::BOTTOM)
            .border_style(state.palette.foreground);

        let in_header = b.inner(area);
        b.render(area, buf);

        let clock = &state.client_state.clock;
        
        let lines = Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
            .flex(Flex::SpaceBetween)
            .split(in_header);

        let rows = Layout::horizontal([Constraint::Percentage(25), Constraint::Max(40), Constraint::Percentage(25)])
            .flex(Flex::SpaceBetween)
            .split(in_header.inner(Margin::new(2, 0)));

        let playing_status = match state.client_state.playback_state {
            PlaybackState::Stopped => Span::styled(nerd_font_symbols::md::MD_PAUSE, state.palette.error),
            PlaybackState::Starting(_) => Span::styled(nerd_font_symbols::md::MD_DOTS_CIRCLE, state.palette.foreground),
            PlaybackState::Playing => Span::styled(nerd_font_symbols::md::MD_PLAY, state.palette.success),
        };
        playing_status.render(lines[1].intersection(rows[0]), buf);

        let clock_status = Line::from(format!("{:.1}bpm   Q{:.0}   {:.2}'", clock.tempo, clock.quantum, clock.beat))
            .style(state.palette.text())
            .centered();
        clock_status.render(lines[0].intersection(rows[1]), buf);

        let percent = if clock.quantum > 0.0 { (clock.phase() * 100.0 / clock.quantum) as u16 } else { 0 };
        let gauge = Gauge::default()
            .gauge_style(if clock.beat % 2.0 < 1.0 { state.palette.accent } else { state.palette.secondary })
            .style(Style::default().bg(state.palette.background))
            .percent(percent)
            .label("");
        gauge.render(lines[1].intersection(rows[1]), buf);

        let mode = Line::from(state.client_state.scene.mode.to_string()).style(state.palette.text()).right_aligned();
        mode.render(lines[1].intersection(rows[2]), buf);
    }
}

// nf md pause play