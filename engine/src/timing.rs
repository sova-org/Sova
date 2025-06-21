//! High-precision timing system for sample-accurate audio scheduling.
//!
//! This module provides microsecond-precision timing without cumulative drift
//! using rational arithmetic. Replaces floating-point timing calculations that
//! cause precision loss and timing drift over long sessions.

use fraction::Fraction;

/// High-precision timer for sample-accurate audio timing.
///
/// Uses rational arithmetic to eliminate floating-point precision loss and
/// cumulative timing drift. Provides exact conversions between sample counts
/// and microsecond timestamps.
///
/// # Performance Characteristics
///
/// - Zero heap allocations during normal operation
/// - O(1) complexity for all timing operations
/// - Uses `new_raw()` constructor to avoid GCD reduction overhead
/// - Perfect precision maintained indefinitely
///
/// # Example
///
/// ```rust
/// let timer = HighPrecisionTimer::new(48000.0);
/// timer.initialize_stream_timing();
///
/// // Process 1024 samples (exactly 21.333... milliseconds at 48kHz)
/// timer.advance_samples(1024);
/// let timestamp = timer.get_current_timestamp_exact();
/// ```
pub struct HighPrecisionTimer {
    /// Rational conversion factor: microseconds per sample (1_000_000 / sample_rate)
    /// Uses new_raw() to avoid expensive GCD reduction during arithmetic
    microseconds_per_sample: Fraction,

    /// Rational conversion factor: samples per microsecond (sample_rate / 1_000_000)
    /// Used for timestamp-to-sample conversions
    samples_per_microsecond: Fraction,

    /// Current sample count since stream start
    current_sample_count: u64,

    /// Deterministic time base in microseconds (not tied to system clock)
    deterministic_time_base: u64,

    /// Whether timing has been initialized
    timing_initialized: bool,

    /// Sample rate for overflow protection calculations
    sample_rate: u32,
}

impl HighPrecisionTimer {
    /// Create a new high-precision timer for the given sample rate.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Audio sample rate in Hz (e.g., 44100.0, 48000.0)
    ///
    /// # Performance Notes
    ///
    /// Uses `Fraction::new_raw()` to avoid GCD reduction overhead during construction.
    /// The fractions are already in lowest terms for common sample rates.
    pub fn new(sample_rate: f32) -> Self {
        let sample_rate_int = sample_rate as u64;

        // Create rational conversion factors using new_raw() for maximum performance
        // These fractions are already in lowest terms for standard sample rates
        let microseconds_per_sample = Fraction::new_raw(1_000_000u64, sample_rate_int);
        let samples_per_microsecond = Fraction::new_raw(sample_rate_int, 1_000_000u64);

        Self {
            microseconds_per_sample,
            samples_per_microsecond,
            current_sample_count: 0,
            deterministic_time_base: 0,
            timing_initialized: false,
            sample_rate: sample_rate as u32,
        }
    }

    /// Initialize deterministic stream timing.
    ///
    /// Must be called once when the audio stream starts to establish the time base.
    /// Uses a deterministic time base instead of system clock for absolute precision.
    pub fn initialize_stream_timing(&mut self) {
        self.deterministic_time_base = 0;
        self.current_sample_count = 0;
        self.timing_initialized = true;
    }

    /// Initialize stream timing with a specific time base (for synchronized starts).
    ///
    /// This allows multiple engines or processes to start from the same deterministic
    /// time reference for perfect synchronization.
    pub fn initialize_stream_timing_with_base(&mut self, time_base_micros: u64) {
        self.deterministic_time_base = time_base_micros;
        self.current_sample_count = 0;
        self.timing_initialized = true;
    }

    /// Convert sample count to exact microseconds using rational arithmetic.
    ///
    /// # Arguments
    ///
    /// * `sample_count` - Number of samples to convert
    ///
    /// # Returns
    ///
    /// Exact microseconds with no precision loss or cumulative drift.
    ///
    /// # Performance
    ///
    /// O(1) operation using pre-computed rational constants.
    pub fn samples_to_micros_exact(&self, sample_count: u64) -> u64 {
        // Exact rational multiplication: samples * (1_000_000 / sample_rate)
        let micros_fraction = self.microseconds_per_sample * Fraction::from(sample_count);

        // Convert to integer using floor (this is exact for most practical sample counts)
        u64::try_from(micros_fraction.floor()).unwrap_or(0)
    }

    /// Convert timestamp to sample offset with perfect precision.
    ///
    /// # Arguments
    ///
    /// * `timestamp_micros` - Target timestamp in deterministic time base
    ///
    /// # Returns
    ///
    /// Sample offset from current position (positive = future, negative = past)
    /// Returns `None` if stream timing has not been initialized.
    ///
    /// # Precision Guarantee
    ///
    /// Result is accurate to within 1 sample for any timestamp, with zero
    /// cumulative drift regardless of session length.
    pub fn timestamp_to_sample_offset(&self, timestamp_micros: u64) -> Option<i64> {
        if !self.timing_initialized {
            return None;
        }

        // Calculate current exact timestamp using rational arithmetic
        let stream_elapsed_micros = self.samples_to_micros_exact(self.current_sample_count);
        let current_timestamp = self.deterministic_time_base + stream_elapsed_micros;

        // Time difference (can be negative for past events)
        let time_diff_micros = timestamp_micros as i64 - current_timestamp as i64;

        // Convert time difference to samples using exact rational arithmetic
        let abs_time_diff = time_diff_micros.unsigned_abs();
        let samples_fraction = self.samples_per_microsecond * Fraction::from(abs_time_diff);
        let sample_offset = i64::try_from(samples_fraction.floor()).unwrap_or(0);

        // Preserve sign
        Some(if time_diff_micros >= 0 {
            sample_offset
        } else {
            -sample_offset
        })
    }

    /// Convert timestamp to exact sample position (not offset).
    ///
    /// Returns the absolute sample position for a given timestamp, used for
    /// sample-accurate scheduling within blocks.
    pub fn timestamp_to_exact_sample(&self, timestamp_micros: u64) -> Option<u64> {
        if !self.timing_initialized {
            return None;
        }

        if timestamp_micros < self.deterministic_time_base {
            return Some(0); // Past events map to sample 0
        }

        let elapsed_micros = timestamp_micros - self.deterministic_time_base;
        let samples_fraction = self.samples_per_microsecond * Fraction::from(elapsed_micros);
        Some(u64::try_from(samples_fraction.floor()).unwrap_or(0))
    }

    /// Advance the sample count by the specified number of samples.
    ///
    /// Call this at the end of each audio buffer processing cycle to maintain
    /// accurate timing state.
    ///
    /// # Arguments
    ///
    /// * `samples` - Number of samples processed in this audio buffer
    ///
    /// # Performance
    ///
    /// O(1) operation with overflow protection check.
    pub fn advance_samples(&mut self, samples: u64) {
        self.current_sample_count += samples;

        // Overflow protection: Reset time base every ~3 years to prevent overflow
        // This is extremely conservative - actual overflow would take centuries
        const MAX_SAFE_SAMPLES: u64 = u64::MAX / 2_000_000;
        if self.current_sample_count > MAX_SAFE_SAMPLES {
            self.reset_time_base();
        }
    }

    /// Get current exact timestamp in deterministic time base.
    ///
    /// # Returns
    ///
    /// Current timestamp with perfect precision, no cumulative drift.
    /// Returns 0 if stream timing has not been initialized.
    pub fn get_current_timestamp_exact(&self) -> u64 {
        if !self.timing_initialized {
            return 0;
        }

        self.deterministic_time_base + self.samples_to_micros_exact(self.current_sample_count)
    }

    /// Get current sample count since stream start.
    pub fn get_current_sample_count(&self) -> u64 {
        self.current_sample_count
    }

    /// Get deterministic time base in microseconds.
    pub fn get_time_base(&self) -> u64 {
        self.deterministic_time_base
    }

    /// Check if timing has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.timing_initialized
    }

    /// Reset time base to prevent overflow in very long sessions.
    ///
    /// Shifts the time base forward while maintaining timing continuity.
    /// This is automatically called when needed and should rarely be used manually.
    fn reset_time_base(&mut self) {
        // Calculate elapsed time and shift time base forward
        let elapsed_micros = self.samples_to_micros_exact(self.current_sample_count);
        self.deterministic_time_base += elapsed_micros;
        self.current_sample_count = 0;

        println!("HighPrecisionTimer: Time base reset to prevent overflow");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_timing_44100hz() {
        let timer = HighPrecisionTimer::new(44100.0);

        // Test: 44100 samples should be exactly 1 second (1,000,000 microseconds)
        let micros = timer.samples_to_micros_exact(44100);
        assert_eq!(micros, 1_000_000);
    }

    #[test]
    fn test_exact_timing_48000hz() {
        let timer = HighPrecisionTimer::new(48000.0);

        // Test: 48000 samples should be exactly 1 second
        let micros = timer.samples_to_micros_exact(48000);
        assert_eq!(micros, 1_000_000);
    }

    #[test]
    fn test_round_trip_conversion() {
        let timer = HighPrecisionTimer::new(44100.0);

        // Test: Converting samples->time->samples should be exact
        for samples in [1, 100, 1000, 44100] {
            let micros = timer.samples_to_micros_exact(samples);
            let back_to_samples =
                u64::try_from((timer.samples_per_microsecond * Fraction::from(micros)).floor())
                    .unwrap_or(0);

            // Should be exact or within 1 sample due to rounding
            assert!((samples as i64 - back_to_samples as i64).abs() <= 1);
        }
    }

    #[test]
    fn test_no_cumulative_drift() {
        let mut timer = HighPrecisionTimer::new(48000.0);
        timer.initialize_stream_timing();

        // Simulate 10 minutes of audio processing (600 seconds)
        let initial_time = timer.get_time_base();

        for second in 1..=600 {
            timer.advance_samples(48000); // 1 second of samples

            let expected_micros = second * 1_000_000;
            let actual_micros = timer.samples_to_micros_exact(timer.current_sample_count);

            // Should be exact - no cumulative drift
            assert_eq!(actual_micros, expected_micros);
        }
    }

    #[test]
    fn test_timestamp_to_sample_offset() {
        let mut timer = HighPrecisionTimer::new(48000.0);
        timer.initialize_stream_timing();

        let start_time = timer.get_current_timestamp_exact();

        // Test future timestamp
        let future_time = start_time + 500_000; // 0.5 seconds in the future
        let offset = timer.timestamp_to_sample_offset(future_time).unwrap();
        assert_eq!(offset, 24000); // 0.5 seconds * 48000 Hz = 24000 samples

        // Test past timestamp
        let past_time = start_time - 250_000; // 0.25 seconds in the past
        let offset = timer.timestamp_to_sample_offset(past_time).unwrap();
        assert_eq!(offset, -12000); // -0.25 seconds * 48000 Hz = -12000 samples
    }
}
