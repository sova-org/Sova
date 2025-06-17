//! Modulation System for Parameter Control
//!
//! This module provides a comprehensive modulation system for controlling audio parameters in real-time.
//!
//! # Types of Modulation
//!
//! ## Static Values
//! The simplest form - a constant value that never changes.
//! ```text
//! Static(440.0) // Always outputs 440.0
//! ```
//!
//! ## Oscillator Modulation (LFO - Low Frequency Oscillator)
//! Cyclical modulation using various waveforms. Essential for vibrato, tremolo, and
//! complex rhythmic effects.
//!
//! **Waveforms Available:**
//! - **Sine**: Smooth, natural modulation. Perfect for vibrato and gentle parameter sweeps
//! - **Triangle**: Linear rise/fall. Good for cyclical automation with sharp direction changes
//! - **Saw**: Linear ramp that resets. Creates "sawtooth" parameter motion
//! - **Square**: Abrupt on/off switching. Useful for rhythmic gating and step changes
//! - **Noise**: Random values. Adds unpredictability and organic variation
//!
//! ```text
//! osc:440.0:50.0:2.0:sine:4.0
//! //  │     │    │   │    └─ Duration (4 seconds)
//! //  │     │    │   └────── Waveform (sine wave)
//! //  │     │    └────────── Rate (2 Hz - 2 cycles per second)
//! //  │     └─────────────── Depth (±50 units of modulation)
//! //  └───────────────────── Base value (center point)
//! ```
//!
//! ## Envelope Modulation
//! One-shot parameter changes from start to end value. Critical for note articulation,
//! filter sweeps, and dramatic parameter changes.
//!
//! ```text
//! env:0.0:1.0:exp:2.0
//! //  │   │   │   └─ Duration (2 seconds)
//! //  │   │   └───── Curve type (exponential)
//! //  │   └─────── End value
//! //  └─────────── Start value
//! ```
//!
//! ## Ramp Modulation
//! Similar to envelopes but with explicit curve control. Ideal for parameter automation
//! and smooth transitions between states.
//!
//! **Curve Types:**
//! - **Linear**: Constant rate of change
//! - **Exp/Quad**: Accelerating change (slow start, fast finish)
//! - **Log**: Decelerating change (fast start, slow finish)
//! - **Cubic**: Extreme acceleration curve
//!
//! ## Sequence Modulation
//! Step through predefined values at a specified rate. Perfect for arpeggios,
//! chord progressions, and rhythmic parameter patterns.
//!
//! ```text
//! seq:440:550:660:880:2.0:8.0
//! //  │              │   └─ Duration (8 seconds total)
//! //  │              └───── Rate (2 steps per second)
//! //  └──────────────────── Values to cycle through
//! ```
//!
//! # Performance Characteristics
//!
//! All modulation calculations are optimized for real-time audio processing:
//! - Fast trigonometric approximations using Taylor series
//! - Efficient random number generation using linear congruential generator
//! - Minimal branching in hot paths
//! - No heap allocations during processing

use crate::memory::MemoryPool;
use std::sync::Arc;

/// Waveform shapes for oscillator-based modulation
///
/// Each shape produces different modulation characteristics:
/// - Sine: Smooth, natural curves
/// - Triangle: Linear rise/fall with sharp peaks
/// - Saw: Linear ramp with abrupt reset
/// - Square: Binary on/off switching
/// - Noise: Pseudo-random values
#[derive(Clone, Copy, Debug)]
pub enum WaveShape {
    Sine,
    Triangle,
    Saw,
    Square,
    Noise,
}

/// Curve types for envelope and ramp modulation
///
/// Controls the interpolation between start and end values:
/// - Linear: Constant rate of change
/// - Exp/Quad: Accelerating (slow → fast)
/// - Log: Decelerating (fast → slow)
/// - Cubic: Extreme acceleration
#[derive(Clone, Copy, Debug)]
pub enum CurveType {
    Linear,
    Exp,
    Log,
    Quad,
    Cubic,
}

/// Core modulation types for real-time parameter control
///
/// Each variant represents a different approach to changing values over time.
/// All modulation types track their own timing state and can be combined
/// or chained for complex parameter automation.
#[derive(Clone, Copy, Debug)]
pub enum Modulation {
    /// Constant value that never changes
    Static(f32),
    /// Oscillating modulation (LFO) with configurable waveform
    Osc {
        /// Center value around which oscillation occurs
        base: f32,
        /// Maximum deviation from base value (±depth)
        depth: f32,
        /// Oscillation frequency in Hz
        rate: f32,
        /// Current phase position (0.0 to 1.0)
        phase: f32,
        /// Waveform shape
        shape: WaveShape,
        /// Total duration (0.0 = infinite)
        duration: f32,
        /// Time elapsed since start
        elapsed: f32,
    },
    /// One-shot envelope from start to end value
    Env {
        /// Initial value
        start: f32,
        /// Final value
        end: f32,
        /// Interpolation curve
        curve: CurveType,
        /// Envelope duration
        duration: f32,
        /// Time elapsed since start
        elapsed: f32,
    },
    /// Linear or curved ramp between two values
    Ramp {
        /// Starting value
        start: f32,
        /// Ending value
        end: f32,
        /// Ramp duration
        duration: f32,
        /// Time elapsed since start
        elapsed: f32,
        /// Interpolation curve
        curve: CurveType,
    },
    /// Step sequencer cycling through predefined values
    Seq {
        /// Array of values to cycle through (max 8)
        values: [f32; 8],
        /// Number of active values in the array
        len: u8,
        /// Step rate in Hz
        rate: f32,
        /// Current sequence position
        phase: f32,
        /// Total sequence duration (0.0 = infinite)
        duration: f32,
        /// Time elapsed since start
        elapsed: f32,
    },
}

impl Modulation {
    /// Updates modulation state and returns current value
    ///
    /// # Arguments
    /// * `dt` - Time delta since last update (in seconds)
    /// * `_envelope_val` - Reserved for future envelope integration
    /// * `rng_state` - Mutable reference to RNG state for noise generation
    ///
    /// # Returns
    /// Current modulated value
    pub fn update(&mut self, dt: f32, _envelope_val: f32, rng_state: &mut u32) -> f32 {
        match self {
            Modulation::Static(value) => *value,

            Modulation::Osc {
                base,
                depth,
                rate,
                phase,
                shape,
                duration,
                elapsed,
            } => {
                *elapsed += dt;
                if *elapsed >= *duration && *duration > 0.0 {
                    return *base;
                }

                *phase += *rate * dt;
                if *phase >= 1.0 {
                    *phase -= 1.0;
                }

                let wave = match shape {
                    WaveShape::Sine => fast_sin(*phase * 2.0 * std::f32::consts::PI),
                    WaveShape::Triangle => 4.0 * (*phase - 0.5).abs() - 1.0,
                    WaveShape::Saw => 2.0 * *phase - 1.0,
                    WaveShape::Square => {
                        if *phase < 0.5 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                    WaveShape::Noise => {
                        *rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                        ((*rng_state >> 16) as f32 / 32768.0) - 1.0
                    }
                };

                *base + wave * *depth
            }

            Modulation::Env {
                start,
                end,
                curve,
                duration,
                elapsed,
            } => {
                *elapsed += dt;
                if *elapsed >= *duration {
                    return *end;
                }

                let t = (*elapsed / *duration).clamp(0.0, 1.0);
                let curved_t = apply_curve(t, curve);

                *start + (*end - *start) * curved_t
            }

            Modulation::Ramp {
                start,
                end,
                duration,
                elapsed,
                curve,
            } => {
                *elapsed += dt;
                if *elapsed >= *duration {
                    return *end;
                }

                let t = (*elapsed / *duration).clamp(0.0, 1.0);
                let curved_t = apply_curve(t, curve);

                *start + (*end - *start) * curved_t
            }

            Modulation::Seq {
                values,
                len,
                rate,
                phase,
                duration,
                elapsed,
            } => {
                *elapsed += dt;
                if *elapsed >= *duration && *duration > 0.0 {
                    return values[0];
                }

                *phase += *rate * dt;
                let idx = (*phase as usize) % (*len as usize);
                if idx < *len as usize {
                    values[idx]
                } else {
                    values[0]
                }
            }
        }
    }

    /// Parses modulation from string format with memory pool
    ///
    /// # String Formats
    /// - Static: "440.0"
    /// - Oscillator: "osc:base:depth:rate:shape:duration"
    /// - Envelope: "env:start:end:curve:duration"
    /// - Ramp: "ramp:start:end:duration:curve"
    /// - Sequence: "seq:val1:val2:...:rate:duration"
    pub fn parse_with_pool(input: &str, _pool: &Arc<MemoryPool>) -> Self {
        let parts: Vec<&str> = input.split(':').collect();
        if parts.len() < 2 {
            return Modulation::Static(parse_f32(input));
        }

        match parts[0] {
            "osc" if parts.len() >= 6 => {
                let base = parse_f32(parts[1]);
                let depth = parse_f32_or_default(parts[2], 50.0);
                let rate = parse_f32_or_default(parts[3], 2.0);
                let shape = parse_wave_shape(parts[4]);
                let duration = parse_f32(parts[5]);

                Modulation::Osc {
                    base,
                    depth,
                    rate,
                    phase: 0.0,
                    shape,
                    duration,
                    elapsed: 0.0,
                }
            }

            "env" if parts.len() >= 5 => {
                let start = parse_f32(parts[1]);
                let end = parse_f32_or_default(parts[2], 1.0);
                let curve = parse_curve_type(parts[3]);
                let duration = parse_f32_or_default(parts[4], 1.0);

                Modulation::Env {
                    start,
                    end,
                    curve,
                    duration,
                    elapsed: 0.0,
                }
            }

            "ramp" if parts.len() >= 5 => {
                let start = parse_f32(parts[1]);
                let end = parse_f32_or_default(parts[2], 1.0);
                let duration = parse_f32_or_default(parts[3], 1.0);
                let curve = parse_curve_type(parts[4]);

                Modulation::Ramp {
                    start,
                    end,
                    duration,
                    elapsed: 0.0,
                    curve,
                }
            }

            "seq" if parts.len() >= 4 => {
                let mut values = [0.0; 8];
                let mut len = 0u8;

                let mut i = 1;
                while i < parts.len() - 2 && len < 8 {
                    values[len as usize] = parse_f32(parts[i]);
                    len += 1;
                    i += 1;
                }

                let rate = parse_f32_or_default(parts[parts.len() - 2], 1.0);
                let duration = parse_f32(parts[parts.len() - 1]);

                Modulation::Seq {
                    values,
                    len,
                    rate,
                    phase: 0.0,
                    duration,
                    elapsed: 0.0,
                }
            }

            _ => Modulation::Static(parse_f32(input)),
        }
    }

    /// Convenience method for parsing without explicit memory pool
    pub fn parse(input: &str) -> Self {
        Self::parse_with_pool(input, &Arc::new(MemoryPool::new(1024)))
    }
}

/// Fast sine approximation using Taylor series
///
/// Optimized for real-time audio processing with acceptable precision.
/// Uses polynomial approximation within each quadrant for efficiency.
#[inline]
fn fast_sin(x: f32) -> f32 {
    let x_norm = x % (2.0 * std::f32::consts::PI);
    let x_abs = x_norm.abs();

    if x_abs <= std::f32::consts::FRAC_PI_2 {
        taylor_sin(x_norm)
    } else if x_abs <= std::f32::consts::PI {
        taylor_sin(std::f32::consts::PI - x_norm)
    } else if x_abs <= 3.0 * std::f32::consts::FRAC_PI_2 {
        -taylor_sin(x_norm - std::f32::consts::PI)
    } else {
        -taylor_sin(2.0 * std::f32::consts::PI - x_norm)
    }
}

/// Taylor series sine approximation for small angles
///
/// More accurate than lookup tables for the range [-π/2, π/2]
#[inline]
fn taylor_sin(x: f32) -> f32 {
    let x2 = x * x;
    x * (1.0 - x2 * (1.0 / 6.0 - x2 * (1.0 / 120.0 - x2 * 1.0 / 5040.0)))
}

/// Applies curve transformation to linear interpolation parameter
///
/// # Arguments
/// * `t` - Linear parameter (0.0 to 1.0)
/// * `curve` - Curve type to apply
///
/// # Returns
/// Curved parameter for non-linear interpolation
#[inline]
fn apply_curve(t: f32, curve: &CurveType) -> f32 {
    match curve {
        CurveType::Linear => t,
        CurveType::Exp => t * t,
        CurveType::Log => t.sqrt(),
        CurveType::Quad => t * t,
        CurveType::Cubic => t * t * t,
    }
}

/// Parses string to f32, defaulting to 0.0 on error
#[inline]
fn parse_f32(s: &str) -> f32 {
    s.parse().unwrap_or(0.0)
}

/// Parses string to f32 with custom default value
#[inline]
fn parse_f32_or_default(s: &str, default: f32) -> f32 {
    s.parse().unwrap_or(default)
}

/// Parses string to WaveShape enum, defaulting to Sine
#[inline]
fn parse_wave_shape(s: &str) -> WaveShape {
    match s {
        "sine" => WaveShape::Sine,
        "triangle" => WaveShape::Triangle,
        "saw" => WaveShape::Saw,
        "square" => WaveShape::Square,
        "noise" => WaveShape::Noise,
        _ => WaveShape::Sine,
    }
}

/// Parses string to CurveType enum, defaulting to Linear
#[inline]
fn parse_curve_type(s: &str) -> CurveType {
    match s {
        "exp" => CurveType::Exp,
        "log" => CurveType::Log,
        "quad" => CurveType::Quad,
        "cubic" => CurveType::Cubic,
        _ => CurveType::Linear,
    }
}
