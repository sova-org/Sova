use std::f32::consts::PI;

/// Biquad filter implementation based on the Audio EQ Cookbook
/// by Robert Bristow-Johnson
///
/// Direct Form II Transposed implementation (more efficient):
/// out = b0*x[n] + w[0]
/// w[0] = b1*x[n] - a1*out + w[1]  
/// w[1] = b2*x[n] - a2*out

#[derive(Clone, Copy, Debug)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    Peak,
    LowShelf,
    HighShelf,
}

/// Biquad filter state for one channel
#[derive(Clone, Copy, Debug)]
pub struct BiquadFilter {
    // Coefficients (normalized by a0)
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    // State variables (Direct Form II Transposed)
    w0: f32,
    w1: f32,
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl BiquadFilter {
    pub fn new() -> Self {
        Self {
            // Unity gain pass-through
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            // State (Direct Form II Transposed)
            w0: 0.0,
            w1: 0.0,
        }
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.w0 = 0.0;
        self.w1 = 0.0;
    }

    /// Process one sample (Direct Form II Transposed)
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.w0;
        self.w0 = self.b1 * input - self.a1 * output + self.w1;
        self.w1 = self.b2 * input - self.a2 * output;
        output
    }

    /// Set coefficients for low-pass filter
    pub fn set_lowpass(&mut self, freq: f32, q: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        self.set_coefficients(b0, b1, b2, a0, a1, a2);
    }

    /// Set coefficients for high-pass filter
    pub fn set_highpass(&mut self, freq: f32, q: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 + cos_omega) / 2.0;
        let b1 = -(1.0 + cos_omega);
        let b2 = (1.0 + cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        self.set_coefficients(b0, b1, b2, a0, a1, a2);
    }

    /// Set coefficients for band-pass filter (constant skirt gain)
    pub fn set_bandpass(&mut self, freq: f32, q: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        self.set_coefficients(b0, b1, b2, a0, a1, a2);
    }

    /// Set coefficients for notch filter
    pub fn set_notch(&mut self, freq: f32, q: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        self.set_coefficients(b0, b1, b2, a0, a1, a2);
    }

    /// Set coefficients for peaking EQ
    pub fn set_peak(&mut self, freq: f32, q: f32, gain_db: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        let a = 10.0_f32.powf(gain_db / 40.0);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        self.set_coefficients(b0, b1, b2, a0, a1, a2);
    }

    /// Set coefficients for low shelf
    pub fn set_lowshelf(&mut self, freq: f32, q: f32, gain_db: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let a = 10.0_f32.powf(gain_db / 40.0);
        let beta = a.sqrt() / q;

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + beta * sin_omega);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - beta * sin_omega);
        let a0 = (a + 1.0) + (a - 1.0) * cos_omega + beta * sin_omega;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) + (a - 1.0) * cos_omega - beta * sin_omega;

        self.set_coefficients(b0, b1, b2, a0, a1, a2);
    }

    /// Set coefficients for high shelf
    pub fn set_highshelf(&mut self, freq: f32, q: f32, gain_db: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let a = 10.0_f32.powf(gain_db / 40.0);
        let beta = a.sqrt() / q;

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_omega + beta * sin_omega);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_omega - beta * sin_omega);
        let a0 = (a + 1.0) - (a - 1.0) * cos_omega + beta * sin_omega;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) - (a - 1.0) * cos_omega - beta * sin_omega;

        self.set_coefficients(b0, b1, b2, a0, a1, a2);
    }

    /// Set raw coefficients and normalize by a0
    fn set_coefficients(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        let inv_a0 = 1.0 / a0;
        self.b0 = b0 * inv_a0;
        self.b1 = b1 * inv_a0;
        self.b2 = b2 * inv_a0;
        self.a1 = a1 * inv_a0;
        self.a2 = a2 * inv_a0;
    }
}

/// Stereo biquad filter (processes two channels)
#[derive(Clone, Copy, Debug)]
pub struct StereoBiquadFilter {
    left: BiquadFilter,
    right: BiquadFilter,
}

impl Default for StereoBiquadFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl StereoBiquadFilter {
    pub fn new() -> Self {
        Self {
            left: BiquadFilter::new(),
            right: BiquadFilter::new(),
        }
    }

    /// Reset both channels
    pub fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }

    /// Process stereo sample
    #[inline]
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        (self.left.process(left_in), self.right.process(right_in))
    }

    /// Set filter type and parameters for both channels
    pub fn set_filter(
        &mut self,
        filter_type: FilterType,
        freq: f32,
        q: f32,
        gain_db: f32,
        sample_rate: f32,
    ) {
        match filter_type {
            FilterType::LowPass => {
                self.left.set_lowpass(freq, q, sample_rate);
                self.right.set_lowpass(freq, q, sample_rate);
            }
            FilterType::HighPass => {
                self.left.set_highpass(freq, q, sample_rate);
                self.right.set_highpass(freq, q, sample_rate);
            }
            FilterType::BandPass => {
                self.left.set_bandpass(freq, q, sample_rate);
                self.right.set_bandpass(freq, q, sample_rate);
            }
            FilterType::Notch => {
                self.left.set_notch(freq, q, sample_rate);
                self.right.set_notch(freq, q, sample_rate);
            }
            FilterType::Peak => {
                self.left.set_peak(freq, q, gain_db, sample_rate);
                self.right.set_peak(freq, q, gain_db, sample_rate);
            }
            FilterType::LowShelf => {
                self.left.set_lowshelf(freq, q, gain_db, sample_rate);
                self.right.set_lowshelf(freq, q, gain_db, sample_rate);
            }
            FilterType::HighShelf => {
                self.left.set_highshelf(freq, q, gain_db, sample_rate);
                self.right.set_highshelf(freq, q, gain_db, sample_rate);
            }
        }
    }
}
