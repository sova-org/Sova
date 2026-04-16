use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;

pub const FFT_SIZE: usize = 2048;
pub const NUM_BANDS: usize = 128;

pub struct SpectrumAnalyzer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    window: Vec<f32>,
    band_edges: Vec<usize>,
    buffer: Vec<Complex<f32>>,
}

impl SpectrumAnalyzer {
    pub fn new(sample_rate: f32) -> Self {
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

    pub fn analyze(&mut self, samples: &[f32]) -> Vec<f32> {
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
