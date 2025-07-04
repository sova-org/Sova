use crate::dsp::math::lerp;
use std::f32::consts::PI;

/// Size of wavetables (power of 2 for efficient indexing)
pub const WAVETABLE_SIZE: usize = 2048;
pub const WAVETABLE_MASK: usize = WAVETABLE_SIZE - 1;

/// Pre-computed sine wave table for efficient oscillators
pub struct SineTable {
    table: [f32; WAVETABLE_SIZE],
}

impl SineTable {
    pub fn new() -> Self {
        let mut table = [0.0; WAVETABLE_SIZE];

        for i in 0..WAVETABLE_SIZE {
            let phase = (i as f32) / (WAVETABLE_SIZE as f32) * 2.0 * PI;
            table[i] = phase.sin();
        }

        Self { table }
    }

    /// Get sine value with linear interpolation
    /// Assumes phase is already wrapped to [0, 1) for performance
    #[inline]
    pub fn sin(&self, phase: f32) -> f32 {
        let index_f = phase * WAVETABLE_SIZE as f32;
        let index = index_f as usize & WAVETABLE_MASK;
        let frac = index_f - index as f32;

        let sample1 = self.table[index];
        let sample2 = self.table[(index + 1) & WAVETABLE_MASK];

        lerp(sample1, sample2, frac)
    }

    /// Get cosine value (phase shifted sine)
    #[inline]
    pub fn cos(&self, phase: f32) -> f32 {
        self.sin(phase + 0.25)
    }
}

impl Default for SineTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Lazy-initialized global sine table
use std::sync::OnceLock;
static SINE_TABLE: OnceLock<SineTable> = OnceLock::new();

/// Get reference to global sine table
pub fn get_sine_table() -> &'static SineTable {
    SINE_TABLE.get_or_init(|| SineTable::new())
}

/// Fast sine using global table
#[inline]
pub fn table_sin(phase: f32) -> f32 {
    get_sine_table().sin(phase)
}

/// Fast cosine using global table
#[inline]
pub fn table_cos(phase: f32) -> f32 {
    get_sine_table().cos(phase)
}
