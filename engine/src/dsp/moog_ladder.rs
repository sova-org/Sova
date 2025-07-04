///
/// Moog VCF, Voltage Controlled Filter
/// Based on Tim Stilson's implementation with improvements by Aaron Krajeski
///
/// Original C++ implementation by Aaron Krajeski
/// Source: https://github.com/ddiakopoulos/MoogLadders
/// Ported to Rust with permission (public domain as stated by Aaron Krajeski, 2018)
///
/// Features:
/// - Audio-rate cutoff and resonance updates
/// - 4-pole ladder filter design
/// - Clean implementation without drive/saturation
///
use std::f32::consts::PI;

/// Moog ladder filter implementation
/// Classic 4-pole analog-modeled filter
#[derive(Clone, Copy, Debug)]
pub struct MoogLadder {
    cutoff: f32,
    resonance: f32,
    sample_rate: f32,

    // Filter state
    state: [f32; 5], // 5 states: input + 4 filter stages
    delay: [f32; 4], // Delay line for each stage

    // Derived parameters
    g: f32,      // Filter coefficient
    g_res: f32,  // Resonance coefficient
    g_comp: f32, // Compensation coefficient
    wc: f32,     // Normalized cutoff frequency
}

impl Default for MoogLadder {
    fn default() -> Self {
        Self::new()
    }
}

impl MoogLadder {
    pub fn new() -> Self {
        let mut filter = Self {
            cutoff: 1000.0,
            resonance: 0.1,
            sample_rate: 44100.0,
            state: [0.0; 5],
            delay: [0.0; 4],
            g: 0.0,
            g_res: 0.0,
            g_comp: 1.0, // As in original
            wc: 0.0,
        };
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.1);
        filter
    }

    /// Initialize the filter with sample rate
    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.reset();
        // Recalculate coefficients for new sample rate
        let cutoff = self.cutoff;
        let resonance = self.resonance;
        self.set_cutoff(cutoff);
        self.set_resonance(resonance);
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.state = [0.0; 5];
        self.delay = [0.0; 4];
    }

    /// Set cutoff frequency in Hz (exact Krajeski implementation)
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff.clamp(20.0, self.sample_rate * 0.45);
        self.wc = 2.0 * PI * self.cutoff / self.sample_rate;
        self.g = 0.9892 * self.wc - 0.4342 * self.wc.powi(2) + 0.1381 * self.wc.powi(3)
            - 0.0202 * self.wc.powi(4);
        // Update resonance coefficient since it depends on wc
        self.update_resonance();
    }

    /// Set resonance (0.0 to 4.0) (exact Krajeski implementation)
    /// Values above 1.0 will start self-oscillating
    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(0.0, 4.0);
        self.update_resonance();
    }

    /// Update resonance coefficient (depends on both resonance and wc)
    fn update_resonance(&mut self) {
        self.g_res = self.resonance
            * (1.0029 + 0.0526 * self.wc - 0.926 * self.wc.powi(2) + 0.0218 * self.wc.powi(3));
    }

    /// Process one sample (exact Krajeski implementation)
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        // Input stage with tanh nonlinearity and resonance feedback
        self.state[0] = (input - 4.0 * self.g_res * (self.state[4] - self.g_comp * input)).tanh();

        // 4-pole ladder filter processing with clamping
        for i in 0..4 {
            self.state[i + 1] = (self.g
                * (0.3 / 1.3 * self.state[i] + 1.0 / 1.3 * self.delay[i] - self.state[i + 1])
                + self.state[i + 1])
                .clamp(-1e30, 1e30);
            self.delay[i] = self.state[i];
        }

        // Output is the final stage
        self.state[4]
    }
}

/// Stereo Moog ladder filter
#[derive(Clone, Copy, Debug)]
pub struct StereoMoogLadder {
    left: MoogLadder,
    right: MoogLadder,
}

impl Default for StereoMoogLadder {
    fn default() -> Self {
        Self::new()
    }
}

impl StereoMoogLadder {
    pub fn new() -> Self {
        Self {
            left: MoogLadder::new(),
            right: MoogLadder::new(),
        }
    }

    /// Initialize both channels
    pub fn init(&mut self, sample_rate: f32) {
        self.left.init(sample_rate);
        self.right.init(sample_rate);
    }

    /// Reset both channels
    pub fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }

    /// Set cutoff for both channels
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.left.set_cutoff(cutoff);
        self.right.set_cutoff(cutoff);
    }

    /// Set resonance for both channels
    pub fn set_resonance(&mut self, resonance: f32) {
        self.left.set_resonance(resonance);
        self.right.set_resonance(resonance);
    }

    /// Process stereo sample
    #[inline]
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        (self.left.process(left_in), self.right.process(right_in))
    }
}
