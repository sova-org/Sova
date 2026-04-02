use eframe::egui;

use crate::settings::ScopeSettings;
use crate::widgets::{self, Waveform};

pub struct ScopeBarPanel {
    pub open: bool,
    height: f32,
    aligned: Vec<f32>,
    line_buffer: Vec<f32>,
    trace: Vec<(f32, f32)>,
}

impl ScopeBarPanel {
    pub fn new(height: f32) -> Self {
        Self {
            open: false,
            height,
            aligned: Vec::new(),
            line_buffer: Vec::new(),
            trace: Vec::new(),
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
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                if scope_data.is_empty() {
                    return;
                }

                let accent = ui.visuals().selection.bg_fill;
                widgets::align_trigger(&mut self.aligned, scope_data);
                let target = (ui.available_width() as usize).clamp(128, 800);
                widgets::downsample_lttb(&mut self.line_buffer, &self.aligned, target);

                let mut waveform = Waveform::from_line(&self.line_buffer, accent)
                    .stroke_width(2.2)
                    .fill_alpha(0.46);

                if scope_settings.persistence > 0.0 {
                    widgets::apply_trace(
                        &mut self.trace,
                        &self.line_buffer,
                        scope_settings.persistence,
                    );
                    waveform = waveform.with_trace(&self.trace);
                }

                waveform.show(ui);
                ctx.request_repaint_after(std::time::Duration::from_millis(33));
            });

        self.height = resp.response.rect.height();
    }
}
