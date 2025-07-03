use crate::dsp::math::lerp;

include!(concat!(env!("OUT_DIR"), "/wavetables.rs"));

pub struct WavetableOscillator {
    phase: f32,
    phase_inc: f32,
    sample_rate: f32,
    wavetable_index: f32,
    wavetable_index_int: usize,
    wavetable_frac: f32,
    wavetables_loaded: bool,
}

impl WavetableOscillator {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            sample_rate: 0.0,
            wavetable_index: 0.0,
            wavetable_index_int: 0,
            wavetable_frac: 0.0,
            wavetables_loaded: false,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        let new_phase_inc = frequency / sample_rate;
        if self.phase_inc != new_phase_inc || self.sample_rate != sample_rate {
            self.phase_inc = new_phase_inc;
            self.sample_rate = sample_rate;
        }
    }

    pub fn set_wavetable_index(&mut self, index: f32) {
        let clamped_index = index.clamp(0.0, (NUM_WAVETABLES - 1) as f32);
        if self.wavetable_index != clamped_index {
            self.wavetable_index = clamped_index;
            self.wavetable_index_int = clamped_index as usize;
            self.wavetable_frac = clamped_index - self.wavetable_index_int as f32;
        }
    }

    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        if !self.wavetables_loaded {
            self.wavetables_loaded = true;
        }

        let tables = get_wavetables();
        if tables.is_empty() {
            return 0.0;
        }

        let table1 = tables[self.wavetable_index_int];
        let table2 = tables[(self.wavetable_index_int + 1) % tables.len()];

        let phase_scaled = self.phase * WAVETABLE_SIZE as f32;
        let index = phase_scaled as usize;
        let frac = phase_scaled - index as f32;

        let sample1_t1 = table1[index % WAVETABLE_SIZE];
        let sample2_t1 = table1[(index + 1) % WAVETABLE_SIZE];
        let interpolated_t1 = lerp(sample1_t1, sample2_t1, frac);

        let sample1_t2 = table2[index % WAVETABLE_SIZE];
        let sample2_t2 = table2[(index + 1) % WAVETABLE_SIZE];
        let interpolated_t2 = lerp(sample1_t2, sample2_t2, frac);

        let output = lerp(interpolated_t1, interpolated_t2, self.wavetable_frac);

        self.phase += self.phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    pub fn get_phase(&self) -> f32 {
        self.phase
    }

    pub fn get_num_wavetables(&self) -> usize {
        NUM_WAVETABLES
    }
}

impl Default for WavetableOscillator {
    fn default() -> Self {
        Self::new()
    }
}