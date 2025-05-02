//! Defines the core structures and enums for handling different communication protocols.
//!
//! This module provides unified ways to represent messages, targets, and errors
//! across various protocols like MIDI, OSC, and internal logging.
//! It relies on submodules for protocol-specific implementations:
//!
//! - `log`: Handles structures and logic for internal logging messages.
//! - `midi`: Contains definitions related to the MIDI protocol
//! - `osc`: Contains definitions for the Open Sound Control (OSC) protocol
//! - `payload`: Defines the `ProtocolPayload` enum which encapsulates protocol-specific
//!   data (MIDI, OSC, Log).
//! - `message`: Defines the `ProtocolMessage` and `TimedMessage` structs representing a
//!   generic message with its target and optional timestamp.
//! - `device`: Defines the `ProtocolDevice` enum to represent device targets
//!   for messages (e.g., a specific MIDI port, an OSC IP address and port).
//! - `error`: Defines the unified `ProtocolError` type for handling errors
//!   related to the different protocols.

pub mod log;
pub mod midi;
pub mod osc;
pub mod payload;
pub mod message;
pub mod device;
pub mod error;
