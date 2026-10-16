use ratatui::{prelude::{Buffer, Rect}, widgets::{Bar, BarChart, StatefulWidget, Widget}};

use crate::app::AppState;

const VUMETER_SMOOTH_FACTOR : f32 = 0.9;

pub struct VUMeter {
    pub channels: usize,
    buffer: Vec<f32>
}

impl VUMeter {
    pub fn new(channels: usize) -> Self {
        Self {
            channels, buffer: Vec::new()
        }
    }

    pub fn add_data(&mut self, data: Vec<f32>) {
        if self.buffer.is_empty() {
            self.buffer = data;
            return;
        }
        if self.buffer.len() < self.channels {
            self.buffer.resize(self.channels, 0.0);
        }
        for i in 0..self.channels {
            let new_peak = data.get(i).copied().unwrap_or_default();
            let old = self.buffer[i];
            let interpolation = 
                (1.0 - VUMETER_SMOOTH_FACTOR) * new_peak +
                VUMETER_SMOOTH_FACTOR * old;
            self.buffer[i] = interpolation;
        }
    }
}

impl StatefulWidget for &VUMeter {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut bars = Vec::with_capacity(self.channels);
        for i in 0..self.channels {
            let peak = self.buffer
                .get(i)
                .copied()
                .unwrap_or_default();
            let peak = (peak * 100.0) as u64;
            let color = if peak < 50 {
                state.palette.success
            } else if peak < 90 {
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