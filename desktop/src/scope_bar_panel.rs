use std::sync::Arc;

use eframe::egui;
use rustfft::{FftPlanner, num_complex::Complex};

use crate::settings::{ScopeBarMode, ScopeSettings, SpectrumSettings};
use crate::widgets::{self, Spectrum, Waveform};

const FFT_SIZE: usize = 2048;
const NUM_BANDS: usize = 128;

struct SpectrumAnalyzer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    window: Vec<f32>,
    band_edges: Vec<usize>,
    buffer: Vec<Complex<f32>>,
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
            buffer: vec![Complex::new(0.0, 0.0); FFT_SIZE],
        }
    }

    fn analyze(&mut self, samples: &[f32]) -> Vec<f32> {
        let n = samples.len().min(FFT_SIZE);
        for (i, buf) in self.buffer.iter_mut().enumerate() {
            *buf = if i < n {
                Complex::new(samples[i] * self.window[i], 0.0)
            } else {
                Complex::new(0.0, 0.0)
            };
        }

        self.fft.process(&mut self.buffer);

        let nyquist = FFT_SIZE / 2;
        let norm = 2.0 / FFT_SIZE as f32;

        (0..NUM_BANDS)
            .map(|i| {
                let lo = self.band_edges[i].min(nyquist);
                let hi = self.band_edges[i + 1].min(nyquist);
                if lo >= hi {
                    return 0.0;
                }
                let sum: f32 = self.buffer[lo..hi].iter().map(|c| c.norm() * norm).sum();
                sum / (hi - lo) as f32
            })
            .collect()
    }
}

pub struct ScopeBarPanel {
    pub open: bool,
    pub mode: ScopeBarMode,
    height: f32,
    // Scope state
    aligned: Vec<f32>,
    line_buffer: Vec<f32>,
    trace: Vec<(f32, f32)>,
    // Spectrum state
    analyzer: Option<SpectrumAnalyzer>,
    bands: Vec<f32>,
    normalized: Vec<f32>,
    last_data_ptr: usize,
}

impl ScopeBarPanel {
    pub fn new(height: f32, mode: ScopeBarMode) -> Self {
        Self {
            open: false,
            mode,
            height,
            aligned: Vec::new(),
            line_buffer: Vec::new(),
            trace: Vec::new(),
            analyzer: None,
            bands: vec![0.0; NUM_BANDS],
            normalized: vec![0.0; NUM_BANDS],
            last_data_ptr: 0,
        }
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    fn prepare_scope(&mut self, scope_data: &[f32]) {
        widgets::align_trigger(&mut self.aligned, scope_data);
    }

    fn prepare_spectrum(&mut self, scope_data: &[f32], spectrum_settings: &SpectrumSettings) {
        let analyzer = self
            .analyzer
            .get_or_insert_with(|| SpectrumAnalyzer::new(44100.0));

        let data_ptr = scope_data.as_ptr() as usize;
        if data_ptr != self.last_data_ptr {
            self.last_data_ptr = data_ptr;
            let raw = analyzer.analyze(scope_data);
            widgets::smooth(&mut self.bands, &raw, spectrum_settings.smoothing);
        }

        let peak = self.bands.iter().cloned().fold(0.0f32, f32::max).max(0.001);
        self.normalized.resize(self.bands.len(), 0.0);
        for (i, &b) in self.bands.iter().enumerate() {
            self.normalized[i] = (b / peak).min(1.0);
        }
    }

    fn show_scope(
        &mut self,
        ui: &mut egui::Ui,
        scope_settings: &ScopeSettings,
    ) {
        let accent = ui.visuals().selection.bg_fill;
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
    }

    fn show_spectrum(
        &mut self,
        ui: &mut egui::Ui,
        spectrum_settings: &SpectrumSettings,
    ) {
        let accent = ui.visuals().selection.bg_fill;
        Spectrum::new(&self.normalized, accent)
            .bar_gap(spectrum_settings.bar_gap)
            .gradient_strength(spectrum_settings.gradient_strength)
            .show(ui);
    }

    pub fn show_bottom_panel(
        &mut self,
        ctx: &egui::Context,
        scope_data: &[f32],
        scope_settings: &ScopeSettings,
        spectrum_settings: &SpectrumSettings,
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

                let needs_scope = matches!(self.mode, ScopeBarMode::Scope | ScopeBarMode::Both);
                let needs_spectrum =
                    matches!(self.mode, ScopeBarMode::Spectrogram | ScopeBarMode::Both);

                if needs_scope {
                    self.prepare_scope(scope_data);
                }
                if needs_spectrum {
                    self.prepare_spectrum(scope_data, spectrum_settings);
                }

                match self.mode {
                    ScopeBarMode::Scope => {
                        self.show_scope(ui, scope_settings);
                    }
                    ScopeBarMode::Spectrogram => {
                        self.show_spectrum(ui, spectrum_settings);
                    }
                    ScopeBarMode::Both => {
                        // Spectrum first (background), then scope overlaid
                        self.show_spectrum(ui, spectrum_settings);
                        // Rewind the cursor to overlay the scope on top
                        let rect = ui.max_rect();
                        let mut overlay = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(rect)
                                .layout(egui::Layout::top_down(egui::Align::Min)),
                        );
                        self.show_scope(&mut overlay, scope_settings);
                    }
                }

                ctx.request_repaint_after(std::time::Duration::from_millis(33));

                let click_resp =
                    ui.interact(ui.max_rect(), ui.id().with("cycle"), egui::Sense::click());
                if click_resp.clicked() {
                    self.mode = self.mode.next();
                }
            });

        self.height = resp.response.rect.height();
    }
}
