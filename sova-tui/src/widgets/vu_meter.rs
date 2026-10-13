use ratatui::{prelude::{Buffer, Rect}, widgets::{Bar, BarChart, StatefulWidget, Widget}};

use crate::app::AppState;

pub struct VUMeter(pub usize);

impl StatefulWidget for VUMeter {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut bars = Vec::with_capacity(self.0);
        for i in 0..self.0 {
            let peak = state.client_state.peak_data
                .get(i)
                .copied()
                .unwrap_or_default();
            let peak = (peak * 100.0) as u64;
            let color = if peak < 50 {
                state.palette.success
            } else if peak < 100 {
                state.palette.warning
            } else {
                state.palette.error
            };
            bars.push(Bar::new(peak).text_value(String::new()).style(color));
        }
        BarChart::vertical(bars)
            .bar_width(1)
            .bar_gap(0)
            .max(100)
            .render(area, buf);
    }
}