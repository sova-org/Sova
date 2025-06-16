use crate::memory::pool::MemoryPool;
use std::ptr::NonNull;

const MAX_VOICES: usize = 128;
const VOICE_BUFFER_SIZE: usize = 4096;
const MODULATION_SLOTS: usize = 32;
const ENVELOPE_STAGES: usize = 4;

pub struct VoiceMemory {
    #[allow(dead_code)]
    pool: MemoryPool,
    voice_buffers: [NonNull<f32>; MAX_VOICES],
    modulation_buffers: [NonNull<f32>; MAX_VOICES * MODULATION_SLOTS],
    envelope_data: [NonNull<f32>; MAX_VOICES * ENVELOPE_STAGES],
    filter_states: [NonNull<f32>; MAX_VOICES * 4],
}

impl Default for VoiceMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceMemory {
    pub fn new() -> Self {
        let total_size = Self::calculate_total_size();
        let pool = MemoryPool::new(total_size);

        let mut voice_buffers = [NonNull::dangling(); MAX_VOICES];
        let mut modulation_buffers = [NonNull::dangling(); MAX_VOICES * MODULATION_SLOTS];
        let mut envelope_data = [NonNull::dangling(); MAX_VOICES * ENVELOPE_STAGES];
        let mut filter_states = [NonNull::dangling(); MAX_VOICES * 4];

        for i in 0..MAX_VOICES {
            voice_buffers[i] = pool.allocate(VOICE_BUFFER_SIZE * 4, 16).unwrap().cast();
        }

        for i in 0..(MAX_VOICES * MODULATION_SLOTS) {
            modulation_buffers[i] = pool.allocate(VOICE_BUFFER_SIZE * 4, 16).unwrap().cast();
        }

        for i in 0..(MAX_VOICES * ENVELOPE_STAGES) {
            envelope_data[i] = pool.allocate(64, 16).unwrap().cast();
        }

        for i in 0..(MAX_VOICES * 4) {
            filter_states[i] = pool.allocate(32, 16).unwrap().cast();
        }

        Self {
            pool,
            voice_buffers,
            modulation_buffers,
            envelope_data,
            filter_states,
        }
    }

    fn calculate_total_size() -> usize {
        let voice_mem = MAX_VOICES * VOICE_BUFFER_SIZE * 4;
        let mod_mem = MAX_VOICES * MODULATION_SLOTS * VOICE_BUFFER_SIZE * 4;
        let env_mem = MAX_VOICES * ENVELOPE_STAGES * 64;
        let filter_mem = MAX_VOICES * 4 * 32;

        voice_mem + mod_mem + env_mem + filter_mem + 65536
    }

    pub fn get_voice_buffer(&self, voice_id: usize) -> Option<&mut [f32]> {
        if voice_id >= MAX_VOICES {
            return None;
        }
        unsafe {
            Some(std::slice::from_raw_parts_mut(
                self.voice_buffers[voice_id].as_ptr(),
                VOICE_BUFFER_SIZE,
            ))
        }
    }

    pub fn get_modulation_buffer(&self, voice_id: usize, mod_slot: usize) -> Option<&mut [f32]> {
        if voice_id >= MAX_VOICES || mod_slot >= MODULATION_SLOTS {
            return None;
        }
        let index = voice_id * MODULATION_SLOTS + mod_slot;
        unsafe {
            Some(std::slice::from_raw_parts_mut(
                self.modulation_buffers[index].as_ptr(),
                VOICE_BUFFER_SIZE,
            ))
        }
    }

    pub fn get_envelope_data(&self, voice_id: usize, stage: usize) -> Option<&mut [f32]> {
        if voice_id >= MAX_VOICES || stage >= ENVELOPE_STAGES {
            return None;
        }
        let index = voice_id * ENVELOPE_STAGES + stage;
        unsafe {
            Some(std::slice::from_raw_parts_mut(
                self.envelope_data[index].as_ptr(),
                16,
            ))
        }
    }

    pub fn get_filter_state(&self, voice_id: usize, filter_id: usize) -> Option<&mut [f32]> {
        if voice_id >= MAX_VOICES || filter_id >= 4 {
            return None;
        }
        let index = voice_id * 4 + filter_id;
        unsafe {
            Some(std::slice::from_raw_parts_mut(
                self.filter_states[index].as_ptr(),
                8,
            ))
        }
    }
}

unsafe impl Send for VoiceMemory {}
unsafe impl Sync for VoiceMemory {}
