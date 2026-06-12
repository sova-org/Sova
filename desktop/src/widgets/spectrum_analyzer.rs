use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;

pub const FFT_SIZE: usize = 4096;
pub const NUM_BANDS: usize = 192;

pub struct SpectrumAnalyzer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    window: Vec<f32>,
    bin_edges: Vec<f32>,
    buffer: Vec<Complex<f32>>,
    ring: Vec<f32>,
    write_pos: usize,
    filled: usize,
    sample_rate: f32,
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

        let bin_edges: Vec<f32> = (0..=NUM_BANDS)
            .map(|i| {
                let freq = (log_min + (log_max - log_min) * i as f32 / NUM_BANDS as f32).exp();
                freq / bin_hz
            })
            .collect();

        Self {
            fft,
            window,
            bin_edges,
            buffer: vec![Complex::new(0.0, 0.0); FFT_SIZE],
            ring: vec![0.0; FFT_SIZE],
            write_pos: 0,
            filled: 0,
            sample_rate,
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn analyze(&mut self, samples: &[f32]) -> Vec<f32> {
        for &s in samples {
            self.ring[self.write_pos] = s;
            self.write_pos = (self.write_pos + 1) % FFT_SIZE;
        }
        self.filled = (self.filled + samples.len()).min(FFT_SIZE);

        for (i, buf) in self.buffer.iter_mut().enumerate() {
            let ring_idx = (self.write_pos + i) % FFT_SIZE;
            *buf = Complex::new(self.ring[ring_idx] * self.window[i], 0.0);
        }

        self.fft.process(&mut self.buffer);

        let nyquist = FFT_SIZE / 2;
        let nyquist_f = nyquist as f32;
        let norm = 2.0 / FFT_SIZE as f32;
        let mag = |k: usize| self.buffer[k.min(nyquist)].norm() * norm;

        (0..NUM_BANDS)
            .map(|i| {
                let lo_f = self.bin_edges[i].min(nyquist_f);
                let hi_f = self.bin_edges[i + 1].min(nyquist_f);
                if hi_f - lo_f >= 1.0 {
                    let lo = lo_f.floor() as usize;
                    let hi = (hi_f.ceil() as usize).min(nyquist).max(lo + 1);
                    let sum: f32 = (lo..hi).map(mag).sum();
                    sum / (hi - lo) as f32
                } else {
                    let mid = (lo_f + hi_f) * 0.5;
                    let lo = mid.floor() as usize;
                    let frac = mid - lo as f32;
                    mag(lo) * (1.0 - frac) + mag(lo + 1) * frac
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f32, sr: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sr).sin())
            .collect()
    }

    #[test]
    fn analyze_low_bands_have_no_zero_holes() {
        let sr = 48_000.0;
        let mut a = SpectrumAnalyzer::new(sr);
        let chunk = sine(440.0, sr, 2048);
        a.analyze(&chunk);
        let bands = a.analyze(&chunk);
        let low = &bands[..32];
        assert!(
            low.iter().all(|&v| v > 0.0),
            "lowest 32 bands must all be > 0 (no comb gaps), got: {:?}",
            low
        );
    }

    #[test]
    fn analyze_returns_correct_band_count() {
        let mut a = SpectrumAnalyzer::new(48_000.0);
        let bands = a.analyze(&vec![0.0; 1024]);
        assert_eq!(bands.len(), NUM_BANDS);
    }
}
