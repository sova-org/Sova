use eframe::egui;

use crate::settings::ScopeBarSettings;
use crate::widgets::Waveform;

pub struct ScopeBarPanel {
    pub open: bool,
    height: f32,
    smoothed: Vec<f32>,
    smoothing: f32,
}

impl ScopeBarPanel {
    pub fn new(settings: ScopeBarSettings) -> Self {
        Self {
            open: false,
            height: settings.height,
            smoothed: Vec::new(),
            smoothing: settings.smoothing,
        }
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn smoothing(&self) -> f32 {
        self.smoothing
    }

    pub fn show_bottom_panel(&mut self, ctx: &egui::Context, scope_data: &[f32]) {
        let resp = egui::TopBottomPanel::bottom("scope_bar")
            .resizable(true)
            .default_height(self.height)
            .max_height(200.0)
            .show(ctx, |ui| {
                if scope_data.is_empty() {
                    return;
                }

                let accent = ui.visuals().selection.bg_fill;
                let a = self.smoothing;

                let data: &[f32] = if a > 0.0 {
                    self.smoothed.resize(scope_data.len(), 0.0);
                    for (i, &s) in scope_data.iter().enumerate() {
                        self.smoothed[i] = self.smoothed[i] * a + s * (1.0 - a);
                    }
                    &self.smoothed
                } else {
                    scope_data
                };

                Waveform::new(data, accent).fill_alpha(0.35).show(ui);
                ctx.request_repaint();
            });

        self.height = resp.response.rect.height();
    }
}
