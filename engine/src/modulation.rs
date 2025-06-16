use crate::memory::MemoryPool;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum WaveShape {
    Sine,
    Triangle,
    Saw,
    Square,
    Noise,
}

#[derive(Clone, Copy, Debug)]
pub enum CurveType {
    Linear,
    Exp,
    Log,
    Quad,
    Cubic,
}

#[derive(Clone, Copy, Debug)]
pub enum Modulation {
    Static(f32),
    Osc {
        base: f32,
        depth: f32,
        rate: f32,
        phase: f32,
        shape: WaveShape,
        duration: f32,
        elapsed: f32,
    },
    Env {
        start: f32,
        end: f32,
        curve: CurveType,
        duration: f32,
        elapsed: f32,
    },
    Ramp {
        start: f32,
        end: f32,
        duration: f32,
        elapsed: f32,
        curve: CurveType,
    },
    Seq {
        values: [f32; 8],
        len: u8,
        rate: f32,
        phase: f32,
        duration: f32,
        elapsed: f32,
    },
}

impl Modulation {
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

    pub fn parse(input: &str) -> Self {
        Self::parse_with_pool(input, &Arc::new(MemoryPool::new(1024)))
    }
}

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

#[inline]
fn taylor_sin(x: f32) -> f32 {
    let x2 = x * x;
    x * (1.0 - x2 * (1.0 / 6.0 - x2 * (1.0 / 120.0 - x2 * 1.0 / 5040.0)))
}

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

#[inline]
fn parse_f32(s: &str) -> f32 {
    s.parse().unwrap_or(0.0)
}

#[inline]
fn parse_f32_or_default(s: &str, default: f32) -> f32 {
    s.parse().unwrap_or(default)
}

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
