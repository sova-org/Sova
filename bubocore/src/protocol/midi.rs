mod control_memory;
pub mod midi_constants;

use midir::os::unix::VirtualOutput;
use midir::{MidiInput, MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};

use control_memory::MidiInMemory;
use midi_constants::*;
use std::fmt::{Debug, Display};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};

/// Représente une erreur dans le traitement MIDI
#[derive(Debug, Default, Clone)]
pub struct MidiError(pub String);

impl<T: ToString> From<T> for MidiError {
    fn from(value: T) -> Self {
        MidiError(value.to_string())
    }
}

/// Message MIDI avec un type de charge utile et un canal
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MIDIMessage {
    pub payload: MIDIMessageType,
    pub channel: u8,
}

impl Display for MIDIMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MIDIMessage sur canal ({}) : [{}]", self.channel, self.payload)
    }
}

/// Types de messages MIDI supportés
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MIDIMessageType {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8, velocity: u8 },
    ControlChange { control: u8, value: u8 },
    ProgramChange { program: u8 },
    PitchBend { value: u16 },
    Aftertouch { note: u8, value: u8 },
    ChannelPressure { value: u8 },
    SystemExclusive { data: Vec<u8> },
    Clock,
    Start,
    Continue,
    Stop,
    Reset,
    Undefined(u8),
}

impl Display for MIDIMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MIDIMessageType::NoteOn { note, velocity } => write!(f, "NoteOn : note = {note} ; velocity = {velocity}"),
            MIDIMessageType::NoteOff { note, velocity } => write!(f, "NoteOff : note = {note} ; velocity = {velocity}"),
            MIDIMessageType::ControlChange { control, value } => write!(f, "ControlChange : control = {control} ; value = {value}"),
            MIDIMessageType::ProgramChange { program } => write!(f, "ProgramChange : program = {program}"),
            MIDIMessageType::PitchBend { value } => write!(f, "PitchBend : pitch = {} ; bend = {}", value % 0x100, value >> 8),
            MIDIMessageType::Aftertouch { note, value } => write!(f, "AfterTouch : note = {note} ; value = {value}"),
            MIDIMessageType::ChannelPressure { value } => write!(f, "ChannelPressure : value = {value}"),
            MIDIMessageType::SystemExclusive { data } => write!(f, "SystemExclusive : data = {:?}", data),
            MIDIMessageType::Clock => write!(f, "Clock"),
            MIDIMessageType::Start => write!(f, "Start"),
            MIDIMessageType::Continue => write!(f, "Continue"),
            MIDIMessageType::Stop => write!(f, "Stop"),
            MIDIMessageType::Reset => write!(f, "Reset"),
            MIDIMessageType::Undefined(x) => write!(f, "Undefined : {x}"),
        }
    }
}

/// Interface commune pour tous les périphériques MIDI
pub trait MidiInterface {
    /// Crée une nouvelle instance de l'interface
    fn new(client_name: String) -> Result<Self, MidiError>
    where
        Self: Sized;
    
    /// Renvoie la liste des ports disponibles
    fn ports(&self) -> Vec<String>;
    
    /// Connecte l'interface au port par défaut
    fn connect(&mut self) -> Result<(), MidiError>;
    
    /// Vérifie si l'interface est connectée
    fn is_connected(&self) -> bool;
}

/// Sortie MIDI pour envoyer des messages
#[derive(Serialize, Deserialize)]
pub struct MidiOut {
    pub name: String,
    #[serde(skip)]
    pub connection: Arc<Mutex<Option<MidiOutputConnection>>>,
    #[serde(skip, default = "default_active_notes")]
    pub active_notes: Mutex<HashMap<u8, HashSet<u8>>>,
}

impl Display for MidiOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiOut({})", self.name)
    }
}

impl Debug for MidiOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiOut({})", self.name)
    }
}

impl MidiOut {
    /// Envoie un message MIDI 
    pub fn send(&self, message: MIDIMessage) -> Result<(), MidiError> {
        let mut connection_opt_guard = self.connection.lock()
            .map_err(|_| MidiError("MidiOut connection Mutex poisoned".to_string()))?;
        
        let Some(connection) = connection_opt_guard.as_mut() else {
            return Err(format!("Interface MIDI {} non connectée à un port MIDI", self.name).into());
        };
        
        let mut active_notes_guard = self.active_notes.lock().unwrap();
        let bytes = match message.payload {
            MIDIMessageType::NoteOn { note, velocity } => {
                let channel_notes = active_notes_guard.entry(message.channel).or_default();
                if channel_notes.contains(&note) { 
                    return Ok(());
                } else { 
                    channel_notes.insert(note); 
                    vec![NOTE_ON_MSG + message.channel, note, velocity] 
                }
            }
            MIDIMessageType::NoteOff { note, velocity } => {
                let channel_notes = active_notes_guard.entry(message.channel).or_default();
                if channel_notes.contains(&note) { 
                    channel_notes.remove(&note); 
                    vec![NOTE_OFF_MSG + message.channel, note, velocity] 
                } else { 
                    return Ok(());
                }
            }
            MIDIMessageType::ControlChange { control, value } => 
                vec![CONTROL_CHANGE_MSG + message.channel, control, value],
            MIDIMessageType::ProgramChange { program } => 
                vec![PROGRAM_CHANGE_MSG + message.channel, program],
            MIDIMessageType::Aftertouch { note, value } => 
                vec![AFTERTOUCH_MSG + message.channel, note, value],
            MIDIMessageType::ChannelPressure { value } => 
                vec![CHANNEL_PRESSURE_MSG + message.channel, value],
            MIDIMessageType::PitchBend { value } => 
                vec![PITCH_BEND_MSG + message.channel, (value & 0x7F) as u8, (value >> 7) as u8],
            MIDIMessageType::Clock => vec![CLOCK_MSG],
            MIDIMessageType::Continue => vec![CONTINUE_MSG],
            MIDIMessageType::Reset => vec![RESET_MSG],
            MIDIMessageType::Start => vec![START_MSG],
            MIDIMessageType::Stop => vec![STOP_MSG],
            MIDIMessageType::SystemExclusive { ref data } => { 
                let mut m = vec![0xF0]; 
                m.extend(data); 
                m.push(0xF7); 
                m 
            },
            MIDIMessageType::Undefined(byte) => vec![byte],
        };
        
        connection.send(&bytes)
            .map_err(|e| format!("Échec d'envoi du message MIDI : {}", e).into())
    }

    /// Connecte à un port MIDI par défaut, avec option pour un port virtuel
    pub fn connect_to_default(&mut self, use_virtual: bool) -> Result<(), MidiError> {
        let midi_out = self.get_midi_out()?;
        let connection_result = if use_virtual {
            #[cfg(not(target_os = "windows"))] { 
                midi_out.create_virtual(&self.name).map_err(|e| e.into()) 
            }
            #[cfg(target_os = "windows")] {
                eprintln!("Ports MIDI virtuels non supportés sous Windows. Retour au mode standard...");
                let ports = midi_out.ports(); 
                if ports.is_empty() { 
                    return Err("Aucun port MIDI disponible".into()); 
                }
                midi_out.connect(&ports[0], &self.name).map_err(|e| e.into())
            }
        } else {
            let ports = midi_out.ports(); 
            if ports.is_empty() { 
                return Err("Aucun port MIDI disponible".into()); 
            }
            midi_out.connect(&ports[0], &self.name).map_err(|e| e.into())
        };
        
        match connection_result {
            Ok(connection) => {
                *self.connection.lock().unwrap() = Some(connection);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    /// Crée une instance de MidiOutput
    fn get_midi_out(&self) -> Result<MidiOutput, MidiError> { 
        MidiOutput::new(&format!("BuboInt-{}", self.name)).map_err(|e| e.into()) 
    }
    
    /// Vide la file d'attente (no-op pour midir)
    pub fn flush(&self) {}
}

impl Drop for MidiOut { 
    fn drop(&mut self) { 
        if let Ok(mut c) = self.connection.lock() { 
            c.take(); 
        } 
    } 
}

impl MidiInterface for MidiOut {
    fn new(name: String) -> Result<Self, MidiError> { 
        Ok(MidiOut { 
            name, 
            connection: Arc::new(Mutex::new(None)), 
            active_notes: Mutex::new(HashMap::new()) 
        }) 
    }
    
    fn ports(&self) -> Vec<String> { 
        self.get_midi_out()
            .map(|m| m.ports().iter()
                .map(|p| m.port_name(p).unwrap_or_default())
                .collect())
            .unwrap_or_default() 
    }
    
    fn connect(&mut self) -> Result<(), MidiError> {
        let midi_out = self.get_midi_out()?; 
        let ports = midi_out.ports(); 
        
        if ports.is_empty() { 
            return Err("Aucun port de sortie MIDI disponible!".into()); 
        }
        
        let target_port = &ports[0]; 
        let target_name = midi_out.port_name(target_port).unwrap_or_default();
        
        match midi_out.connect(target_port, &self.name) {
            Ok(connection) => {
                *self.connection.lock().unwrap() = Some(connection);
                Ok(())
            },
            Err(e) => Err(format!("Échec de connexion '{}' à '{}': {}", self.name, target_name, e).into()),
        }
    }
    
    fn is_connected(&self) -> bool { 
        self.connection.lock().unwrap().is_some() 
    }
}

/// Entrée MIDI pour recevoir des messages
#[derive(Serialize, Deserialize)]
pub struct MidiIn {
    pub name: String,
    #[serde(skip)] 
    pub connection: Arc<Mutex<Option<midir::MidiInputConnection<()>>>>,
    #[serde(skip)] 
    pub memory: Arc<Mutex<MidiInMemory>>,
}

impl Debug for MidiIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiIn({})", self.name)
    }
}

impl Display for MidiIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiIn({})", self.name)
    }
}

impl MidiIn { 
    /// Crée une instance de MidiInput
    fn get_midi_in(&self) -> Result<MidiInput, MidiError> { 
        MidiInput::new(&format!("BuboInt-{}", self.name)).map_err(|e| e.into()) 
    } 
}

impl MidiInterface for MidiIn {
    fn new(name: String) -> Result<Self, MidiError> { 
        Ok(MidiIn { 
            name, 
            connection: Arc::new(Mutex::new(None)), 
            memory: Arc::new(Mutex::new(MidiInMemory::new())) 
        }) 
    }
    
    fn ports(&self) -> Vec<String> { 
        self.get_midi_in()
            .map(|m| m.ports().iter()
                .map(|p| m.port_name(p).unwrap_or_default())
                .collect())
            .unwrap_or_default() 
    }
    
    fn connect(&mut self) -> Result<(), MidiError> {
        let midi_in = self.get_midi_in()?; 
        let ports = midi_in.ports(); 
        
        if ports.is_empty() { 
            return Err("Aucun port d'entrée MIDI disponible!".into()); 
        }
        
        let target_port = &ports[0];
        let memory_clone = Arc::clone(&self.memory);

        let connection = midi_in.connect(
            target_port,
            &format!("BuboCoreIn-{}", self.name),
            move |_timestamp, message, _| {
                if message.len() == 3 && (message[0] & 0xF0) == CONTROL_CHANGE_MSG {
                    let channel = (message[0] & 0x0F) as i8;
                    let control = message[1] as i8;
                    let value = message[2] as i8;
                    let mut memory_guard = memory_clone.lock().unwrap();
                    (*memory_guard).set(channel, control, value);
                }
            },
            (),
        )?;
        
        *self.connection.lock().unwrap() = Some(connection);
        Ok(())
    }
    
    fn is_connected(&self) -> bool { 
        self.connection.lock().unwrap().is_some() 
    }
}

/// Crée un Mutex contenant une HashMap vide pour les notes actives
fn default_active_notes() -> Mutex<HashMap<u8, HashSet<u8>>> { 
    Mutex::new(HashMap::new()) 
}

