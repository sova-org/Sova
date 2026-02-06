#[cfg(feature = "audio")]
pub use doux_sova::{AudioEngineState, DouxConfig, DouxManager};

#[cfg(not(feature = "audio"))]
mod stub {
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AudioEngineState {
        pub running: bool,
        pub device: Option<String>,
        pub sample_rate: f32,
        pub channels: usize,
        pub buffer_size: Option<u32>,
        pub active_voices: usize,
        pub sample_paths: Vec<PathBuf>,
        pub error: Option<String>,
        pub cpu_load: f32,
        pub peak_voices: usize,
        pub max_voices: usize,
        pub schedule_depth: usize,
        pub sample_pool_mb: f32,
    }

    impl Default for AudioEngineState {
        fn default() -> Self {
            Self {
                running: false,
                device: None,
                sample_rate: 0.0,
                channels: 0,
                buffer_size: None,
                active_voices: 0,
                sample_paths: Vec::new(),
                error: Some("Audio disabled at compile time".to_string()),
                cpu_load: 0.0,
                peak_voices: 0,
                max_voices: 0,
                schedule_depth: 0,
                sample_pool_mb: 0.0,
            }
        }
    }
}

#[cfg(not(feature = "audio"))]
pub use stub::AudioEngineState;
