use std::sync::Arc;

use eframe::egui;
use rustfft::{FftPlanner, num_complex::Complex};

use crate::widgets::Spectrum;

const FFT_SIZE: usize = 1024;
const NUM_BANDS: usize = 32;
const SMOOTHING: f32 = 0.85;

struct SpectrumAnalyzer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    window: [f32; FFT_SIZE],
    band_edges: [usize; NUM_BANDS + 1],
}

impl SpectrumAnalyzer {
    fn new(sample_rate: f32) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        let mut window = [0.0f32; FFT_SIZE];
        for (i, w) in window.iter_mut().enumerate() {
            let t = i as f32 / (FFT_SIZE - 1) as f32;
            *w = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * t).cos();
        }

        let min_freq: f32 = 20.0;
        let max_freq = (sample_rate * 0.5).min(16000.0);
        let log_min = min_freq.ln();
        let log_max = max_freq.ln();
        let bin_hz = sample_rate / FFT_SIZE as f32;

        let mut band_edges = [0usize; NUM_BANDS + 1];
        for (i, edge) in band_edges.iter_mut().enumerate() {
            let freq = (log_min + (log_max - log_min) * i as f32 / NUM_BANDS as f32).exp();
            *edge = (freq / bin_hz) as usize;
        }

        Self {
            fft,
            window,
            band_edges,
        }
    }

    fn analyze(&self, samples: &[f32]) -> [f32; NUM_BANDS] {
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .take(FFT_SIZE)
            .enumerate()
            .map(|(i, &s)| Complex::new(s * self.window[i], 0.0))
            .collect();

        if buffer.len() < FFT_SIZE {
            buffer.resize(FFT_SIZE, Complex::new(0.0, 0.0));
        }

        self.fft.process(&mut buffer);

        let nyquist = FFT_SIZE / 2;
        let norm = 2.0 / FFT_SIZE as f32;

        let mut bands = [0.0f32; NUM_BANDS];
        for (i, band) in bands.iter_mut().enumerate() {
            let lo = self.band_edges[i].min(nyquist);
            let hi = self.band_edges[i + 1].min(nyquist);
            if lo >= hi {
                continue;
            }
            let sum: f32 = buffer[lo..hi].iter().map(|c| c.norm() * norm).sum();
            *band = sum / (hi - lo) as f32;
        }

        bands
    }
}

pub struct SpectrumPanel {
    pub open: bool,
    analyzer: Option<SpectrumAnalyzer>,
    bands: [f32; NUM_BANDS],
}

impl SpectrumPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            analyzer: None,
            bands: [0.0; NUM_BANDS],
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, scope_data: &[(f32, f32)]) {
        let mut open = self.open;
        egui::Window::new("Audio Spectrum")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([400.0, 150.0])
            .show(ctx, |ui| {
                if scope_data.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No audio data received");
                    self.analyzer = None;
                    self.bands = [0.0; NUM_BANDS];
                } else {
                    let samples: Vec<f32> = scope_data.iter().map(|(l, _)| *l).collect();
                    let accent = ui.visuals().selection.bg_fill;

                    let analyzer = self
                        .analyzer
                        .get_or_insert_with(|| SpectrumAnalyzer::new(44100.0));

                    let raw = analyzer.analyze(&samples);
                    for (i, &r) in raw.iter().enumerate() {
                        self.bands[i] = self.bands[i] * SMOOTHING + r * (1.0 - SMOOTHING);
                    }

                    let peak = self.bands.iter().cloned().fold(0.0f32, f32::max).max(0.001);
                    let normalized: Vec<f32> =
                        self.bands.iter().map(|&b| (b / peak).min(1.0)).collect();

                    Spectrum::new(&normalized, accent).show(ui);
                    ctx.request_repaint();
                }
            });
        self.open = open;
    }
}
