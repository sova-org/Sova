//! Core type definitions.
//!
//! This module provides the fundamental types used throughout the audio engine,
//! including voice and track identifiers, message scheduling, and engine commands.
//! All types are designed for zero-allocation operation in real-time audio contexts.

use std::cmp::Ordering;

/// Unique identifier for a voice instance in the audio engine.
///
/// Voice IDs are assigned sequentially and allow the engine to track and manage
/// individual sound instances across the polyphonic voice pool. The 32-bit range
/// provides sufficient unique identifiers for long-running audio sessions.
pub type VoiceId = u32;

/// Unique identifier for an audio track.
///
/// Track IDs support up to 256 concurrent audio tracks, which is sufficient for
/// most live coding and performance scenarios. Each track can host multiple voices
/// and applies global effects to all voices routed through it.
pub type TrackId = u8;

/// Time-scheduled message wrapper for deferred execution of engine commands.
///
/// This structure enables precise timing control for audio events by storing
/// messages with their execution timestamps. It implements a reverse-ordered
/// priority queue to ensure messages are processed in chronological order.
///
/// # Performance Characteristics
///
/// - Zero heap allocation during creation and comparison
/// - Optimized for use in `BinaryHeap` with reverse ordering
/// - Deterministic comparison operations for real-time safety
///
/// # Usage
///
/// ```rust
/// let scheduled = ScheduledMessage {
///     due_time_ms: 1000, // Execute in 1 second
///     message: EngineMessage::Play { /* ... */ },
/// };
/// ```
#[derive(Debug)]
pub struct ScheduledMessage {
    /// Absolute timestamp in milliseconds when this message should be executed
    pub due_time_ms: u64,
    /// The engine command to execute at the scheduled time
    pub message: EngineMessage,
}

impl PartialEq for ScheduledMessage {
    fn eq(&self, other: &Self) -> bool {
        self.due_time_ms == other.due_time_ms
    }
}

impl Eq for ScheduledMessage {}

impl PartialOrd for ScheduledMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        other.due_time_ms.cmp(&self.due_time_ms)
    }
}

/// Error types that can occur during audio engine operation.
///
/// These errors provide detailed information about what went wrong during
/// audio processing, voice management, or parameter handling, enabling
/// proper error reporting to the user interface.
#[derive(Debug, Clone)]
pub enum EngineError {
    /// An invalid or unknown source module was requested.
    InvalidSource { 
        source_name: String, 
        voice_id: VoiceId,
        available_sources: Vec<String>,
    },
    /// A sample folder or specific sample was not found.
    SampleNotFound { 
        folder: String, 
        index: usize, 
        voice_id: VoiceId,
        available_folders: Vec<String>,
    },
    /// Sample loading failed due to file format or other issues.
    SampleLoadFailed { 
        path: String, 
        reason: String, 
        voice_id: VoiceId 
    },
    /// Invalid parameter name or value provided.
    ParameterError { 
        param: String, 
        value: String, 
        reason: String, 
        voice_id: VoiceId,
        valid_params: Vec<String>,
    },
    /// Audio device or stream error.
    AudioDeviceError { 
        reason: String 
    },
    /// Memory allocation failed.
    MemoryAllocationFailed { 
        voice_id: VoiceId,
        requested_size: usize,
    },
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::InvalidSource { source_name, voice_id, available_sources } => {
                write!(f, "Unknown source '{}' for voice {}. Available sources: [{}]", 
                    source_name, voice_id, available_sources.join(", "))
            },
            EngineError::SampleNotFound { folder, index, voice_id, available_folders } => {
                write!(f, "Sample folder '{}' (index {}) not found for voice {}. Available folders: [{}]", 
                    folder, index, voice_id, available_folders.join(", "))
            },
            EngineError::SampleLoadFailed { path, reason, voice_id } => {
                write!(f, "Failed to load sample '{}' for voice {}: {}", path, voice_id, reason)
            },
            EngineError::ParameterError { param, value, reason, voice_id, valid_params } => {
                write!(f, "Invalid parameter '{}' = '{}' for voice {}: {}. Valid parameters: [{}]", 
                    param, value, voice_id, reason, valid_params.join(", "))
            },
            EngineError::AudioDeviceError { reason } => {
                write!(f, "Audio device error: {}", reason)
            },
            EngineError::MemoryAllocationFailed { voice_id, requested_size } => {
                write!(f, "Memory allocation failed for voice {} ({} bytes)", voice_id, requested_size)
            },
        }
    }
}

/// Status messages from the audio engine for monitoring and debugging.
///
/// These messages provide feedback about engine state, performance,
/// and operational status to help users understand what's happening.
#[derive(Debug, Clone)]
pub enum EngineStatusMessage {
    /// Critical or non-critical errors.
    Error(EngineError),
    /// Warning messages about potential issues.
    Warning(String),
    /// Informational messages about engine state.
    Info(String),
    /// Debug information for development.
    Debug(String),
}

/// Commands that control the audio engine's behavior and voice management.
///
/// This enum defines all possible operations that can be performed on the audio engine,
/// from triggering new sounds to updating parameters and controlling playback state.
/// All commands are designed to be efficiently processed in the real-time audio thread.
///
/// # Message Types
///
/// - **Play**: Start a new voice with specified source and parameters
/// - **Update**: Modify parameters of an existing voice during playback
/// - **Stop**: Halt all audio processing immediately
/// - **Panic**: Emergency stop with immediate voice cleanup
///
/// # Parameter System
///
/// The parameter system uses type-erased values (`Box<dyn Any + Send>`) to support
/// arbitrary parameter types while maintaining thread safety. Common parameter types
/// include `f32` for continuous values, `String` for sample names, and custom structs
/// for complex configurations.
///
/// # Performance Notes
///
/// While parameters use boxed values, this allocation occurs in the message thread,
/// not the audio thread. The audio thread processes pre-allocated parameter updates
/// from shared memory buffers.
#[derive(Debug)]
pub enum EngineMessage {
    /// Start a new voice with the specified source and parameters.
    ///
    /// # Fields
    ///
    /// - `voice_id`: Unique identifier for this voice instance
    /// - `track_id`: Target track for audio routing and effects
    /// - `source_name`: Name of the audio source module (e.g., "sine", "sample")
    /// - `parameters`: Type-erased parameter map for source configuration
    ///
    /// # Example Parameters
    ///
    /// ```rust
    /// let mut params = HashMap::new();
    /// params.insert("frequency".to_string(), Box::new(440.0f32) as Box<dyn Any + Send>);
    /// params.insert("amplitude".to_string(), Box::new(0.8f32) as Box<dyn Any + Send>);
    /// ```
    Play {
        voice_id: VoiceId,
        track_id: TrackId,
        source_name: String,
        parameters: std::collections::HashMap<String, Box<dyn std::any::Any + Send>>,
    },

    /// Update parameters of an active voice during playback.
    ///
    /// This allows real-time parameter modulation for live performance and
    /// automation. Parameter updates are applied atomically to avoid audio artifacts.
    ///
    /// # Fields
    ///
    /// - `voice_id`: Target voice to update
    /// - `track_id`: Track containing the voice
    /// - `parameters`: New parameter values to apply
    Update {
        voice_id: VoiceId,
        track_id: TrackId,
        parameters: std::collections::HashMap<String, Box<dyn std::any::Any + Send>>,
    },

    /// Stop all audio processing immediately.
    ///
    /// This command halts the audio engine gracefully, allowing voices to complete
    /// their current audio block before stopping. Use for controlled shutdown.
    Stop,

    /// Emergency stop with immediate voice cleanup.
    ///
    /// This command immediately silences all voices and clears the processing state.
    /// Use when immediate audio halt is required, such as during live performance
    /// emergencies or when audio artifacts occur.
    Panic,
}
