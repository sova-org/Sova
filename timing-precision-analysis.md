# Floating-Point Precision Issues in BuboCore Timing

## Critical Problem Areas

### 1. **Cumulative Beat Accumulation** (Most Critical)
**Location**: `schedule.rs:178-250` in `frame_index()`

```rust
let mut cumulative_beats_in_line = 0.0;  // ❌ Accumulates errors
for frame_idx_in_range in 0..effective_num_frames {
    // ... calculations ...
    cumulative_beats_in_line += total_frame_len_beats;  // Each addition loses precision
}
```

**Problem**: Classic floating-point error accumulation. Each frame adds a small number to a growing sum, compounding precision loss over time.

**Impact**: After thousands of frames, timing can drift by several milliseconds.

### 2. **Conversion Round-Trips** 
**Location**: `clock.rs:323-346`

```rust
// Events get converted back and forth repeatedly:
beats → micros → beats → micros → beats
```

**Problem**: Each conversion introduces rounding errors that accumulate.

**Impact**: Sub-millisecond timing drift accumulates over session duration.

### 3. **Modulo on Large Numbers**
**Location**: `link.rs:40`, `schedule.rs:167`

```rust
beat % self.quantum  // ❌ Less precise as beat grows
let beat_in_effective_loop = current_absolute_beat % effective_loop_length_beats;
```

**Problem**: As songs run longer, beat numbers grow large. Floating-point precision decreases with magnitude, making modulo operations less accurate.

**Impact**: Phase drift increases over time, especially in long sessions.

### 4. **Speed Factor Divisions**
**Location**: `schedule.rs:188`

```rust
let single_rep_len_beats = line.frame_len(absolute_frame_index) / speed_factor;
```

**Problem**: Non-power-of-2 speed factors (e.g., 1.33, 0.75) cannot be represented exactly in binary floating-point.

**Impact**: Timing drift proportional to speed factor precision error.

### 5. **Loop Iteration Scaling**
**Location**: `schedule.rs:219`

```rust
let absolute_beat_at_loop_start = loop_iteration as f64 * effective_loop_length_beats;
```

**Problem**: Large loop iterations multiply accumulated precision errors.

**Impact**: Timing becomes less accurate in later loop iterations.

## Solutions by Priority

### 🔴 **Critical Fixes**

#### 1. Fixed-Point Cumulative Calculations
Replace floating-point accumulation with integer arithmetic:

```rust
// Instead of:
let mut cumulative_beats_in_line = 0.0;
cumulative_beats_in_line += total_frame_len_beats;

// Use fixed-point (e.g., microseconds):
let mut cumulative_micros_in_line = 0u64;
cumulative_micros_in_line += clock.beats_to_micros(total_frame_len_beats);
```

#### 2. Integer Beat Tracking
Use Link's native microsecond timeline as the source of truth:

```rust
// Track position in microseconds, derive beats only when needed
struct PositionTracker {
    current_micros: u64,
    loop_start_micros: u64,
    loop_length_micros: u64,
}

impl PositionTracker {
    fn current_beat(&self, clock: &Clock) -> f64 {
        clock.beat_at_date(self.current_micros)
    }
    
    fn phase_in_loop(&self) -> f64 {
        let elapsed_micros = self.current_micros - self.loop_start_micros;
        (elapsed_micros as f64) / (self.loop_length_micros as f64)
    }
}
```

### 🟡 **High Impact Fixes**

#### 3. Reduce Conversion Frequency
Cache converted values and minimize round-trips:

```rust
struct CachedTiming {
    last_tempo: f64,
    micros_per_beat: f64,  // Cache the conversion ratio
    
    fn beats_to_micros_cached(&mut self, beats: f64, current_tempo: f64) -> u64 {
        if self.last_tempo != current_tempo {
            self.micros_per_beat = 60_000_000.0 / current_tempo;
            self.last_tempo = current_tempo;
        }
        (beats * self.micros_per_beat) as u64
    }
}
```

#### 4. High-Precision Critical Paths
Use `f64` instead of `f32` for critical timing calculations, or even 128-bit fixed-point for ultra-precision:

```rust
type PreciseBeat = f64;  // Or even better: fixed-point u128
type PreciseTime = u64;  // Microseconds as integers
```

### 🟢 **Medium Impact Fixes**

#### 5. Periodic Re-synchronization
Re-synchronize with Link's authoritative timeline periodically:

```rust
if frame_count % 1000 == 0 {  // Every ~20ms at 48kHz
    // Re-sync from Link instead of accumulating
    let authoritative_beat = clock.beat();
    self.reset_position_from_beat(authoritative_beat);
}
```

#### 6. Speed Factor Optimization
Pre-compute reciprocals and use rational approximations for common speed factors:

```rust
// Pre-compute common speed factors as rational numbers
const SPEED_FACTORS: &[(f64, (u64, u64))] = &[
    (0.5, (1, 2)),
    (0.75, (3, 4)),
    (1.33, (4, 3)),  // Approximate 4/3
    (1.5, (3, 2)),
];

fn precise_speed_division(length: f64, speed: f64) -> f64 {
    if let Some((num, den)) = SPEED_FACTORS.iter()
        .find(|(s, _)| (s - speed).abs() < 1e-10)
        .map(|(_, rational)| rational) 
    {
        length * (*den as f64) / (*num as f64)  // Exact rational arithmetic
    } else {
        length / speed  // Fallback to floating-point
    }
}
```

## Expected Improvements

- **Immediate**: Sub-millisecond precision maintained over hours of playback
- **Long-term**: Elimination of gradual timing drift in complex arrangements
- **Collaborative**: Perfect sync with Ableton even in extended sessions

## Implementation Priority

1. **Start with**: Fixed-point cumulative calculations (biggest impact)
2. **Next**: Integer beat tracking architecture 
3. **Then**: Reduce conversion frequency
4. **Finally**: Speed factor and re-sync optimizations

The cumulative beat accumulation fix alone should eliminate most of the drift issues you're experiencing with Ableton Link alignment.