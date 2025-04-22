//! Manages connections to external and virtual devices (MIDI, OSC, Log)
//! and maps internal events to protocol-specific messages for output.
//!
//! This module provides the `DeviceMap` struct, which serves as the central
//! registry for devices known to the BuboCore system. It handles:
//! - Discovering available system MIDI ports.
//! - Connecting to and disconnecting from MIDI devices (physical and virtual).
//! - Creating and removing virtual MIDI ports.
//! - Creating and removing OSC output endpoints.
//! - Assigning unique, user-friendly names to connected devices.
//! - Mapping devices to numbered slots (1 to `MAX_DEVICE_SLOTS`) for easy referencing.
//!   Slot 0 is reserved for the internal Log device.
//! - Translating `ConcreteEvent`s into `ProtocolMessage`s 
//!   (like `MIDIMessage`, `OSCMessage`, `LogMessage`)
//!   based on the target device (specified by name or slot ID).
//! - Providing a list of available and connected devices (`DeviceInfo`).

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    net::{SocketAddr, IpAddr},
    str::FromStr,
};

use crate::{
    clock::{SyncTime, Clock},
    lang::event::ConcreteEvent,
    protocol::{
        log::{LogMessage, Severity, LOG_NAME},
        midi::{MIDIMessage, MIDIMessageType, MidiIn, MidiOut, MidiInterface},
        osc::{OSCMessage, Argument as OscArgument},
        ProtocolDevice, ProtocolMessage, TimedMessage,
    },
    shared_types::{DeviceInfo, DeviceKind},
    lang::variable::VariableValue,
};

use midir::{MidiInput, MidiOutput, Ignore};

/// A tuple representing a registered device, containing its user-assigned name
/// and a reference-counted, thread-safe `ProtocolDevice` instance.
pub type DeviceItem = (String, Arc<ProtocolDevice>);

/// Maximum number of user-assignable device slots (1-based).
const MAX_DEVICE_SLOTS: usize = 16;

/// Manages device connections, slot assignments, and event-to-protocol mapping.
///
/// Provides thread-safe access to device information and handles the underlying
/// MIDI and OSC communication setup via `midir` and `rosc`.
pub struct DeviceMap {
    /// Currently connected input devices, keyed by their unique address/identifier string.
    /// Values are `DeviceItem`s containing the assigned name and the device handle.
    pub input_connections: Mutex<HashMap<String, DeviceItem>>,
    /// Currently connected output devices, keyed by their unique address/identifier string.
    /// Values are `DeviceItem`s containing the assigned name and the device handle.
    pub output_connections: Mutex<HashMap<String, DeviceItem>>,
    /// Maps user-assigned Slot IDs (1-N) to the system or virtual device name assigned to it.
    /// Slot 0 is implicitly the Log device and is not stored here.
    pub slot_assignments: Mutex<HashMap<usize, String>>,
    /// Optional handle to the system's MIDI input interface, managed by `midir`.
    midi_in: Option<Arc<Mutex<MidiInput>>>,
    /// Optional handle to the system's MIDI output interface, managed by `midir`.
    midi_out: Option<Arc<Mutex<MidiOutput>>>,
}

impl DeviceMap {
    /// Creates a new `DeviceMap` and attempts to initialize system MIDI interfaces.
    /// The internal Log device is handled implicitly and not registered here.
    pub fn new() -> Self {
        let midi_in = match MidiInput::new("BuboCore Input") {
            Ok(mut input) => {
                input.ignore(Ignore::None);
                println!("[+] MIDI Input initialized successfully.");
                Some(Arc::new(Mutex::new(input)))
            }
            Err(e) => {
                eprintln!("[!] Failed to initialize MIDI Input: {}", e);
                None
            }
        };

        let midi_out = match MidiOutput::new("BuboCore Output") {
            Ok(output) => {
                println!("[+] MIDI Output initialized successfully.");
                Some(Arc::new(Mutex::new(output)))
            }
            Err(e) => {
                eprintln!("[!] Failed to initialize MIDI Output: {}", e);
                None
            }
        };

        DeviceMap {
            input_connections: Default::default(),
            output_connections: Default::default(),
            slot_assignments: Default::default(), 
            midi_in,
            midi_out,
        }
    }

    /// Registers a connected input device.
    ///
    /// Associates the given `name` with the `device` and stores it in the
    /// `input_connections` map, keyed by the device's address.
    pub fn register_input_connection(&self, name: String, device: ProtocolDevice) {
        let address = device.address().to_owned();
        let item = (name, Arc::new(device));
        self.input_connections.lock().unwrap().insert(address, item);
    }

    /// Registers a connected output device.
    ///
    /// Associates the given `name` with the `device` and stores it in the
    /// `output_connections` map, keyed by the device's address.
    /// Note: This only registers the connection; slot assignment is separate.
    pub fn register_output_connection(&self, name: String, device: ProtocolDevice) {
        let address = device.address().to_owned();
        let item = (name, Arc::new(device));
        self.output_connections
            .lock()
            .unwrap()
            .insert(address, item);
    }

    /// Assigns a device (identified by its `device_name`) to a specific slot ID (1-N).
    ///
    /// # Arguments
    /// * `slot_id` - The 1-based slot number to assign the device to. Must be between 1 and `MAX_DEVICE_SLOTS`.
    /// * `device_name` - The name of the device to assign (must match a name from discovery or creation).
    ///
    /// # Behavior
    /// - If the `slot_id` is already assigned, the previous assignment is removed.
    /// - If the `device_name` is already assigned to a different slot, that assignment is removed.
    /// - A device can only be assigned to one slot at a time.
    ///
    /// # Returns
    /// - `Ok(())` on successful assignment.
    /// - `Err(String)` if the `slot_id` is invalid.
    pub fn assign_slot(&self, slot_id: usize, device_name: &str) -> Result<(), String> {
        if !(1..=MAX_DEVICE_SLOTS).contains(&slot_id) {
            return Err(format!("Invalid slot ID: {}. Must be between 1 and {}.", slot_id, MAX_DEVICE_SLOTS));
        }

        let mut assignments = self.slot_assignments.lock().unwrap();

        // Remove any existing assignment for the target slot_id
        assignments.remove(&slot_id);
        
        // Remove any existing assignment for the target device_name (a device can only be in one slot)
        assignments.retain(|_s_id, assigned_name| assigned_name != device_name);

        // Create the new assignment
        assignments.insert(slot_id, device_name.to_string());
        println!("[+] Assigned device '{}' to Slot {}", device_name, slot_id);
        Ok(())
    }

    /// Unassigns whatever device is currently in the specified slot ID (1-N).
    ///
    /// If the slot was already empty, it prints a message indicating so.
    ///
    /// # Arguments
    /// * `slot_id` - The 1-based slot number to clear. Must be between 1 and `MAX_DEVICE_SLOTS`.
    ///
    /// # Returns
    /// - `Ok(())` if the slot was cleared or was already empty.
    /// - `Err(String)` if the `slot_id` is invalid.
    pub fn unassign_slot(&self, slot_id: usize) -> Result<(), String> {
         if !(1..=MAX_DEVICE_SLOTS).contains(&slot_id) {
             return Err(format!("Invalid slot ID: {}. Must be between 1 and {}.", slot_id, MAX_DEVICE_SLOTS));
         }
         let mut assignments = self.slot_assignments.lock().unwrap();
         if let Some(removed_name) = assignments.remove(&slot_id) {
             println!("[-] Unassigned device '{}' from Slot {}", removed_name, slot_id);
         } else {
             println!("[~] Slot {} was already empty.", slot_id);
         }
         Ok(())
    }
    
    /// Unassigns a specific device name from whichever slot it might be in.
    ///
    /// If the device was assigned to a slot, it removes the assignment and prints a message.
    /// If the device was not assigned to any slot, it does nothing.
    pub fn unassign_device_by_name(&self, device_name: &str) {
        let mut assignments = self.slot_assignments.lock().unwrap();
        let mut found_slot = None;
        assignments.retain(|slot, name| {
            if name == device_name {
                found_slot = Some(*slot);
                false // Remove the entry
            } else {
                true // Keep other entries
            }
        });
        if let Some(slot) = found_slot {
             println!("[-] Unassigned device '{}' from Slot {}", device_name, slot);
        }
    }

    /// Finds the device name assigned to a specific slot ID (1-N).
    ///
    /// Returns `None` if the slot is invalid or not currently assigned.
    pub fn get_name_for_slot(&self, slot_id: usize) -> Option<String> {
        self.slot_assignments.lock().unwrap().get(&slot_id).cloned()
    }

    /// Finds the slot ID (1-N) assigned to a specific device name.
    ///
    /// Returns `None` if the device name is not assigned to any slot.
    pub fn get_slot_for_name(&self, device_name: &str) -> Option<usize> {
         self.slot_assignments.lock().unwrap().iter()
            .find_map(|(slot, name)| if name == device_name { Some(*slot) } else { None })
    }

    /// Generates `TimedMessage`s containing `MIDIMessage` payloads from a `ConcreteEvent`.
    ///
    /// Handles mapping various `ConcreteEvent::Midi*` variants to their corresponding
    /// MIDI message types (NoteOn/Off, CC, ProgramChange, etc.).
    /// Note durations are handled by scheduling a corresponding NoteOff message.
    /// MIDI channels are converted from 1-based (in `ConcreteEvent`) to 0-based (in `MIDIMessage`).
    /// System messages (Start, Stop, etc.) are sent on channel 0.
    fn generate_midi_message(
        &self,
        payload: ConcreteEvent,
        date: SyncTime,
        device: Arc<ProtocolDevice>,
    ) -> Vec<TimedMessage> {
        match payload {
            ConcreteEvent::MidiNote(note, vel, chan, dur, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8; // Convert to 0-based MIDI channel
                vec![
                    // NoteOn
                    ProtocolMessage {
                        payload: MIDIMessage {
                            payload: MIDIMessageType::NoteOn {
                                note: note as u8,
                                velocity: vel as u8,
                            },
                            channel: midi_chan,
                        }
                        .into(),
                        device: Arc::clone(&device),
                    }
                    .timed(date),
                    // NoteOff
                    ProtocolMessage {
                        payload: MIDIMessage {
                            payload: MIDIMessageType::NoteOff {
                                note: note as u8,
                                velocity: 0,
                            },
                            channel: midi_chan,
                        }
                        .into(),
                        device: Arc::clone(&device),
                    }
                    .timed(date + dur),
                ]
            }
            ConcreteEvent::MidiControl(control, value, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8;
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::ControlChange {
                            control: control as u8,
                            value: value as u8,
                        },
                        channel: midi_chan,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiProgram(program, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8;
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::ProgramChange {
                            program: program as u8,
                        },
                        channel: midi_chan,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiAftertouch(note, pressure, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8;
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::Aftertouch {
                            note: note as u8,
                            value: pressure as u8,
                        },
                        channel: midi_chan,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiChannelPressure(pressure, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8;
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::ChannelPressure {
                            value: pressure as u8,
                        },
                        channel: midi_chan,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiStart(_device_id) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::Start {},
                        channel: 0, // System messages use channel 0
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiStop(_device_id) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::Stop {},
                        channel: 0,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiContinue(_device_id) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::Continue {},
                        channel: 0,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiClock(_device_id) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::Clock {},
                        channel: 0,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiReset(_device_id) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::Reset {},
                        channel: 0,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiSystemExclusive(data, _device_id) => {
                let data = data.iter().map(|x| *x as u8).collect();
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::SystemExclusive { data },
                        channel: 0,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            _ => Vec::new(), // Ignore Nop or other non-MIDI events
        }
    }

    /// Generates a `TimedMessage` containing a `LogMessage` payload from a `ConcreteEvent`.
    ///
    /// Wraps the original event within the `LogMessage` for context.
    fn generate_log_message(
        &self,
        payload: ConcreteEvent,
        date: SyncTime,
        device: Arc<ProtocolDevice>, // Expects ProtocolDevice::Log
    ) -> Vec<TimedMessage> {
        vec![ProtocolMessage {
            // Use the LogMessage constructor to store the event directly
            payload: LogMessage::from_event(Severity::Info, payload).into(),
            device: Arc::clone(&device),
        }
        .timed(date)]
    }

    /// Maps a `ConcreteEvent` to `TimedMessage`s for a target device specified by its `target_device_name`.
    ///
    /// This function handles the core logic of translating events into the appropriate
    /// protocol messages (MIDI, OSC, Log) based on the type of the target device.
    ///
    /// # Arguments
    /// * `target_device_name` - The name of the destination device.
    /// * `event` - The `ConcreteEvent` to be mapped.
    /// * `date` - The timestamp (`SyncTime`) for the generated message(s).
    /// * `clock` - A reference to the `Clock` for time-sensitive calculations (e.g., for Dirt/SuperDirt messages).
    ///
    /// # Behavior
    /// - If `target_device_name` is `"log"` (case-sensitive), it generates a `LogMessage`.
    /// - Otherwise, it looks up the device in `output_connections`.
    /// - If the device is not found, it generates an error `LogMessage`.
    /// - If the device is found, it dispatches based on the `ProtocolDevice` type:
    ///   - `OSCOutputDevice`: Maps the event to an `OSCMessage`. Handles `ConcreteEvent::Osc` directly
    ///     and maps `ConcreteEvent::Dirt` to a SuperDirt `/dirt/play` message, calculating
    ///     context parameters (cps, cycle, delta, orbit) using the provided `clock`. Also includes
    ///     legacy mappings for some MIDI events to generic OSC paths (e.g., `/midi/noteon`).
    ///   - `MIDIOutDevice` or `VirtualMIDIOutDevice`: Calls `generate_midi_message`.
    ///   - `Log`: Calls `generate_log_message` (defensive, usually handled by the initial check).
    ///   - Other types: Prints an error and returns an empty vector.
    ///
    /// # Returns
    /// A `Vec<TimedMessage>` containing zero or more protocol messages ready for scheduling/sending.
    /// Note that some events (like `MidiNote`) generate multiple messages (NoteOn and NoteOff).
    pub fn map_event_for_device_name(
        &self,
        target_device_name: &str,
        event: ConcreteEvent,
        date: SyncTime,
        clock: &Clock, // Required for time context, e.g., Dirt messages
    ) -> Vec<TimedMessage> {

        // Handle Log Device implicitly first
        if target_device_name == LOG_NAME {
            // generate_log_message now stores the event.
            return self.generate_log_message(event, date, Arc::new(ProtocolDevice::Log));
        }

        // Look up the device in connected outputs
        let device_opt = self.output_connections.lock().unwrap().values()
            .find(|(name, _)| name == target_device_name)
            .map(|(_, device_arc)| Arc::clone(device_arc));

        let Some(device) = device_opt else {
            // Log error if the device name was not "log" and wasn't found
            return vec![ProtocolMessage {
                payload: LogMessage::error(
                    format!("Device name '{}' not found or not connected.", target_device_name)
                ).into(),
                device: Arc::new(ProtocolDevice::Log), // Send error to the log device
            }
            .timed(date)];
        };

        // Dispatch based on the found device type
        match &*device {
            ProtocolDevice::OSCOutputDevice {..} => {
                let osc_payload_opt: Option<OSCMessage> = match event {
                    // Handle Generic OSC Event (pass-through)
                    ConcreteEvent::Osc { message, device_id: _ } => {
                         Some(message)
                    }
                    // Handle Dirt Event (map to /dirt/play with context)
                    ConcreteEvent::Dirt { sound, params, device_id: _ } => {
                        // Calculate SuperDirt context using the clock
                        let tempo_bpm = clock.tempo();
                        let cps_val = tempo_bpm / 60.0;
                        let cycle_val = clock.beat_at_date(date); // Beat at the event's specific time
                        let delta_micros = clock.beats_to_micros(1.0); // Use 1 beat for delta
                        let delta_val = delta_micros as f64 / 1_000_000.0;
                        let orbit_val = 0i32; // Default orbit

                        let capacity = 4 * 2 + 2 + params.len() * 2;
                        let mut args: Vec<OscArgument> = Vec::with_capacity(capacity);

                        // Add context parameters
                        args.push(OscArgument::String("cps".to_string()));
                        args.push(OscArgument::Float(cps_val as f32));
                        args.push(OscArgument::String("cycle".to_string()));
                        args.push(OscArgument::Float(cycle_val as f32));
                        args.push(OscArgument::String("delta".to_string()));
                        args.push(OscArgument::Float(delta_val as f32));
                        args.push(OscArgument::String("orbit".to_string()));
                        args.push(OscArgument::Int(orbit_val));

                        // Add sound ("s") parameter
                        args.push(OscArgument::String("s".to_string()));
                        let sound_arg = match sound {
                            VariableValue::Integer(i) => OscArgument::Int(i as i32),
                            VariableValue::Float(f) => OscArgument::Float(f as f32),
                            VariableValue::Str(s) => OscArgument::String(s),
                            _ => OscArgument::String("default".to_string()), // Fallback
                        };
                        args.push(sound_arg);

                        // Add other parameters
                        for (key, value) in params {
                            args.push(OscArgument::String(key.clone()));
                            let param_arg = match value {
                                VariableValue::Integer(i) => OscArgument::Int(i as i32),
                                VariableValue::Float(f) => OscArgument::Float(f as f32),
                                VariableValue::Str(s) => OscArgument::String(s),
                                _ => {
                                     eprintln!("[WARN] Dirt to OSC: Unsupported param type {:?} for key '{}'. Sending Int 0.", value, key);
                                     OscArgument::Int(0)
                                }
                            };
                            args.push(param_arg);
                        }

                        Some(OSCMessage {
                            addr: "/dirt/play".to_string(),
                            args,
                        })
                    }
                    // Legacy MIDI-to-OSC mappings (consider removal/refinement)
                    ConcreteEvent::MidiNote(note, vel, chan, _dur, _device_id) => {
                        Some(OSCMessage {
                            addr: "/midi/noteon".to_string(),
                            args: vec![
                                OscArgument::Int(note as i32),
                                OscArgument::Int(vel as i32),
                                OscArgument::Int(chan as i32),
                            ],
                        })
                    }
                    ConcreteEvent::MidiControl(control, value, chan, _device_id) => {
                        Some(OSCMessage {
                            addr: "/midi/cc".to_string(),
                            args: vec![
                                OscArgument::Int(control as i32),
                                OscArgument::Int(value as i32),
                                OscArgument::Int(chan as i32),
                            ],
                        })
                    }
                     ConcreteEvent::MidiProgram(program, chan, _device_id) => {
                         Some(OSCMessage {
                             addr: "/midi/program".to_string(),
                             args: vec![
                                 OscArgument::Int(program as i32),
                                 OscArgument::Int(chan as i32),
                             ],
                         })
                     }
                    _ => None, // Ignore other events for OSC for now
                };

                if let Some(osc_payload) = osc_payload_opt {
                    vec![ProtocolMessage {
                        payload: osc_payload.into(),
                        device: Arc::clone(&device),
                    }
                    .timed(date)]
                } else {
                    vec![] // No mapping found for this event to OSC
                }
            },
            ProtocolDevice::MIDIOutDevice(_) | ProtocolDevice::VirtualMIDIOutDevice {..} => {
                // Generate MIDI messages using the helper function
                self.generate_midi_message(event, date, device)
            }
            ProtocolDevice::Log => {
                // Should be unreachable due to the initial check, but kept defensively.
                self.generate_log_message(event, date, device)
            }
            _ => {
                eprintln!("[!] map_event_for_device_name: Unhandled ProtocolDevice type for {}", target_device_name);
                 vec![] // Or generate an error log message
            }
        }
    }

    /// Maps a `ConcreteEvent` to `TimedMessage`s for a target device specified by its `target_slot_id`.
    ///
    /// # Arguments
    /// * `target_slot_id` - The slot ID of the destination device.
    ///   - Slot `0` implicitly targets the internal Log device.
    ///   - Slots `1` to `MAX_DEVICE_SLOTS` target the device assigned via `assign_slot`.
    /// * `event` - The `ConcreteEvent` to be mapped.
    /// * `date` - The timestamp (`SyncTime`) for the generated message(s).
    /// * `clock` - A reference to the `Clock` for time-sensitive calculations (passed to `map_event_for_device_name`).
    ///
    /// # Behavior
    /// - If `target_slot_id` is `0`, it calls `map_event_for_device_name` with `LOG_NAME`.
    /// - If `target_slot_id` is `1` to `MAX_DEVICE_SLOTS`:
    ///   - It looks up the device name assigned to that slot using `get_name_for_slot`.
    ///   - If a name is found, it calls `map_event_for_device_name` with that name.
    ///   - If no device is assigned to the slot, it generates a warning `LogMessage` containing the original event.
    /// - If `target_slot_id` is invalid (outside 0-N range), behavior is currently undefined by slot lookup,
    ///   but `get_name_for_slot` would return `None`, leading to the warning log message.
    ///
    /// # Returns
    /// A `Vec<TimedMessage>` resulting from the call to `map_event_for_device_name` or a single warning `LogMessage`.
    pub fn map_event_for_slot_id(
        &self,
        target_slot_id: usize,
        event: ConcreteEvent,
        date: SyncTime,
        clock: &Clock, // Pass clock through
    ) -> Vec<TimedMessage> {
        if target_slot_id == 0 {
            // Slot 0 always targets the Log device
            self.map_event_for_device_name(LOG_NAME, event, date, clock)
        } else {
            // Look up the device name assigned to the slot ID (1-N)
            let device_name_opt = self.get_name_for_slot(target_slot_id);

            match device_name_opt {
                Some(device_name) => {
                    // Found an assigned device, map using its name
                    self.map_event_for_device_name(&device_name, event, date, clock)
                }
                None => {
                    // Slot is not assigned, generate a warning log message
                    vec![ProtocolMessage {
                        payload: LogMessage {
                            level: Severity::Warn,
                            event: Some(event), // Include the original event for context
                            msg: format!("Slot {} is not assigned", target_slot_id),
                        }
                        .into(),
                        device: Arc::new(ProtocolDevice::Log), // Send warning to log
                    }
                    .timed(date)]
                }
            }
        }
    }

    /// Generates a list of discoverable and currently connected devices.
    ///
    /// This function aggregates information from:
    /// - System MIDI input ports (via `midir`).
    /// - System MIDI output ports (via `midir`).
    /// - Currently connected output devices (`output_connections`), which includes
    ///   connected physical MIDI, virtual MIDI, and OSC devices.
    ///
    /// It attempts to provide a consolidated view, including the assigned slot ID (if any),
    /// connection status, and address (for OSC devices).
    ///
    /// # Returns
    /// A `Vec<DeviceInfo>` sorted primarily by assigned slot ID (ascending, with 0/unassigned last)
    /// and secondarily by name (alphabetical for unassigned devices).
    /// The internal Log device is excluded from this list.
    pub fn device_list(&self) -> Vec<DeviceInfo> {
        println!("[~] Generating device list (excluding implicit log)...");
        let mut discovered_devices_map: HashMap<String, DeviceInfo> = HashMap::new();
        let slot_map = self.slot_assignments.lock().unwrap();
        let connected_map = self.output_connections.lock().unwrap(); // Lock output connections once

        // Helper to create DeviceInfo, checking slot assignment and connection status
        let create_device_info = |name: String, kind: DeviceKind, device_ref_opt: Option<&ProtocolDevice>| -> DeviceInfo {
            let assigned_slot_id = slot_map.iter()
                .find_map(|(slot, assigned_name)| if assigned_name == &name { Some(*slot) } else { None })
                .unwrap_or(0); // 0 if not assigned

            // Determine connection status based on presence in connected_map for outputs
            // For system ports discovered but not explicitly connected via BuboCore, this might show false.
            let is_connected = connected_map.values().any(|(conn_name, _)| conn_name == &name);

            // Extract address specifically for OSC devices using the provided reference
            let address = if kind == DeviceKind::Osc {
                 device_ref_opt.and_then(|device| match device {
                     ProtocolDevice::OSCOutputDevice { address, .. } => Some(address.to_string()),
                     _ => None,
                 })
            } else {
                 None
            };

            DeviceInfo {
                id: assigned_slot_id,
                name,
                kind,
                is_connected,
                address,
            }
        };

        // Discover system MIDI output ports
        if let Some(midi_out_arc) = &self.midi_out {
            if let Ok(midi_out) = midi_out_arc.lock() {
                for port in midi_out.ports() {
                    if let Ok(name) = midi_out.port_name(&port) {
                        if !discovered_devices_map.contains_key(&name) {
                             // Pass None for device_ref_opt as this is just discovery
                             discovered_devices_map.insert(name.clone(), create_device_info(name, DeviceKind::Midi, None));
                        }
                    }
                }
            }
        }

        // Discover system MIDI input ports
        if let Some(midi_in_arc) = &self.midi_in {
            if let Ok(midi_in) = midi_in_arc.lock() {
                for port in midi_in.ports() {
                     if let Ok(name) = midi_in.port_name(&port) {
                         if !discovered_devices_map.contains_key(&name) {
                            // Pass None for device_ref_opt
                              discovered_devices_map.insert(name.clone(), create_device_info(name, DeviceKind::Midi, None));
                         }
                     }
                }
            }
        }

        // Add currently connected devices (MIDI & OSC) from output_connections, potentially overwriting discovered info
        // This ensures `is_connected` is true and OSC address is included for these.
        for (_device_addr, (name, device_arc)) in connected_map.iter() {
             // Determine kind and get device reference
             let (kind, device_ref) = match &**device_arc {
                ProtocolDevice::MIDIOutDevice { .. } => (DeviceKind::Midi, Some(&**device_arc)),
                 ProtocolDevice::VirtualMIDIOutDevice { .. } => (DeviceKind::Midi, Some(&**device_arc)), // Treat virtual as MIDI
                 ProtocolDevice::OSCOutputDevice { .. } => (DeviceKind::Osc, Some(&**device_arc)),
                 _ => (DeviceKind::Other, None), // Skip Log, In, etc.
             };

             if kind == DeviceKind::Midi || kind == DeviceKind::Osc {
                 // Insert or update the entry using create_device_info with the device reference
                 discovered_devices_map.insert(name.clone(), create_device_info(name.clone(), kind, device_ref));
             }
        }
        drop(connected_map); // Release lock

        let mut final_list: Vec<DeviceInfo> = discovered_devices_map.into_values().collect();

        // Sort: Assigned devices first (by Slot ID), then unassigned devices (alphabetically)
        final_list.sort_by(|a, b| {
            match (a.id, b.id) {
                (0, 0) => a.name.cmp(&b.name), // Both unassigned: sort by name
                (0, _) => std::cmp::Ordering::Greater, // Unassigned goes after assigned
                (_, 0) => std::cmp::Ordering::Less, // Assigned goes before unassigned
                (id_a, id_b) => id_a.cmp(&id_b), // Both assigned: sort by slot ID
            }
        });

        println!("[~] Device list generated. Count: {}", final_list.len());
        final_list
    }

    /// Connects to a physical MIDI device specified by its exact name (bidirectional).
    ///
    /// Attempts to open both the MIDI input and output ports matching the given name.
    /// If successful, registers the device in both `input_connections` and `output_connections`.
    ///
    /// # Arguments
    /// * `device_name` - The exact name of the MIDI device as reported by the system.
    ///
    /// # Returns
    /// - `Ok(())` on successful connection and registration of both input and output.
    /// - `Err(String)` if the device is already connected, if either input or output port cannot be opened,
    ///   or if there's an error creating the internal handlers.
    pub fn connect_midi_by_name(&self, device_name: &str) -> Result<(), String> {
        println!("[🔌] Attempting to connect MIDI device (In/Out): {}", device_name);

        // Check if already connected (using output_connections as the primary check)
        if self.output_connections.lock().unwrap().values().any(|(name, _)| name == device_name) {
             return Err(format!("Device '{}' is already connected.", device_name));
        }

        // Create MidiIn and MidiOut handlers
        let mut midi_in_handler = MidiIn::new(device_name.to_string())
            .map_err(|e| format!("Failed to create MidiIn handler: {:?}", e))?;
        let mut midi_out_handler = MidiOut::new(device_name.to_string())
            .map_err(|e| format!("Failed to create MidiOut handler: {:?}", e))?;

        // Attempt to connect Input first
        match midi_in_handler.connect_to_port_by_name(device_name) {
            Ok(_) => {
                 println!("[✅] Connected MIDI Input: {}", device_name);
                 // Input succeeded, now try Output
                 match midi_out_handler.connect_to_port_by_name(device_name) {
                     Ok(_) => {
                         println!("[✅] Connected MIDI Output: {}", device_name);
                         // Both connected successfully, register them
                         let in_device = ProtocolDevice::MIDIInDevice(Arc::new(Mutex::new(midi_in_handler)));
                         let out_device = ProtocolDevice::MIDIOutDevice(Arc::new(Mutex::new(midi_out_handler)));
                         self.register_input_connection(device_name.to_string(), in_device);
                         self.register_output_connection(device_name.to_string(), out_device);
                         println!("[✅] Registered MIDI device: {}", device_name);
                         Ok(())
                     }
                     Err(e) => {
                         // Output failed after Input succeeded. The input connection will be implicitly closed
                         // when midi_in_handler goes out of scope.
                         eprintln!("[!] Failed to connect MIDI Output '{}' after Input succeeded: {:?}", device_name, e);
                         Err(format!("Failed to connect MIDI Output '{}': {:?}", device_name, e))
                     }
                 }
            }
            Err(e) => {
                 // Input failed
                 eprintln!("[!] Failed to connect MIDI Input '{}': {:?}", device_name, e);
                 Err(format!("Failed to connect MIDI Input '{}': {:?}", device_name, e))
             }
        }
    }

    /// Disconnects a MIDI device specified by name (bidirectional).
    ///
    /// Removes the device from both `input_connections` and `output_connections`.
    /// Also unassigns the device from any slot it might occupy.
    /// The underlying MIDI connections are closed when the `MidiIn`/`MidiOut` handlers are dropped.
    ///
    /// # Arguments
    /// * `device_name` - The name of the MIDI device to disconnect.
    ///
    /// # Returns
    /// - `Ok(())` on successful removal from registrations.
    /// - `Err(String)` if the device was not found in the connections or if the state is inconsistent
    ///   (e.g., found in input but not output).
    pub fn disconnect_midi_by_name(&self, device_name: &str) -> Result<(), String> {
         println!("[🔌] Attempting to disconnect MIDI device (In/Out): {}", device_name);
         let mut output_connections = self.output_connections.lock().unwrap();
         let mut input_connections = self.input_connections.lock().unwrap();

         // Find the keys (addresses) associated with the device name
         let output_key_to_remove = output_connections.iter()
             .find(|(_address, (name, _device))| name == device_name)
             .map(|(address, _item)| address.clone());

         let input_key_to_remove = input_connections.iter()
             .find(|(_address, (name, _device))| name == device_name)
             .map(|(address, _item)| address.clone());

        match (output_key_to_remove, input_key_to_remove) {
            (Some(out_key), Some(in_key)) => {
                // Remove from both maps. Dropping the Arc<ProtocolDevice> will eventually drop
                // the MidiIn/MidiOut handlers, closing the ports.
                let out_removed = output_connections.remove(&out_key).is_some();
                let in_removed = input_connections.remove(&in_key).is_some();

                if out_removed && in_removed {
                     println!("[✅] Disconnected and removed registration for MIDI In/Out '{}'", device_name);
                     // Release locks before calling another method that might lock
                     drop(output_connections);
                     drop(input_connections);
                     // Unassign from any slot
                     self.unassign_device_by_name(device_name);
                    Ok(())
                } else {
                      // This indicates an internal logic error if keys were found but removal failed
                      eprintln!("[!] Mismatch removing connections for '{}'. Out removed: {}, In removed: {}", device_name, out_removed, in_removed);
                     Err(format!("Internal error removing connections for {}", device_name))
                }
            }
            (None, None) => {
                 eprintln!("[!] Cannot disconnect MIDI device '{}': Not found in connections.", device_name);
                 Err(format!("Device '{}' not found or not connected.", device_name))
             }
             _ => {
                 // Found in one map but not the other - indicates an inconsistent state
                  eprintln!("[!] Cannot disconnect MIDI device '{}': Inconsistent connection state (In/Out mismatch).", device_name);
                  // Attempt removal from wherever it was found to try and clean up? Or just error?
                  // For now, just error out. Consider adding cleanup logic if this state occurs.
                  Err(format!("Device '{}' has inconsistent connection state.", device_name))
             }
         }
    }

    /// Creates a virtual MIDI port pair (Input and Output) with the specified name.
    ///
    /// Uses the `midir` library to create platform-specific virtual MIDI endpoints.
    /// If successful, registers the port pair in `input_connections` and `output_connections`.
    /// Does NOT assign the new virtual port to a slot automatically.
    ///
    /// # Arguments
    /// * `desired_name` - The name for the new virtual MIDI port pair.
    ///
    /// # Returns
    /// - `Ok(String)` containing the actual name used (usually `desired_name`) on success.
    /// - `Err(String)` if a device with that name already exists (checked via `device_list`),
    ///   or if the underlying `midir` calls fail to create the virtual ports.
    pub fn create_virtual_midi_port(&self, desired_name: &str) -> Result<String, String> {
        println!("[✨] Creating virtual MIDI port (In/Out): '{}'", desired_name);

        // Check if name is already used by any known device (system or virtual)
        if self.device_list().iter().any(|d| d.name == desired_name) {
            return Err(format!("Device name '{}' already exists.", desired_name));
        }

        // Create handlers
        let mut midi_in_handler = MidiIn::new(desired_name.to_string())
            .map_err(|e| format!("Failed to create MidiIn handler for virtual port: {:?}", e))?;
        let mut midi_out_handler = MidiOut::new(desired_name.to_string())
            .map_err(|e| format!("Failed to create MidiOut handler for virtual port: {:?}", e))?;

        // Attempt to create virtual Output source first
        match midi_out_handler.create_virtual_port() {
            Ok(_) => {
                 println!("[✅] Virtual MIDI Output source created: '{}'", desired_name);

                // Now create the virtual Input destination
                match midi_in_handler.create_virtual_port() {
                    Ok(_) => {
                        println!("[✅] Virtual MIDI Input destination created: '{}'", desired_name);

                        // Both endpoints created, register them
                         let in_device = ProtocolDevice::MIDIInDevice(Arc::new(Mutex::new(midi_in_handler)));
                         // Use VirtualMIDIOutDevice variant? Or stick to MIDIOutDevice?
                         // Sticking to MIDIOutDevice simplifies matching later. The underlying handler is correct.
                         let out_device = ProtocolDevice::MIDIOutDevice(Arc::new(Mutex::new(midi_out_handler)));
                         // Let's use a specific VirtualMIDIOutDevice type for clarity if needed elsewhere
                         // let out_device = ProtocolDevice::VirtualMIDIOutDevice { name: desired_name.to_string(), handler: Arc::new(Mutex::new(midi_out_handler))};

                         self.register_input_connection(desired_name.to_string(), in_device);
                         self.register_output_connection(desired_name.to_string(), out_device);
                         println!("[✅] Registered virtual MIDI port pair: '{}'", desired_name);
                         Ok(desired_name.to_string()) // Return the name on success
                    }
                    Err(e) => {
                        // Input creation failed after Output succeeded. Output port will close automatically.
                        eprintln!("[!] Failed to create Virtual MIDI Input destination '{}' after Output source creation: {:?}", desired_name, e);
                        Err(format!("Failed to create Virtual MIDI Input destination '{}': {:?}", desired_name, e))
                    }
                }
            }
            Err(e) => {
                // Output creation failed
                eprintln!("[!] Failed to create Virtual MIDI Output source '{}': {:?}", desired_name, e);
                Err(format!("Failed to create Virtual MIDI Output source '{}': {:?}", desired_name, e))
            }
        }
    }

    /// Creates and registers a new OSC Output device targeting a specific IP address and port.
    ///
    /// Attempts to bind a local UDP socket for sending messages.
    ///
    /// # Arguments
    /// * `name` - A unique name for this OSC output device.
    /// * `ip_str` - The target IP address as a string (e.g., "127.0.0.1").
    /// * `port` - The target UDP port number.
    ///
    /// # Returns
    /// - `Ok(())` on successful creation, connection (socket binding), and registration.
    /// - `Err(String)` if the IP address format is invalid, if the name already exists,
    ///   if another OSC device already targets the same address:port, or if the UDP socket
    ///   cannot be bound.
    pub fn create_osc_output_device(&self, name: &str, ip_str: &str, port: u16) -> Result<(), String> {
        println!("[✨] Creating OSC Output device: '{}' @ {}:{}", name, ip_str, port);

        // Parse target IP and create SocketAddr
        let target_ip_addr = IpAddr::from_str(ip_str)
            .map_err(|e| format!("Invalid IP address format '{}': {}", ip_str, e))?;
        let target_socket_addr = SocketAddr::new(target_ip_addr, port);

        // Check for existing name or address collision
        { // Scope for lock
            let output_connections = self.output_connections.lock().unwrap();
            for (existing_name, device_arc) in output_connections.values() {
                if existing_name == name {
                    let err_msg = format!("Cannot create OSC device: Name '{}' already exists.", name);
                    eprintln!("[!] {}", err_msg);
                    return Err(err_msg);
                }
                // Check specifically for OSC address collision
                if let ProtocolDevice::OSCOutputDevice { address: existing_addr, .. } = &**device_arc {
                    if *existing_addr == target_socket_addr {
                        let err_msg = format!("Cannot create OSC device '{}': Another OSC device already targets address '{}'.", name, target_socket_addr);
                        eprintln!("[!] {}", err_msg);
                        return Err(err_msg);
                    }
                }
            }
        } // Lock released here

        // Create the OSCOutputDevice instance
        let mut osc_device = ProtocolDevice::OSCOutputDevice {
            name: name.to_string(),
            address: target_socket_addr,
            latency: 0.02, // Default latency
            socket: None, // Socket will be created in connect()
        };

        // Attempt to connect (bind local socket)
        match osc_device.connect() {
            Ok(_) => {
                println!("[✅] OSC Output device '{}' socket created successfully.", name);
                // Register the now-connected device
                self.register_output_connection(name.to_string(), osc_device);
                println!("[✅] Registered OSC Output device: '{}'", name);
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Failed to connect/bind socket for OSC device '{}': {:?}", name, e);
                 eprintln!("[!] {}", err_msg);
                 Err(err_msg)
            }
        }
    }

    /// Removes an OSC Output device by its name.
    ///
    /// Removes the device registration from `output_connections`. The underlying socket
    /// will be closed when the `ProtocolDevice::OSCOutputDevice` is dropped.
    /// Also unassigns the device from any slot it might occupy.
    ///
    /// # Arguments
    /// * `name` - The name of the OSC Output device to remove.
    ///
    /// # Returns
    /// - `Ok(())` on successful removal from registration.
    /// - `Err(String)` if no OSC Output device with the given name is found.
    pub fn remove_osc_output_device(&self, name: &str) -> Result<(), String> {
        println!("[🗑️] Removing OSC Output device: '{}'", name);
        let mut output_connections = self.output_connections.lock().unwrap();

        // Find the key (address string) associated with the named OSC device
        let key_to_remove = output_connections.iter()
            .find(|(_address, (n, device))| n == name && matches!(**device, ProtocolDevice::OSCOutputDevice{..}))
            .map(|(address, _item)| address.clone());

        match key_to_remove {
            Some(key) => {
                if output_connections.remove(&key).is_some() {
                    println!("[✅] Removed OSC Output device registration: '{}'", name);
                    // Release lock before potentially calling another method
                    drop(output_connections);
                    // Unassign from any slot
                    self.unassign_device_by_name(name);
                    Ok(())
                } else {
                    // Should not happen if key was found
                    let err_msg = format!("Internal error removing OSC device '{}'", name);
                    eprintln!("[!] {}", err_msg);
                    Err(err_msg)
                }
            }
            None => {
                let err_msg = format!("Cannot remove OSC device '{}': Not found or not an OSC device.", name);
                eprintln!("[!] {}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Sends the MIDI "All Notes Off" message (Control Change 123, Value 0)
    /// to all connected MIDI output devices (physical and virtual) on all 16 channels.
    ///
    /// This is a utility function to stop hanging notes.
    pub fn panic_all_midi_outputs(&self) {
        println!("[!] Sending MIDI Panic (All Notes Off CC 123) to all outputs...");
        let connections = self.output_connections.lock().unwrap();

        for (_device_addr, (name, device_arc)) in connections.iter() {
            // Target MIDIOutDevice (covers both physical and virtual)
            if let ProtocolDevice::MIDIOutDevice(midi_out_mutex) = &**device_arc {
                println!("[!] Sending Panic to MIDI device: {}", name);
                if let Ok(midi_out) = midi_out_mutex.lock() {
                    for chan in 0..16 { // Send on all 16 channels (0-15)
                        let msg = MIDIMessage {
                            payload: MIDIMessageType::ControlChange { control: 123, value: 0 },
                            channel: chan,
                        };
                        // Attempt to send, log errors but continue
                        if let Err(e) = midi_out.send(msg) {
                            eprintln!("[!] Error sending panic CC 123 chan {} to {}: {:?}", chan, name, e);
                        }
                    }
                     // Optionally send "All Sound Off" (CC 120) as well?
                     // for chan in 0..16 {
                     //     let msg = MIDIMessage {
                     //         payload: MIDIMessageType::ControlChange { control: 120, value: 0 },
                     //         channel: chan,
                     //     };
                     //     if let Err(e) = midi_out.send(msg) {
                     //         eprintln!("[!] Error sending panic CC 120 chan {} to {}: {:?}", chan, name, e);
                     //     }
                     // }
                } else {
                     eprintln!("[!] Could not lock Mutex for MIDI device: {}", name);
                }
            }
        }
         println!("[!] MIDI Panic finished.");
    }
}

impl Default for DeviceMap {
    fn default() -> Self {
        Self::new()
    }
}
