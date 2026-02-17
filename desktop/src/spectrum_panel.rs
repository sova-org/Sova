use std::sync::Arc;

use eframe::egui;
use rustfft::{FftPlanner, num_complex::Complex};

use crate::settings::{AppearanceSettings, SpectrumSettings};
use crate::widgets::{self, Spectrum};

const FFT_SIZE: usize = 2048;
const NUM_BANDS: usize = 128;

struct SpectrumAnalyzer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    window: Vec<f32>,
    band_edges: Vec<usize>,
}

impl SpectrumAnalyzer {
    fn new(sample_rate: f32) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        let window: Vec<f32> = (0..FFT_SIZE)
            .map(|i| {
                let t = i as f32 / (FFT_SIZE - 1) as f32;
                0.5 - 0.5 * (2.0 * std::f32::consts::PI * t).cos()
            })
            .collect();

        let min_freq: f32 = 20.0;
        let max_freq = (sample_rate * 0.5).min(20000.0);
        let log_min = min_freq.ln();
        let log_max = max_freq.ln();
        let bin_hz = sample_rate / FFT_SIZE as f32;

        let band_edges: Vec<usize> = (0..=NUM_BANDS)
            .map(|i| {
                let freq = (log_min + (log_max - log_min) * i as f32 / NUM_BANDS as f32).exp();
                (freq / bin_hz) as usize
            })
            .collect();

        Self {
            fft,
            window,
            band_edges,
        }
    }

    fn analyze(&self, samples: &[f32]) -> Vec<f32> {
        let n = samples.len().min(FFT_SIZE);
        let mut buffer: Vec<Complex<f32>> = (0..FFT_SIZE)
            .map(|i| {
                if i < n {
                    Complex::new(samples[i] * self.window[i], 0.0)
                } else {
                    Complex::new(0.0, 0.0)
                }
            })
            .collect();

        self.fft.process(&mut buffer);

        let nyquist = FFT_SIZE / 2;
        let norm = 2.0 / FFT_SIZE as f32;

        (0..NUM_BANDS)
            .map(|i| {
                let lo = self.band_edges[i].min(nyquist);
                let hi = self.band_edges[i + 1].min(nyquist);
                if lo >= hi {
                    return 0.0;
                }
                let sum: f32 = buffer[lo..hi].iter().map(|c| c.norm() * norm).sum();
                sum / (hi - lo) as f32
            })
            .collect()
    }
}

pub struct SpectrumPanel {
    pub open: bool,
    pub settings: SpectrumSettings,
    analyzer: Option<SpectrumAnalyzer>,
    bands: Vec<f32>,
}

impl SpectrumPanel {
    pub fn new(settings: SpectrumSettings) -> Self {
        Self {
            open: false,
            settings,
            analyzer: None,
            bands: vec![0.0; NUM_BANDS],
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        scope_data: &[f32],
        appearance: &AppearanceSettings,
    ) {
        if !self.open {
            return;
        }
        if self.settings.detached {
            self.show_detached(ctx, scope_data, appearance);
        } else {
            self.show_embedded(ctx, scope_data);
        }
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let hint = |r: &egui::Response, text: &'static str| {
            if r.hovered() { crate::widgets::hint::set(&ctx, text); }
        };
        hint(&ui.add(
            egui::Slider::new(&mut self.settings.smoothing, 0.0..=0.99).text("Smoothing"),
        ), "Temporal smoothing — higher values give slower, smoother bars");
        hint(&ui.add(
            egui::Slider::new(&mut self.settings.bar_gap, 0.0..=4.0).text("Bar Gap"),
        ), "Spacing between frequency bars in pixels");
        hint(&ui.add(
            egui::Slider::new(&mut self.settings.gradient_strength, 0.0..=1.0).text("Gradient"),
        ), "Vertical color gradient intensity on bars");
    }

    fn content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, scope_data: &[f32]) {
        if scope_data.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No audio data received");
            self.analyzer = None;
            self.bands.fill(0.0);
            return;
        }

        let accent = ui.visuals().selection.bg_fill;

        let analyzer = self
            .analyzer
            .get_or_insert_with(|| SpectrumAnalyzer::new(44100.0));

        let smoothing = self.settings.smoothing;
        let raw = analyzer.analyze(scope_data);
        for (i, &r) in raw.iter().enumerate() {
            self.bands[i] = self.bands[i] * smoothing + r * (1.0 - smoothing);
        }

        let peak = self.bands.iter().cloned().fold(0.0f32, f32::max).max(0.001);
        let normalized: Vec<f32> = self.bands.iter().map(|&b| (b / peak).min(1.0)).collect();

        Spectrum::new(&normalized, accent)
            .bar_gap(self.settings.bar_gap)
            .gradient_strength(self.settings.gradient_strength)
            .show(ui);
        ctx.request_repaint();
    }

    fn show_embedded(&mut self, ctx: &egui::Context, scope_data: &[f32]) {
        let mut open = self.open;
        egui::Window::new("Audio Spectrum")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([400.0, 150.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let r = ui.button(crate::icons::POPOUT).on_hover_text("Pop out");
                    if r.hovered() {
                        crate::widgets::hint::set(ctx, "Detach spectrum into its own window");
                    }
                    if r.clicked() {
                        self.settings.detached = true;
                    }
                    egui::CollapsingHeader::new("Settings")
                        .default_open(false)
                        .show(ui, |ui| self.settings_ui(ui));
                });
                self.content(ui, ctx, scope_data);
            });
        self.open = open;
    }

    fn show_detached(
        &mut self,
        ctx: &egui::Context,
        scope_data: &[f32],
        appearance: &AppearanceSettings,
    ) {
        let mut open = self.open;
        let mut detached = self.settings.detached;
        widgets::show_detached_viewport(
            ctx,
            &mut open,
            &mut detached,
            "spectrum_viewport",
            "Sova - Audio Spectrum",
            [400.0, 200.0],
            appearance,
            |ui| {
                egui::CollapsingHeader::new("Settings")
                    .default_open(false)
                    .show(ui, |ui| self.settings_ui(ui));
                self.content(ui, ctx, scope_data);
            },
        );
        self.open = open;
        self.settings.detached = detached;
    }
}
