use ratatui::{layout::{Constraint::Percentage, Layout, Margin}, prelude::{Buffer, Rect}, style::Stylize, text::{Line, Span}, widgets::{Block, StatefulWidget, Widget}};

use crate::app::AppState;

pub struct Footer;

impl StatefulWidget for Footer {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Block::default()
            .bg(state.palette.surface)
            .render(area, buf);
        let area = area.inner(Margin::new(1, 0));

        let layout = Layout::horizontal([
            Percentage(33),
            Percentage(34),
            Percentage(33)
        ]).split(area);

        let serv_status = Line::from(if state.is_server_running() {
            vec![
                Span::styled(nerd_font_symbols::cod::COD_CIRCLE_FILLED, state.palette.success),
                Span::styled(" Server running", state.palette.text())
            ]
        } else if state.client.is_some() {
            let ip = state.client.as_ref().unwrap().ip.clone();
            vec![
                Span::styled(nerd_font_symbols::md::MD_WIFI, state.palette.success),
                Span::styled(format!(" ({ip})"), state.palette.text())
            ]
        } else {
            vec![Span::styled("Not connected", state.palette.muted)]
        });

        serv_status.render(layout[0], buf);

        if state.client.is_some() {
            let lang = state.selected_frame().map(|f| f.script().lang()).unwrap_or_default();
            let selected = Line::from(format!("({},{}) {}", state.selected.0, state.selected.1, lang))
                .style(state.palette.text())
                .right_aligned();
            selected.render(layout[2], buf);
        }
    }
}