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
use crate::protocol::midi::{MidiOut, MidiInterface};

use midir::{MidiInput, MidiOutput, Ignore};
// Import the necessary trait for create_virtual (on Unix-like systems)
#[cfg(target_family = "unix")] 
use midir::os::unix::VirtualOutput;

pub type DeviceItem = (String, Arc<ProtocolDevice>);

pub struct DeviceMap {
    pub input_connections: Mutex<HashMap<String, DeviceItem>>,
    pub output_connections: Mutex<HashMap<String, DeviceItem>>,
    midi_in: Option<MidiInput>,
    midi_out: Option<MidiOutput>,
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
                Some(input)
            }
            Err(e) => {
                eprintln!("[!] Failed to initialize MIDI Input: {}", e);
                None
            }
        };

        let midi_out = match MidiOutput::new("BuboCore Output") {
            Ok(output) => {
                println!("[+] MIDI Output initialized successfully.");
                Some(output)
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

    fn generate_midi_message(
        &self,
        payload: ConcreteEvent,
        date: SyncTime,
        device: Arc<ProtocolDevice>,
    ) -> Vec<TimedMessage> {
        match payload {
            ConcreteEvent::MidiNote(note, vel, chan, dur, _) => {
                //let chan = chan.unwrap_or(0);
                //let vel = vel.unwrap_or(90);
                vec![
                    ProtocolMessage {
                        payload: MIDIMessage {
                            payload: MIDIMessageType::NoteOn {
                                note: note as u8,
                                velocity: vel as u8,
                            },
                            channel: chan as u8,
                        }
                        .into(),
                        device: Arc::clone(&device),
                    }
                    .timed(date),
                    ProtocolMessage {
                        payload: MIDIMessage {
                            payload: MIDIMessageType::NoteOff {
                                note: note as u8,
                                velocity: 0,
                            },
                            channel: chan as u8,
                        }
                        .into(),
                        device: Arc::clone(&device),
                    }
                    .timed(date + dur),
                ]
                /*notes.iter().map(|n|
                .chain(notes.iter().map(|n|
                )).collect()*/
            }
            ConcreteEvent::MidiControl(control, value, chan, _) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::ControlChange {
                            control: control as u8,
                            value: value as u8,
                        },
                        channel: chan as u8,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiProgram(program, chan, _) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::ProgramChange {
                            program: program as u8,
                        },
                        channel: chan as u8,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiAftertouch(note, pressure, chan, _) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::Aftertouch {
                            note: note as u8,
                            value: pressure as u8,
                        },
                        channel: chan as u8,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiChannelPressure(pressure, channel, _) => {
                vec![ProtocolMessage {
                    payload: MIDIMessage {
                        payload: MIDIMessageType::ChannelPressure {
                            value: pressure as u8,
                        },
                        channel: channel as u8,
                    }
                    .into(),
                    device: Arc::clone(&device),
                }
                .timed(date)]
            }
            ConcreteEvent::MidiStart(_) => {
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
            ConcreteEvent::MidiStop(_) => {
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
            ConcreteEvent::MidiContinue(_) => {
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
            ConcreteEvent::MidiClock(_) => {
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
            ConcreteEvent::MidiReset(_) => {
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
            ConcreteEvent::MidiSystemExclusive(data, _) => {
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
            _ => Vec::new(),
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
        let (dev_name, opt_device) = self.find_device(&event);
        let Some(device) = opt_device else {
            return vec![ProtocolMessage {
                payload: LogMessage {
                    level: Severity::Error,
                    msg: format!("Unable to find device {:?}", dev_name),
                }
                .into(),
                device: Arc::new(ProtocolDevice::Log),
            }
            .timed(date)];
        };
        match &*device {
            ProtocolDevice::OSCOutDevice => todo!(),
            ProtocolDevice::MIDIOutDevice(_) => {
                self.generate_midi_message(event, date, device)
            }
            ProtocolDevice::Log => self.generate_log_message(event, date, device),
            _ => Vec::new(),
        }
    }

    pub fn find_device(&self, event: &ConcreteEvent) -> (String, Option<Arc<ProtocolDevice>>) {
        let cons = self.output_connections.lock().unwrap();
        match event {
            ConcreteEvent::Nop => (
                LOG_NAME.to_string(),
                cons.get(LOG_NAME).map(|x| Arc::clone(&x.1)),
            ),
            ConcreteEvent::MidiNote(_, _, _, _, dev)
            | ConcreteEvent::MidiControl(_, _, _, dev)
            | ConcreteEvent::MidiProgram(_, _, dev)
            | ConcreteEvent::MidiStart(dev)
            | ConcreteEvent::MidiStop(dev)
            | ConcreteEvent::MidiReset(dev)
            | ConcreteEvent::MidiClock(dev)
            | ConcreteEvent::MidiContinue(dev)
            | ConcreteEvent::MidiSystemExclusive(_, dev)
            | ConcreteEvent::MidiChannelPressure(_, _, dev)
            | ConcreteEvent::MidiAftertouch(_, _, _, dev) => {
                (dev.to_string(), cons.get(dev).map(|x| Arc::clone(&x.1)))
            }
        }
    }

    pub fn device_list(&self) -> Vec<DeviceInfo> {
        println!("[~] Generating device list...");
        let mut discovered_devices_info: HashMap<String, DeviceInfo> = HashMap::new();

        // --- Discover system ports (MIDI Out) ---
        if let Some(midi_out) = &self.midi_out {
            let output_port_names: Vec<String> = midi_out.ports().iter().map(|p| midi_out.port_name(p).unwrap_or_else(|e| format!("<Err: {}>", e))).collect();
            println!("[~] device_list: Discovered MIDI Outputs via midir: {:?}", output_port_names);
            for port in midi_out.ports() {
                if let Ok(name) = midi_out.port_name(&port) {
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
        } else {
             eprintln!("[!] device_list: MIDI Output interface (self.midi_out) is None!");
        }

        // --- Discover system ports (MIDI In) ---
        if let Some(midi_in) = &self.midi_in {
             let input_port_names: Vec<String> = midi_in.ports().iter().map(|p| midi_in.port_name(p).unwrap_or_else(|e| format!("<Err: {}>", e))).collect();
             println!("[~] device_list: Discovered MIDI Inputs via midir: {:?}", input_port_names);
            for port in midi_in.ports() {
                if let Ok(name) = midi_in.port_name(&port) {
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
        } else {
            eprintln!("[!] device_list: MIDI Input interface (self.midi_in) is None!");
        }

        // --- Add Log device ---
        println!("[~] device_list: Adding LOG device.");
        discovered_devices_info.insert(LOG_NAME.to_string(), DeviceInfo {
            id: LOG_DEVICE_ID,
            name: LOG_NAME.to_string(),
            kind: DeviceKind::Log,
            is_connected: true,
        });

        // --- Mark connected status based on registered connections ---
        println!("[~] device_list: Checking registered connections to mark status...");
        let connections = self.output_connections.lock().unwrap();
        println!("[~] device_list: Registered output connections: {:?}", connections.values().map(|(name, dev)| (name, dev.address())).collect::<Vec<_>>());
        for (registered_name, _protocol_device_arc) in connections.values() {
            println!("[~] device_list: Checking registered connection '{}'", registered_name);
            // Ensure the registered device also exists in our discovered list (could be virtual)
            if !discovered_devices_info.contains_key(registered_name) {
                 // This happens for virtual devices which aren't discoverable by midir::ports()
                 // We need to add them to the list now.
                 let id = self.ensure_device_id(registered_name);
                  println!("[~] device_list: Adding registered (likely virtual) device '{}' (ID {}) to list.", registered_name, id);
                 discovered_devices_info.insert(registered_name.clone(), DeviceInfo {
                    id, 
                    name: registered_name.clone(),
                    kind: DeviceKind::Midi, // Assume MIDI for now
                    is_connected: false, // Will be marked true below
                 });
            }
            
            if let Some(device_info) = discovered_devices_info.get_mut(registered_name) {
                // Mark as connected
                 println!("[~] device_list: Marking '{}' (ID {}) as connected.", registered_name, device_info.id);
                device_info.is_connected = true;
            } else {
                // This case should theoretically not be reached after the check above, but log if it does.
                 println!("[!] device_list: Registered connection '{}' could not be found or added to list.", registered_name);
            }
        }
        drop(connections);

        let mut final_list: Vec<DeviceInfo> = discovered_devices_info.into_values().collect();
        // Sort by ID for stable ordering
        final_list.sort_by_key(|d| d.id);
        println!("[~] device_list: Final generated list (sorted by ID): {:?}", final_list);
        final_list
    }

    /// Attempts to connect to the specified MIDI output device by ID.
    pub fn connect_midi_output(&self, device_id: usize) -> Result<(), String> {
        let Some(device_name) = self.get_device_name_by_id(device_id) else {
             return Err(format!("Cannot connect: Invalid device ID {}", device_id));
        };

        println!("[~] connect_midi_output: Request to connect ID {} ('{}')", device_id, device_name);

        if self.output_connections.lock().unwrap().values().any(|(name, _)| name == &device_name) {
            println!("[~] MIDI Output '{}' is already connected.", device_name);
            return Ok(());
        }

        let mut midi_out_handler = MidiOut::new(device_name.clone())
            .map_err(|e| format!("Failed to create MidiOut handler for '{}': {}", device_name, e.0))?;
        
        midi_out_handler.connect()
            .map_err(|e| format!("Failed to connect to MIDI output '{}': {}", device_name, e.0))?;

        let device = ProtocolDevice::MIDIOutDevice(midi_out_handler);
        self.register_output_connection(device_name.clone(), device); // register_output_connection ensures ID again
        println!("[+] Registered connection for MIDI Output '{}' (ID {})", device_name, device_id);
        Ok(())
    }

    /// Disconnects the specified MIDI output device by ID.
    pub fn disconnect_midi_output(&self, device_id: usize) -> Result<(), String> {
        let Some(device_name) = self.get_device_name_by_id(device_id) else {
             return Err(format!("Cannot disconnect: Invalid device ID {}", device_id));
        };
         println!("[~] disconnect_midi_output: Request to disconnect ID {} ('{}')", device_id, device_name);

        let mut connections = self.output_connections.lock().unwrap();
        
        let key_to_remove = connections.iter()
            .find(|(_address, (name, _device))| name == &device_name)
            .map(|(address, _item)| address.clone());

        match key_to_remove {
            Some(key) => {
                if connections.remove(&key).is_some() {
                    // Note: We don't remove the ID from the id_map or name_map, 
                    // so it remains stable if the device reappears.
                    println!("[+] Disconnected and removed registration for MIDI Output '{}' (ID {})", device_name, device_id);
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
        println!("[~] Attempting to create virtual MIDI output: '{}'", device_name);

        // Check if a device (real or virtual) with this name already exists in registered connections
        // OR if the name is already assigned an ID (even if not currently registered)
        if self.device_name_to_id_map.lock().unwrap().contains_key(device_name) {
            let err_msg = format!("Device name '{}' is already in use (registered or previously assigned ID).", device_name);
            eprintln!("[!] create_virtual_midi_output: {}", err_msg);
            return Err(err_msg);
        }
        // Also check if the name exists in the system ports discovered by the main midi_out (avoid conflicts)
        if let Some(main_midi_out) = &self.midi_out {
             if main_midi_out.ports().iter().any(|p| main_midi_out.port_name(p).map_or(false, |n| n == device_name)) {
                 let err_msg = format!("Device name '{}' already exists as a system MIDI port.", device_name);
                 eprintln!("[!] create_virtual_midi_output: {}", err_msg);
                 return Err(err_msg);
             }
        }

        // Create a *temporary* MidiOutput instance to create the virtual port.
        let temp_client_name = format!("BuboCore_Virtual_{}", device_name);
        let temp_midi_out = match MidiOutput::new(&temp_client_name) {
             Ok(instance) => instance,
             Err(e) => {
                 let err_msg = format!("Failed to create temporary MidiOutput for virtual device: {}", e);
                 eprintln!("[!] create_virtual_midi_output: {}", err_msg);
                 return Err(err_msg);
             }
        };

        // Use the temporary instance to call create_virtual
        match temp_midi_out.create_virtual(device_name) {
            Ok(connection) => {
                println!("[+] Successfully created virtual MIDI output: '{}'", device_name);
                // Ensure ID is assigned *before* registering
                let new_id = self.ensure_device_id(device_name);
                // Wrap the connection in our new ProtocolDevice variant
                let device = ProtocolDevice::VirtualMIDIOutDevice {
                     name: device_name.to_string(),
                     connection: Some(connection),
                };
                // Register this new virtual device (will use the name as key)
                self.register_output_connection(device_name.to_string(), device);
                println!("[+] Registered virtual MIDI output: '{}' (ID {})", device_name, new_id);
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Failed to create virtual MIDI output '{}': {}", device_name, e);
                eprintln!("[!] create_virtual_midi_output: {}", err_msg);
                Err(err_msg)
            }
        }
    }
}
