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

use midir::{MidiInput, MidiOutput, Ignore, MidiOutputPort};
// Import the necessary trait for create_virtual (on Unix-like systems)
#[cfg(target_family = "unix")] 
use midir::os::unix::VirtualOutput;

pub type DeviceItem = (String, Arc<ProtocolDevice>);

pub struct DeviceMap {
    pub input_connections: Mutex<HashMap<String, DeviceItem>>,
    pub output_connections: Mutex<HashMap<String, DeviceItem>>,
    midi_in: Option<Arc<Mutex<MidiInput>>>,
    midi_out: Option<Arc<Mutex<MidiOutput>>>,
    // For assigning stable IDs
    next_device_id: Mutex<usize>,
    device_id_map: Mutex<HashMap<usize, String>>, // Maps ID -> Name
    device_name_to_id_map: Mutex<HashMap<String, usize>>, // Maps Name -> ID 
}

pub const LOG_NAME: &str = "log";
const LOG_DEVICE_ID: usize = 0; // Assign a fixed ID for the log device

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
            midi_in,
            midi_out,
            next_device_id: Mutex::new(LOG_DEVICE_ID + 1), // Start IDs after LOG
            device_id_map: Mutex::new(HashMap::new()),
            device_name_to_id_map: Mutex::new(HashMap::new()),
        };
        // Register Log device with its fixed ID
        let mut id_map = devices.device_id_map.lock().unwrap();
        let mut name_map = devices.device_name_to_id_map.lock().unwrap();
        id_map.insert(LOG_DEVICE_ID, LOG_NAME.to_string());
        name_map.insert(LOG_NAME.to_string(), LOG_DEVICE_ID);
        drop(id_map);
        drop(name_map);
        devices.register_output_connection(LOG_NAME.to_owned(), ProtocolDevice::Log);
        devices
    }

    /// Assigns a stable ID to a device name if it doesn't already have one.
    fn ensure_device_id(&self, name: &str) -> usize {
        let mut name_map = self.device_name_to_id_map.lock().unwrap();
        if let Some(id) = name_map.get(name) {
            return *id;
        }
        // Name not found, assign a new ID
        let mut next_id_guard = self.next_device_id.lock().unwrap();
        let new_id = *next_id_guard;
        *next_id_guard += 1;
        drop(next_id_guard);

        name_map.insert(name.to_string(), new_id);
        drop(name_map);

        let mut id_map = self.device_id_map.lock().unwrap();
        id_map.insert(new_id, name.to_string());
        drop(id_map);

        println!("[~] Assigned new device ID {} to '{}'", new_id, name);
        new_id
    }

    /// Gets the name associated with a device ID.
    fn get_device_name_by_id(&self, id: usize) -> Option<String> {
        self.device_id_map.lock().unwrap().get(&id).cloned()
    }

    pub fn register_input_connection(&self, name: String, device: ProtocolDevice) {
        let address = device.address().to_owned();
        let item = (name, Arc::new(device));
        self.input_connections.lock().unwrap().insert(address, item);
    }

    pub fn register_output_connection(&self, name: String, device: ProtocolDevice) {
        // Ensure the device has an ID when registered
        self.ensure_device_id(&name);
        let address = device.address().to_owned();
        let item = (name, Arc::new(device));
        self.output_connections
            .lock()
            .unwrap()
            .insert(address, item);
    }

    /// Finds a registered output device Arc by its assigned ID.
    fn find_device_by_id(&self, id: usize) -> Option<Arc<ProtocolDevice>> {
        // 1. Get the name from the ID map within a limited scope for the lock
        let device_name = {
            let id_map = self.device_id_map.lock().unwrap();
            id_map.get(&id).cloned() // Clone the String if found, returns Option<String>
        }?; // Propagate None if the ID wasn't found; lock is released here

        // 2. Find the device in output_connections by matching the cloned name
        let connections = self.output_connections.lock().unwrap();
        connections.values()
            .find(|(name, _device)| name == &device_name) // Compare with the cloned name
            .map(|(_name, device_arc)| Arc::clone(device_arc))
        // Lock on connections is released here
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

    pub fn map_event(
        &self,
        event: ConcreteEvent,
        date: SyncTime,
    ) -> Vec<TimedMessage> {
        let opt_device: Option<Arc<ProtocolDevice>>;
        let error_dev_identifier: String;

        // Extract device ID from *any* Midi* event
        match &event {
            ConcreteEvent::Nop => {
                 // Nop doesn't target a device, handle separately or default to log
                 opt_device = self.find_device_by_id(LOG_DEVICE_ID); // Target Log device
                 error_dev_identifier = format!("Nop event");
            }
            ConcreteEvent::MidiNote(_, _, _, _, device_id)
            | ConcreteEvent::MidiControl(_, _, _, device_id)
            | ConcreteEvent::MidiProgram(_, _, device_id)
            | ConcreteEvent::MidiAftertouch(_, _, _, device_id)
            | ConcreteEvent::MidiChannelPressure(_, _, device_id)
            | ConcreteEvent::MidiSystemExclusive(_, device_id)
            | ConcreteEvent::MidiStart(device_id)
            | ConcreteEvent::MidiStop(device_id)
            | ConcreteEvent::MidiReset(device_id)
            | ConcreteEvent::MidiContinue(device_id)
            | ConcreteEvent::MidiClock(device_id) => {
                opt_device = self.find_device_by_id(*device_id);
                error_dev_identifier = format!("ID {}", device_id);
            }
        }

        // Handle device not found (same as before)
        let Some(device) = opt_device else {
            return vec![ProtocolMessage {
                payload: LogMessage {
                    level: Severity::Error,
                    msg: format!("Unable to find target device {}", error_dev_identifier),
                }
                .into(),
                device: Arc::new(ProtocolDevice::Log),
            }
            .timed(date)];
        };

        // Dispatch based on the *type* of the found device Arc (same as before)
        match &*device {
            ProtocolDevice::OSCOutDevice => todo!("OSC output not implemented in map_event"),
            ProtocolDevice::MIDIOutDevice(_) | ProtocolDevice::VirtualMIDIOutDevice {..} => {
                self.generate_midi_message(event, date, device)
            }
            ProtocolDevice::Log => {
                self.generate_log_message(event, date, device)
            }
            _ => {
                eprintln!("[!] map_event: Unhandled ProtocolDevice type for {}", error_dev_identifier);
                 vec![]
            }
        }
    }

    pub fn device_list(&self) -> Vec<DeviceInfo> {
        println!("[~] Générant la liste des périphériques...");
        let mut discovered_devices_info: HashMap<String, DeviceInfo> = HashMap::new();

        // --- Discover system ports (MIDI Out) ---
        if let Some(midi_out) = &self.midi_out {
            let ports_result = midi_out.lock().map(|guard| guard.ports());
            
            match ports_result {
                Ok(ports) => {
                    if !ports.is_empty() {
                        println!("[~] Découverte de {} ports MIDI output", ports.len());
                    }
                    
                    let mut output_port_names = Vec::new();
                    for port in &ports {
                        if let Ok(port_name) = midi_out.lock().unwrap().port_name(port) {
                            output_port_names.push(port_name);
                        } else {
                            output_port_names.push("<Error getting name>".to_string());
                        }
                    }
                    
                    // Process each port
                    for port in &ports {
                        if let Ok(name) = midi_out.lock().unwrap().port_name(port) {
                            if !discovered_devices_info.contains_key(&name) {
                                let id = self.ensure_device_id(&name); // Assign ID if new
                                discovered_devices_info.insert(name.clone(), DeviceInfo {
                                     id,
                                     name,
                                     kind: DeviceKind::Midi,
                                     is_connected: false,
                                });
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("[!] Échec de verrouillage midi_out: {:?}", e);
                }
            }
        } else {
             eprintln!("[!] Interface MIDI Output (self.midi_out) est None!");
        }

        // --- Discover system ports (MIDI In) ---
        if let Some(midi_in) = &self.midi_in {
            let ports_result = midi_in.lock().map(|guard| guard.ports());
            
            match ports_result {
                Ok(ports) => {
                    if !ports.is_empty() {
                        println!("[~] Découverte de {} ports MIDI input", ports.len());
                    }
                    
                    let mut input_port_names = Vec::new();
                    for port in &ports {
                        if let Ok(port_name) = midi_in.lock().unwrap().port_name(port) {
                            input_port_names.push(port_name);
                        } else {
                            input_port_names.push("<Error getting name>".to_string());
                        }
                    }
                    
                    // Process each port
                    for port in &ports {
                        if let Ok(name) = midi_in.lock().unwrap().port_name(port) {
                            if !discovered_devices_info.contains_key(&name) {
                                let id = self.ensure_device_id(&name); // Assign ID if new
                                discovered_devices_info.insert(name.clone(), DeviceInfo {
                                    id,
                                    name,
                                    kind: DeviceKind::Midi,
                                    is_connected: false,
                                });
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("[!] Échec de verrouillage midi_in: {:?}", e);
                }
            }
        } else {
            eprintln!("[!] Interface MIDI Input (self.midi_in) est None!");
        }

        // --- Add Log device ---
        discovered_devices_info.insert(LOG_NAME.to_string(), DeviceInfo {
            id: LOG_DEVICE_ID,
            name: LOG_NAME.to_string(),
            kind: DeviceKind::Log,
            is_connected: true,
        });

        // --- Mark connected status based on registered connections ---
        let connections_result = self.output_connections.lock();
        match connections_result {
            Ok(connections) => {
                for (registered_name, _device_arc) in connections.values() {
                    // Ensure the registered device also exists in our discovered list (could be virtual)
                    if !discovered_devices_info.contains_key(registered_name) {
                        // This happens for virtual devices which aren't discoverable by midir::ports()
                        // We need to add them to the list now.
                        let id = self.ensure_device_id(registered_name);
                        discovered_devices_info.insert(registered_name.clone(), DeviceInfo {
                            id, 
                            name: registered_name.clone(),
                            kind: DeviceKind::Midi, // Assume MIDI for now
                            is_connected: false, // Will be marked true below
                        });
                    }
                    
                    if let Some(device_info) = discovered_devices_info.get_mut(registered_name) {
                        // Mark as connected
                        device_info.is_connected = true;
                    }
                }
            },
            Err(e) => {
                eprintln!("[!] Échec de verrouillage output_connections: {:?}", e);
                return Vec::new(); // Return empty list on error
            }
        }

        let mut final_list: Vec<DeviceInfo> = discovered_devices_info.into_values().collect();
        // Sort by ID for stable ordering
        final_list.sort_by_key(|d| d.id);
        println!("[~] Liste de {} périphériques générée", final_list.len());
        final_list
    }

    /// Attempts to connect to the specified MIDI output device by ID.
    pub fn connect_midi_output(&self, device_id: usize) -> Result<(), String> {
        println!("[🔌] Attempting to connect MIDI Output device ID: {}", device_id);

        let Some(device_name) = self.get_device_name_by_id(device_id) else {
             return Err(format!("Cannot connect: Invalid device ID {}", device_id));
        };

        // Create a temporary MidiOutput instance to find the port and connect
        // This avoids moving out of the shared MutexGuard
        let temp_midi_out = MidiOutput::new(&format!("BuboCore-Temp-Connector-{}", device_name))
            .map_err(|e| format!("Failed to create temporary MidiOutput: {}", e))?;

        // Find the midir port using the temporary instance
        let port_opt: Option<MidiOutputPort> = temp_midi_out.ports().into_iter().find(|p| {
            // Need to handle potential error from port_name
            temp_midi_out.port_name(p).map_or(false, |name| name == device_name)
        });
        let port = port_opt.ok_or(format!("MIDI Output port '{}' not found by midir.", device_name))?;
        println!("   Found midir port using temporary instance.");

        // Perform the connection using the temporary instance (which consumes it)
        match temp_midi_out.connect(&port, &format!("BuboCore-Connection-{}", device_name)) {
            Ok(connection) => {
                println!("[✅] Successfully connected to MIDI Output: {}", device_name);

                // Create the MidiOut struct for storage, wrapping the actual connection
                let midi_out_handler = MidiOut {
                    name: device_name.clone(),
                    active_notes: Default::default(),
                    // Store the actual connection obtained from the temporary MidiOutput
                    connection: Arc::new(Mutex::new(Some(connection))),
                };

                // Wrap the handler in Arc<Mutex<>> for ProtocolDevice
                let device = ProtocolDevice::MIDIOutDevice(Arc::new(Mutex::new(midi_out_handler)));

                // Register/update the connection in the map
                self.register_output_connection(device_name.clone(), device);

                println!("[✅] Registered/Updated connection for MIDI Output '{}' (ID {})", device_name, device_id);
                Ok(())
            },
            Err(e) => {
                eprintln!("[!] Failed to connect MIDI Output '{}': {}", device_name, e);
                Err(format!("Failed to connect MIDI Output '{}': {}", device_name, e))
            }
        }
        // No MutexGuard lock to release here as we used a temporary instance
    }

    /// Disconnects the specified MIDI output device by ID.
    pub fn disconnect_midi_output(&self, device_id: usize) -> Result<(), String> {
        let Some(device_name) = self.get_device_name_by_id(device_id) else {
             return Err(format!("Cannot disconnect: Invalid device ID {}", device_id));
        };
         println!("[🔌] Attempting to disconnect MIDI Output device ID: {}", device_id);

        let mut connections = self.output_connections.lock().unwrap();
        
        let key_to_remove = connections.iter()
            .find(|(_address, (name, _device))| name == &device_name)
            .map(|(address, _item)| address.clone());

        match key_to_remove {
            Some(key) => {
                if connections.remove(&key).is_some() {
                    // Note: We don't remove the ID from the id_map or name_map, 
                    // so it remains stable if the device reappears.
                    println!("[✅] Disconnected and removed registration for MIDI Output '{}' (ID {})", device_name, device_id);
                    Ok(())
                } else {
                     eprintln!("[!] Failed to remove connection for key '{}' (name: '{}') even though it was found.", key, device_name);
                     Err(format!("Internal error removing connection for {}", device_name))
                }
            }
            None => {
                eprintln!("[!] Cannot disconnect MIDI Output '{}' (ID {}): Not found in registered connections.", device_name, device_id);
                Err(format!("Device '{}' (ID {}) not registered/connected.", device_name, device_id))
            }
        }
    }

    /// Creates a virtual MIDI output port and registers it.
    pub fn create_virtual_midi_output(&self, device_name: &str) -> Result<(), String> {
        println!("[✨] Création du port MIDI virtuel: '{}'", device_name);

        // Check if a device (real or virtual) with this name already exists in registered connections
        // OR if the name is already assigned an ID (even if not currently registered)
        let name_exists_result = self.device_name_to_id_map.lock();
        match name_exists_result {
            Ok(name_map) => {
                if name_map.contains_key(device_name) {
                    let err_msg = format!("Le nom '{}' est déjà utilisé (enregistré ou avec un ID précédemment assigné).", device_name);
                    eprintln!("[!] {}", err_msg);
                    return Err(err_msg);
                }
                // Drop the lock before proceeding
                drop(name_map);
            },
            Err(e) => {
                let err_msg = format!("Échec de verrouillage device_name_to_id_map: {:?}", e);
                eprintln!("[!] {}", err_msg);
                return Err(err_msg);
            }
        }
        
        // Also check if the name exists in the system ports discovered by the main midi_out (avoid conflicts)
        if let Some(main_midi_out) = &self.midi_out {
            let midi_out_lock_result = main_midi_out.lock();
            match midi_out_lock_result {
                Ok(midi_out_guard) => {
                    // Get ports while we have the lock
                    let ports = midi_out_guard.ports();
                    
                    // Drop the lock before checking ports individually
                    drop(midi_out_guard);
                    
                    // Check each port for name match
                    for port in ports.iter() {
                        // Re-acquire lock for port_name
                        let port_name_result = main_midi_out.lock().unwrap().port_name(port);
                        match port_name_result {
                            Ok(port_name) => {
                                if port_name == device_name {
                                    let err_msg = format!("Le nom '{}' existe déjà comme port MIDI système.", device_name);
                                    eprintln!("[!] {}", err_msg);
                                    return Err(err_msg);
                                }
                            },
                            Err(_) => {
                                // Continue checking other ports even if this one failed
                            }
                        }
                    }
                },
                Err(e) => {
                    let err_msg = format!("Échec de verrouillage midi_out: {:?}", e);
                    eprintln!("[!] {}", err_msg);
                    return Err(err_msg);
                }
            }
        }

        // Use a temporary MidiOutput instance just to create the virtual port
        let temp_midi_out_result = MidiOutput::new(&format!("BuboCore-Virtual-Creator-{}", device_name));
        let temp_midi_out = match temp_midi_out_result {
            Ok(output) => {
                output
            },
            Err(e) => {
                let err_msg = format!("Échec de création du MidiOutput temporaire: {}", e);
                eprintln!("[!] {}", err_msg);
                return Err(err_msg);
            }
        };

        println!("[~] Création du port MIDI virtuel (peut prendre du temps)...");
        match temp_midi_out.create_virtual(device_name) {
            Ok(connection) => {
                println!("[✅] Port MIDI virtuel créé: '{}'", device_name);
                
                // Ensure ID is assigned *before* registering
                let new_id = self.ensure_device_id(device_name);

                // Create the ProtocolDevice variant, wrapping the connection correctly
                let virtual_device = ProtocolDevice::VirtualMIDIOutDevice {
                    name: device_name.to_string(),
                    // Wrap the Option<MidiOutputConnection> in Arc<Mutex<>>
                    connection: Arc::new(Mutex::new(Some(connection))),
                };

                // Register this new virtual device (will use the name as key)
                self.register_output_connection(device_name.to_string(), virtual_device);
                println!("[✅] Port MIDI virtuel '{}' (ID {}) enregistré", device_name, new_id);
                Ok(())
            }
            Err(e) => {
                eprintln!("[!] Échec de création du port MIDI virtuel '{}': {}", device_name, e);
                Err(format!("Échec de création du port MIDI virtuel '{}': {}", device_name, e))
            }
        }
    }
}

impl Default for DeviceMap {
    fn default() -> Self {
        Self::new()
    }
}
