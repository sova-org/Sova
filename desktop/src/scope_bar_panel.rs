use eframe::egui;

use crate::settings::{ScopeBarMode, ScopeSettings, SpectrumSettings};
use crate::widgets::{self, Spectrum, Waveform};

pub struct ScopeBarPanel {
    pub open: bool,
    pub mode: ScopeBarMode,
    height: f32,
    // Scope state
    line_buffer: Vec<f32>,
    trace: Vec<(f32, f32)>,
    // Spectrum state
    bands: Vec<f32>,
    normalized: Vec<f32>,
}

impl ScopeBarPanel {
    pub fn new(height: f32, mode: ScopeBarMode) -> Self {
        Self {
            open: false,
            mode,
            height,
            line_buffer: Vec::new(),
            trace: Vec::new(),
            bands: vec![0.0; crate::widgets::spectrum_analyzer::NUM_BANDS],
            normalized: vec![0.0; crate::widgets::spectrum_analyzer::NUM_BANDS],
        }
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    fn show_scope(
        &mut self,
        ui: &mut egui::Ui,
        aligned: &[f32],
        scope_settings: &ScopeSettings,
    ) {
        let accent = ui.visuals().selection.bg_fill;
        let target = (ui.available_width() as usize).clamp(128, 800);
        widgets::downsample_lttb(&mut self.line_buffer, aligned, target);

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
    }

    fn show_spectrum(
        &mut self,
        ui: &mut egui::Ui,
        raw_bands: &[f32],
        spectrum_settings: &SpectrumSettings,
    ) {
        widgets::smooth(&mut self.bands, raw_bands, spectrum_settings.smoothing);

        let peak = self.bands.iter().cloned().fold(0.0f32, f32::max).max(0.001);
        self.normalized.resize(self.bands.len(), 0.0);
        for (i, &b) in self.bands.iter().enumerate() {
            self.normalized[i] = (b / peak).min(1.0);
        }

        let accent = ui.visuals().selection.bg_fill;
        Spectrum::new(&self.normalized, accent)
            .bar_gap(spectrum_settings.bar_gap)
            .gradient_strength(spectrum_settings.gradient_strength)
            .show(ui);
    }

    pub fn show_bottom_panel(
        &mut self,
        ctx: &egui::Context,
        aligned: &[f32],
        raw_bands: &[f32],
        scope_settings: &ScopeSettings,
        spectrum_settings: &SpectrumSettings,
    ) {
        let resp = egui::TopBottomPanel::bottom("scope_bar")
            .resizable(true)
            .default_height(self.height)
            .max_height(200.0)
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                if aligned.is_empty() && raw_bands.is_empty() {
                    return;
                }

                match self.mode {
                    ScopeBarMode::Scope => {
                        self.show_scope(ui, aligned, scope_settings);
                    }
                    ScopeBarMode::Spectrogram => {
                        self.show_spectrum(ui, raw_bands, spectrum_settings);
                    }
                    ScopeBarMode::Both => {
                        self.show_spectrum(ui, raw_bands, spectrum_settings);
                        let rect = ui.max_rect();
                        let mut overlay = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(rect)
                                .layout(egui::Layout::top_down(egui::Align::Min)),
                        );
                        self.show_scope(&mut overlay, aligned, scope_settings);
                    }
                }

                let click_resp =
                    ui.interact(ui.max_rect(), ui.id().with("cycle"), egui::Sense::click());
                if click_resp.clicked() {
                    self.mode = self.mode.next();
                }
            });

        self.height = resp.response.rect.height();
    }
}
