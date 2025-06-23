//! Centralized constants for the Sova audio engine
//! Contains only the constants that are actually used throughout the codebase.

// Audio processing constants - Used in main.rs CLI defaults and engine.rs
pub const DEFAULT_SAMPLE_RATE: u32 = 44100;
pub const DEFAULT_BLOCK_SIZE: u32 = 512;
pub const DEFAULT_BUFFER_SIZE: usize = 1024;
pub const AUDIO_BLOCK_SIZE_FALLBACK: usize = 256;

// Memory allocation constants - Used in engine.rs and main.rs
pub const DEFAULT_MEMORY_SIZE: usize = 64 * 1024 * 1024; // 64MB
pub const DEFAULT_MAX_AUDIO_BUFFERS: usize = 2048;

// Voice and track limits - Used in main.rs and engine.rs
pub const DEFAULT_MAX_VOICES: usize = 128;
pub const MAX_TRACKS: usize = 10;

// Sample library constants - Used in main.rs and engine.rs
pub const DEFAULT_SAMPLE_DIR: &str = "./samples";
pub const DEFAULT_SAMPLE_COUNT: usize = 1024;

// Network and OSC constants - Used in main.rs and server.rs
pub const DEFAULT_OSC_PORT: u16 = 12345;
pub const OSC_STRING_BUFFER_SIZE: usize = 1024;
pub const PARAMETER_HASHMAP_CAPACITY: usize = 16;
pub const MICROSECONDS_PER_SECOND: f64 = 1_000_000.0;

// Thread priority constants - Used in main.rs
pub const DEFAULT_AUDIO_PRIORITY: u8 = 80;

// Engine parameter indices - Used in voice.rs and engine.rs
pub const ENGINE_PARAM_AMP: usize = 0;
pub const ENGINE_PARAM_PAN: usize = 1;
pub const ENGINE_PARAM_ATTACK: usize = 2;
pub const ENGINE_PARAM_DECAY: usize = 3;
pub const ENGINE_PARAM_SUSTAIN: usize = 4;
pub const ENGINE_PARAM_RELEASE: usize = 5;
pub const ENGINE_PARAM_DUR: usize = 6;
pub const ENGINE_PARAM_ATTACK_CURVE: usize = 7;
pub const ENGINE_PARAM_DECAY_CURVE: usize = 8;
pub const ENGINE_PARAM_RELEASE_CURVE: usize = 9;
pub const ENGINE_PARAM_COUNT: usize = 11;
pub const ENGINE_TX_CHANNEL_BOUND: usize = 1024;

// Default parameter values - Used in registry.rs
pub const DEFAULT_AMP: f32 = 1.0;
pub const DEFAULT_PAN: f32 = 0.0;
pub const DEFAULT_ATTACK: f32 = 0.0125;
pub const DEFAULT_DECAY: f32 = 0.1;
pub const DEFAULT_SUSTAIN: f32 = 0.7;
pub const DEFAULT_RELEASE: f32 = 0.3;
pub const DEFAULT_DURATION: f32 = 1.0;
pub const DEFAULT_ATTACK_CURVE: f32 = 0.3;
pub const DEFAULT_DECAY_CURVE: f32 = 0.3;
pub const DEFAULT_RELEASE_CURVE: f32 = 0.3;
pub const DEFAULT_TRACK: f32 = 1.0;

// Parameter ranges - Used in registry.rs
pub const AMP_MIN: f32 = 0.0;
pub const AMP_MAX: f32 = 2.0;
pub const PAN_MIN: f32 = -1.0;
pub const PAN_MAX: f32 = 1.0;
pub const ATTACK_MIN: f32 = 0.01;
pub const ATTACK_MAX: f32 = 10.0;
pub const DECAY_MIN: f32 = 0.001;
pub const DECAY_MAX: f32 = 10.0;
pub const SUSTAIN_MIN: f32 = 0.0;
pub const SUSTAIN_MAX: f32 = 1.0;
pub const RELEASE_MIN: f32 = 0.001;
pub const RELEASE_MAX: f32 = 10.0;
pub const DURATION_MIN: f32 = 0.001;
pub const DURATION_MAX: f32 = 60.0;
pub const CURVE_MIN: f32 = 0.0;
pub const CURVE_MAX: f32 = 1.0;
pub const TRACK_MIN: f32 = 1.0;
pub const TRACK_MAX: f32 = 10.0;
