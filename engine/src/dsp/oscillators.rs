use crate::dsp::math::{freq_to_phase_inc, wrap_phase};
use crate::dsp::polyblep::PolyBlepSaw;
use crate::dsp::tables::table_sin;

/// Efficient sawtooth oscillator with PolyBLEP anti-aliasing
///
/// This is a basic building block that other oscillators can use.
/// Caches phase increment for performance.
pub struct SawOscillator {
    osc: PolyBlepSaw,
    phase_inc: f32,
    sample_rate: f32,
}

impl SawOscillator {
    pub fn new() -> Self {
        Self {
            osc: PolyBlepSaw::new(),
            phase_inc: 0.0,
            sample_rate: 0.0,
        }
    }

    /// Set frequency (only recalculates phase increment when needed)
    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        let new_phase_inc = freq_to_phase_inc(frequency, sample_rate);
        if self.phase_inc != new_phase_inc || self.sample_rate != sample_rate {
            self.phase_inc = new_phase_inc;
            self.sample_rate = sample_rate;
            self.osc.set_frequency(frequency, sample_rate);
        }
    }

    /// Generate next sample
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        self.osc.next_sample()
    }

    /// Reset phase
    pub fn reset_phase(&mut self) {
        self.osc.reset_phase();
    }

    /// Get current phase
    pub fn get_phase(&self) -> f32 {
        self.osc.get_phase()
    }
}

impl Default for SawOscillator {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple LFO (Low Frequency Oscillator) using efficient sine table
pub struct TableLfo {
    phase: f32,
    phase_inc: f32,
}

impl TableLfo {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
        }
    }

    /// Set LFO frequency (only recalculates when needed)
    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        let new_phase_inc = freq_to_phase_inc(frequency, sample_rate);
        if self.phase_inc != new_phase_inc {
            self.phase_inc = new_phase_inc;
        }
    }

    /// Generate next LFO sample
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let output = table_sin(self.phase);
        self.phase = wrap_phase(self.phase + self.phase_inc);
        output
    }

    /// Reset phase
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }
}

impl Default for TableLfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Efficient sine oscillator using wavetable lookup
pub struct SineOscillator {
    phase: f32,
    phase_inc: f32,
    sample_rate: f32,
}

impl SineOscillator {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            sample_rate: 0.0,
        }
    }

    /// Set frequency (only recalculates phase increment when needed)
    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        let new_phase_inc = freq_to_phase_inc(frequency, sample_rate);
        if self.phase_inc != new_phase_inc || self.sample_rate != sample_rate {
            self.phase_inc = new_phase_inc;
            self.sample_rate = sample_rate;
        }
    }

    /// Generate next sample
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let output = table_sin(self.phase);
        self.phase = wrap_phase(self.phase + self.phase_inc);
        output
    }

    /// Reset phase
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    /// Get current phase
    pub fn get_phase(&self) -> f32 {
        self.phase
    }
}

impl Default for SineOscillator {
    fn default() -> Self {
        Self::new()
    }
}

/// Efficient triangle oscillator using integrated sawtooth approach
pub struct TriangleOscillator {
    phase: f32,
    phase_inc: f32,
    sample_rate: f32,
}

impl TriangleOscillator {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            sample_rate: 0.0,
        }
    }

    /// Set frequency (only recalculates phase increment when needed)
    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        let new_phase_inc = freq_to_phase_inc(frequency, sample_rate);
        if self.phase_inc != new_phase_inc || self.sample_rate != sample_rate {
            self.phase_inc = new_phase_inc;
            self.sample_rate = sample_rate;
        }
    }

    /// Generate next sample
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        // Triangle wave: integrate a square wave
        // Convert phase [0,1) to triangle [-1,1]
        let output = if self.phase < 0.5 {
            4.0 * self.phase - 1.0
        } else {
            3.0 - 4.0 * self.phase
        };

        self.phase = wrap_phase(self.phase + self.phase_inc);
        output
    }

    /// Reset phase
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    /// Get current phase
    pub fn get_phase(&self) -> f32 {
        self.phase
    }
}

impl Default for TriangleOscillator {
    fn default() -> Self {
        Self::new()
    }
}

/// High-quality white noise generator using LCG algorithm
pub struct NoiseGenerator {
    state: u32,
}

impl NoiseGenerator {
    pub fn new() -> Self {
        Self {
            state: 1, // Non-zero seed
        }
    }

    /// Seed the noise generator with a specific value
    pub fn seed(&mut self, seed: u32) {
        self.state = if seed == 0 { 1 } else { seed };
    }

    /// Generate next noise sample
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        // Linear Congruential Generator (LCG) - same constants as used in many implementations
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);

        // Convert to float [-1.0, 1.0)
        let normalized = (self.state as i32) as f32 / 2147483648.0;
        normalized
    }

    /// Reset generator state
    pub fn reset(&mut self) {
        self.state = 1;
    }
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Efficient square/pulse wave oscillator with adjustable duty cycle
pub struct SquareOscillator {
    phase: f32,
    phase_inc: f32,
    sample_rate: f32,
    duty_cycle: f32,
}

impl SquareOscillator {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            sample_rate: 0.0,
            duty_cycle: 0.5, // 50% duty cycle = square wave
        }
    }

    /// Set frequency (only recalculates phase increment when needed)
    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        let new_phase_inc = freq_to_phase_inc(frequency, sample_rate);
        if self.phase_inc != new_phase_inc || self.sample_rate != sample_rate {
            self.phase_inc = new_phase_inc;
            self.sample_rate = sample_rate;
        }
    }

    /// Set duty cycle (0.0 to 1.0, where 0.5 = square wave)
    pub fn set_duty_cycle(&mut self, duty_cycle: f32) {
        self.duty_cycle = duty_cycle.clamp(0.01, 0.99);
    }

    /// Generate next sample
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        // Square wave based on duty cycle comparison
        let output = if self.phase < self.duty_cycle {
            1.0
        } else {
            -1.0
        };

        self.phase = wrap_phase(self.phase + self.phase_inc);
        output
    }

    /// Reset phase
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    /// Get current phase
    pub fn get_phase(&self) -> f32 {
        self.phase
    }
}

impl Default for SquareOscillator {
    fn default() -> Self {
        Self::new()
    }
}
