use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::protocol::{
    payload::ProtocolPayload,
    device::ProtocolDevice,
};
use crate::clock::SyncTime;
use crate::protocol::error::ProtocolError;
use std::cmp::Ordering;
use std::fmt::Display;

/// Associates a protocol-specific payload with its target device.
///
/// Holds the message content (`payload`) and a reference-counted handle
/// to the destination `ProtocolDevice`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// The target device for this message.
    pub device: Arc<ProtocolDevice>,
    /// The actual message content (MIDI, OSC, Log).
    pub payload: ProtocolPayload,
}

impl ProtocolMessage {
    /// Sends the message to its target device immediately.
    ///
    /// Note: For time-sensitive protocols like OSC, the `time` parameter might be used
    /// internally by the device's `send` method to schedule the message appropriately
    /// (e.g., using OSC bundles with timestamps).
    ///
    /// # Arguments
    /// * `time` - The intended send time (`SyncTime`). Primarily relevant for scheduling OSC bundles.
    ///
    /// # Returns
    /// - `Ok(())` on successful sending (or queuing).
    /// - `Err(ProtocolError)` if sending fails (e.g., connection error, invalid format).
    pub fn send(self, time: SyncTime) -> Result<(), ProtocolError> {
        self.device.send(self.payload, time)
    }

    /// Wraps the `ProtocolMessage` in a `TimedMessage` with the specified timestamp.
    pub fn timed(self, time: SyncTime) -> TimedMessage {
        TimedMessage {
            message: self,
            time,
        }
    }
}

impl Display for ProtocolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] -> Device: {}", self.payload, self.device)
    }
}


/// Associates a `ProtocolMessage` with a specific time (`SyncTime`).
///
/// Used for scheduling messages in time-ordered queues (like a priority queue).
/// Implements `Ord` based *inversely* on time, so earlier times have higher priority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimedMessage {
    /// The underlying message (payload and device).
    pub message: ProtocolMessage,
    /// The timestamp associated with the message.
    pub time: SyncTime,
}

impl Display for TimedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Delegate formatting to ProtocolMessage and add the time
        write!(f, "{} @ Time: {}", self.message, self.time)
    }
}

impl TimedMessage {
    /// Consumes the `TimedMessage` and returns its components.
    pub fn untimed(self) -> (ProtocolMessage, SyncTime) {
        (self.message, self.time)
    }
}

impl Eq for TimedMessage {}

/// Ordering for `TimedMessage` is based on the `time` field, but reversed.
///
/// This means messages with *earlier* timestamps are considered "greater",
/// making them higher priority in a standard `BinaryHeap` (which acts as a min-heap
/// based on this reversed ordering, effectively becoming a max-heap on priority/earliness).
impl Ord for TimedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse the comparison: earlier time means greater ordering priority
        other.time.cmp(&self.time)
    }
}

/// Partial ordering follows the total ordering defined by `Ord`.
impl PartialOrd for TimedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}