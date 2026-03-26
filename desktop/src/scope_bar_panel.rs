use eframe::egui;

use crate::settings::ScopeSettings;
use crate::widgets::{self, Waveform};

pub struct ScopeBarPanel {
    pub open: bool,
    height: f32,
    smoothed: Vec<f32>,
}

impl ScopeBarPanel {
    pub fn new(height: f32) -> Self {
        Self {
            open: false,
            height,
            smoothed: Vec::new(),
        }
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn show_bottom_panel(
        &mut self,
        ctx: &egui::Context,
        scope_data: &[f32],
        scope_settings: &ScopeSettings,
    ) {
        let resp = egui::TopBottomPanel::bottom("scope_bar")
            .resizable(true)
            .default_height(self.height)
            .max_height(200.0)
            .show(ctx, |ui| {
                if scope_data.is_empty() {
                    return;
                }

                let accent = ui.visuals().selection.bg_fill;
                let a = scope_settings.smoothing;

                let data: &[f32] = if a > 0.0 {
                    widgets::smooth(&mut self.smoothed, scope_data, a);
                    &self.smoothed
                } else {
                    scope_data
                };

                Waveform::new(data, accent)
                    .stroke_width(scope_settings.stroke_width)
                    .fill_alpha(scope_settings.fill_alpha)
                    .glow(scope_settings.glow)
                    .show(ui);
                ctx.request_repaint_after(std::time::Duration::from_millis(33));
            });

        self.height = resp.response.rect.height();
    }
}
