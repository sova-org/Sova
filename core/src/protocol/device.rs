use crate::clock::{Clock, SyncTime};
use crate::protocol::audio_engine_proxy::{AudioEnginePayload, AudioEngineProxy};
use crate::protocol::error::ProtocolError;
use crate::protocol::log;
use crate::protocol::midi::{MIDIMessage, MidiIn};
use crate::protocol::osc::{OSCMessage, OSCOut};
use crate::protocol::{midi::MidiOut, payload::ProtocolPayload};
use crate::vm::event::ConcreteEvent;
use crate::{LogMessage, log_eprintln};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};

// SystemTime and UNIX_EPOCH no longer needed - using target_time directly
// Placeholder for richer device info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub slot_id: Option<usize>,
    pub name: String,
    pub kind: DeviceKind,
    pub direction: DeviceDirection,
    pub is_connected: bool,
    pub address: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum DeviceKind {
    Midi,
    VirtualMidi,
    Osc,
    Log,
    AudioEngine,
    Missing,
    #[default]
    Other,
}

impl Display for DeviceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceKind::Midi => write!(f, "Midi"),
            DeviceKind::VirtualMidi => write!(f, "VirtualMidi"),
            DeviceKind::Osc => write!(f, "Osc"),
            DeviceKind::Log => write!(f, "Log"),
            DeviceKind::AudioEngine => write!(f, "AudioEngine"),
            DeviceKind::Missing => write!(f, "Missing"),
            DeviceKind::Other => write!(f, "Other"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum DeviceDirection {
    #[default]
    Output,
    Input,
}

impl Display for DeviceDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceDirection::Output => write!(f, "Output"),
            DeviceDirection::Input => write!(f, "Input"),
        }
    }
}

/// Represents the different types of devices the system can interact with.
///
/// Each variant encapsulates the specific logic for communicating with a type
/// of device, whether it's an input (MIDI, OSC) or an output (Log, MIDI, OSC).
/// Input devices are typically used for discovery and mapping,
/// while output devices handle sending messages.
pub enum ProtocolDevice {
    /// Internal logging device, typically writing to standard output.
    Log,
    /// A physical or virtual MIDI input device, wrapping a `MidiIn` handler.
    /// Access is shared and thread-safe via `Arc<Mutex<>>`.
    MIDIInDevice(MidiIn),
    /// A physical MIDI output device, wrapping a `MidiOut` handler.
    /// Access is shared and thread-safe via `Arc<Mutex<>>`.
    MIDIOutDevice(MidiOut),
    /// A physical or virtual MIDI input device, wrapping a `MidiIn` handler.
    /// Access is shared and thread-safe via `Arc<Mutex<>>`.
    VirtualMIDIInDevice(MidiIn),
    /// A physical MIDI output device, wrapping a `MidiOut` handler.
    /// Access is shared and thread-safe via `Arc<Mutex<>>`.
    VirtualMIDIOutDevice(MidiOut),
    /// Represents an OSC input source. (Future functionality)
    OSCInDevice,
    /// An OSC output device targeting a specific network address.
    OSCOutDevice(OSCOut),
    /// Internal audio engine (Sova) - no external connectivity required
    AudioEngine(AudioEngineProxy),
}

impl ProtocolDevice {
    /// Attempts to establish or verify the necessary connection for the device.
    ///
    /// Behavior depends on the device type:
    /// - `OSCOutDevice`: Attempts to bind a local UDP socket if one doesn't already exist.
    /// - `VirtualMIDIOutDevice`: Checks if the internal `midir` connection is active.
    /// - MIDI devices (`MIDIInDevice`, `MIDIOutDevice`): Connection is typically
    ///   managed externally (e.g., by `DeviceMap`). This method might do nothing
    ///   or display an informational message.
    /// - `Log`, `OSCInDevice`: No connection action is currently required.
    ///
    /// # Errors
    ///
    /// Returns `Err(ProtocolError)` if the connection cannot be established
    /// (e.g., UDP socket bind failure, virtual MIDI connection not found)
    /// or if the Mutex protecting the internal state is poisoned.
    pub fn connect(&mut self) -> Result<(), ProtocolError> {
        match self {
            ProtocolDevice::OSCInDevice => {
                // Placeholder: Implement OSC input connection logic if needed
                crate::log_eprintln!(
                    "ProtocolDevice::connect() called for OSCInDevice (Not Implemented)"
                );
                Ok(())
            }
            ProtocolDevice::MIDIInDevice(midi_in)
            | ProtocolDevice::VirtualMIDIInDevice(midi_in) => midi_in.connect(),
            ProtocolDevice::MIDIOutDevice(midi_out)
            | ProtocolDevice::VirtualMIDIOutDevice(midi_out) => midi_out.connect(),
            ProtocolDevice::OSCOutDevice(osc_out) => osc_out.connect(),
            ProtocolDevice::Log => Ok(()), // Log device doesn't need connection
            ProtocolDevice::AudioEngine { .. } => Ok(()), // AudioEngine doesn't need external connection
        }
    }

    /// Sends a message (`ProtocolPayload`) via this device.
    ///
    /// Handles protocol-specific sending logic:
    /// - `MIDIOutDevice`/`VirtualMIDIOutDevice`: Sends MIDI bytes via `midir`.
    /// - `OSCOutDevice`: Encodes the `OSCMessage` into an OSC `OscBundle`
    ///   with a timestamp (`now + latency`) via `rosc` and sends it over the UDP socket.
    /// - `Log`: Prints the `LogMessage` content to standard output.
    /// - Input devices (`MIDIInDevice`, `OSCInDevice`): Returns an error as sending
    ///   to an input is not possible.
    ///
    /// # Arguments
    /// * `message` - The `ProtocolPayload` to send. The inner type must match
    ///   the `ProtocolDevice` type (e.g., `ProtocolPayload::MIDI` for `MIDIOutDevice`).
    /// * `target_time` - The intended execution time (`SyncTime`). Used for precise
    ///   OSC bundle timestamping to enable sample-accurate timing.
    ///
    /// # Errors
    ///
    /// Returns `Err(ProtocolError)` if:
    /// - The `message` format is incompatible with the device type.
    /// - A network error occurs (e.g., UDP send failure).
    /// - The device is not connected (socket not bound, MIDI connection absent).
    /// - An OSC encoding error occurs.
    /// - The Mutex protecting the internal state is poisoned.
    /// - The system time cannot be read.
    pub fn send(&self, message: ProtocolPayload) -> Result<(), ProtocolError> {
        // target_time used for precise OSC timestamping and protocol timing
        match self {
            ProtocolDevice::MIDIOutDevice(midi_out)
            | ProtocolDevice::VirtualMIDIOutDevice(midi_out) => {
                let ProtocolPayload::MIDI(midi_msg) = message else {
                    return Err(ProtocolError(
                        "Invalid message format for MIDI device!".to_owned(),
                    ));
                };
                midi_out.send(midi_msg)
            }
            ProtocolDevice::OSCOutDevice(osc_out) => {
                let ProtocolPayload::OSC(crate_osc_msg) = message else {
                    return Err(ProtocolError(format!(
                        "Invalid message format for OSC device '{}'!",
                        osc_out.name
                    )));
                };
                osc_out.send(crate_osc_msg)
            }
            ProtocolDevice::Log => {
                let ProtocolPayload::LOG(log_msg) = message else {
                    return Err(ProtocolError(
                        "Invalid message format for Log device!".to_owned(),
                    ));
                };
                // Simple stdout logging implementation
                crate::log_println!("[{}] {}", log_msg.level, log_msg.msg);
                if let Some(event) = log_msg.event {
                    // Use debug formatting for the associated event if present
                    crate::log_println!("    Associated Event: {:?}", event);
                }
                Ok(())
            }
            ProtocolDevice::AudioEngine(proxy) => {
                let ProtocolPayload::AudioEngine(msg) = message else {
                    return Err(ProtocolError(
                        "Invalid message format for AudioEngine device!".to_owned(),
                    ));
                };
                proxy.send(msg)
            }
            ProtocolDevice::MIDIInDevice(_)
            | ProtocolDevice::VirtualMIDIInDevice(_)
            | ProtocolDevice::OSCInDevice => {
                // Cannot send to input devices
                Err(ProtocolError(format!(
                    "Cannot send message to input device: {}",
                    self.address()
                )))
            }
        }
    }

    /// Flushes the outgoing message buffer for the device, if applicable.
    ///
    /// Behavior depends on the device type:
    /// - `MIDIOutDevice`: Calls the `flush` method of the underlying `MidiOut` handler.
    /// - Others (`VirtualMIDIOutDevice`, `OSCOutDevice`, `Log`, `MIDIInDevice`, `OSCInDevice`):
    ///   This operation is typically a no-op, as sending is immediate (UDP, virtual midir)
    ///   or not applicable (Log, inputs).
    pub fn flush(&self) {
        match self {
            ProtocolDevice::MIDIOutDevice(midi_out)
            | ProtocolDevice::VirtualMIDIOutDevice(midi_out) => {
                midi_out.flush();
            }
            ProtocolDevice::OSCOutDevice(osc_out) => {
                // UDP sends are typically fire-and-forget, no explicit flush needed at socket level.
                crate::log_println!(
                    "Flush called on OSCOutDevice '{}' @ {} (no-op for UDP)",
                    osc_out.name,
                    osc_out.address
                );
            }
            ProtocolDevice::Log
            | ProtocolDevice::MIDIInDevice(_)
            | ProtocolDevice::VirtualMIDIInDevice(_)
            | ProtocolDevice::OSCInDevice
            | ProtocolDevice::AudioEngine { .. } => {
                // No flushing mechanism for Log, AudioEngine, Control, or input devices
            }
        }
    }

    /// Returns a unique textual identifier or address for the device.
    ///
    /// This identifier is used for comparisons (`PartialEq`), display (`Display`, `Debug`),
    /// and potentially as a key in data structures.
    /// - `Log`: Returns the string "log".
    /// - MIDI devices (Input/Output/Virtual): Returns the device name as reported
    ///   by the system or given during creation (for virtual devices).
    /// - `OSCOutDevice`: Returns the name assigned during creation.
    /// - `OSCInDevice`: Returns a placeholder string ("OSC_IN_ADDRESS_TBD").
    pub fn address(&self) -> String {
        match self {
            ProtocolDevice::Log => log::LOG_NAME.to_string(), // Use constant if available
            ProtocolDevice::OSCInDevice => "OSC_IN_ADDRESS_TBD".to_string(), // Placeholder
            ProtocolDevice::MIDIInDevice(midi_in)
            | ProtocolDevice::VirtualMIDIInDevice(midi_in) => midi_in.name.clone(),
            ProtocolDevice::MIDIOutDevice(midi_out)
            | ProtocolDevice::VirtualMIDIOutDevice(midi_out) => midi_out.name.clone(),
            ProtocolDevice::OSCOutDevice(osc_out) => osc_out.address.to_string(),
            ProtocolDevice::AudioEngine { .. } => "Internal".to_string(),
        }
    }

    pub fn kind(&self) -> DeviceKind {
        match self {
            ProtocolDevice::Log => DeviceKind::Log,
            ProtocolDevice::MIDIInDevice(_) | ProtocolDevice::MIDIOutDevice(_) => DeviceKind::Midi,
            ProtocolDevice::VirtualMIDIInDevice(_) | ProtocolDevice::VirtualMIDIOutDevice(_) => {
                DeviceKind::VirtualMidi
            }
            ProtocolDevice::OSCOutDevice(_) | ProtocolDevice::OSCInDevice => DeviceKind::Osc,
            ProtocolDevice::AudioEngine { .. } => DeviceKind::AudioEngine,
        }
    }

    pub fn translate_event(
        &self,
        event: ConcreteEvent,
        date: SyncTime,
        clock: &Clock,
    ) -> Vec<(ProtocolPayload, SyncTime)> {
        match self {
            ProtocolDevice::OSCOutDevice(out) => {
                OSCMessage::generate_messages(out, event, date, clock)
            }
            ProtocolDevice::MIDIOutDevice(midi_out)
            | ProtocolDevice::VirtualMIDIOutDevice(midi_out) => {
                MIDIMessage::generate_messages(event, date, midi_out.epsilon)
            }
            ProtocolDevice::Log => {
                // Should be unreachable due to the initial check, but kept defensively.
                LogMessage::generate_messages(event, date)
            }
            ProtocolDevice::AudioEngine { .. } => {
                AudioEnginePayload::generate_messages(event, date)
            }
            _ => {
                log_eprintln!(
                    "map_event_for_device_name: Unhandled ProtocolDevice type for {}",
                    self.address()
                );
                vec![] // Or generate an error log message
            }
        }
    }
}

impl From<MidiOut> for ProtocolDevice {
    /// Creates a `ProtocolDevice::MIDIOutDevice` from a `MidiOut` handler.
    fn from(value: MidiOut) -> Self {
        Self::MIDIOutDevice(value)
    }
}

impl From<MidiIn> for ProtocolDevice {
    /// Creates a `ProtocolDevice::MIDIInDevice` from a `MidiIn` handler.
    fn from(value: MidiIn) -> Self {
        Self::MIDIInDevice(value)
    }
}

impl From<OSCOut> for ProtocolDevice {
    fn from(value: OSCOut) -> Self {
        Self::OSCOutDevice(value)
    }
}

// Custom Debug implementation to avoid printing the full internal state
// of handlers (MidiIn/Out, UdpSocket, MidiOutputConnection) which can be large.
impl Debug for ProtocolDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolDevice::Log => write!(f, "Log"),
            ProtocolDevice::OSCInDevice => write!(f, "OSCInDevice"),
            ProtocolDevice::MIDIInDevice(midi_in)
            | ProtocolDevice::VirtualMIDIInDevice(midi_in) => Debug::fmt(midi_in, f),
            ProtocolDevice::MIDIOutDevice(midi_out)
            | ProtocolDevice::VirtualMIDIOutDevice(midi_out) => Debug::fmt(midi_out, f),
            ProtocolDevice::OSCOutDevice(osc_out) => Debug::fmt(osc_out, f),
            ProtocolDevice::AudioEngine { .. } => write!(f, "AudioEngine"),
        }
    }
}

impl Display for ProtocolDevice {
    /// Formats the device for display, typically using its name or type.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolDevice::Log => write!(f, "Log"),
            ProtocolDevice::OSCInDevice => write!(f, "OSCInDevice"),
            ProtocolDevice::MIDIInDevice(midi_in)
            | ProtocolDevice::VirtualMIDIInDevice(midi_in) => Display::fmt(midi_in, f),
            ProtocolDevice::MIDIOutDevice(midi_out)
            | ProtocolDevice::VirtualMIDIOutDevice(midi_out) => Display::fmt(midi_out, f),
            ProtocolDevice::OSCOutDevice(osc_out) => write!(f, "OSCOutDevice({})", osc_out.name),
            ProtocolDevice::AudioEngine { .. } => write!(f, "AudioEngine"),
        }
    }
}

impl PartialEq for ProtocolDevice {
    /// Compares two `ProtocolDevice` instances based on their `address()`.
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for ProtocolDevice {}
