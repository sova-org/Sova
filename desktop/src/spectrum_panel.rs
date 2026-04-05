use eframe::egui;

use crate::settings::{AppearanceSettings, SpectrumSettings};
use crate::widgets::{self, Spectrum};

pub struct SpectrumPanel {
    pub open: bool,
    pub settings: SpectrumSettings,
    bands: Vec<f32>,
    normalized: Vec<f32>,
}

impl SpectrumPanel {
    pub fn new(settings: SpectrumSettings) -> Self {
        Self {
            open: false,
            settings,
            bands: vec![0.0; crate::widgets::spectrum_analyzer::NUM_BANDS],
            normalized: vec![0.0; crate::widgets::spectrum_analyzer::NUM_BANDS],
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        raw_bands: &[f32],
        appearance: &AppearanceSettings,
    ) {
        if !self.open {
            return;
        }
        if self.settings.detached {
            self.show_detached(ctx, raw_bands, appearance);
        } else {
            self.show_embedded(ctx, raw_bands);
        }
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        use crate::widgets::hint;
        let r = ui.add(
            egui::Slider::new(&mut self.settings.smoothing, 0.0..=0.99)
                .text(t!("spectrum.smoothing").as_ref()),
        );
        hint::on_hover(ui.ctx(), &r, t!("spectrum.hint.smoothing"));
        let r = ui.add(
            egui::Slider::new(&mut self.settings.bar_gap, 0.0..=4.0)
                .text(t!("spectrum.bar_gap").as_ref()),
        );
        hint::on_hover(ui.ctx(), &r, t!("spectrum.hint.bar_gap"));
        let r = ui.add(
            egui::Slider::new(&mut self.settings.gradient_strength, 0.0..=1.0)
                .text(t!("spectrum.gradient").as_ref()),
        );
        hint::on_hover(ui.ctx(), &r, t!("spectrum.hint.gradient"));
    }

    fn content(&mut self, ui: &mut egui::Ui, raw_bands: &[f32]) {
        if raw_bands.is_empty() {
            ui.colored_label(egui::Color32::GRAY, t!("spectrum.no_data"));
            self.bands.fill(0.0);
            return;
        }

        let accent = ui.visuals().selection.bg_fill;

        widgets::smooth(&mut self.bands, raw_bands, self.settings.smoothing);

        let peak = self.bands.iter().cloned().fold(0.0f32, f32::max).max(0.001);
        self.normalized.resize(self.bands.len(), 0.0);
        for (i, &b) in self.bands.iter().enumerate() {
            self.normalized[i] = (b / peak).min(1.0);
        }

        Spectrum::new(&self.normalized, accent)
            .bar_gap(self.settings.bar_gap)
            .gradient_strength(self.settings.gradient_strength)
            .show(ui);
    }

    fn show_embedded(&mut self, ctx: &egui::Context, raw_bands: &[f32]) {
        let mut open = self.open;
        egui::Window::new(t!("spectrum.title"))
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([400.0, 150.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let r = ui
                        .button(crate::icons::rich(crate::icons::POPOUT))
                        .on_hover_text(t!("common.pop_out"));
                    if r.hovered() {
                        crate::widgets::hint::set(ctx, t!("spectrum.hint.detach"));
                    }
                    if r.clicked() {
                        self.settings.detached = true;
                    }
                    egui::CollapsingHeader::new(t!("common.settings"))
                        .default_open(false)
                        .show(ui, |ui| self.settings_ui(ui));
                });
                self.content(ui, raw_bands);
            });
        self.open = open;
    }

    fn show_detached(
        &mut self,
        ctx: &egui::Context,
        raw_bands: &[f32],
        appearance: &AppearanceSettings,
    ) {
        let mut open = self.open;
        let mut detached = self.settings.detached;
        widgets::show_detached_viewport(
            ctx,
            &mut open,
            &mut detached,
            "spectrum_viewport",
            &t!("spectrum.detached_title"),
            [400.0, 200.0],
            appearance,
            |ui| {
                egui::CollapsingHeader::new(t!("common.settings"))
                    .default_open(false)
                    .show(ui, |ui| self.settings_ui(ui));
                self.content(ui, raw_bands);
            },
        );
        self.open = open;
        self.settings.detached = detached;
    }
}
