//! Manages connections to external and virtual devices (MIDI, OSC, Log)
//! and maps internal events to protocol-specific messages for output.
//!
//! This module provides the `DeviceMap` struct, which serves as the central
//! registry for devices known to the Sova system. It handles:
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
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{
    clock::{Clock, SyncTime},
    log_eprintln, log_println,
    protocol::{
        DeviceDirection, DeviceInfo, DeviceKind, ProtocolDevice, ProtocolMessage, TimedMessage,
        audio_engine_proxy::AudioEngineProxy,
        log::{LOG_NAME, LogMessage, Severity},
        midi::{MIDIMessage, MIDIMessageType, MidiIn, MidiInterface, MidiOut},
        osc::OSCOut,
    },
    vm::event::ConcreteEvent,
};

use midir::{Ignore, MidiInput, MidiOutput};

/// Maximum number of user-assignable device slots (1-based).
const MAX_DEVICE_SLOTS: usize = 16;

/// Manages device connections, slot assignments, and event-to-protocol mapping.
///
/// Provides thread-safe access to device information and handles the underlying
/// MIDI and OSC communication setup via `midir` and `rosc`.
pub struct DeviceMap {
    /// Currently connected input devices, keyed by their unique user-given name.
    /// Values are `DeviceItem`s containing the assigned name and the device handle.
    pub input_connections: Mutex<BTreeMap<String, Arc<ProtocolDevice>>>,
    /// Currently connected output devices, keyed by their unique user-given name.
    /// Values are `DeviceItem`s containing the assigned name and the device handle.
    pub output_connections: Mutex<BTreeMap<String, Arc<ProtocolDevice>>>,
    /// Maps user-assigned Slot IDs (1-N) to the system or virtual device name assigned to it.
    /// Slot 0 is implicitly the Log device and is not stored here.
    pub slot_assignments: Mutex<[Option<String>; MAX_DEVICE_SLOTS]>,
    /// Log device
    pub log_device: Arc<ProtocolDevice>,
    /// Optional handle to the system's MIDI input interface, managed by `midir`.
    midi_in: Option<Arc<Mutex<MidiInput>>>,
    /// Optional handle to the system's MIDI output interface, managed by `midir`.
    midi_out: Option<Arc<Mutex<MidiOutput>>>,
    /// Names of devices from snapshot that couldn't be restored (unplugged physical devices).
    /// These are reconstructed as DeviceInfo in device_list() with is_missing: true.
    missing_devices: Mutex<BTreeSet<String>>,
}

impl DeviceMap {
    /// Creates a new `DeviceMap` and attempts to initialize system MIDI interfaces.
    /// The internal Log device is handled implicitly and not registered here.
    pub fn new() -> Self {
        let midi_in = match MidiInput::new("Sova Input") {
            Ok(mut input) => {
                input.ignore(Ignore::None);
                log_println!("MIDI Input initialized successfully.");
                Some(Arc::new(Mutex::new(input)))
            }
            Err(e) => {
                log_eprintln!("Failed to initialize MIDI Input: {}", e);
                None
            }
        };

        let midi_out = match MidiOutput::new("Sova Output") {
            Ok(output) => {
                log_println!("MIDI Output initialized successfully.");
                Some(Arc::new(Mutex::new(output)))
            }
            Err(e) => {
                log_eprintln!("Failed to initialize MIDI Output: {}", e);
                None
            }
        };

        DeviceMap {
            input_connections: Default::default(),
            output_connections: Default::default(),
            slot_assignments: Default::default(),
            log_device: Arc::new(ProtocolDevice::Log),
            midi_in,
            midi_out,
            missing_devices: Default::default(),
        }
    }

    /// Registers a connected input device.
    ///
    /// Associates the given `name` with the `device` and stores it in the
    /// `input_connections` map, keyed by the device's address.
    pub fn register_input_connection(&self, name: String, device: ProtocolDevice) {
        self.input_connections
            .lock()
            .unwrap()
            .insert(name, Arc::new(device));
    }

    /// Registers a connected output device.
    ///
    /// Associates the given `name` with the `device` and stores it in the
    /// `output_connections` map, keyed by the device's address.
    /// Note: This only registers the connection; slot assignment is separate.
    pub fn register_output_connection(&self, name: String, device: ProtocolDevice) {
        self.output_connections
            .lock()
            .unwrap()
            .insert(name, Arc::new(device));
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
        if slot_id == 0 || slot_id > MAX_DEVICE_SLOTS {
            return Err(format!(
                "Invalid slot ID: {}. Must be between 1 and {}.",
                slot_id, MAX_DEVICE_SLOTS
            ));
        }

        let mut assignments = self.slot_assignments.lock().unwrap();

        // Create the new assignment
        let slot_index = slot_id - 1;
        assignments[slot_index] = Some(device_name.to_owned());

        // Remove any existing assignment for the target device_name (a device can only be in one slot)
        for (index, assignment) in assignments.iter_mut().enumerate() {
            let Some(name) = assignment else {
                continue;
            };
            if index != slot_index && name == device_name {
                *assignment = None;
            }
        }

        log_println!("Assigned device '{}' to Slot {}", device_name, slot_id);
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
        if slot_id == 0 || slot_id > MAX_DEVICE_SLOTS {
            return Err(format!(
                "Invalid slot ID: {}. Must be between 1 and {}.",
                slot_id, MAX_DEVICE_SLOTS
            ));
        }
        let slot_index = slot_id - 1;
        let mut assignments = self.slot_assignments.lock().unwrap();
        if let Some(removed_name) = assignments[slot_index].take() {
            log_println!(
                "[-] Unassigned device '{}' from Slot {}",
                removed_name,
                slot_id
            );
        } else {
            log_println!("Slot {} was already empty.", slot_id);
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
        for (index, assignment) in assignments.iter_mut().enumerate() {
            let Some(name) = assignment else {
                continue;
            };
            if name == device_name {
                found_slot = Some(index + 1);
                *assignment = None;
            }
        }
        if let Some(slot) = found_slot {
            log_println!("[-] Unassigned device '{}' from Slot {}", device_name, slot);
        }
    }

    /// Finds the device name assigned to a specific slot ID (1-N).
    ///
    /// Returns `None` if the slot is invalid or not currently assigned.
    pub fn get_name_for_slot(&self, slot_id: usize) -> Option<String> {
        if slot_id == 0 || slot_id > MAX_DEVICE_SLOTS {
            return None;
        }
        self.slot_assignments.lock().unwrap()[slot_id - 1].clone()
    }

    /// Finds the slot ID (1-N) assigned to a specific device name.
    ///
    /// Returns `None` if the device name is not assigned to any slot.
    pub fn get_slot_for_name(&self, device_name: &str) -> Option<usize> {
        for (index, assignment) in self.slot_assignments.lock().unwrap().iter().enumerate() {
            let Some(name) = assignment else {
                continue;
            };
            if name == device_name {
                return Some(index + 1);
            }
        }
        None
    }

    pub fn get_out_device_at_slot(&self, slot_id: usize) -> Option<Arc<ProtocolDevice>> {
        self.get_name_for_slot(slot_id).and_then(|name| {
            let outputs = self.output_connections.lock().unwrap();
            let dev_item = outputs.get(&name);
            dev_item.map(Arc::clone)
        })
    }

    fn map_event_to_device(
        device: &Arc<ProtocolDevice>,
        event: ConcreteEvent,
        date: SyncTime,
        clock: &Clock,
    ) -> Vec<TimedMessage> {
        let timed = device.translate_event(event, date, clock);
        timed
            .into_iter()
            .map(|(payload, time)| {
                ProtocolMessage {
                    device: Arc::clone(device),
                    payload,
                }
                .timed(time)
            })
            .collect()
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
    ///   - `OSCOutDevice`: Maps the event to an `OSCMessage`. Handles `ConcreteEvent::Osc` directly
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
            return Self::map_event_to_device(&self.log_device, event, date, clock);
        }

        // Look up the device in connected outputs
        let device_opt = self
            .output_connections
            .lock()
            .unwrap()
            .get(target_device_name)
            .map(Arc::clone);

        let Some(device) = device_opt else {
            // Log error if the device name was not "log" and wasn't found
            return vec![
                ProtocolMessage {
                    payload: LogMessage::error(format!(
                        "Device name '{}' not found or not connected.",
                        target_device_name
                    ))
                    .into(),
                    device: Arc::clone(&self.log_device), // Send error to the log device
                }
                .timed(date),
            ];
        };

        Self::map_event_to_device(&device, event, date, clock)
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
            return Self::map_event_to_device(&self.log_device, event, date, clock);
        } else {
            // Look up the device name assigned to the slot ID (1-N)
            match self.get_name_for_slot(target_slot_id) {
                Some(device_name) => {
                    // Found an assigned device, map using its name
                    self.map_event_for_device_name(&device_name, event, date, clock)
                }
                None => {
                    // Slot is not assigned, generate a warning log message
                    vec![
                        ProtocolMessage {
                            payload: LogMessage {
                                level: Severity::Warn,
                                event: Some(event), // Include the original event for context
                                msg: format!("Slot {} is not assigned", target_slot_id),
                            }
                            .into(),
                            device: Arc::clone(&self.log_device), // Send warning to log
                        }
                        .timed(date),
                    ]
                }
            }
        }
    }

    pub fn map_event(
        &self,
        event: ConcreteEvent,
        date: SyncTime,
        clock: &Clock, // Pass clock through
    ) -> Vec<TimedMessage> {
        let Some(device_id) = event.device_id() else {
            return Vec::new();
        };
        self.map_event_for_slot_id(device_id, event, date, clock)
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
        let mut discovered_devices_map: BTreeMap<String, DeviceInfo> = BTreeMap::new();
        let connected_map = self.output_connections.lock().unwrap(); // Lock output connections once

        // Helper to create DeviceInfo, checking slot assignment and connection status
        let create_device_info = |name: String,
                                  kind: DeviceKind,
                                  direction: DeviceDirection,
                                  device_ref_opt: Option<&ProtocolDevice>|
         -> DeviceInfo {
            let assigned_slot_id = self.get_slot_for_name(&name);

            // Determine connection status based on presence in connected_map for outputs
            // For system ports discovered but not explicitly connected via Sova, this might show false.
            let is_connected = connected_map.contains_key(&name);

            // Extract address specifically for OSC devices using the provided reference
            let address = match device_ref_opt {
                Some(d) => Some(d.address()),
                _ => None,
            };

            DeviceInfo {
                slot_id: assigned_slot_id,
                name,
                kind,
                direction,
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
                            discovered_devices_map.insert(
                                name.clone(),
                                create_device_info(
                                    name,
                                    DeviceKind::Midi,
                                    DeviceDirection::Output,
                                    None,
                                ),
                            );
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
                            discovered_devices_map.insert(
                                name.clone(),
                                create_device_info(
                                    name,
                                    DeviceKind::Midi,
                                    DeviceDirection::Input,
                                    None,
                                ),
                            );
                        }
                    }
                }
            }
        }

        // Add currently connected devices from output_connections, potentially overwriting discovered info
        // This ensures `is_connected` is true and addresses are included for these.
        for (name, device_arc) in connected_map.iter() {
            // Determine kind and get device reference
            let kind = device_arc.kind();

            if kind == DeviceKind::Midi
                || kind == DeviceKind::Osc
                || kind == DeviceKind::AudioEngine
            {
                // Insert or update the entry using create_device_info with the device reference
                discovered_devices_map.insert(
                    name.clone(),
                    create_device_info(
                        name.clone(),
                        kind,
                        DeviceDirection::Output,
                        Some(&device_arc),
                    ),
                );
            }
        }
        drop(connected_map); // Release lock

        // Add missing devices (from snapshot that couldn't be restored)
        for missing_name in self.missing_devices.lock().unwrap().iter() {
            if !discovered_devices_map.contains_key(missing_name) {
                discovered_devices_map.insert(
                    missing_name.clone(),
                    DeviceInfo {
                        slot_id: self.get_slot_for_name(missing_name),
                        name: missing_name.clone(),
                        kind: DeviceKind::Midi,
                        direction: DeviceDirection::Output,
                        is_connected: false,
                        address: None,
                    },
                );
            }
        }

        let mut final_list: Vec<DeviceInfo> = discovered_devices_map.into_values().collect();

        // Sort: Assigned devices first (by Slot ID), then unassigned devices (alphabetically)
        final_list.sort_by(|a, b| {
            match (a.slot_id, b.slot_id) {
                (None, None) => a.name.cmp(&b.name), // Both unassigned: sort by name
                (None, _) => std::cmp::Ordering::Greater, // Unassigned goes after assigned
                (_, None) => std::cmp::Ordering::Less, // Assigned goes before unassigned
                (Some(id_a), Some(id_b)) => id_a.cmp(&id_b), // Both assigned: sort by slot ID
            }
        });

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
        log_println!(
            "Attempting to connect MIDI device (In/Out): {}",
            device_name
        );

        // Check if already connected (using output_connections as the primary check)
        if self
            .output_connections
            .lock()
            .unwrap()
            .contains_key(device_name)
        {
            return Err(format!("Device '{}' is already connected.", device_name));
        }

        // Create MidiIn and MidiOut handlers
        let mut midi_in_handler = MidiIn::new(device_name.to_string())
            .map_err(|e| format!("Failed to create MidiIn handler: {:?}", e))?;
        let mut midi_out_handler = MidiOut::new(device_name.to_string())
            .map_err(|e| format!("Failed to create MidiOut handler: {:?}", e))?;

        // Attempt to connect Input first
        match midi_in_handler.connect() {
            Ok(_) => {
                log_println!("[✅] Connected MIDI Input: {}", device_name);
                // Input succeeded, now try Output
                match midi_out_handler.connect() {
                    Ok(_) => {
                        log_println!("[✅] Connected MIDI Output: {}", device_name);
                        // Both connected successfully, register them
                        let in_device = ProtocolDevice::MIDIInDevice(midi_in_handler);
                        let out_device = ProtocolDevice::MIDIOutDevice(midi_out_handler);
                        self.register_input_connection(device_name.to_string(), in_device);
                        self.register_output_connection(device_name.to_string(), out_device);
                        log_println!("[✅] Registered MIDI device: {}", device_name);
                        Ok(())
                    }
                    Err(e) => {
                        // Output failed after Input succeeded. The input connection will be implicitly closed
                        // when midi_in_handler goes out of scope.
                        log_eprintln!(
                            "Failed to connect MIDI Output '{}' after Input succeeded: {:?}",
                            device_name,
                            e
                        );
                        Err(format!(
                            "Failed to connect MIDI Output '{}': {:?}",
                            device_name, e
                        ))
                    }
                }
            }
            Err(e) => {
                // Input failed
                log_eprintln!("Failed to connect MIDI Input '{}': {:?}", device_name, e);
                Err(format!(
                    "Failed to connect MIDI Input '{}': {:?}",
                    device_name, e
                ))
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
        log_println!(
            "Attempting to disconnect MIDI device (In/Out): {}",
            device_name
        );

        let (input, output) = (
            self.output_connections
                .lock()
                .unwrap()
                .remove(device_name)
                .is_some(),
            self.input_connections
                .lock()
                .unwrap()
                .remove(device_name)
                .is_some(),
        );

        if input && output {
            log_println!(
                "[✅] Disconnected and removed registration for MIDI In/Out '{}'",
                device_name
            );
            Ok(())
        } else if input || output {
            log_eprintln!(
                "Cannot disconnect MIDI device '{}': Inconsistent connection state (In/Out mismatch).",
                device_name
            );
            Err(format!(
                "Device '{}' has inconsistent connection state.",
                device_name
            ))
        } else {
            log_eprintln!(
                "Cannot disconnect MIDI device '{}': Not found in connections.",
                device_name
            );
            Err(format!(
                "Device '{}' not found or not connected.",
                device_name
            ))
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
        log_println!(
            "[✨] Creating virtual MIDI port (In/Out): '{}'",
            desired_name
        );

        // Check if name is already used by a connected device
        if self
            .output_connections
            .lock()
            .unwrap()
            .contains_key(desired_name)
        {
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
                log_println!(
                    "[✅] Virtual MIDI Output source created: '{}'",
                    desired_name
                );

                // Now create the virtual Input destination
                match midi_in_handler.create_virtual_port() {
                    Ok(_) => {
                        log_println!(
                            "[✅] Virtual MIDI Input destination created: '{}'",
                            desired_name
                        );

                        // Both endpoints created, register them
                        let in_device = ProtocolDevice::VirtualMIDIInDevice(midi_in_handler);
                        // Use VirtualMIDIOutDevice variant? Or stick to MIDIOutDevice?
                        // Sticking to MIDIOutDevice simplifies matching later. The underlying handler is correct.
                        let out_device = ProtocolDevice::VirtualMIDIOutDevice(midi_out_handler);
                        // Let's use a specific VirtualMIDIOutDevice type for clarity if needed elsewhere
                        // let out_device = ProtocolDevice::VirtualMIDIOutDevice { name: desired_name.to_string(), handler: Arc::new(Mutex::new(midi_out_handler))};

                        self.register_input_connection(desired_name.to_string(), in_device);
                        self.register_output_connection(desired_name.to_string(), out_device);
                        log_println!("[✅] Registered virtual MIDI port pair: '{}'", desired_name);
                        Ok(desired_name.to_string()) // Return the name on success
                    }
                    Err(e) => {
                        // Input creation failed after Output succeeded. Output port will close automatically.
                        log_eprintln!(
                            "Failed to create Virtual MIDI Input destination '{}' after Output source creation: {:?}",
                            desired_name,
                            e
                        );
                        Err(format!(
                            "Failed to create Virtual MIDI Input destination '{}': {:?}",
                            desired_name, e
                        ))
                    }
                }
            }
            Err(e) => {
                // Output creation failed
                log_eprintln!(
                    "Failed to create Virtual MIDI Output source '{}': {:?}",
                    desired_name,
                    e
                );
                Err(format!(
                    "Failed to create Virtual MIDI Output source '{}': {:?}",
                    desired_name, e
                ))
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
    pub fn create_osc_output_device(
        &self,
        name: &str,
        ip_str: &str,
        port: u16,
    ) -> Result<(), String> {
        log_println!(
            "[✨] Creating OSC Output device: '{}' @ {}:{}",
            name,
            ip_str,
            port
        );

        // Parse target IP and create SocketAddr
        let target_ip_addr = IpAddr::from_str(ip_str)
            .map_err(|e| format!("Invalid IP address format '{}': {}", ip_str, e))?;
        let target_socket_addr = SocketAddr::new(target_ip_addr, port);

        // Check for existing name or address collision
        {
            // Scope for lock
            let output_connections = self.output_connections.lock().unwrap();
            for (existing_name, device_arc) in output_connections.iter() {
                if existing_name == name {
                    let err_msg =
                        format!("Cannot create OSC device: Name '{}' already exists.", name);
                    log_eprintln!("{}", err_msg);
                    return Err(err_msg);
                }
                // Check specifically for OSC address collision
                if let ProtocolDevice::OSCOutDevice(osc_out) = &**device_arc {
                    if osc_out.address == target_socket_addr {
                        let err_msg = format!(
                            "Cannot create OSC device '{}': Another OSC device already targets address '{}'.",
                            name, target_socket_addr
                        );
                        log_eprintln!("{}", err_msg);
                        return Err(err_msg);
                    }
                }
            }
        } // Lock released here

        // Create the OSCOutDevice instance
        let mut osc_device = OSCOut {
            name: name.to_string(),
            address: target_socket_addr,
            latency: 0.02, // Default latency
            socket: None,  // Socket will be created in connect()
        };

        // Attempt to connect (bind local socket)
        match osc_device.connect() {
            Ok(_) => {
                log_println!(
                    "[✅] OSC Output device '{}' socket created successfully.",
                    name
                );
                // Register the now-connected device
                self.register_output_connection(
                    name.to_string(),
                    ProtocolDevice::OSCOutDevice(osc_device),
                );
                log_println!("[✅] Registered OSC Output device: '{}'", name);
                Ok(())
            }
            Err(e) => {
                let err_msg = format!(
                    "Failed to connect/bind socket for OSC device '{}': {:?}",
                    name, e
                );
                log_eprintln!("{}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Removes an output device by its name.
    ///
    /// Removes the device registration from `output_connections`. The underlying socket
    /// will be closed when the `ProtocolDevice::OSCOutDevice` is dropped.
    /// Also unassigns the device from any slot it might occupy.
    ///
    /// # Arguments
    /// * `name` - The name of the OSC Output device to remove.
    ///
    /// # Returns
    /// - `Ok(())` on successful removal from registration.
    /// - `Err(String)` if no OSC Output device with the given name is found.
    pub fn remove_input_device(&self, name: &str) -> Result<(), String> {
        log_println!("[🗑️] Removing OSC Output device: '{}'", name);
        let mut input_connections = self.input_connections.lock().unwrap();

        if input_connections.remove(name).is_some() {
            log_println!("[✅] Removed OSC Output device registration: '{}'", name);
            // Release lock before potentially calling another method
            drop(input_connections);
            // Unassign from any slot
            self.unassign_device_by_name(name);
            Ok(())
        } else {
            let err_msg = format!(
                "Cannot remove OSC device '{}': Not found or not an OSC device.",
                name
            );
            log_eprintln!("{}", err_msg);
            Err(err_msg)
        }
    }

    /// Removes an output device by its name.
    ///
    /// Removes the device registration from `output_connections`. The underlying socket
    /// will be closed when the `ProtocolDevice::OSCOutDevice` is dropped.
    /// Also unassigns the device from any slot it might occupy.
    ///
    /// # Arguments
    /// * `name` - The name of the OSC Output device to remove.
    ///
    /// # Returns
    /// - `Ok(())` on successful removal from registration.
    /// - `Err(String)` if no OSC Output device with the given name is found.
    pub fn remove_output_device(&self, name: &str) -> Result<(), String> {
        log_println!("[🗑️] Removing OSC Output device: '{}'", name);
        let mut output_connections = self.output_connections.lock().unwrap();

        if output_connections.remove(name).is_some() {
            log_println!("[✅] Removed OSC Output device registration: '{}'", name);
            // Release lock before potentially calling another method
            drop(output_connections);
            // Unassign from any slot
            self.unassign_device_by_name(name);
            Ok(())
        } else {
            let err_msg = format!(
                "Cannot remove OSC device '{}': Not found or not an OSC device.",
                name
            );
            log_eprintln!("{}", err_msg);
            Err(err_msg)
        }
    }

    pub fn connect_audio_engine(&self, name: &str, proxy: AudioEngineProxy) -> Result<(), String> {
        log_println!("[✨] Registering Audio Engine device: '{}'", name);
        let device = ProtocolDevice::AudioEngine(proxy);
        self.register_output_connection(name.to_owned(), device);
        log_println!(
            "[✅] Audio engine device '{}' registered successfully.",
            name
        );
        Ok(())
    }

    /// Creates a snapshot of all connected output devices for save/restore.
    ///
    /// Returns a Vec<DeviceInfo> containing virtual MIDI, physical MIDI, and OSC devices.
    /// Uses DeviceKind::VirtualMidi vs DeviceKind::Midi to distinguish virtual from physical.
    pub fn create_device_snapshot(&self) -> Vec<DeviceInfo> {
        let output_connections = self.output_connections.lock().unwrap();

        output_connections
            .iter()
            .filter_map(|(name, device_arc)| {
                Some(DeviceInfo {
                    slot_id: self.get_slot_for_name(name),
                    name: name.clone(),
                    kind: device_arc.kind(),
                    direction: DeviceDirection::Output,
                    is_connected: true,
                    address: Some(device_arc.address()),
                })
            })
            .collect()
    }

    /// Restores devices from a list of DeviceInfo.
    ///
    /// - Clears existing virtual MIDI and OSC devices (physical devices are left alone)
    /// - Recreates virtual MIDI devices (kind = VirtualMidi)
    /// - Recreates OSC devices (kind = Osc, parses ip:port from address)
    /// - Attempts to connect physical MIDI devices if present on system (kind = Midi)
    /// - Restores slot assignments from device.slot_id
    ///
    /// Returns a list of device names that couldn't be restored (missing physical devices).
    pub fn restore_from_snapshot(&self, devices: Vec<DeviceInfo>) -> Vec<String> {
        let mut missing = Vec::new();

        // Clear any previously tracked missing devices
        self.missing_devices.lock().unwrap().clear();

        // Get current system MIDI ports to check availability
        let mut system_midi_ports: BTreeSet<String> = BTreeSet::new();
        if let Some(midi_out_arc) = &self.midi_out {
            if let Ok(midi_out) = midi_out_arc.lock() {
                for port in midi_out.ports() {
                    if let Ok(name) = midi_out.port_name(&port) {
                        system_midi_ports.insert(name);
                    }
                }
            }
        }

        // Clear existing virtual MIDI and OSC devices
        {
            let mut output_connections = self.output_connections.lock().unwrap();
            let mut input_connections = self.input_connections.lock().unwrap();

            let names_to_remove: Vec<String> = output_connections
                .iter()
                .filter_map(|(name, device_arc)| match &**device_arc {
                    ProtocolDevice::VirtualMIDIOutDevice(_) => Some(name.clone()),
                    ProtocolDevice::OSCOutDevice(_) => Some(name.clone()),
                    _ => None,
                })
                .collect();

            for name in names_to_remove {
                output_connections.remove(&name);
                input_connections.remove(&name);
            }
        }

        {
            let mut assignments = self.slot_assignments.lock().unwrap();
            for slot in assignments.iter_mut() {
                *slot = None;
            }
        }

        // Recreate devices
        for device in devices {
            match device.kind {
                DeviceKind::VirtualMidi => {
                    if let Err(e) = self.create_virtual_midi_port(&device.name) {
                        log_eprintln!("Failed to restore virtual MIDI '{}': {}", device.name, e);
                        missing.push(device.name.clone());
                    }
                }
                DeviceKind::Osc => {
                    // Parse address "ip:port" format
                    if let Some((ip, port)) =
                        device.address.as_ref().and_then(|a| parse_socket_addr(a))
                    {
                        if let Err(e) = self.create_osc_output_device(&device.name, &ip, port) {
                            log_eprintln!("Failed to restore OSC device '{}': {}", device.name, e);
                            missing.push(device.name.clone());
                        }
                    } else {
                        log_eprintln!(
                            "Invalid OSC address for '{}': {:?}",
                            device.name,
                            device.address
                        );
                        missing.push(device.name.clone());
                    }
                }
                DeviceKind::Midi => {
                    // Physical MIDI - check if available on system
                    if system_midi_ports.contains(&device.name) {
                        let already_connected = self
                            .output_connections
                            .lock()
                            .unwrap()
                            .contains_key(&device.name);
                        if !already_connected {
                            if let Err(e) = self.connect_midi_by_name(&device.name) {
                                log_eprintln!(
                                    "Failed to restore physical MIDI '{}': {}",
                                    device.name,
                                    e
                                );
                                missing.push(device.name.clone());
                            }
                        }
                    } else {
                        // Physical device not available - store name for UI display
                        log_println!(
                            "Physical MIDI device '{}' not available on system",
                            device.name
                        );
                        missing.push(device.name.clone());
                        self.missing_devices
                            .lock()
                            .unwrap()
                            .insert(device.name.clone());
                    }
                }
                _ => {} // Skip Log, AudioEngine, Other
            }

            // Restore slot assignment
            if let Some(slot_id) = device.slot_id {
                if let Err(e) = self.assign_slot(slot_id, &device.name) {
                    log_eprintln!("Failed to restore slot {} assignment: {}", slot_id, e);
                }
            }
        }

        missing
    }

    /// Sends the MIDI "All Notes Off" message (Control Change 123, Value 0)
    /// to all connected MIDI output devices (physical and virtual) on all 16 channels.
    ///
    /// This is a utility function to stop hanging notes.
    pub fn panic_all_midi_outputs(&self) {
        log_println!("Sending MIDI Panic (All Notes Off CC 123) to all outputs...");
        let connections = self.output_connections.lock().unwrap();

        for (name, device_arc) in connections.iter() {
            // Target MIDIOutDevice (covers both physical and virtual)
            if let ProtocolDevice::MIDIOutDevice(midi_out) = &**device_arc {
                log_println!("Sending Panic to MIDI device: {}", name);
                for chan in 0..16 {
                    // Send on all 16 channels (0-15)
                    let msg = MIDIMessage {
                        payload: MIDIMessageType::ControlChange {
                            control: 123,
                            value: 0,
                        },
                        channel: chan,
                    };
                    // Attempt to send, log errors but continue
                    if let Err(e) = midi_out.send(msg) {
                        log_eprintln!(
                            "Error sending panic CC 123 chan {} to {}: {:?}",
                            chan,
                            name,
                            e
                        );
                    }
                }
            }
        }
        log_println!("MIDI Panic finished.");
    }
}

/// Parses a socket address string "ip:port" into (ip, port) tuple.
fn parse_socket_addr(addr: &str) -> Option<(String, u16)> {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() == 2 {
        parts[1]
            .parse()
            .ok()
            .map(|port| (parts[0].to_string(), port))
    } else {
        None
    }
}

impl Default for DeviceMap {
    fn default() -> Self {
        Self::new()
    }
}
