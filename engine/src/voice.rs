use crate::dsp::adsr::{Envelope, EnvelopeParams, EnvelopeState};
use crate::dsp::dc_blocker::DcBlocker;
use crate::memory::VoiceMemory;
use crate::modulation::Modulation;
use crate::modules::{Frame, LocalEffect, Source};
use crate::types::{TrackId, VoiceId};
use std::sync::Arc;

const INLINE_MODULATION_SLOTS: usize = 4; // 90% of voices use ≤4 slots
const EXTENDED_MODULATION_SLOTS: usize = 12; // Additional slots available from pool (4+12=16 total)
const MAX_MODULATION_SLOTS: usize = 16; // Reduced from 32, still plenty
const MAX_ENVELOPE_BUFFER_SIZE: usize = 1024; // Right-sized for max expected block size
const DEFAULT_DURATION: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
struct ParameterSmoother {
    target: f32,
    current: f32,
    rate: f32,
}

impl Default for ParameterSmoother {
    fn default() -> Self {
        Self {
            target: 0.0,
            current: 0.0,
            rate: 0.1,
        }
    }
}

impl ParameterSmoother {
    fn new(initial_value: f32) -> Self {
        Self {
            target: initial_value,
            current: initial_value,
            rate: 0.1,
        }
    }

    fn new_with_rate(initial_value: f32, rate: f32) -> Self {
        Self {
            target: initial_value,
            current: initial_value,
            rate: rate.clamp(0.001, 1.0),
        }
    }

    #[inline]
    fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    #[inline]
    fn set_target_immediate(&mut self, target: f32) {
        self.target = target;
        self.current = target;
    }

    #[inline]
    fn update(&mut self) -> f32 {
        self.current += (self.target - self.current) * self.rate;
        self.current
    }
}

/// Real-time polyphonic voice processor for audio synthesis.
///
/// A `Voice` represents a single sound instance in the audio engine, capable of generating
/// audio through a source module, applying local effects, and managing amplitude envelopes.
/// Multiple voices can be played simultaneously to achieve polyphonic audio synthesis.
///
/// # Performance Characteristics
///
/// This implementation is optimized for real-time audio processing:
/// - Zero heap allocations during audio processing
/// - Pre-allocated modulation slots with fixed-size arrays
/// - Lock-free parameter updates via shared memory buffers
/// - Block-based audio processing for optimal CPU cache usage
/// - Deterministic execution time for all operations
///
/// # Memory Management
///
/// Voice uses pre-allocated shared memory pools for audio buffers and modulation data:
/// - Audio buffers are allocated from `VoiceMemory` to avoid real-time allocation
/// - Modulation values are cached in fixed-size arrays
/// - All temporary data structures are stack-allocated
///
/// # Architecture
///
/// Each voice consists of:
/// - **Source**: Audio generator (oscillator, sampler, etc.)
/// - **Local Effects**: Per-voice processing chain (filters, distortion, etc.)
/// - **Envelope**: ADSR amplitude control
/// - **Modulations**: Parameter automation system (up to 16 slots)
/// - **Panning**: Stereo positioning
///
/// # Usage
///
/// ```rust
/// let mut voice = Voice::new(voice_id, track_id, buffer_size);
/// voice.set_voice_memory(memory_pool);
///
/// // Configure voice
/// voice.amp = 0.8;
/// voice.pan = -0.2;  // Slightly left
///
/// // Add modulation
/// voice.add_modulation("freq", Modulation::Osc { /* params */ });
///
/// // Process audio
/// voice.trigger();
/// voice.process(&mut output_buffer, sample_rate);
/// ```
pub struct Voice {
    /// Unique identifier for this voice instance
    pub id: VoiceId,
    /// Track identifier that owns this voice
    pub track_id: TrackId,
    /// Voice amplitude (0.0 to 1.0)
    pub amp: f32,
    /// Stereo pan position (-1.0 = left, 0.0 = center, 1.0 = right)
    pub pan: f32,
    /// ADSR envelope parameters for amplitude control
    pub envelope_params: EnvelopeParams,
    /// ADSR envelope state for this voice
    pub envelope_state: EnvelopeState,
    /// Voice duration in seconds (0.0 = infinite until release)
    pub duration: f32,
    /// Audio source module (oscillator, sampler, etc.)
    pub source: Option<Box<dyn Source>>,
    /// Chain of local effects applied to this voice
    pub local_effects: Vec<Box<dyn LocalEffect>>,
    /// Whether voice is currently active and should be processed
    pub is_active: bool,
    /// Index into voice memory pools for buffer allocation
    pub voice_index: usize,
    /// Shared memory pool for pre-allocated audio and modulation buffers
    voice_memory: Option<Arc<VoiceMemory>>,
    /// Inline modulation slots for common case (90% use ≤4)
    inline_modulations: [Modulation; INLINE_MODULATION_SLOTS],
    /// Inline modulation values cache
    inline_mod_values: [f32; INLINE_MODULATION_SLOTS],
    /// Inline parameter names
    inline_mod_names: [&'static str; INLINE_MODULATION_SLOTS],
    /// Extended modulation storage for voices needing >4 slots (pre-allocated from pool)
    extended_modulations: [Modulation; EXTENDED_MODULATION_SLOTS],
    /// Extended modulation values cache (pre-allocated from pool)
    extended_mod_values: [f32; EXTENDED_MODULATION_SLOTS],
    /// Extended parameter names (pre-allocated from pool)
    extended_mod_names: [&'static str; EXTENDED_MODULATION_SLOTS],
    /// Number of active modulation slots
    mod_count: u8,
    /// Random number generator state for noise modulations
    rng_state: u32,
    /// Per-voice DC blocker for removing DC offset
    dc_blocker: DcBlocker,
    /// Parameter smoothers to prevent zipper noise
    amp_smoother: ParameterSmoother,
    pan_smoother: ParameterSmoother,
    /// Crossfade smoother for voice transitions
    crossfade_smoother: ParameterSmoother,
    /// Voice transition state
    is_crossfading: bool,
    /// Chain-level gain reduction for managing effect explosions
    chain_gain_reduction: f32,
    /// Peak tracking for voice culling optimization
    pub peak_tracker: f32,
    /// Right-sized envelope buffer (fixed size based on max expected block size)
    envelope_buffer: [f32; MAX_ENVELOPE_BUFFER_SIZE],
}

impl Voice {
    /// Creates a new voice instance with the specified identifiers.
    ///
    /// The voice is created in an inactive state with default parameters:
    /// - amp: 1.0 (full volume)
    /// - pan: 0.0 (center)
    /// - duration: 1.0 second
    /// - envelope: default ADSR settings
    ///
    /// # Arguments
    ///
    /// * `id` - Unique voice identifier
    /// * `track_id` - Track that owns this voice
    /// * `_buffer_size` - Buffer size hint (currently unused)
    ///
    /// # Performance Notes
    ///
    /// No heap allocations are performed during construction. All arrays are
    /// stack-allocated with fixed sizes for deterministic memory usage.
    pub fn new(id: VoiceId, track_id: TrackId, _buffer_size: usize) -> Self {
        Self {
            id,
            track_id,
            amp: 1.0,
            pan: 0.0,
            envelope_params: EnvelopeParams::default(),
            envelope_state: EnvelopeState::default(),
            duration: DEFAULT_DURATION,
            source: None,
            local_effects: Vec::new(),
            is_active: false,
            voice_index: id as usize,
            voice_memory: None,
            inline_modulations: [Modulation::Static(0.0); INLINE_MODULATION_SLOTS],
            inline_mod_values: [0.0; INLINE_MODULATION_SLOTS],
            inline_mod_names: [""; INLINE_MODULATION_SLOTS],
            extended_modulations: [Modulation::Static(0.0); EXTENDED_MODULATION_SLOTS],
            extended_mod_values: [0.0; EXTENDED_MODULATION_SLOTS],
            extended_mod_names: [""; EXTENDED_MODULATION_SLOTS],
            mod_count: 0,
            rng_state: 1,
            dc_blocker: DcBlocker::new(),
            amp_smoother: ParameterSmoother::new(1.0),
            pan_smoother: ParameterSmoother::new(0.0),
            crossfade_smoother: ParameterSmoother::new_with_rate(1.0, 0.05),
            is_crossfading: false,
            chain_gain_reduction: 1.0,
            peak_tracker: 0.0,
            envelope_buffer: [0.0; MAX_ENVELOPE_BUFFER_SIZE], // Pre-allocated fixed size
        }
    }

    /// Assigns shared memory pool for voice audio and modulation buffers.
    ///
    /// This must be called before processing audio to ensure the voice has access
    /// to pre-allocated memory pools for zero-allocation audio processing.
    ///
    /// # Arguments
    ///
    /// * `voice_memory` - Shared memory pool for audio buffers and modulation data
    pub fn set_voice_memory(&mut self, voice_memory: Arc<VoiceMemory>) {
        self.voice_memory = Some(voice_memory);
    }

    pub fn set_voice_index(&mut self, index: usize) {
        self.voice_index = index;
    }

    /// Processes a block of audio samples for this voice.
    ///
    /// This is the core audio processing method that:
    /// 1. Generates audio from the source module
    /// 2. Applies local effects in order
    /// 3. Updates modulations for the current block
    /// 4. Applies envelope, amplitude, and panning
    /// 5. Adds the result to the output buffer
    ///
    /// # Arguments
    ///
    /// * `output` - Output buffer to add voice audio to
    /// * `sample_rate` - Current audio sample rate
    ///
    /// # Performance Notes
    ///
    /// - Returns early if voice is inactive
    /// - Uses pre-allocated memory from VoiceMemory
    /// - All calculations are performed in-place where possible
    /// - Automatically deactivates voice when envelope finishes
    #[inline]
    pub fn process(&mut self, output: &mut [Frame], sample_rate: f32) {
        if !self.is_active {
            return;
        }

        let buffer = if let Some(ref memory) = self.voice_memory {
            if let Some(voice_buffer) = memory.get_voice_buffer(self.voice_index) {
                let max_frames = voice_buffer.len() / 2;
                let len = output.len().min(max_frames);

                let ptr = voice_buffer.as_ptr();
                debug_assert_eq!(
                    ptr as usize % std::mem::align_of::<Frame>(),
                    0,
                    "Buffer not aligned for Frame"
                );
                debug_assert!(len <= max_frames, "Buffer length exceeds capacity");
                debug_assert!(!ptr.is_null(), "Buffer pointer is null");
                debug_assert!(
                    len * std::mem::size_of::<Frame>() <= std::mem::size_of_val(voice_buffer),
                    "Frame buffer would exceed f32 buffer bounds"
                );

                // Safety: We've verified alignment, bounds, and non-null pointer
                // The lifetime is tied to the voice_memory which outlives this call
                unsafe { std::slice::from_raw_parts_mut(ptr as *mut Frame, len) }
            } else {
                return;
            }
        } else {
            return;
        };

        Frame::process_block_zero(buffer);

        if let Some(source) = &mut self.source {
            source.generate(buffer, sample_rate);
        }

        // Apply DC blocking after source, before effects
        self.dc_blocker.process_block_optimized(buffer);

        // Process local effects with safety measures
        self.process_effects_chain(buffer, sample_rate);

        let _sample_dt = 1.0 / sample_rate;
        let block_dt = buffer.len() as f32 / sample_rate;

        // Use envelope buffer (fixed size, no dynamic allocation)
        let envelope_len = buffer.len().min(MAX_ENVELOPE_BUFFER_SIZE);

        {
            let env_slice = &mut self.envelope_buffer[..envelope_len];
            Envelope::process_block(
                &self.envelope_params,
                &mut self.envelope_state,
                env_slice,
                sample_rate,
            );
        }

        let env_avg =
            self.envelope_buffer[..envelope_len].iter().sum::<f32>() / envelope_len as f32;
        self.update_modulations(block_dt, env_avg);

        let smooth_amp = self.amp_smoother.update();
        let smooth_pan = self.pan_smoother.update();
        let crossfade_level = self.crossfade_smoother.update();

        if self.is_crossfading && crossfade_level <= 0.001 {
            self.immediate_reset();
            return;
        }

        let pan_factor = (smooth_pan + 1.0) * 0.5;
        let left_gain = (1.0 - pan_factor).max(0.0);
        let right_gain = pan_factor.max(0.0);

        for (i, frame) in buffer.iter().enumerate() {
            if i >= output.len() {
                break;
            }

            let env_level = if i < self.envelope_buffer.len() {
                self.envelope_buffer[i]
            } else {
                0.0
            };
            let envelope_amp = env_level * crossfade_level;
            let mixed_left = frame.left * envelope_amp * smooth_amp * left_gain;
            let mixed_right = frame.right * envelope_amp * smooth_amp * right_gain;

            // Track peak level for voice culling (running average with decay)
            self.peak_tracker =
                self.peak_tracker * 0.99 + (mixed_left.abs() + mixed_right.abs()) * 0.01;

            output[i].left += mixed_left;
            output[i].right += mixed_right;
        }

        // Voice is inactive if envelope finished OR output is below silence threshold (-60dB)
        if self.envelope_state.is_finished() || self.peak_tracker < 0.001 {
            self.is_active = false;
        }
    }

    /// Activates the voice and triggers its envelope to begin audio generation.
    ///
    /// This starts the ADSR envelope from the attack phase and marks the voice
    /// as active for audio processing.
    #[inline]
    pub fn trigger(&mut self) {
        self.is_active = true;
        self.is_crossfading = false;
        self.crossfade_smoother.set_target_immediate(1.0);
        self.envelope_state.trigger();
        self.peak_tracker = 0.0; // Reset peak tracking for fresh voice

        if let Some(source) = &mut self.source {
            // Check if it's a sampler and trigger it
            if let Some(sampler) = source
                .as_any_mut()
                .downcast_mut::<crate::modules::source::sample::StereoSampler>()
            {
                sampler.trigger();
            }
        }
    }

    /// Advances the envelope by the specified time for sub-sample precision.
    ///
    /// This method enables sample-accurate timing by pre-advancing the envelope
    /// when events are scheduled mid-block. The envelope is processed for the
    /// exact fractional time until the true trigger point.
    ///
    /// # Arguments
    ///
    /// * `time_seconds` - Time in seconds to advance the envelope
    /// * `sample_rate` - Current audio sample rate for accurate timing calculations
    pub fn advance_envelope_by_time(&mut self, time_seconds: f32, sample_rate: f32) {
        if time_seconds <= 0.0 {
            return;
        }

        // Advance envelope by exact sub-sample time for sample-accurate timing
        // Use actual sample rate for precise dt calculation
        let dt = 1.0 / sample_rate;
        let steps = (time_seconds / dt).round() as usize;

        // Process envelope for exact fractional sample timing
        for _ in 0..steps.min(64) {
            // Limit iterations for realtime safety
            crate::dsp::adsr::Envelope::get_amplitude(
                &self.envelope_params,
                &mut self.envelope_state,
                dt,
            );
        }
    }

    /// Begins the release phase of the voice envelope.
    ///
    /// The envelope will transition to its release phase, fading the voice
    /// to silence over the configured release time. The voice will automatically
    /// deactivate when the envelope reaches zero amplitude.
    pub fn release(&mut self) {
        self.envelope_state.release();
    }

    /// Immediately stops the voice and deactivates it.
    ///
    /// This bypasses the envelope release phase and instantly silences the voice.
    /// The voice becomes inactive and will not be processed until triggered again.
    pub fn stop(&mut self) {
        self.is_active = false;
        self.envelope_state.set_idle();
    }

    /// Completely resets voice state for reuse.
    pub fn reset_for_reuse(&mut self) {
        self.start_crossfade_out();
    }

    /// Safely prepares voice for reuse with crossfading
    fn start_crossfade_out(&mut self) {
        if self.is_active && !self.envelope_state.is_finished() {
            self.is_crossfading = true;
            self.crossfade_smoother.set_target(0.0);
        } else {
            self.immediate_reset();
        }
    }

    /// Immediately resets voice (use only when safe)
    pub fn immediate_reset(&mut self) {
        self.is_active = false;
        self.amp = 1.0;
        self.pan = 0.0;
        self.duration = DEFAULT_DURATION;
        self.envelope_params = EnvelopeParams::default();
        self.envelope_state = EnvelopeState::default();
        self.dc_blocker = DcBlocker::new();
        self.amp_smoother = ParameterSmoother::new(1.0);
        self.pan_smoother = ParameterSmoother::new(0.0);
        self.crossfade_smoother = ParameterSmoother::new_with_rate(1.0, 0.05);
        self.is_crossfading = false;
        self.chain_gain_reduction = 1.0;
        self.peak_tracker = 0.0;

        self.inline_modulations = [Modulation::Static(0.0); INLINE_MODULATION_SLOTS];
        self.inline_mod_values = [0.0; INLINE_MODULATION_SLOTS];
        self.inline_mod_names = [""; INLINE_MODULATION_SLOTS];
        self.extended_modulations = [Modulation::Static(0.0); EXTENDED_MODULATION_SLOTS];
        self.extended_mod_values = [0.0; EXTENDED_MODULATION_SLOTS];
        self.extended_mod_names = [""; EXTENDED_MODULATION_SLOTS];
        self.mod_count = 0;

        self.source = None;
        self.local_effects.clear();
        self.rng_state = 1;
    }

    /// Adds a modulation source to control voice parameters dynamically.
    ///
    /// Modulation sources can control any voice parameter, source parameter,
    /// or effect parameter by name. Up to 16 modulation slots are available
    /// per voice for real-time parameter automation.
    ///
    /// # Arguments
    ///
    /// * `name` - Parameter name to modulate (e.g., "freq", "amp", "cutoff")
    /// * `modulation` - Modulation source configuration
    ///
    /// # Performance Notes
    ///
    /// Modulation slots are pre-allocated with fixed-size arrays for zero-allocation
    /// operation during audio processing.
    #[inline]
    pub fn add_modulation(&mut self, name: &'static str, modulation: Modulation) {
        let idx = self.mod_count as usize;

        if idx < INLINE_MODULATION_SLOTS {
            // Use inline storage for first 4 modulations
            self.inline_modulations[idx] = modulation;
            self.inline_mod_names[idx] = name;
        } else if idx < MAX_MODULATION_SLOTS {
            // Use extended storage for modulations 5-16 (pre-allocated fixed array)
            let ext_idx = idx - INLINE_MODULATION_SLOTS;
            self.extended_modulations[ext_idx] = modulation;
            self.extended_mod_names[ext_idx] = name;
            self.extended_mod_values[ext_idx] = 0.0;
        } else {
            return; // Exceed max modulations, ignore
        }

        self.mod_count += 1;
    }

    /// Updates all active modulation sources for the current audio block.
    ///
    /// This method processes all modulation slots and applies their current values
    /// to the appropriate parameters. Modulation values are computed based on the
    /// current time delta and envelope state.
    ///
    /// # Arguments
    ///
    /// * `dt` - Time delta for this audio block (block_size / sample_rate)
    /// * `envelope_val` - Current envelope level for envelope-based modulations
    ///
    /// # Performance Notes
    ///
    /// Uses pre-allocated modulation buffers from VoiceMemory when available.
    /// Falls back to local storage if memory pool is unavailable.
    pub fn update_modulations(&mut self, dt: f32, envelope_val: f32) {
        let total_count = self.mod_count as usize;

        // Process inline modulations (first 4)
        let inline_count = total_count.min(INLINE_MODULATION_SLOTS);
        for i in 0..inline_count {
            let value = if let Some(ref memory) = self.voice_memory {
                if let Some(mod_buffer) = memory.get_modulation_buffer(self.voice_index, i) {
                    mod_buffer[0] =
                        self.inline_modulations[i].update(dt, envelope_val, &mut self.rng_state);
                    mod_buffer[0]
                } else {
                    self.inline_modulations[i].update(dt, envelope_val, &mut self.rng_state)
                }
            } else {
                self.inline_modulations[i].update(dt, envelope_val, &mut self.rng_state)
            };

            self.inline_mod_values[i] = value;
            self.apply_modulation_value(self.inline_mod_names[i], value);
        }

        // Process extended modulations if any (5-16)
        if total_count > INLINE_MODULATION_SLOTS {
            let extended_count = total_count - INLINE_MODULATION_SLOTS;

            for i in 0..extended_count {
                let value = if let Some(ref memory) = self.voice_memory {
                    if let Some(mod_buffer) =
                        memory.get_modulation_buffer(self.voice_index, i + INLINE_MODULATION_SLOTS)
                    {
                        mod_buffer[0] = self.extended_modulations[i].update(
                            dt,
                            envelope_val,
                            &mut self.rng_state,
                        );
                        mod_buffer[0]
                    } else {
                        self.extended_modulations[i].update(dt, envelope_val, &mut self.rng_state)
                    }
                } else {
                    self.extended_modulations[i].update(dt, envelope_val, &mut self.rng_state)
                };

                self.extended_mod_values[i] = value;
                self.apply_modulation_value(self.extended_mod_names[i], value);
            }
        }
    }

    #[inline]
    fn apply_modulation_value(&mut self, param_name: &'static str, value: f32) {
        if let Some(param_index) = self.get_engine_param_index(param_name) {
            self.set_engine_parameter(param_index, value);
        } else {
            if let Some(source) = &mut self.source {
                source.set_parameter(param_name, value);
            }
            for effect in &mut self.local_effects {
                effect.set_parameter(param_name, value);
            }
        }
    }

    /// Gets the parameter index for built-in engine parameters.
    ///
    /// Returns the index if the parameter name matches a built-in engine parameter,
    /// allowing direct parameter updates through the registry system.
    fn get_engine_param_index(&self, name: &str) -> Option<usize> {
        use crate::registry::*;
        get_engine_parameter_index(name)
    }

    /// Sets a built-in engine parameter by index.
    ///
    /// Handles direct updates to voice parameters like amplitude, pan, duration,
    /// and envelope settings through the parameter registry system.
    ///
    /// # Arguments
    ///
    /// * `param_index` - Parameter index from the registry
    /// * `value` - New parameter value
    #[inline]
    pub fn set_engine_parameter(&mut self, param_index: usize, value: f32) {
        use crate::constants::{
            ENGINE_PARAM_AMP, ENGINE_PARAM_ATTACK, ENGINE_PARAM_ATTACK_CURVE, ENGINE_PARAM_DECAY,
            ENGINE_PARAM_DECAY_CURVE, ENGINE_PARAM_DUR, ENGINE_PARAM_PAN, ENGINE_PARAM_RELEASE,
            ENGINE_PARAM_RELEASE_CURVE, ENGINE_PARAM_SUSTAIN,
        };
        match param_index {
            ENGINE_PARAM_AMP => {
                self.amp = value;
                self.amp_smoother.set_target_immediate(value);
            }
            ENGINE_PARAM_PAN => {
                self.pan = value;
                self.pan_smoother.set_target(value);
            }
            ENGINE_PARAM_DUR => {
                self.duration = value;
                self.envelope_state.scale_to_duration(value);
            }
            ENGINE_PARAM_ATTACK => self.envelope_params.attack = value,
            ENGINE_PARAM_DECAY => self.envelope_params.decay = value,
            ENGINE_PARAM_SUSTAIN => self.envelope_params.sustain = value,
            ENGINE_PARAM_RELEASE => self.envelope_params.release = value,
            ENGINE_PARAM_ATTACK_CURVE => self.envelope_params.attack_curve = value.clamp(0.0, 1.0),
            ENGINE_PARAM_DECAY_CURVE => self.envelope_params.decay_curve = value.clamp(0.0, 1.0),
            ENGINE_PARAM_RELEASE_CURVE => {
                self.envelope_params.release_curve = value.clamp(0.0, 1.0)
            }
            _ => {}
        }
    }

    /// Soft limiting function to prevent audio explosions between effects
    #[inline]
    fn soft_limit(x: f32) -> f32 {
        let abs_x = x.abs();
        if abs_x <= 0.7 {
            x
        } else if abs_x <= 1.0 {
            let t = abs_x - 0.7;
            let soft = 0.7 + t * (0.3 - 0.1 * t);
            x.signum() * soft
        } else {
            let normalized = abs_x.min(2.0) / 2.0;
            let soft = normalized * (2.0 - normalized * normalized) * 0.5;
            (x.signum() * soft).clamp(-1.0, 1.0)
        }
    }

    /// Processes the local effects chain with safety measures
    fn process_effects_chain(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.local_effects.is_empty() {
            return;
        }

        for effect in &mut self.local_effects {
            if effect.is_active() {
                // Process the effect
                effect.process(buffer, sample_rate);

                // Apply soft limiting after each effect to prevent explosions
                for frame in buffer.iter_mut() {
                    frame.left = Self::soft_limit(frame.left);
                    frame.right = Self::soft_limit(frame.right);
                }
            }
        }

        // Apply chain-level gain management
        self.apply_chain_gain_management(buffer);
    }

    /// Apply automatic gain reduction to manage overall chain levels
    fn apply_chain_gain_management(&mut self, buffer: &mut [Frame]) {
        // Calculate peak level for this block
        let mut peak = 0.0f32;
        for frame in buffer.iter() {
            let level = frame.left.abs().max(frame.right.abs());
            if level > peak {
                peak = level;
            }
        }

        // Update peak tracker with decay
        const PEAK_DECAY: f32 = 0.99;
        self.peak_tracker = self.peak_tracker * PEAK_DECAY + peak * (1.0 - PEAK_DECAY);

        // Calculate gain reduction if peak is too high
        const THRESHOLD: f32 = 0.8;
        const RATIO: f32 = 4.0;
        const ATTACK: f32 = 0.01;
        const RELEASE: f32 = 0.1;

        let target_gain = if self.peak_tracker > THRESHOLD {
            let overshoot = self.peak_tracker - THRESHOLD;
            let reduction = overshoot / RATIO;
            (1.0 - reduction).max(0.1) // Minimum 10% gain
        } else {
            1.0
        };

        // Smooth gain changes
        let rate = if target_gain < self.chain_gain_reduction {
            ATTACK
        } else {
            RELEASE
        };
        self.chain_gain_reduction += (target_gain - self.chain_gain_reduction) * rate;

        // Apply gain reduction
        if self.chain_gain_reduction < 0.99 {
            for frame in buffer.iter_mut() {
                frame.left *= self.chain_gain_reduction;
                frame.right *= self.chain_gain_reduction;
            }
        }
    }
}
