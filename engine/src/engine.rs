use crate::memory::{MemoryPool, SampleLibrary, VoiceMemory};
use crate::modulation::Modulation;
use crate::modules::Frame;
use crate::registry::ModuleRegistry;
use crate::server::ScheduledEngineMessage;
use crate::track::Track;
use crate::types::{EngineMessage, ScheduledMessage, TrackId, VoiceId};
use crate::voice::Voice;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};


pub struct AudioEngine {
    pub voices: Vec<Voice>,
    pub tracks: Vec<Track>,
    pub registry: ModuleRegistry,
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub block_size: usize,
    max_voices: usize,
    next_voice_id: VoiceId,
    block_buffer: Vec<Frame>,
    temp_effects: Vec<Box<dyn crate::modules::LocalEffect>>,
    temp_effect_params: Vec<(String, Vec<(String, f32)>)>,
    scheduled_messages: BinaryHeap<ScheduledMessage>,
    voice_memory: Arc<VoiceMemory>,
    sample_library: Arc<Mutex<SampleLibrary>>,
}

impl AudioEngine {
    pub fn new(sample_rate: f32, buffer_size: usize, max_voices: usize, block_size: usize) -> Self {
        let mut registry = ModuleRegistry::new();
        registry.register_default_modules();

        let global_pool = Arc::new(MemoryPool::new(64 * 1024 * 1024));
        let voice_memory = Arc::new(VoiceMemory::new());
        let sample_library = Arc::new(Mutex::new(SampleLibrary::new(1024, "./samples", sample_rate as u32)));

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
        let global_pool = Arc::new(MemoryPool::new(64 * 1024 * 1024));
        let voice_memory = Arc::new(VoiceMemory::new());
        let sample_library = Arc::new(Mutex::new(SampleLibrary::new(1024, "./samples", sample_rate as u32)));

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
        sample_library: Arc<Mutex<SampleLibrary>>,
    ) -> Self {
        const MAX_TRACKS: usize = 10;
        let effective_block_size = if block_size == 0 { 256 } else { block_size };

        let mut voices = Vec::with_capacity(max_voices);

        for i in 0..max_voices {
            voices.push(Voice::new(i as VoiceId, 0, buffer_size));
        }

        let mut tracks = Vec::with_capacity(MAX_TRACKS);
        for i in 0..MAX_TRACKS {
            let mut track = Track::new(i as TrackId, buffer_size);
            track.set_memory_pool(Arc::clone(&global_pool));
            track.initialize_global_effects(&registry);
            tracks.push(track);
        }

        for voice in &mut voices {
            voice.set_voice_memory(Arc::clone(&voice_memory));
        }

        let block_buffer = if let Some(ptr) =
            global_pool.allocate(effective_block_size * std::mem::size_of::<Frame>(), 16)
        {
            unsafe {
                std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut Frame, effective_block_size)
                    .fill(Frame::ZERO);
                Vec::from_raw_parts(
                    ptr.as_ptr() as *mut Frame,
                    effective_block_size,
                    effective_block_size,
                )
            }
        } else {
            vec![Frame::ZERO; effective_block_size]
        };

        Self {
            voices,
            tracks,
            registry,
            sample_rate,
            buffer_size,
            block_size: effective_block_size,
            max_voices,
            next_voice_id: 0,
            block_buffer,
            temp_effects: Vec::with_capacity(16),
            temp_effect_params: Vec::with_capacity(16),
            scheduled_messages: BinaryHeap::new(),
            voice_memory,
            sample_library,
        }
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
            self.process_scheduled_messages();

            let remaining = len - processed;
            let block_len = remaining.min(self.block_size);

            if let Some(voice_buffer) = self.voice_memory.get_voice_buffer(0) {
                (0..block_len).for_each(|i| {
                    voice_buffer[i] = 0.0;
                });
            }

            let block_slice = &mut self.block_buffer[..block_len];
            Frame::process_block_zero(block_slice);

            for track in &mut self.tracks {
                track.process(&mut self.voices, block_slice, self.sample_rate);
            }

            for frame in block_slice.iter_mut() {
                frame.left = Self::flush_denormals(frame.left);
                frame.right = Self::flush_denormals(frame.right);
            }

            Self::post_processing(block_slice);

            output[processed..processed + block_len].copy_from_slice(block_slice);
            processed += block_len;
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

    fn process_scheduled_messages(&mut self) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        while let Some(scheduled) = self.scheduled_messages.peek() {
            if scheduled.due_time_ms <= now_ms {
                let scheduled = self.scheduled_messages.pop().unwrap();
                self.handle_message_immediate(&scheduled.message);
            } else {
                break;
            }
        }
    }

    fn handle_message_immediate(&mut self, message: &EngineMessage) {
        match message {
            EngineMessage::Play {
                voice_id: _,
                track_id,
                source_name,
                parameters,
            } => {
                let source = self.registry.create_source(source_name);

                if source.is_none() {
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
                                    .any(|d| d.name == key || d.aliases.contains(&key.as_str()));

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

                        if let Some(effect_name) = self.registry.is_global_effect_wet_parameter(key) {
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
                        } else {
                            for effect_name in self.registry.global_effects.keys() {
                                if let Some(temp_effect) =
                                    self.registry.create_global_effect(effect_name)
                                {
                                    let param_exists = temp_effect
                                        .get_parameter_descriptors()
                                        .iter()
                                        .any(|d| d.name == key || d.aliases.contains(&key.as_str()));

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
                    self.prepare_sample_data(parameters)
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
                            voice.set_engine_parameter(crate::registry::ENGINE_PARAM_DUR, duration);
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
                }

                let track_idx = *track_id as usize;
                if track_idx < self.tracks.len() {
                    for (effect_name, params) in &self.temp_effect_params {
                        self.tracks[track_idx].update_global_effect(effect_name, params);
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

    pub fn schedule_message(&mut self, message: EngineMessage, due_time_ms: u64) {
        self.scheduled_messages.push(ScheduledMessage {
            due_time_ms,
            message,
        });
    }

    pub fn start_audio_thread(
        engine: Arc<Mutex<AudioEngine>>,
        block_size: u32,
        max_voices: usize,
        sample_rate: u32,
        buffer_size: usize,
        output_device: Option<String>,
        message_rx: mpsc::Receiver<ScheduledEngineMessage>,
    ) -> thread::JoinHandle<()> {
        thread::Builder::new()
            .name("audio".to_string())
            .spawn(move || {
                Self::run_audio_thread(
                    engine,
                    sample_rate,
                    buffer_size,
                    output_device,
                    message_rx,
                    block_size,
                    max_voices,
                );
            })
            .expect("Failed to spawn audio thread")
    }

    fn run_audio_thread(
        engine: Arc<Mutex<AudioEngine>>,
        sample_rate: u32,
        buffer_size: usize,
        output_device: Option<String>,
        message_rx: mpsc::Receiver<ScheduledEngineMessage>,
        block_size: u32,
        _max_voices: usize,
    ) {
        use cpal::StreamConfig;
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        let host = cpal::default_host();

        let device = if let Some(device_name) = output_device {
            host.output_devices()
                .unwrap()
                .find(|d| d.name().unwrap_or_default() == device_name)
                .unwrap_or_else(|| {
                    host.default_output_device()
                        .expect("No output device available")
                })
        } else {
            host.default_output_device()
                .expect("No output device available")
        };


        let config = StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(buffer_size as u32),
        };

        let mut pre_allocated_buffer = vec![Frame::ZERO; buffer_size];
        let _effective_block_size = block_size.min(buffer_size as u32) as usize;

        let engine_clone = Arc::clone(&engine);
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let frames_needed = data.len() / 2;
                    let actual_frames = frames_needed.min(buffer_size);
                    let buffer_slice = &mut pre_allocated_buffer[..actual_frames];

                    // Always clear the buffer first
                    Frame::process_block_zero(buffer_slice);
                    
                    // Try to get engine lock with minimal hold time
                    if let Ok(mut engine_lock) = engine_clone.try_lock() {
                        // Process audio - this is the only place we hold the lock
                        engine_lock.process(buffer_slice);
                        // Lock is automatically released here
                    } else {
                        // Lock failed - output silence instead of dropping frames
                        // This prevents audio dropouts when OSC thread holds the lock
                    }

                    // Always fill output buffer (even if it's silence)
                    data.fill(0.0);
                    
                    for (i, frame) in buffer_slice.iter().enumerate() {
                        let idx = i * 2;
                        if idx + 1 < data.len() {
                            data[idx] = frame.left;
                            data[idx + 1] = frame.right;
                        }
                    }
                },
                |err| {
                    eprintln!("Audio stream error: {}", err);
                },
                None,
            )
            .expect("Failed to build output stream");

        stream.play().expect("Failed to start audio stream");

        // Collect messages in batches to minimize lock frequency
        let mut pending_messages = Vec::with_capacity(32);
        
        loop {
            // Collect all available messages
            pending_messages.clear();
            while let Ok(scheduled_msg) = message_rx.try_recv() {
                match scheduled_msg {
                    ScheduledEngineMessage::Immediate(EngineMessage::Stop)
                    | ScheduledEngineMessage::Immediate(EngineMessage::Panic) => return,
                    _ => {
                        pending_messages.push(scheduled_msg);
                        if pending_messages.len() >= 32 {
                            break; // Process in smaller batches to avoid long lock holds
                        }
                    }
                }
            }

            // Process messages in batch with a single lock acquisition
            if !pending_messages.is_empty() {
                // Use blocking lock since this is message thread, not audio thread
                if let Ok(mut engine_lock) = engine.lock() {
                    for scheduled_msg in pending_messages.drain(..) {
                        match scheduled_msg {
                            ScheduledEngineMessage::Immediate(msg) => {
                                engine_lock.handle_message_immediate(&msg);
                            }
                            ScheduledEngineMessage::Scheduled(scheduled) => {
                                engine_lock.schedule_message(scheduled.message, scheduled.due_time_ms);
                            }
                        }
                    }
                    // Lock released here
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }


    fn prepare_sample_data(
        &mut self,
        parameters: &std::collections::HashMap<String, Box<dyn std::any::Any + Send>>,
    ) -> Option<(Vec<f32>, f32)> {
        let sample_name = parameters
            .get("sample_name")
            .and_then(|v| v.downcast_ref::<String>())?
            .clone();

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

        // Use blocking lock since this is called during voice setup, not real-time audio processing
        if let Ok(mut sample_lib) = self.sample_library.lock() {
            if let Some(sample_data) = sample_lib.get_sample(&sample_name, sample_index) {
                let mut final_data = sample_data.to_vec();
                let base_duration = (final_data.len() / 2) as f32 / self.sample_rate;
                let adjusted_duration = base_duration / speed.abs();

                if mix_factor > 0.0 {
                    if let Some(next_sample_data) =
                        sample_lib.get_sample(&sample_name, sample_index + 1)
                    {
                        let len = final_data.len().min(next_sample_data.len());
                        for i in 0..len {
                            final_data[i] = final_data[i] * (1.0 - mix_factor)
                                + next_sample_data[i] * mix_factor;
                        }
                    }
                }

                return Some((final_data, adjusted_duration));
            }
        }
        None
    }
}
