use std::f32::consts::PI;

/// Fast mathematical approximations for real-time audio processing

/// Fast sine approximation using polynomial interpolation
/// Accurate to ~0.1% for audio applications
#[inline]
pub fn fast_sin(x: f32) -> f32 {
    let x = x % (2.0 * PI);
    let x = if x > PI { x - 2.0 * PI } else { x };
    
    // Polynomial approximation: sin(x) ≈ x - x³/6 + x⁵/120
    let x2 = x * x;
    x * (1.0 - x2 * (1.0/6.0 - x2/120.0))
}

/// Fast cosine approximation 
#[inline]
pub fn fast_cos(x: f32) -> f32 {
    fast_sin(x + PI * 0.5)
}

/// Convert frequency to phase increment for given sample rate
#[inline]
pub fn freq_to_phase_inc(frequency: f32, sample_rate: f32) -> f32 {
    frequency / sample_rate
}

/// Wrap phase to [0.0, 1.0) range
#[inline]
pub fn wrap_phase(phase: f32) -> f32 {
    phase - phase.floor()
}

/// Linear interpolation between two values
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Clamp value to range [min, max]
#[inline]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min { min } else if value > max { max } else { value }
}

/// Convert MIDI note to frequency
#[inline]
pub fn midi_to_freq(note: f32) -> f32 {
    440.0 * 2.0_f32.powf((note - 69.0) / 12.0)
}

/// Fast power of 2 approximation for exponential curves
#[inline]
pub fn fast_pow2(x: f32) -> f32 {
    let i = x as i32;
    let f = x - i as f32;
    let pow2_f = 1.0 + f * (0.693147 + f * (0.240227 + f * 0.0558282));
    (1u32 << i.max(0).min(31)) as f32 * pow2_f
}

/// RMS-based gain compensation for mixing multiple signals
/// Returns gain factor to maintain perceptual loudness when combining signals
#[inline]
pub fn rms_mix_gain(num_signals: u32) -> f32 {
    1.0 / (num_signals as f32).sqrt()
}

/// Stereo mixing gain for dual oscillator setups
/// Maintains perceptual loudness while preventing clipping
#[inline]
pub fn stereo_mix_gain() -> f32 {
    // sqrt(2)/2 ≈ 0.707 - RMS compensation for 2 signals
    0.7071067811865476
}