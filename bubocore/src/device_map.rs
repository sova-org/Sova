use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    clock::SyncTime,
    lang::event::ConcreteEvent,
    protocol::{
        log::{LogMessage, Severity},
        midi::{MIDIMessage, MIDIMessageType},
        ProtocolDevice, ProtocolMessage, TimedMessage,
    },
    shared_types::{DeviceInfo, DeviceKind},
};
use crate::protocol::midi::MidiOut;
use crate::protocol::midi::midi_constants::CONTROL_CHANGE_MSG;

use midir::{MidiInput, MidiOutput, Ignore, MidiOutputPort};
// Import the necessary trait for create_virtual (on Unix-like systems)
#[cfg(target_family = "unix")] 
use midir::os::unix::VirtualOutput;

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

    pub fn register_input_connection(&self, name: String, device: ProtocolDevice) {
        let address = device.address().to_owned();
        let item = (name, Arc::new(device));
        self.input_connections.lock().unwrap().insert(address, item);
    }

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
        date: SyncTime,
        device: Arc<ProtocolDevice>,
    ) -> Vec<TimedMessage> {
        vec![ProtocolMessage {
            payload: LogMessage {
                level: Severity::Info,
                msg: format!("{:?}", payload),
            }
            .into(),
            device: Arc::clone(&device),
        }
        .timed(date)]
    }

    /// Maps a ConcreteEvent to ProtocolMessages for a target device specified by NAME.
    /// The resolution of Slot ID -> Name must happen *before* calling this.
    pub fn map_event_for_device_name(
        &self,
        target_device_name: &str,
        event: ConcreteEvent, // Event now contains slot_id, but we ignore it here
        date: SyncTime,
    ) -> Vec<TimedMessage> {
        
        // Find the ProtocolDevice using the name.
        // Search in output_connections first.
        let device_opt = self.output_connections.lock().unwrap().values()
            .find(|(name, _)| name == target_device_name)
            .map(|(_, device_arc)| Arc::clone(device_arc));
            
        // TODO: Also check input_connections if necessary?

        let Some(device) = device_opt else {
            // Log error if device name is not found in active connections
            return vec![ProtocolMessage {
                payload: LogMessage {
                    level: Severity::Error,
                    msg: format!("Device name '{}' not found or not connected.", target_device_name),
                }
                .into(),
                // Need a default log device reference if not using the map for it
                device: Arc::new(ProtocolDevice::Log),
            }
            .timed(date)];
        };

        // Dispatch based on the *type* of the found device Arc
        match &*device {
            ProtocolDevice::OSCOutDevice => todo!("OSC output not implemented in map_event"),
            ProtocolDevice::MIDIOutDevice(_) | ProtocolDevice::VirtualMIDIOutDevice {..} => {
                self.generate_midi_message(event, date, device)
            }
            ProtocolDevice::Log => { // Handle explicit logging if needed
                self.generate_log_message(event, date, device)
            }
            _ => {
                eprintln!("[!] map_event_for_device_name: Unhandled ProtocolDevice type for {}", target_device_name);
                 vec![]
            }
        }
    }

    pub fn device_list(&self) -> Vec<DeviceInfo> {
        println!("[~] Generating device list with slot assignments...");
        let mut discovered_devices_map: HashMap<String, DeviceInfo> = HashMap::new();
        let slot_map = self.slot_assignments.lock().unwrap();
        let connected_map = self.output_connections.lock().unwrap(); // Lock once

        // Helper to create DeviceInfo, checking slot assignment and connection status
        let create_device_info = |name: String, kind: DeviceKind| -> DeviceInfo {
            let assigned_slot_id = slot_map.iter()
                .find_map(|(slot, assigned_name)| if assigned_name == &name { Some(*slot) } else { None })
                .unwrap_or(0); // 0 if not assigned
                
            // Check connection status based on the locked connected_map
            let is_connected = connected_map.values().any(|(conn_name, _)| conn_name == &name);
            
            DeviceInfo {
                id: assigned_slot_id, // Slot ID (0 if unassigned)
                name,
                kind,
                is_connected,
            }
        };

        // --- Discover system MIDI ports (Out) ---
        if let Some(midi_out_arc) = &self.midi_out {
            if let Ok(midi_out) = midi_out_arc.lock() {
                for port in midi_out.ports() {
                    if let Ok(name) = midi_out.port_name(&port) {
                        // Only add if not already processed (e.g., virtual port already added)
                        if !discovered_devices_map.contains_key(&name) {
                             discovered_devices_map.insert(name.clone(), create_device_info(name, DeviceKind::Midi));
                        }
                    }
                }
            } // midi_out lock dropped here
        }

        // --- Discover system MIDI ports (In) --- 
        if let Some(midi_in_arc) = &self.midi_in {
            if let Ok(midi_in) = midi_in_arc.lock() {
                for port in midi_in.ports() {
                     if let Ok(name) = midi_in.port_name(&port) {
                         if !discovered_devices_map.contains_key(&name) {
                              discovered_devices_map.insert(name.clone(), create_device_info(name, DeviceKind::Midi));
                         }
                     }
                }
            } // midi_in lock dropped here
        }
        
        // --- Add currently connected devices that might not have been discovered (e.g., just created virtual) ---
        // Clone names *while holding the lock* to avoid borrow error
        let connected_names: Vec<String> = connected_map.values().map(|(name, _)| name.clone()).collect();
        // Lock is dropped here automatically when connected_map goes out of scope
        // drop(connected_map); // No longer needed explicitly
        
        for name in connected_names {
             if !discovered_devices_map.contains_key(&name) {
                 // Assume MIDI for now if we only know it from connections
                 // Might need more info in ProtocolDevice later
                 discovered_devices_map.insert(name.clone(), create_device_info(name, DeviceKind::Midi));
             }
        }

        // Don't add Log device to the user list unless explicitly assigned to a slot
        // Add OSC devices later if needed

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

    /// Connects to a MIDI output device by NAME.
    pub fn connect_midi_output_by_name(&self, device_name: &str) -> Result<(), String> {
        println!("[🔌] Attempting to connect MIDI Output device: {}", device_name);

        // Check if already connected (i.e., in output_connections)
        if self.output_connections.lock().unwrap().values().any(|(name, _)| name == device_name) {
             return Err(format!("Device '{}' is already connected.", device_name));
        }
        
        // Find the midir port using a temporary instance
        let temp_midi_out = MidiOutput::new(&format!("BuboCore-Temp-Connector-{}", device_name))
            .map_err(|e| format!("Failed to create temporary MidiOutput: {}", e))?;
        let port_opt: Option<MidiOutputPort> = temp_midi_out.ports().into_iter().find(|p| {
            temp_midi_out.port_name(p).map_or(false, |name| name == device_name)
        });
        let port = port_opt.ok_or_else(|| format!("MIDI Output port '{}' not found by midir.", device_name))?;

        match temp_midi_out.connect(&port, &format!("BuboCore-Connection-{}", device_name)) {
            Ok(connection) => {
                println!("[✅] Successfully connected to MIDI Output: {}", device_name);
                let midi_out_handler = MidiOut {
                    name: device_name.to_string(),
                    active_notes: Default::default(),
                    connection: Arc::new(Mutex::new(Some(connection))),
                };
                let device = ProtocolDevice::MIDIOutDevice(Arc::new(Mutex::new(midi_out_handler)));
                self.register_output_connection(device_name.to_string(), device);
                Ok(())
            },
            Err(e) => {
                eprintln!("[!] Failed to connect MIDI Output '{}': {}", device_name, e);
                Err(format!("Failed to connect MIDI Output '{}': {}", device_name, e))
            }
        }
    }

    /// Disconnects a MIDI output device by NAME.
    pub fn disconnect_midi_output_by_name(&self, device_name: &str) -> Result<(), String> {
         println!("[🔌] Attempting to disconnect MIDI Output device: {}", device_name);
        let mut connections = self.output_connections.lock().unwrap();
        let key_to_remove = connections.iter()
             .find(|(_address, (name, _device))| name == device_name)
            .map(|(address, _item)| address.clone());

        match key_to_remove {
            Some(key) => {
                if connections.remove(&key).is_some() {
                     println!("[✅] Disconnected and removed registration for MIDI Output '{}'", device_name);
                     // Also unassign from any slot it might be in
                     drop(connections); // Release lock before calling another method
                     self.unassign_device_by_name(device_name);
                    Ok(())
                } else {
                      eprintln!("[!] Failed to remove connection for key '{}' (name: '{}').", key, device_name);
                     Err(format!("Internal error removing connection for {}", device_name))
                }
            }
            None => {
                 eprintln!("[!] Cannot disconnect MIDI Output '{}': Not connected.", device_name);
                 Err(format!("Device '{}' not connected.", device_name))
             }
         }
    }

    /// Creates a virtual MIDI output port and returns its name on success.
    /// Does NOT assign it to a slot automatically.
    pub fn create_virtual_midi_output(&self, desired_name: &str) -> Result<String, String> {
        println!("[✨] Creating virtual MIDI port: '{}'", desired_name);
        
        // Check if name is already used by any known device (system or virtual)
        let known_devices = self.device_list(); // Get current list including assignments
        if known_devices.iter().any(|d| d.name == desired_name) {
             return Err(format!("Device name '{}' already exists.", desired_name));
        }

        // Use temporary MidiOutput to create the port
        let temp_midi_out = MidiOutput::new(&format!("BuboCore-Virtual-Creator-{}", desired_name))
            .map_err(|e| format!("Failed to create temporary MidiOutput: {}", e))?;

        match temp_midi_out.create_virtual(desired_name) {
            Ok(connection) => {
                println!("[✅] Virtual MIDI port created: '{}'", desired_name);
                let virtual_device = ProtocolDevice::VirtualMIDIOutDevice {
                    name: desired_name.to_string(),
                    connection: Arc::new(Mutex::new(Some(connection))),
                };
                // Register the connection so it can be found immediately
                self.register_output_connection(desired_name.to_string(), virtual_device);
                println!("[✅] Virtual MIDI port '{}' registered.", desired_name);
                Ok(desired_name.to_string()) // Return the name
            }
            Err(e) => {
                eprintln!("[!] Failed to create virtual MIDI port '{}': {}", desired_name, e);
                Err(format!("Failed to create virtual MIDI port '{}': {}", desired_name, e))
            }
        }
    }

    /// Sends the "All Notes Off" message (CC 123) to all connected MIDI outputs on all channels.
    pub fn panic_all_midi_outputs(&self) {
        println!("[!] Sending MIDI Panic (All Notes Off CC 123) to all outputs...");
        let connections = self.output_connections.lock().unwrap();

        for (_device_addr, (name, device_arc)) in connections.iter() {
            match &**device_arc {
                ProtocolDevice::MIDIOutDevice(midi_out_mutex) => {
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
                         eprintln!("[!] Could not lock Mutex for MIDI device: {}", name);
                    }
                }
                ProtocolDevice::VirtualMIDIOutDevice { name: virtual_name, connection: virtual_conn_mutex } => {
                     println!("[!] Sending Panic to Virtual MIDI device: {}", virtual_name);
                     if let Ok(mut conn_opt_guard) = virtual_conn_mutex.lock() {
                         if let Some(conn) = conn_opt_guard.as_mut() {
                             for chan in 0..16 {
                                 let bytes = vec![CONTROL_CHANGE_MSG + chan, 123, 0];
                                 if let Err(e) = conn.send(&bytes) {
                                     eprintln!("[!] Error sending panic to {}: {:?}", virtual_name, e);
                                 }
                             }
                         } else {
                             eprintln!("[!] Virtual MIDI device {} is not connected.", virtual_name);
                         }
                     } else {
                         eprintln!("[!] Could not lock Mutex for Virtual MIDI device: {}", virtual_name);
                     }
                }
                // Ignore non-MIDI output devices
                _ => {}
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
