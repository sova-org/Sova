/// PolyBLEP (Polynomial Band-Limited Step) anti-aliasing for oscillators
///
/// PolyBLEP is an efficient method to reduce aliasing in oscillators by
/// smoothing discontinuities with polynomial functions.

/// PolyBLEP correction function
/// t: normalized distance from discontinuity (0.0 to 1.0)
#[inline]
fn polyblep_correction(t: f32) -> f32 {
    if t < 1.0 {
        let t2 = t * t;
        if t < 0.5 {
            -t2 + t - 0.25
        } else {
            t2 - t + 0.25
        }
    } else {
        0.0
    }
}

/// PolyBLEP sawtooth oscillator
pub struct PolyBlepSaw {
    phase: f32,
    phase_inc: f32,
    last_output: f32,
}

impl PolyBlepSaw {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            last_output: 0.0,
        }
    }

    /// Set frequency (updates phase increment)
    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        self.phase_inc = frequency / sample_rate;
    }

    /// Generate next sawtooth sample with PolyBLEP anti-aliasing
    pub fn next_sample(&mut self) -> f32 {
        // Basic sawtooth: ramp from -1 to 1
        let mut output = 2.0 * self.phase - 1.0;

        // Apply PolyBLEP correction at discontinuity
        if self.phase_inc > 0.0 {
            // Calculate distance to discontinuity
            let t = self.phase / self.phase_inc;

            // Apply correction if near discontinuity
            if t < 1.0 {
                output += polyblep_correction(t);
            }

            // Check for wraparound discontinuity
            let t_wrap = (self.phase - 1.0) / self.phase_inc;
            if t_wrap < 0.0 && t_wrap >= -1.0 {
                output += polyblep_correction(-t_wrap);
            }
        }

        // Advance phase
        self.phase += self.phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        self.last_output = output;
        output
    }

    /// Reset phase to zero
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
        self.last_output = 0.0;
    }

    /// Get current phase [0.0, 1.0)
    pub fn get_phase(&self) -> f32 {
        self.phase
    }
}

impl Default for PolyBlepSaw {
    fn default() -> Self {
        Self::new()
    }
}
