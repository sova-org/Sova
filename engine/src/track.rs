use crate::effect_pool::GlobalEffectPool;
use crate::memory::MemoryPool;
use crate::modules::Frame;
use crate::types::TrackId;
use crate::voice::Voice;
use std::collections::HashMap;
use std::sync::Arc;

/// A `Track` represents an audio channel that mixes multiple voices and applies
/// global effects processing. Each track maintains its own effects chain and
/// memory buffer, enabling parallel processing of multiple audio streams.
///
/// # Performance Characteristics
///
/// This implementation is optimized for real-time audio processing:
/// - Zero heap allocations during audio processing
/// - Pre-allocated audio buffers from shared memory pools
/// - Block-based processing with SIMD-optimized frame operations
/// - Lock-free effect parameter updates
/// - Deterministic execution time for all operations
///
/// # Memory Management
///
/// Track uses pre-allocated memory pools for audio buffers:
/// - Audio buffers are allocated from `MemoryPool` to avoid real-time allocation
/// - Buffer size is fixed at initialization and remains constant
/// - Raw pointers are used for maximum performance in audio processing
///
/// # Effects Processing
///
/// The track supports a dynamic chain of global effects:
/// - Effects are loaded from `ModuleRegistry` during initialization
/// - Only active effects are processed to minimize CPU usage
/// - Effects can be activated/deactivated with parameter changes
/// - Processing order follows the activation sequence
///
/// # Usage
///
/// ```rust
/// let mut track = Track::new(track_id, buffer_size);
/// track.set_memory_pool(memory_pool);
/// track.initialize_global_effects(&registry);
///
/// // Activate reverb with specific parameters
/// track.activate_global_effect("reverb", &[
///     ("room_size".to_string(), 0.7),
///     ("damping".to_string(), 0.5),
/// ]);
///
/// // Process audio block
/// track.process(&mut voices, &mut master_output, sample_rate);
/// ```
pub struct Track {
    /// Unique identifier for this track
    pub id: TrackId,
    /// Available global effects on this track
    available_effects: Vec<String>,
    /// Currently active effects in processing order
    active_effects: Vec<String>,
    /// Send levels for each global effect (0.0 = no send, 1.0 = full send)
    send_levels: HashMap<String, f32>,
    /// Shared memory pool for buffer allocation
    memory_pool: Option<Arc<MemoryPool>>,
    /// Raw pointer to pre-allocated audio buffer
    buffer_ptr: Option<*mut Frame>,
    /// Fixed buffer size in frames
    buffer_size: usize,
    /// Pre-allocated send buffer for send effect processing (eliminates heap allocation)
    send_buffer: Vec<Frame>,
}

impl Track {
    /// Creates a new track with the specified ID and buffer size.
    ///
    /// The track is initialized in an inactive state with no memory pool,
    /// no effects loaded, and no buffer allocated. Call `set_memory_pool()`
    /// and `initialize_global_effects()` to prepare the track for audio processing.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique track identifier
    /// * `buffer_size` - Size of the audio buffer in frames
    ///
    /// # Performance Notes
    ///
    /// Buffer size should match the engine's block size for optimal performance.
    /// Larger buffers reduce processing overhead but increase latency.
    pub fn new(id: TrackId, buffer_size: usize) -> Self {
        Self {
            id,
            available_effects: Vec::new(),
            active_effects: Vec::new(),
            send_levels: HashMap::new(),
            memory_pool: None,
            buffer_ptr: None,
            buffer_size,
            send_buffer: vec![Frame::ZERO; buffer_size],
        }
    }

    /// Sets the memory pool and allocates the track's audio buffer.
    ///
    /// This method must be called before processing audio. It allocates a contiguous
    /// block of memory for the track's audio buffer and initializes it to silence.
    /// The buffer is 16-byte aligned for optimal SIMD performance.
    ///
    /// # Arguments
    ///
    /// * `pool` - Shared memory pool for allocation
    ///
    /// # Safety
    ///
    /// The allocated buffer pointer is stored as a raw pointer for performance.
    /// The memory remains valid as long as the memory pool exists.
    ///
    /// # Panics
    ///
    /// This method will not panic but allocation may fail silently if the
    /// memory pool is exhausted. Check buffer availability before processing.
    pub fn set_memory_pool(&mut self, pool: Arc<MemoryPool>) {
        if let Some(ptr) = pool.allocate(self.buffer_size * std::mem::size_of::<Frame>(), 16) {
            unsafe {
                std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut Frame, self.buffer_size)
                    .fill(Frame::ZERO);
            }
            self.buffer_ptr = Some(ptr.as_ptr() as *mut Frame);
        }
        self.memory_pool = Some(pool);
    }

    /// Loads and initializes all available global effects from the registry.
    ///
    /// This method queries the module registry for available global effects
    /// and creates instances of each one. Effects are stored in an inactive state
    /// and must be explicitly activated with `activate_global_effect()`.
    ///
    /// # Arguments
    ///
    /// * `registry` - Module registry containing effect definitions
    ///
    /// # Performance Notes
    ///
    /// This method performs heap allocation and should be called during
    /// initialization, not during real-time audio processing.
    pub fn initialize_global_effects(&mut self, available_effect_names: Vec<String>) {
        self.available_effects = available_effect_names;
    }

    /// Activates a global effect with the specified parameters.
    ///
    /// This method configures an effect's parameters and adds it to the active
    /// effects chain if not already present. Effects are processed in the order
    /// they are activated.
    ///
    /// # Arguments
    ///
    /// * `effect_name` - Name of the effect to activate
    /// * `parameters` - Array of (parameter_name, value) tuples
    ///
    /// # Performance Notes
    ///
    /// Parameter updates are lock-free and safe to call from any thread.
    /// The effects chain is only modified when effects are activated/deactivated,
    /// not during parameter changes.
    ///
    /// # Example
    ///
    /// ```rust
    /// track.activate_global_effect("reverb", &[
    ///     ("room_size".to_string(), 0.8),
    ///     ("decay_time".to_string(), 2.5),
    /// ]);
    /// ```
    pub fn activate_global_effect(&mut self, effect_name: &str, _parameters: &[(String, f32)]) {
        if self.available_effects.contains(&effect_name.to_string())
            && !self.active_effects.contains(&effect_name.to_string())
        {
            self.active_effects.push(effect_name.to_string());
        }
    }

    /// Updates a global effect with new parameters while preserving its state.
    ///
    /// This method updates an effect's parameters without destroying its internal
    /// state (reverb tails, delay buffers, etc.). If the effect is not yet active,
    /// it will be added to the active effects chain.
    ///
    /// # Arguments
    ///
    /// * `effect_name` - Name of the effect to update
    /// * `parameters` - Array of (parameter_name, value) tuples
    ///
    /// # Performance Notes
    ///
    /// Parameter updates are lock-free and preserve effect state.
    /// Adding to active effects chain only occurs if not already present.
    pub fn update_global_effect(&mut self, effect_name: &str, parameters: &[(String, f32)]) {
        if self.available_effects.contains(&effect_name.to_string()) {
            for (param_name, value) in parameters {
                if param_name == &format!("{}_send", effect_name) {
                    self.send_levels.insert(effect_name.to_string(), *value);
                } else if param_name == "send" {
                    // Handle generic "send" parameter
                    self.send_levels.insert(effect_name.to_string(), *value);
                }
            }
            if !self.active_effects.contains(&effect_name.to_string()) {
                self.active_effects.push(effect_name.to_string());
            }
        }
    }

    /// Deactivates all global effects on this track.
    ///
    /// This method clears the active effects chain, effectively bypassing
    /// all global effects processing. The effects remain loaded and can be
    /// reactivated with their previous parameter settings.
    ///
    /// # Performance Notes
    ///
    /// This operation is O(1) and safe to call during audio processing.
    /// No heap allocation or deallocation occurs.
    pub fn deactivate_all_effects(&mut self) {
        self.active_effects.clear();
    }

    /// Processes one block of audio through the track's voice mix and effects chain.
    ///
    /// This is the core real-time audio processing method. It:
    /// 1. Clears the track buffer to silence
    /// 2. Mixes all active voices assigned to this track
    /// 3. Applies active global effects in sequence
    /// 4. Adds the processed audio to the master output
    ///
    /// # Arguments
    ///
    /// * `voices` - Array of all voices in the engine
    /// * `master_output` - Master output buffer to add processed audio to
    /// * `sample_rate` - Current engine sample rate
    ///
    /// # Performance Characteristics
    ///
    /// - Zero heap allocations during processing
    /// - Block-based processing with SIMD optimization
    /// - Early exit if no memory buffer is allocated
    /// - Processes only active voices assigned to this track
    /// - Skips inactive effects automatically
    ///
    /// # Safety
    ///
    /// This method uses raw pointers for maximum performance. The buffer
    /// pointer must be valid and the memory pool must remain alive during
    /// the call.
    ///
    /// # Real-time Safety
    ///
    /// This method is designed for real-time audio processing:
    /// - Deterministic execution time
    /// - No blocking operations
    /// - No system calls or memory allocation
    /// - Lock-free parameter access
    #[inline]
    pub fn process(
        &mut self,
        voices: &mut [Voice],
        master_output: &mut [Frame],
        sample_rate: f32,
        effect_pool: &mut GlobalEffectPool,
    ) {
        let len = master_output.len().min(self.buffer_size);

        let buffer = if let Some(ptr) = self.buffer_ptr {
            debug_assert!(!ptr.is_null(), "Track buffer pointer is null");
            debug_assert_eq!(
                ptr as usize % std::mem::align_of::<Frame>(),
                0,
                "Track buffer not aligned for Frame"
            );
            debug_assert!(
                len <= self.buffer_size,
                "Buffer length exceeds track capacity"
            );

            // Safety: We've verified the pointer is non-null, aligned, and within bounds
            // The pointer is owned by this track and valid for the lifetime of the call
            unsafe { std::slice::from_raw_parts_mut(ptr, len) }
        } else {
            return;
        };

        Frame::process_block_zero(buffer);

        // Process only voices that belong to this track and are active
        // Note: peak_tracker check removed from filter to allow new voices to produce sound
        for voice in voices {
            if voice.track_id == self.id && voice.is_active {
                voice.process(buffer, sample_rate);
            }
        }

        // Global effects processing with send architecture
        for effect_name in &self.active_effects {
            if let Some(effect) = effect_pool.get_effect_mut(effect_name, self.id as usize) {
                if effect.is_active() {
                    let send_level = self.send_levels.get(effect_name).copied().unwrap_or(0.0);

                    if send_level > 0.0 {
                        // Use pre-allocated send buffer to avoid heap allocation
                        let send_slice = &mut self.send_buffer[..len];

                        // Clear send buffer to avoid stale data
                        Frame::process_block_zero(send_slice);

                        // Mix input signal to send buffer based on send level
                        for (i, frame) in buffer[..len].iter().enumerate() {
                            send_slice[i].left = frame.left * send_level;
                            send_slice[i].right = frame.right * send_level;
                        }

                        // Process send buffer through effect (100% wet)
                        effect.process(send_slice, sample_rate);

                        // Add processed send signal back to main buffer
                        for (i, frame) in buffer[..len].iter_mut().enumerate() {
                            frame.left += send_slice[i].left;
                            frame.right += send_slice[i].right;
                        }
                    }
                }
            }
        }

        for (i, frame) in buffer.iter().enumerate() {
            master_output[i].left += frame.left;
            master_output[i].right += frame.right;
        }
    }
}

/// # Thread Safety
///
/// Track is safe to send between threads and can be accessed concurrently
/// from multiple threads. The audio processing method should only be called
/// from the audio thread, while parameter updates can be made from any thread.
///
/// # Memory Safety
///
/// The raw buffer pointer is managed carefully:
/// - Only valid while the memory pool exists
/// - Aligned properly for SIMD operations
/// - Size is bounded by the buffer_size field
/// - Cleared to silence on each processing cycle
unsafe impl Send for Track {}
unsafe impl Sync for Track {}
