use crate::constants::{
    AUDIO_BLOCK_SIZE_FALLBACK, DEFAULT_MEMORY_SIZE, DEFAULT_SAMPLE_COUNT, DEFAULT_SAMPLE_DIR,
    ENGINE_PARAM_DUR, MAX_TRACKS,
};
use crate::effect_pool::GlobalEffectPool;
use crate::memory::{
    MemoryPool, PredictiveSampleManager, SampleLibrary, SampleResult, VoiceMemory,
};
use crate::modulation::Modulation;
use crate::modules::Frame;
use crate::registry::ModuleRegistry;
use crate::server::ScheduledEngineMessage;
use crate::timing::HighPrecisionTimer;
use crate::track::Track;
use crate::types::{
    EngineError, EngineMessage, EngineStatusMessage, ScheduledMessage, TrackId, VoiceId,
};
use crate::voice::Voice;

// Real-time safe logging - local macro
#[cfg(feature = "rt-safe")]
macro_rules! rt_eprintln {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "rt-safe"))]
macro_rules! rt_eprintln {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}
use crossbeam_channel::{Receiver, Sender};
use std::collections::BinaryHeap;
use std::sync::{Arc, mpsc};
use std::thread;
use thread_priority::{ThreadPriority, ThreadPriorityValue, set_current_thread_priority};

/// Maps user priority (0-99) to platform-appropriate priority range
fn map_to_platform_priority(user_priority: u8) -> u8 {
    // Clamp user input to 0-99 range
    let user_priority = user_priority.min(99);

    // Platform-specific mapping
    #[cfg(target_os = "macos")]
    {
        // macOS: range 15-47
        let min_priority = 15u8;
        let max_priority = 47u8;
        let range = max_priority - min_priority;
        min_priority + ((user_priority as u16 * range as u16) / 99) as u8
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: range 1-99 for SCHED_FIFO/SCHED_RR
        user_priority.max(1)
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: different priority classes, map to reasonable range
        // ThreadPriorityValue supports different ranges on Windows
        user_priority.min(31) // Conservative upper bound
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        // Other platforms: conservative mapping
        user_priority.min(50)
    }
}

pub struct AudioEngine {
    pub voices: Vec<Voice>,
    pub tracks: Vec<Track>,
    pub registry: ModuleRegistry,
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub block_size: usize,
    max_voices: usize,
    next_voice_id: VoiceId,
    block_buffer_ptr: *mut Frame,
    block_buffer_len: usize,
    temp_effects: Vec<Box<dyn crate::modules::LocalEffect>>,
    temp_effect_params: Vec<(String, Vec<(String, f32)>)>,
    scheduled_messages: BinaryHeap<ScheduledMessage>,
    voice_memory: Arc<VoiceMemory>,
    sample_library: Arc<SampleLibrary>,
    global_effect_pool: GlobalEffectPool,
    // High-precision timing system
    precision_timer: HighPrecisionTimer,
    // Predictive sample loading system
    predictive_sample_manager: PredictiveSampleManager,
    // Pre-allocated buffer for sample mixing (real-time safe)
    sample_mix_buffer: Box<[f32]>,
}

impl AudioEngine {
    pub fn new(sample_rate: f32, buffer_size: usize, max_voices: usize, block_size: usize) -> Self {
        let mut registry = ModuleRegistry::new();
        registry.register_default_modules();

        let global_pool = Arc::new(MemoryPool::new(DEFAULT_MEMORY_SIZE));
        let voice_memory = Arc::new(VoiceMemory::new());
        let sample_library = Arc::new(SampleLibrary::new(
            DEFAULT_SAMPLE_COUNT,
            DEFAULT_SAMPLE_DIR,
            sample_rate as u32,
        ));

        Self::new_with_memory(
            sample_rate,
            buffer_size,
            max_voices,
            block_size,
            registry,
            global_pool,
            voice_memory,
            sample_library,
        )
    }

    pub fn new_with_registry(
        sample_rate: f32,
        buffer_size: usize,
        max_voices: usize,
        block_size: usize,
        registry: ModuleRegistry,
    ) -> Self {
        let global_pool = Arc::new(MemoryPool::new(DEFAULT_MEMORY_SIZE));
        let voice_memory = Arc::new(VoiceMemory::new());
        let sample_library = Arc::new(SampleLibrary::new(
            DEFAULT_SAMPLE_COUNT,
            DEFAULT_SAMPLE_DIR,
            sample_rate as u32,
        ));

        Self::new_with_memory(
            sample_rate,
            buffer_size,
            max_voices,
            block_size,
            registry,
            global_pool,
            voice_memory,
            sample_library,
        )
    }

    pub fn new_with_memory(
        sample_rate: f32,
        buffer_size: usize,
        max_voices: usize,
        block_size: usize,
        registry: ModuleRegistry,
        global_pool: Arc<MemoryPool>,
        voice_memory: Arc<VoiceMemory>,
        sample_library: Arc<SampleLibrary>,
    ) -> Self {
        let effective_block_size = if block_size == 0 {
            AUDIO_BLOCK_SIZE_FALLBACK
        } else {
            block_size
        };

        let mut voices = Vec::with_capacity(max_voices);

        for i in 0..max_voices {
            voices.push(Voice::new(i as VoiceId, 0, buffer_size));
        }

        let global_effect_pool =
            GlobalEffectPool::new(&registry, Arc::clone(&global_pool), MAX_TRACKS);

        let available_effects: Vec<String> = global_effect_pool
            .get_available_effects()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let mut tracks = Vec::with_capacity(MAX_TRACKS);
        for i in 0..MAX_TRACKS {
            let mut track = Track::new(i as TrackId, buffer_size);
            track.set_memory_pool(Arc::clone(&global_pool));
            track.initialize_global_effects(available_effects.clone());
            tracks.push(track);
        }

        for voice in &mut voices {
            voice.set_voice_memory(Arc::clone(&voice_memory));
        }

        let (block_buffer_ptr, block_buffer_len) = if let Some(ptr) =
            global_pool.allocate(effective_block_size * std::mem::size_of::<Frame>(), 16)
        {
            let frame_ptr = ptr.as_ptr() as *mut Frame;
            debug_assert!(!frame_ptr.is_null(), "Allocated buffer pointer is null");
            debug_assert_eq!(
                frame_ptr as usize % std::mem::align_of::<Frame>(),
                0,
                "Buffer not aligned for Frame"
            );

            // Safety:
            // - Pointer is non-null and properly aligned (checked above)
            // - Size is exactly what we allocated
            // - Memory is owned by the global pool and will outlive this engine
            // - We initialize the memory to zero
            unsafe {
                std::slice::from_raw_parts_mut(frame_ptr, effective_block_size).fill(Frame::ZERO);
            }
            (frame_ptr, effective_block_size)
        } else {
            panic!("Failed to allocate block buffer from memory pool");
        };

        // Initialize predictive sample manager with 2 worker threads
        let predictive_sample_manager = PredictiveSampleManager::new(
            Arc::clone(&sample_library),
            2, // Number of background loader threads
        );

        // Pre-allocate mix buffer for real-time safe sample mixing
        // Size: 2 seconds at max sample rate stereo (covers 90% of samples, analysis shows most <2 seconds)
        let max_sample_size = (sample_rate as usize) * 2 * 2; // 2 seconds stereo
        let sample_mix_buffer = vec![0.0f32; max_sample_size].into_boxed_slice();

        Self {
            voices,
            tracks,
            registry,
            sample_rate,
            buffer_size,
            block_size: effective_block_size,
            max_voices,
            next_voice_id: 0,
            block_buffer_ptr,
            block_buffer_len,
            temp_effects: Vec::with_capacity(16),
            temp_effect_params: Vec::with_capacity(16),
            scheduled_messages: BinaryHeap::new(),
            voice_memory,
            sample_library,
            global_effect_pool,
            precision_timer: HighPrecisionTimer::new(sample_rate),
            predictive_sample_manager,
            sample_mix_buffer,
        }
    }

    /// Initialize stream timing when audio processing starts
    pub fn initialize_stream_timing(&mut self) {
        self.precision_timer.initialize_stream_timing();
    }

    /// Initialize stream timing with Link time base for synchronized timing
    pub fn initialize_stream_timing_with_link_time(&mut self, link_time_base_micros: u64) {
        self.precision_timer
            .initialize_stream_timing_with_base(link_time_base_micros);
    }

    /// Convert timestamp to exact sample position for sample-accurate scheduling
    fn timestamp_to_exact_sample(&self, timestamp_micros: u64) -> Option<u64> {
        self.precision_timer
            .timestamp_to_exact_sample(timestamp_micros)
    }

    /// Start preloading common samples in the background
    /// Call this after engine initialization to improve responsiveness
    pub fn start_sample_preloading(&self) {
        self.predictive_sample_manager.preload_common_samples();
    }

    pub fn allocate_voice(&mut self) -> &mut Voice {
        let voice_index = self.next_voice_id as usize % self.max_voices;
        let voice = &mut self.voices[voice_index];

        voice.immediate_reset();
        voice.id = self.next_voice_id;
        voice.set_voice_index(voice_index);
        self.next_voice_id += 1;
        voice
    }

    pub fn release_voice(&mut self, voice_id: VoiceId) {
        for voice in &mut self.voices {
            if voice.id == voice_id && voice.is_active {
                voice.release();
                return;
            }
        }
    }

    pub fn stop_voice(&mut self, voice_id: VoiceId) {
        for voice in self.voices.iter_mut() {
            if voice.id == voice_id {
                voice.immediate_reset();
                return;
            }
        }
    }

    pub fn stop_all_voices(&mut self) {
        for voice in self.voices.iter_mut() {
            if voice.is_active {
                voice.immediate_reset();
            }
        }
    }

    fn post_processing(buffer: &mut [Frame]) {
        for frame in buffer.iter_mut() {
            frame.left = Self::soft_clip(frame.left);
            frame.right = Self::soft_clip(frame.right);
        }
    }

    #[inline]
    fn flush_denormals(x: f32) -> f32 {
        const DENORMAL_THRESHOLD: f32 = 1e-15;
        if x.abs() < DENORMAL_THRESHOLD { 0.0 } else { x }
    }

    #[inline]
    fn soft_clip(x: f32) -> f32 {
        let abs_x = x.abs();
        if abs_x <= 0.5 {
            x
        } else if abs_x <= 1.0 {
            let t = abs_x - 0.5;
            let soft = 0.5 + t * (0.75 - 0.25 * t);
            x.signum() * soft
        } else {
            let normalized = abs_x.min(2.0) / 2.0;
            let soft = normalized * (3.0 - normalized * normalized) * 0.5;
            (x.signum() * soft).clamp(-1.0, 1.0)
        }
    }

    pub fn process(&mut self, output: &mut [Frame]) {
        let len = output.len().min(self.buffer_size);
        let mut processed = 0;

        while processed < len {
            let remaining = len - processed;
            let block_len = remaining.min(self.block_size);

            // Process sample-accurate scheduled messages for this block
            self.process_scheduled_messages_sample_accurate(block_len, None);

            if let Some(voice_buffer) = self.voice_memory.get_voice_buffer(0) {
                (0..block_len).for_each(|i| {
                    voice_buffer[i] = 0.0;
                });
            }

            // Safety: We're creating a slice from our pre-allocated buffer
            // - The pointer is valid and aligned (checked at allocation)
            // - The length is bounded by our allocation size
            // - The lifetime is tied to self, which owns the memory
            let block_slice = unsafe {
                debug_assert!(block_len <= self.block_buffer_len);
                std::slice::from_raw_parts_mut(self.block_buffer_ptr, block_len)
            };

            Frame::process_block_zero(block_slice);

            // Update pending samples - hot-swap loaded samples into playing voices
            self.update_pending_samples();

            for track in &mut self.tracks {
                track.process(
                    &mut self.voices,
                    block_slice,
                    self.sample_rate,
                    &mut self.global_effect_pool,
                );
            }

            for frame in block_slice.iter_mut() {
                frame.left = Self::flush_denormals(frame.left);
                frame.right = Self::flush_denormals(frame.right);
            }

            Self::post_processing(block_slice);

            output[processed..processed + block_len].copy_from_slice(block_slice);
            processed += block_len;

            // Update sample count for timing accuracy
            self.precision_timer.advance_samples(block_len as u64);
        }

        self.cleanup_finished_voices();
    }

    fn cleanup_finished_voices(&mut self) {
        for voice in self.voices.iter_mut() {
            if voice.is_active && voice.envelope_state.is_finished() {
                voice.immediate_reset();
            }
        }
    }

    /// Process scheduled messages with sample-accurate timing within the block.
    ///
    /// This provides maximum precision by checking for scheduled events at every
    /// sample within the block, enabling sub-sample accurate timing.
    ///
    /// Accepts all messages unconditionally:
    /// - Late messages (past due time): Execute immediately
    /// - Future messages: Execute at precise sample timing
    fn process_scheduled_messages_sample_accurate(
        &mut self,
        block_len: usize,
        status_tx: Option<&mpsc::Sender<EngineStatusMessage>>,
    ) {
        let base_sample_count = self.precision_timer.get_current_sample_count();

        // Collect all messages that should fire within this block
        let mut block_messages = Vec::with_capacity(16);

        let current_time = self.precision_timer.get_current_timestamp_exact();

        while let Some(scheduled) = self.scheduled_messages.peek() {
            if let Some(target_sample) = self.timestamp_to_exact_sample(scheduled.due_time_micros) {
                let sample_offset = target_sample as i64 - base_sample_count as i64;

                if sample_offset >= 0 && sample_offset < block_len as i64 {
                    let scheduled = self.scheduled_messages.pop().unwrap();
                    block_messages.push((sample_offset as usize, scheduled));
                } else if sample_offset < 0 {
                    let scheduled = self.scheduled_messages.pop().unwrap();
                    block_messages.push((0, scheduled));
                } else {
                    break;
                }
            } else if scheduled.due_time_micros <= current_time {
                let scheduled = self.scheduled_messages.pop().unwrap();
                block_messages.push((0, scheduled));
            } else {
                break;
            }
        }

        block_messages.sort_by_key(|(sample_offset, _)| *sample_offset);

        for (sample_offset, scheduled) in block_messages {
            let fractional_offset = sample_offset as f32;
            self.handle_message_with_exact_sample_timing(
                &scheduled.message,
                fractional_offset,
                status_tx,
            );
        }
    }

    fn handle_message_with_exact_sample_timing(
        &mut self,
        message: &EngineMessage,
        _fractional_offset: f32,
        status_tx: Option<&mpsc::Sender<EngineStatusMessage>>,
    ) {
        self.handle_message_with_optional_timing(message, None, status_tx);
    }

    fn handle_message_immediate(
        &mut self,
        message: &EngineMessage,
        status_tx: Option<&mpsc::Sender<EngineStatusMessage>>,
    ) {
        self.handle_message_with_optional_timing(message, None, status_tx);
    }

    fn handle_message_with_optional_timing(
        &mut self,
        message: &EngineMessage,
        sample_offset: Option<usize>,
        status_tx: Option<&mpsc::Sender<EngineStatusMessage>>,
    ) {
        match message {
            EngineMessage::Play {
                voice_id: _,
                track_id,
                source_name,
                parameters,
            } => {
                let source =
                    match std::panic::catch_unwind(|| self.registry.create_source(source_name)) {
                        Ok(src) => src,
                        Err(_) => {
                            return;
                        }
                    };

                if source.is_none() {
                    if let Some(tx) = status_tx {
                        let available_sources = self.registry.sources.keys().cloned().collect();
                        let error = EngineError::InvalidSource {
                            source_name: source_name.clone(),
                            voice_id: self.next_voice_id,
                            available_sources,
                        };
                        let _ = tx.send(EngineStatusMessage::Error(error));
                    } else {
                        let _available_sources: Vec<String> =
                            self.registry.sources.keys().cloned().collect();
                    }
                    return;
                }

                self.temp_effects.clear();
                self.temp_effect_params.clear();

                for (key, value) in parameters {
                    let should_check_effects = value.downcast_ref::<f32>().is_some()
                        || value.downcast_ref::<Modulation>().is_some();

                    if should_check_effects && !crate::registry::is_engine_parameter(key) {
                        for effect_name in self.registry.local_effects.keys() {
                            if let Some(temp_effect) =
                                self.registry.create_local_effect(effect_name)
                            {
                                let param_exists = temp_effect
                                    .get_parameter_descriptors()
                                    .iter()
                                    .any(|d| d.matches_name(key));

                                if param_exists {
                                    let already_added = self.temp_effects.iter().any(|e| {
                                        let e_params = e.get_parameter_descriptors();
                                        let temp_params = temp_effect.get_parameter_descriptors();
                                        e_params.len() == temp_params.len()
                                            && e_params
                                                .iter()
                                                .zip(temp_params.iter())
                                                .all(|(a, b)| a.name == b.name)
                                    });

                                    if !already_added {
                                        self.temp_effects.push(temp_effect);
                                    }
                                    break;
                                }
                            }
                        }

                        if let Some(effect_name) = self.registry.is_global_effect_wet_parameter(key)
                        {
                            if let Some(value_f32) = value.downcast_ref::<f32>() {
                                let effect_name_owned = effect_name.to_string();
                                if let Some((_, params)) = self
                                    .temp_effect_params
                                    .iter_mut()
                                    .find(|(name, _)| name == &effect_name_owned)
                                {
                                    params.push((key.clone(), *value_f32));
                                } else {
                                    self.temp_effect_params
                                        .push((effect_name_owned, vec![(key.clone(), *value_f32)]));
                                }
                            }
                        } else {
                            for effect_name in self.registry.global_effects.keys() {
                                if let Some(temp_effect) =
                                    self.registry.create_global_effect(effect_name)
                                {
                                    let param_exists = temp_effect
                                        .get_parameter_descriptors()
                                        .iter()
                                        .any(|d| d.matches_name(key));
                                    if param_exists {
                                        if let Some(value_f32) = value.downcast_ref::<f32>() {
                                            let effect_name_owned = effect_name.to_string();
                                            if let Some((_, params)) = self
                                                .temp_effect_params
                                                .iter_mut()
                                                .find(|(name, _)| name == &effect_name_owned)
                                            {
                                                params.push((key.clone(), *value_f32));
                                            } else {
                                                self.temp_effect_params.push((
                                                    effect_name_owned,
                                                    vec![(key.clone(), *value_f32)],
                                                ));
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                let sample_result = if source_name == "sample" {
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        self.prepare_sample_data(parameters, status_tx, self.next_voice_id)
                    })) {
                        Ok(result) => result,
                        Err(_) => None,
                    }
                } else {
                    None
                };

                let voice_index = {
                    let voice = self.allocate_voice();
                    voice.track_id = *track_id;
                    voice.voice_index
                };

                {
                    let voice = &mut self.voices[voice_index];
                    if let Some(s) = source {
                        voice.source = Some(s);
                    }

                    voice.local_effects = std::mem::take(&mut self.temp_effects);

                    if let (Some((data, duration)), Some(source_box)) =
                        (sample_result, &mut voice.source)
                    {
                        if let Some(sampler) = source_box
                            .as_any_mut()
                            .downcast_mut::<crate::modules::source::sample::StereoSampler>(
                        ) {
                            sampler.load_sample_data(data);
                        }

                        if !parameters.contains_key("dur") {
                            voice.set_engine_parameter(ENGINE_PARAM_DUR, duration);
                        }
                    }

                    for (key, value) in parameters {
                        if let Some(value_f32) = value.downcast_ref::<f32>() {
                            if let Some(param_index) =
                                crate::registry::get_engine_parameter_index(key)
                            {
                                voice.set_engine_parameter(param_index, *value_f32);
                            } else {
                                if let Some(source) = &mut voice.source {
                                    source.set_parameter(key, *value_f32);
                                }

                                for effect in &mut voice.local_effects {
                                    effect.set_parameter(key, *value_f32);
                                }
                            }
                        } else if let Some(modulation) = value.downcast_ref::<Modulation>() {
                            let param_name = Box::leak(key.clone().into_boxed_str());
                            voice.add_modulation(param_name, *modulation);
                        }
                    }

                    voice.trigger();

                    // Sub-sample precision: advance envelope by exact sample offset
                    if let Some(offset) = sample_offset {
                        let sample_time = offset as f32 / self.sample_rate;
                        voice.advance_envelope_by_time(sample_time, self.sample_rate);
                    }
                }

                let track_idx = *track_id as usize;
                if track_idx < self.tracks.len() {
                    for (effect_name, params) in &self.temp_effect_params {
                        self.tracks[track_idx].update_global_effect(effect_name, params);
                        // Also update the actual effect in the pool
                        if let Some(effect) = self
                            .global_effect_pool
                            .get_effect_mut(effect_name, track_idx)
                        {
                            for (param_name, value) in params {
                                if !param_name.ends_with("_wet") {
                                    effect.set_parameter(param_name, *value);
                                }
                            }
                        }
                    }
                }
            }
            EngineMessage::Update {
                voice_id,
                track_id: _,
                parameters,
            } => {
                for voice in &mut self.voices {
                    if voice.id == *voice_id && voice.is_active {
                        for (key, value) in parameters {
                            if let Some(value_f32) = value.downcast_ref::<f32>() {
                                if let Some(param_index) =
                                    crate::registry::get_engine_parameter_index(key)
                                {
                                    voice.set_engine_parameter(param_index, *value_f32);
                                } else if let Some(source) = &mut voice.source {
                                    source.set_parameter(key, *value_f32);
                                } else {
                                    for effect in &mut voice.local_effects {
                                        effect.set_parameter(key, *value_f32);
                                    }
                                }
                            } else if let Some(modulation) = value.downcast_ref::<Modulation>() {
                                let param_name = Box::leak(key.clone().into_boxed_str());
                                voice.add_modulation(param_name, *modulation);
                            }
                        }
                        break;
                    }
                }
            }
            EngineMessage::Stop => {
                self.stop_all_voices();
            }
            EngineMessage::Panic => {
                self.stop_all_voices();
            }
        }
    }

    pub fn schedule_message(&mut self, message: EngineMessage, due_time_micros: u64) {
        self.scheduled_messages.push(ScheduledMessage {
            due_time_micros,
            message,
        });
    }

    /// Start lock-free audio thread implementation using crossbeam channels
    pub fn start_audio_thread(
        engine: AudioEngine,
        block_size: u32,
        max_voices: usize,
        sample_rate: u32,
        buffer_size: usize,
        output_device: Option<String>,
        command_rx: Receiver<ScheduledEngineMessage>,
        status_tx: Option<Sender<EngineStatusMessage>>,
        audio_priority: u8,
    ) -> thread::JoinHandle<()> {
        thread::Builder::new()
            .name("audio".to_string())
            .spawn(move || {
                Self::run_audio_thread(
                    engine,
                    sample_rate,
                    buffer_size,
                    output_device,
                    command_rx,
                    status_tx,
                    block_size,
                    max_voices,
                    audio_priority,
                );
            })
            .expect("Failed to spawn audio thread")
    }

    fn run_audio_thread(
        mut engine: AudioEngine,
        sample_rate: u32,
        buffer_size: usize,
        output_device: Option<String>,
        command_rx: Receiver<ScheduledEngineMessage>,
        _status_tx: Option<Sender<EngineStatusMessage>>,
        block_size: u32,
        _max_voices: usize,
        audio_priority: u8,
    ) {
        // Set real-time priority for audio thread (if enabled)
        if audio_priority > 0 {
            // Map user priority (0-99) to platform-appropriate range
            let platform_priority = map_to_platform_priority(audio_priority);

            match ThreadPriorityValue::try_from(platform_priority) {
                Ok(priority_value) => {
                    let priority = ThreadPriority::Crossplatform(priority_value);
                    match set_current_thread_priority(priority) {
                        Ok(_) => println!(
                            "Audio thread real-time priority set to {} (platform: {})",
                            audio_priority, platform_priority
                        ),
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to set audio thread real-time priority: {}",
                                e
                            );
                            eprintln!(
                                "Consider running with elevated privileges for better audio performance"
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Invalid priority value {}: {}",
                        platform_priority, e
                    );
                }
            }
        } else {
            println!("Audio thread real-time priority disabled (priority = 0)");
        }

        use crate::device_selector::{DeviceSelector, SelectionResult};
        use cpal::StreamConfig;
        use cpal::traits::{DeviceTrait, StreamTrait};

        let selector = DeviceSelector::new(sample_rate);
        let device_info = match selector.select_output_device(output_device) {
            SelectionResult::Success(info) => {
                println!(
                    "Successfully selected audio device: {} {}",
                    info.name,
                    if info.is_default { "(default)" } else { "" }
                );
                info
            }
            SelectionResult::Fallback(info, reason) => {
                println!("Audio device fallback: {}", reason);
                info
            }
            SelectionResult::Error(err) => {
                eprintln!("Failed to select audio device: {}", err);
                std::process::exit(1);
            }
        };

        let device = device_info.device;

        let config = StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(buffer_size as u32),
        };

        let mut pre_allocated_buffer = vec![Frame::ZERO; buffer_size];
        let _effective_block_size = block_size.min(buffer_size as u32) as usize;

        let mut stream_initialized = false;
        let audio_should_exit = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let audio_exit_flag = audio_should_exit.clone();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let frames_needed = data.len() / 2;
                    let actual_frames = frames_needed.min(buffer_size);
                    let buffer_slice = &mut pre_allocated_buffer[..actual_frames];

                    Frame::process_block_zero(buffer_slice);

                    if !stream_initialized {
                        // ALWAYS wait for first message to get correct Link time
                        if let Ok(ScheduledEngineMessage::Scheduled(scheduled)) =
                            command_rx.try_recv()
                        {
                            let init_time = scheduled.due_time_micros.saturating_sub(100_000);
                            engine.initialize_stream_timing_with_link_time(init_time);
                            engine.schedule_message(scheduled.message, scheduled.due_time_micros);
                            stream_initialized = true;
                        } else {
                            return;
                        }
                    }

                    // Process all pending commands (lock-free!)
                    while let Ok(scheduled_msg) = command_rx.try_recv() {
                        match scheduled_msg {
                            ScheduledEngineMessage::Immediate(msg) => {
                                if matches!(msg, crate::types::EngineMessage::Stop) {
                                    audio_exit_flag
                                        .store(true, std::sync::atomic::Ordering::Relaxed);
                                }
                                engine.handle_message_immediate(&msg, None);
                            }
                            ScheduledEngineMessage::Scheduled(scheduled) => {
                                engine
                                    .schedule_message(scheduled.message, scheduled.due_time_micros);
                            }
                        }
                    }

                    // Process audio (no mutex!)
                    engine.process(buffer_slice);

                    // Fill output buffer
                    data.fill(0.0);
                    for (i, frame) in buffer_slice.iter().enumerate() {
                        let idx = i * 2;
                        if idx + 1 < data.len() {
                            data[idx] = frame.left;
                            data[idx + 1] = frame.right;
                        }
                    }
                },
                |_| {},
                None, // No timeout
            )
            .expect("Failed to build audio stream");

        stream.play().expect("Failed to start audio stream");

        println!(
            "Audio thread started at {}Hz, buffer: {}",
            sample_rate, buffer_size
        );

        // Keep the stream alive until shutdown signal
        loop {
            if audio_should_exit.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        drop(stream);
    }

    /// Prepare sample data using predictive loading system (REAL-TIME SAFE)
    ///
    /// This function maintains the same API as before but eliminates all blocking I/O
    /// and heap allocations from the audio thread. Instead, it returns:
    /// - Ready samples immediately if already loaded
    /// - Silent voice with pending sample replacement if loading
    /// - Error for truly missing samples
    fn prepare_sample_data(
        &mut self,
        parameters: &std::collections::HashMap<String, Box<dyn std::any::Any + Send>>,
        status_tx: Option<&mpsc::Sender<EngineStatusMessage>>,
        voice_id: VoiceId,
    ) -> Option<(Vec<f32>, f32)> {
        let sample_name = parameters
            .get("sample_name")
            .and_then(|v| v.downcast_ref::<String>());

        if sample_name.is_none() {
            if let Some(tx) = status_tx {
                let error = EngineError::ParameterError {
                    param: "sample_name".to_string(),
                    value: "missing".to_string(),
                    reason: "sample_name parameter is required for sample playback".to_string(),
                    voice_id,
                    valid_params: vec![
                        "sample_name".to_string(),
                        "sample_number".to_string(),
                        "speed".to_string(),
                    ],
                };
                let _ = tx.send(EngineStatusMessage::Error(error));
            }
            return None;
        }

        let sample_name = sample_name.unwrap().clone();

        let sample_number = parameters
            .get("sample_number")
            .and_then(|v| v.downcast_ref::<f32>())
            .copied()
            .unwrap_or(0.0);

        let sample_index = sample_number.floor() as usize;
        let mix_factor = sample_number.fract();

        let speed = parameters
            .get("speed")
            .and_then(|v| v.downcast_ref::<f32>())
            .copied()
            .unwrap_or(1.0);

        // Use predictive sample manager (REAL-TIME SAFE)
        match self
            .predictive_sample_manager
            .get_sample_immediate(&sample_name, sample_index)
        {
            SampleResult::Ready(sample_data) => {
                // Sample is immediately available
                let base_duration = (sample_data.len() / 2) as f32 / self.sample_rate;
                let adjusted_duration = base_duration / speed.abs();

                let final_data = if mix_factor > 0.0 {
                    // Try to get the next sample for mixing
                    match self
                        .predictive_sample_manager
                        .get_sample_immediate(&sample_name, sample_index + 1)
                    {
                        SampleResult::Ready(next_sample_data) => {
                            // Mix the two samples using pre-allocated buffer (REAL-TIME SAFE)
                            let len = sample_data.len().min(next_sample_data.len());

                            // Ensure our pre-allocated buffer is large enough
                            if len <= self.sample_mix_buffer.len() {
                                // Use pre-allocated buffer slice for mixing
                                let mix_slice = &mut self.sample_mix_buffer[..len];

                                for i in 0..len {
                                    mix_slice[i] = sample_data[i] * (1.0 - mix_factor)
                                        + next_sample_data[i] * mix_factor;
                                }

                                // Return slice as Vec (copies from pre-allocated buffer)
                                mix_slice.to_vec()
                            } else {
                                // Fallback: sample too large for buffer, use current sample only
                                rt_eprintln!(
                                    "Sample too large for mix buffer: {} > {}",
                                    len,
                                    self.sample_mix_buffer.len()
                                );
                                sample_data
                            }
                        }
                        _ => {
                            // Next sample not available, use current sample only
                            sample_data
                        }
                    }
                } else {
                    sample_data
                };

                Some((final_data, adjusted_duration))
            }
            SampleResult::Loading(loading_sample_name, loading_sample_index) => {
                // Sample is being loaded - register as pending voice
                self.predictive_sample_manager.register_pending_voice(
                    voice_id,
                    loading_sample_name,
                    loading_sample_index,
                );

                // Send user feedback about loading
                if let Some(tx) = status_tx {
                    let _ = tx.send(EngineStatusMessage::Info(format!(
                        "Loading sample: {}",
                        sample_name
                    )));
                }

                // Return None to create silent voice that will be replaced when sample loads
                None
            }
            SampleResult::NotFound => {
                // Sample not found in library
                if let Some(tx) = status_tx {
                    let available_folders: Vec<String> = self.sample_library.get_folders();
                    let error = EngineError::SampleNotFound {
                        folder: sample_name.clone(),
                        index: sample_index,
                        voice_id,
                        available_folders,
                    };
                    let _ = tx.send(EngineStatusMessage::Error(error));
                }
                None
            }
        }
    }

    /// Update pending voices with loaded samples (called each audio block)
    /// This enables hot-swapping from silence to real sample when loading completes
    fn update_pending_samples(&mut self) {
        let ready_samples = self.predictive_sample_manager.update_pending_samples();

        for (voice_id, sample_data) in ready_samples {
            // Find the voice with this ID
            if let Some(voice) = self.voices.iter_mut().find(|v| v.id == voice_id) {
                if voice.is_active {
                    // Hot-swap the sample data into the playing voice
                    if let Some(sampler) = voice.source.as_mut().and_then(|s| {
                        s.as_any_mut()
                            .downcast_mut::<crate::modules::source::sample::StereoSampler>()
                    }) {
                        sampler.load_sample_data(sample_data);
                        sampler.trigger(); // Restart from beginning with new sample
                    }
                }
            }
        }
    }
}

// SAFETY: AudioEngine is Send because:
// - block_buffer_ptr points to memory owned by the thread-safe MemoryPool
// - All other fields are already Send
unsafe impl Send for AudioEngine {}

// SAFETY: AudioEngine is Sync because:
// - It's only accessed from the audio thread during processing
// - The memory pool handles synchronization internally
unsafe impl Sync for AudioEngine {}
