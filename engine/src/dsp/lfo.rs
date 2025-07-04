use crate::dsp::tables::table_sin;

#[derive(Clone)]
pub struct LFO {
    pub phase: f32,
    phase_increment: f32,
}

impl LFO {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_increment: 0.0,
        }
    }

    pub fn new_with_random_phase() -> Self {
        // Simple pseudo-random phase initialization using stack address
        let stack_var = 42u32;
        let random_phase =
            ((&stack_var as *const u32 as usize) as f32).fract() * std::f32::consts::TAU;
        Self {
            phase: random_phase,
            phase_increment: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        self.phase_increment = frequency * std::f32::consts::TAU / sample_rate;
    }

    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let output = table_sin(self.phase);
        self.phase += self.phase_increment;

        // Wrap phase to avoid precision loss
        if self.phase >= std::f32::consts::TAU {
            self.phase -= std::f32::consts::TAU;
        }

        output
    }

    #[inline]
    pub fn next_sample_range(&mut self, min: f32, max: f32) -> f32 {
        let sine_value = self.next_sample(); // -1.0 to 1.0
        // Map from [-1, 1] to [min, max]
        min + (sine_value + 1.0) * 0.5 * (max - min)
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}
