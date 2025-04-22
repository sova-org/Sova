use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    net::{SocketAddr, IpAddr},
    str::FromStr,
};

use crate::{
    clock::SyncTime,
    lang::event::ConcreteEvent,
    protocol::{
        log::{LogMessage, Severity, LOG_NAME},
        midi::{MIDIMessage, MIDIMessageType, MidiIn, MidiOut, MidiInterface},
        osc::{OSCMessage, Argument as OscArgument},
        ProtocolDevice, ProtocolMessage, TimedMessage,
    },
    shared_types::{DeviceInfo, DeviceKind},
};

use midir::{MidiInput, MidiOutput, Ignore};
pub type DeviceItem = (String, Arc<ProtocolDevice>);

// Maximum number of user-assignable slots
const MAX_DEVICE_SLOTS: usize = 16; // Example: 16 slots

pub struct DeviceMap {
    pub input_connections: Mutex<HashMap<String, DeviceItem>>,
    pub output_connections: Mutex<HashMap<String, DeviceItem>>,
    /// Maps Slot ID (1-N) to the system device name assigned to it.
    pub slot_assignments: Mutex<HashMap<usize, String>>, 
    midi_in: Option<Arc<Mutex<MidiInput>>>,
    midi_out: Option<Arc<Mutex<MidiOutput>>>,
}

impl DeviceMap {
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

        let devices = DeviceMap {
            input_connections: Default::default(),
            output_connections: Default::default(),
            slot_assignments: Default::default(), // Initialize empty slot map
            midi_in,
            midi_out,
        };
        
        // Register Log device directly without assigning a slot ID
        // Its name "log" will be its identifier in the connections map.
        // devices.register_output_connection(LOG_NAME.to_owned(), ProtocolDevice::Log);
        // Let's not register log here, handle log messages specifically if needed
        devices
    }

    /// Registers a connected input device.
    pub fn register_input_connection(&self, name: String, device: ProtocolDevice) {
        let address = device.address().to_owned();
        let item = (name, Arc::new(device));
        self.input_connections.lock().unwrap().insert(address, item);
    }

    /// Registers a connected output device.
    pub fn register_output_connection(&self, name: String, device: ProtocolDevice) {
        // Just register the connection by name (address)
        // Slot assignment is separate
        let address = device.address().to_owned();
        let item = (name, Arc::new(device));
        self.output_connections
            .lock()
            .unwrap()
            .insert(address, item);
    }

    /// Assigns a device (by name) to a specific slot ID (1-N).
    /// Removes any previous assignment for this slot or this device name.
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

    /// Finds the device name assigned to a specific slot ID.
    pub fn get_name_for_slot(&self, slot_id: usize) -> Option<String> {
        self.slot_assignments.lock().unwrap().get(&slot_id).cloned()
    }

    /// Finds the slot ID assigned to a specific device name.
    pub fn get_slot_for_name(&self, device_name: &str) -> Option<usize> {
         self.slot_assignments.lock().unwrap().iter()
            .find_map(|(slot, name)| if name == device_name { Some(*slot) } else { None })
    }

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
                            channel: midi_chan, // Use converted channel
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
                            channel: midi_chan, // Use converted channel
                        }
                        .into(),
                        device: Arc::clone(&device),
                    }
                    .timed(date + dur),
                ]
            }
            ConcreteEvent::MidiControl(control, value, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8; // Convert to 0-based MIDI channel
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::ControlChange {
                            control: control as u8,
                            value: value as u8,
                        },
                        channel: midi_chan, // Use converted channel
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiProgram(program, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8; // Convert to 0-based MIDI channel
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::ProgramChange {
                            program: program as u8,
                        },
                        channel: midi_chan, // Use converted channel
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiAftertouch(note, pressure, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8; // Convert to 0-based MIDI channel
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::Aftertouch {
                            note: note as u8,
                            value: pressure as u8,
                        },
                        channel: midi_chan, // Use converted channel
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiChannelPressure(pressure, chan, _device_id) => { // Renamed channel to chan for consistency
                let midi_chan = (chan.saturating_sub(1) % 16) as u8; // Convert to 0-based MIDI channel
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::ChannelPressure {
                            value: pressure as u8,
                        },
                        channel: midi_chan, // Use converted channel
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            // System messages (Start, Stop, Continue, Clock, Reset, Sysex) typically don't use a channel,
            // so no conversion needed here. Keep channel 0 as specified in the original code.
            ConcreteEvent::MidiStart(_device_id) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::Start {},
                        channel: 0, 
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
            _ => Vec::new(), // Handle Nop or other non-MIDI events
        }
    }

    fn generate_log_message(
        &self,
        payload: ConcreteEvent,
        _date: SyncTime, // Date not used directly for formatting anymore
        device: Arc<ProtocolDevice>,
    ) -> Vec<TimedMessage> {
        vec![ProtocolMessage {
            // Use the new constructor to store the event directly
            payload: LogMessage::from_event(Severity::Info, payload).into(), 
            device: Arc::clone(&device),
        }
        // .timed() needs date, maybe get it from the message later?
        // For now, let's assume the caller provides the correct time
        // If date is needed for formatting here, we need clock access.
         .timed(_date)] // Still need to time the message wrapper
    }

    /// Maps a ConcreteEvent to ProtocolMessages for a target device specified by NAME.
    /// The resolution of Slot ID -> Name must happen *before* calling this.
    pub fn map_event_for_device_name(
        &self,
        target_device_name: &str,
        event: ConcreteEvent, // Event now contains slot_id, but we ignore it here
        date: SyncTime,
    ) -> Vec<TimedMessage> {

        // --- Handle Log Device Implicitly FIRST ---
        if target_device_name == LOG_NAME {
            // If the target is "log", generate the log message directly.
            // generate_log_message now stores the event.
            return self.generate_log_message(event, date, Arc::new(ProtocolDevice::Log));
        }
        
        // --- Handle other devices via lookup --- (Existing logic)
        let device_opt = self.output_connections.lock().unwrap().values()
            .find(|(name, _)| name == target_device_name)
            .map(|(_, device_arc)| Arc::clone(device_arc));
            
        // TODO: Also check input_connections if necessary?

        let Some(device) = device_opt else {
            // Log error IF THE DEVICE NAME WAS *NOT* "log" and wasn't found
            // Use LogMessage::error which sets event = None
            return vec![ProtocolMessage {
                payload: LogMessage::error(
                    format!("Device name '{}' not found or not connected.", target_device_name)
                ).into(),
                device: Arc::new(ProtocolDevice::Log), // Send error to the log device
            }
            .timed(date)];
        };

        // --- Dispatch for FOUND devices (MIDI, OSC, etc.) --- (Existing logic)
        match &*device {
            ProtocolDevice::OSCOutputDevice {..} => {
                // Map ConcreteEvent to OSCMessage
                let osc_payload_opt = match event {
                    // --- Handle Generic OSC Event --- 
                    ConcreteEvent::Osc { message, device_id: _ } => {
                         // The message is already constructed, just pass it through
                         Some(message)
                     }
                    // --- Handle Dirt Event --- 
                    ConcreteEvent::Dirt { data, device_id: _ } => {
                        // Construct args: [key1, val1, key2, val2, ...]
                        let mut args: Vec<OscArgument> = Vec::with_capacity(data.len() * 2);
                        for (key, value) in data {
                            args.push(OscArgument::String(key));
                            args.push(value); // Value is already OscArgument
                        }
                        Some(OSCMessage {
                            addr: "/dirt/play".to_string(), // Standard SuperDirt address
                            args,
                        })
                    }
                    // --- Handle other generic OSC mappings (from previous step) ---
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
                    // Add mappings for other relevant events here
                    // e.g., PitchBend, Aftertouch, maybe custom events?
                    _ => None, // Ignore other events for OSC for now
                };

                // If a mapping exists, create the TimedMessage
                if let Some(osc_payload) = osc_payload_opt {
                    vec![ProtocolMessage {
                        payload: osc_payload.into(), // Convert OSCMessage to ProtocolPayload::OSC
                        device: Arc::clone(&device),
                    }
                    .timed(date)]
                } else {
                    // No mapping for this event type to OSC
                    vec![]
                }
            },
            ProtocolDevice::MIDIOutDevice(_) | ProtocolDevice::VirtualMIDIOutDevice {..} => {
                self.generate_midi_message(event, date, device)
            }
            ProtocolDevice::Log => { 
                // This case should be unreachable now due to the initial check,
                // but kept defensively. generate_log_message handles it.
                self.generate_log_message(event, date, device)
            }
            _ => {
                eprintln!("[!] map_event_for_device_name: Unhandled ProtocolDevice type for {}", target_device_name);
                 vec![] // Or generate an error log message
            }
        }
    }

    /// Maps a ConcreteEvent to ProtocolMessages for a target device specified by SLOT ID.
    /// Slot 0 is implicitly mapped to the internal Log device.
    /// Slots 1-N are mapped using the current slot assignments.
    pub fn map_event_for_slot_id(
        &self,
        target_slot_id: usize,
        event: ConcreteEvent,
        date: SyncTime,
    ) -> Vec<TimedMessage> {
        if target_slot_id == 0 {
            // Slot 0 always targets the Log device
            // Directly use the name-based function which handles the implicit log
            self.map_event_for_device_name(LOG_NAME, event, date)
        } else {
            // Look up the device name assigned to the slot ID (1-N)
            let device_name_opt = self.get_name_for_slot(target_slot_id);

            match device_name_opt {
                Some(device_name) => {
                    // Found an assigned device, use the name-based mapping
                    self.map_event_for_device_name(&device_name, event, date)
                }
                None => {
                    // Slot is not assigned, generate a warning log message
                    // Include the original event in the LogMessage for context
                    vec![ProtocolMessage {
                        payload: LogMessage {
                            level: Severity::Warn, 
                            event: Some(event), // Store the event that failed to map
                            msg: format!("Slot {} is not assigned", target_slot_id), // Simple message
                        }
                        .into(),
                        // Use the implicit log device for the error message
                        device: Arc::new(ProtocolDevice::Log),
                    }
                    .timed(date)]
                }
            }
        }
    }

    pub fn device_list(&self) -> Vec<DeviceInfo> {
        println!("[~] Generating device list (excluding implicit log)...");
        let mut discovered_devices_map: HashMap<String, DeviceInfo> = HashMap::new();
        let slot_map = self.slot_assignments.lock().unwrap();
        let connected_map = self.output_connections.lock().unwrap(); // Lock output connections once

        // Helper to create DeviceInfo, checking slot assignment and connection status
        // Updated to handle address for OSC devices
        let create_device_info = |name: String, kind: DeviceKind, device_ref_opt: Option<&ProtocolDevice>| -> DeviceInfo {
            let assigned_slot_id = slot_map.iter()
                .find_map(|(slot, assigned_name)| if assigned_name == &name { Some(*slot) } else { None })
                .unwrap_or(0); // 0 if not assigned

            // Check connection status based on the locked connected_map
            // For OSC, connection is always true if it exists in the map
            let is_connected = match kind {
                 DeviceKind::Osc => device_ref_opt.is_some(), // Connected if found
                 DeviceKind::Midi => {
                      // Check actual MIDI connection state if possible, or just presence in map
                      // For simplicity, let's check presence for now.
                      connected_map.values().any(|(conn_name, _)| conn_name == &name)
                 }
                _ => false, // Log, Other are not 'connected' in the same way
            };
            
            // Extract address for OSC devices
            let address = if kind == DeviceKind::Osc {
                 device_ref_opt.and_then(|device| match device {
                     ProtocolDevice::OSCOutputDevice { address, .. } => Some(address.to_string()),
                     _ => None, // Should not happen if kind is Osc
                 })
            } else {
                 None
            };

            DeviceInfo {
                id: assigned_slot_id,
                name,
                kind,
                is_connected,
                address, // Add the address field
            }
        };

        // --- Discover system MIDI ports (Out) ---
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

        // --- Discover system MIDI ports (In) --- 
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
        
        // --- Add currently connected devices (MIDI & OSC) from output_connections --- 
        // Iterate through the locked connected_map directly
        for (device_addr, (name, device_arc)) in connected_map.iter() {
            if !discovered_devices_map.contains_key(name) {
                 // Determine kind and pass device reference to helper
                 let (kind, address_str) = match &**device_arc {
                    ProtocolDevice::MIDIOutDevice { .. } => (DeviceKind::Midi, None), // MIDI device
                     ProtocolDevice::VirtualMIDIOutDevice { .. } => (DeviceKind::Midi, None), // Also MIDI
                     ProtocolDevice::OSCOutputDevice { address, .. } => (DeviceKind::Osc, Some(address.to_string())), // OSC device
                     _ => (DeviceKind::Other, None), // Skip Log, In, etc.
                 };
                 
                 // Only add MIDI or OSC types found here
                 if kind == DeviceKind::Midi || kind == DeviceKind::Osc {
                     // Pass the actual device reference to create_device_info
                     discovered_devices_map.insert(name.clone(), create_device_info(name.clone(), kind, Some(&**device_arc)));
                 }
             }
             // If already discovered (e.g., MIDI system port), update its info if needed?
             // For now, discovery takes precedence.
        }
        // Drop the lock explicitly after use if needed, though it drops at end of scope
        drop(connected_map);

        // Don't add Log device to the user list unless explicitly assigned to a slot

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

    /// Connects to a MIDI device by NAME (bidirectional).
    pub fn connect_midi_by_name(&self, device_name: &str) -> Result<(), String> {
        println!("[🔌] Attempting to connect MIDI device (In/Out): {}", device_name);

        // Check if already connected (i.e., in output_connections)
        if self.output_connections.lock().unwrap().values().any(|(name, _)| name == device_name) {
             return Err(format!("Device '{}' is already connected.", device_name));
        }
        
        // Create MidiIn and MidiOut handlers first
        let mut midi_in_handler = MidiIn::new(device_name.to_string())
            .map_err(|e| format!("Failed to create MidiIn handler: {:?}", e))?;
        let mut midi_out_handler = MidiOut::new(device_name.to_string())
            .map_err(|e| format!("Failed to create MidiOut handler: {:?}", e))?;

        // Attempt to connect both
        match midi_in_handler.connect_to_port_by_name(device_name) {
            Ok(_) => {
                 println!("[✅] Connected MIDI Input: {}", device_name);
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
                         // Output failed, Input succeeded but we need full bidirectionality
                         eprintln!("[!] Failed to connect MIDI Output '{}' after Input succeeded: {:?}", device_name, e);
                         // Input handler will be dropped automatically, disconnecting it.
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

    /// Disconnects a MIDI device by NAME (bidirectional).
    pub fn disconnect_midi_by_name(&self, device_name: &str) -> Result<(), String> {
         println!("[🔌] Attempting to disconnect MIDI device (In/Out): {}", device_name);
         let mut output_connections = self.output_connections.lock().unwrap();
         let mut input_connections = self.input_connections.lock().unwrap();

         let output_key_to_remove = output_connections.iter()
             .find(|(_address, (name, _device))| name == device_name)
             .map(|(address, _item)| address.clone());

         let input_key_to_remove = input_connections.iter()
             .find(|(_address, (name, _device))| name == device_name)
             .map(|(address, _item)| address.clone());

        // We expect both to be present if connected
        match (output_key_to_remove, input_key_to_remove) {
            (Some(out_key), Some(in_key)) => {
                let out_removed = output_connections.remove(&out_key).is_some();
                let in_removed = input_connections.remove(&in_key).is_some();

                if out_removed && in_removed {
                     println!("[✅] Disconnected and removed registration for MIDI In/Out '{}'", device_name);
                     // Also unassign from any slot it might be in
                     drop(output_connections); // Release locks before calling another method
                     drop(input_connections);
                     self.unassign_device_by_name(device_name);
                    Ok(())
                } else {
                      // This case should ideally not happen if connect logic is sound
                      eprintln!("[!] Mismatch removing connections for '{}'. Out removed: {}, In removed: {}", device_name, out_removed, in_removed);
                     Err(format!("Internal error removing connections for {}", device_name))
                }
            }
            (None, None) => {
                 eprintln!("[!] Cannot disconnect MIDI device '{}': Not connected.", device_name);
                 Err(format!("Device '{}' not connected.", device_name))
             }
             _ => {
                 // One exists but not the other - indicates an inconsistent state
                  eprintln!("[!] Cannot disconnect MIDI device '{}': Inconsistent connection state (In/Out mismatch).", device_name);
                  Err(format!("Device '{}' has inconsistent connection state.", device_name))
             }
         }
    }

    /// Creates a virtual MIDI port (In/Out) and returns its name on success.
    /// Does NOT assign it to a slot automatically.
    pub fn create_virtual_midi_port(&self, desired_name: &str) -> Result<String, String> {
        println!("[✨] Creating virtual MIDI port (In/Out): '{}'", desired_name);

        // Check if name is already used by any known device (system or virtual)
        // Use device_list for a consolidated check
        let known_devices = self.device_list();
        if known_devices.iter().any(|d| d.name == desired_name) {
            return Err(format!("Device name '{}' already exists.", desired_name));
        }

        // Create handlers first
        let mut midi_in_handler = MidiIn::new(desired_name.to_string())
            .map_err(|e| format!("Failed to create MidiIn handler for virtual port: {:?}", e))?;
        let mut midi_out_handler = MidiOut::new(desired_name.to_string())
            .map_err(|e| format!("Failed to create MidiOut handler for virtual port: {:?}", e))?;

        // Attempt to create virtual output first
        match midi_out_handler.create_virtual_port() {
            Ok(_) => {
                 println!("[✅] Virtual MIDI Output source created: '{}'", desired_name);

                // Now explicitly create the virtual input destination
                match midi_in_handler.create_virtual_port() {
                    Ok(_) => {
                        println!("[✅] Virtual MIDI Input destination created: '{}'", desired_name);

                        // Both virtual endpoints created successfully, register them
                         let in_device = ProtocolDevice::MIDIInDevice(Arc::new(Mutex::new(midi_in_handler)));
                         // Use MIDIOutDevice for consistency, even for virtual ports
                         let out_device = ProtocolDevice::MIDIOutDevice(Arc::new(Mutex::new(midi_out_handler)));
                         self.register_input_connection(desired_name.to_string(), in_device);
                         self.register_output_connection(desired_name.to_string(), out_device);
                         println!("[✅] Registered virtual MIDI port pair: '{}'", desired_name);
                         Ok(desired_name.to_string())
                    }
                    Err(e) => {
                        // Virtual Input creation failed after Output creation succeeded
                        eprintln!("[!] Failed to create Virtual MIDI Input destination '{}' after Output source creation: {:?}", desired_name, e);
                        // Output handler will be dropped automatically, disconnecting it.
                        Err(format!("Failed to create Virtual MIDI Input destination '{}': {:?}", desired_name, e))
                    }
                }
            }
            Err(e) => {
                // Virtual output source creation failed
                eprintln!("[!] Failed to create Virtual MIDI Output source '{}': {:?}", desired_name, e);
                Err(format!("Failed to create Virtual MIDI Output source '{}': {:?}", desired_name, e))
            }
        }
    }

    /// Creates and registers a new OSC Output device.
    pub fn create_osc_output_device(&self, name: &str, ip_str: &str, port: u16) -> Result<(), String> {
        println!("[✨] Creating OSC Output device: '{}' @ {}:{}", name, ip_str, port);

        // 1. Parse target IP and create SocketAddr *first*
        let target_ip_addr = IpAddr::from_str(ip_str)
            .map_err(|e| format!("Invalid IP address format '{}': {}", ip_str, e))?;
        let target_socket_addr = SocketAddr::new(target_ip_addr, port);

        // 2. Check for existing name or address collision within the lock
        let output_connections = self.output_connections.lock().unwrap();
        for (existing_name, device_arc) in output_connections.values() {
            // Check for name collision
            if existing_name == name {
                let err_msg = format!("Cannot create OSC device: Name '{}' already exists.", name);
                eprintln!("[!]	{}", err_msg);
                return Err(err_msg);
            }
            // Check for address collision *specifically for OSC outputs*
            if let ProtocolDevice::OSCOutputDevice { address: existing_addr, .. } = &**device_arc {
                if *existing_addr == target_socket_addr {
                     let err_msg = format!("Cannot create OSC device '{}': Another OSC device already targets address '{}'.", name, target_socket_addr);
                     eprintln!("[!]	{}", err_msg);
                    return Err(err_msg);
                }
            }
        }
        drop(output_connections); // Release lock

        // 3. Create the OSCOutputDevice instance (address already parsed)
        let mut osc_device = ProtocolDevice::OSCOutputDevice {
            name: name.to_string(),
            address: target_socket_addr, // Use the already parsed address
            socket: None, 
        };

        // 4. Attempt to connect (create the socket)
        match osc_device.connect() {
            Ok(_) => {
                println!("[✅] OSC Output device '{}' socket created successfully.", name);
                // 5. Register the connected device
                self.register_output_connection(name.to_string(), osc_device);
                println!("[✅] Registered OSC Output device: '{}'", name);
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Failed to connect/bind socket for OSC device '{}': {:?}", name, e);
                 eprintln!("[!]	{}", err_msg);
                 Err(err_msg)
            }
        }
    }

    /// Removes an OSC Output device by its name.
    pub fn remove_osc_output_device(&self, name: &str) -> Result<(), String> {
        println!("[🗑️] Removing OSC Output device: '{}'", name);
        let mut output_connections = self.output_connections.lock().unwrap();
        
        let key_to_remove = output_connections.iter()
            .find(|(_address, (n, device))| n == name && matches!(**device, ProtocolDevice::OSCOutputDevice{..}))
            .map(|(address, _item)| address.clone());
            
        match key_to_remove {
            Some(key) => {
                if output_connections.remove(&key).is_some() {
                    println!("[✅] Removed OSC Output device registration: '{}'", name);
                    drop(output_connections); // Release lock before calling other method
                    self.unassign_device_by_name(name); // Unassign from any slot
                    Ok(())
                } else {
                    // Should not happen if key was found
                    let err_msg = format!("Internal error removing OSC device '{}'", name);
                    eprintln!("[!]	{}", err_msg);
                    Err(err_msg)
                }
            }
            None => {
                let err_msg = format!("Cannot remove OSC device '{}': Not found or not an OSC device.", name);
                eprintln!("[!]	{}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Sends the "All Notes Off" message (CC 123) to all connected MIDI outputs on all channels.
    pub fn panic_all_midi_outputs(&self) {
        println!("[!] Sending MIDI Panic (All Notes Off CC 123) to all outputs...");
        let connections = self.output_connections.lock().unwrap();

        for (_device_addr, (name, device_arc)) in connections.iter() {
            // Only target MIDIOutDevice (covers both physical and virtual now)
            if let ProtocolDevice::MIDIOutDevice(midi_out_mutex) = &**device_arc {
                println!("[!] Sending Panic to MIDI device: {}", name);
                if let Ok(midi_out) = midi_out_mutex.lock() {
                    for chan in 0..16 {
                        let msg = MIDIMessage {
                            payload: MIDIMessageType::ControlChange { control: 123, value: 0 },
                            channel: chan,
                        };
                        if let Err(e) = midi_out.send(msg) {
                            eprintln!("[!] Error sending panic to {}: {:?}", name, e);
                        }
                    }
                } else {
                     println!("[!] Could not lock Mutex for MIDI device: {}", name);
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
