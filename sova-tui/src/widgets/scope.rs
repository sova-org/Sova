use ratatui::{prelude::{self, Buffer}, symbols::Marker, widgets::{Axis, Chart, Dataset, GraphType, StatefulWidget, Widget}};
use ratatui_themes::Style;

use crate::app::AppState;

pub struct Scope;

impl StatefulWidget for Scope {
    type State = AppState;

    fn render(self, area: prelude::Rect, buf: &mut Buffer, state: &mut Self::State) {
        let data : Vec<(f64, f64)> = state.client_state.scope_data.iter().enumerate().map(|(i,v)| {
            (i as f64, *v as f64)
        }).collect();
        let set = Dataset::default()
            .graph_type(GraphType::Line)
            .marker(Marker::Braille)
            .style(state.palette.accent)
            .data(&data);
    
        Chart::new(vec![set])
            .x_axis(Axis::default().bounds([0.0, 2048.0]))
            .y_axis(Axis::default().bounds([-1.0, 1.0]))
            .style(Style::default().bg(state.palette.background))
            .render(area, buf);
    }
}