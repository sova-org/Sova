use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// Version of the BuboCore protocol
pub const SOVA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Messages sent between relay server and BuboCore instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayMessage {
    /// Instance registration request
    RegisterInstance {
        instance_name: String,
        version: String,
        session_token: Option<String>,
    },
    
    /// Registration response
    RegistrationResponse {
        success: bool,
        message: String,
        assigned_id: Option<Uuid>,
        current_instances: Vec<InstanceInfo>,
    },

    /// State update from an instance to be relayed
    StateUpdate {
        source_instance_id: Uuid,
        timestamp: u64,
        update_data: Vec<u8>, // Serialized ClientMessage from core
    },

    /// Broadcast of state update to other instances
    StateBroadcast {
        source_instance_name: String,
        timestamp: u64,
        update_data: Vec<u8>,
    },

    /// Instance disconnection notification
    InstanceDisconnected {
        instance_id: Uuid,
        instance_name: String,
    },

    /// Ping for connection health check
    Ping { timestamp: u64 },
    
    /// Pong response
    Pong { timestamp: u64 },

    /// Error message
    Error { message: String },
}

/// Information about a connected instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub connected_at: SystemTime,
    pub last_activity: SystemTime,
}

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub messages_per_minute: u32,
    pub max_message_size: usize,
    pub cleanup_interval_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            messages_per_minute: 1000,
            max_message_size: 1024 * 1024, // 1MB
            cleanup_interval_secs: 60,
        }
    }
}

/// Error types for the relay server
#[derive(thiserror::Error, Debug)]
pub enum RelayError {
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
    
    #[error("Instance name '{name}' already in use")]
    InstanceNameTaken { name: String },
    
    #[error("Maximum instances ({max}) reached")]
    MaxInstancesReached { max: usize },
    
    #[error("Rate limit exceeded for instance {instance_name}")]
    RateLimitExceeded { instance_name: String },
    
    #[error("Message too large: {size} bytes (max: {max})")]
    MessageTooLarge { size: usize, max: usize },
    
    #[error("Instance {id} not found")]
    InstanceNotFound { id: Uuid },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),
    
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] rmp_serde::decode::Error),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl RelayMessage {
    /// Serialize message to bytes using MessagePack
    pub fn to_bytes(&self) -> Result<Vec<u8>, RelayError> {
        Ok(rmp_serde::to_vec_named(self)?)
    }
    
    /// Deserialize message from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, RelayError> {
        Ok(rmp_serde::from_slice(data)?)
    }
    
    /// Get the size of the serialized message
    pub fn serialized_size(&self) -> Result<usize, RelayError> {
        Ok(self.to_bytes()?.len())
    }
}

/// Rate limiter for instance connections
#[derive(Debug)]
pub struct RateLimit {
    pub instance_id: Uuid,
    pub message_count: u32,
    pub window_start: SystemTime,
    pub total_bytes: usize,
}

impl RateLimit {
    pub fn new(instance_id: Uuid) -> Self {
        Self {
            instance_id,
            message_count: 0,
            window_start: SystemTime::now(),
            total_bytes: 0,
        }
    }
    
    pub fn check_and_update(&mut self, config: &RateLimitConfig, message_size: usize) -> bool {
        let now = SystemTime::now();
        
        // Reset window if a minute has passed
        if now.duration_since(self.window_start).unwrap_or_default().as_secs() >= 60 {
            self.message_count = 0;
            self.total_bytes = 0;
            self.window_start = now;
        }
        
        // Check limits
        if self.message_count >= config.messages_per_minute {
            return false;
        }
        
        if message_size > config.max_message_size {
            return false;
        }
        
        // Update counters
        self.message_count += 1;
        self.total_bytes += message_size;
        
        true
    }
}