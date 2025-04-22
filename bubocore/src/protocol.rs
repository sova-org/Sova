use std::{cmp::Ordering, fmt::{self, Debug, Display}, sync::{Arc, Mutex}};
use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

use log::LogMessage;
use osc::OSCMessage;
use midi::{MIDIMessage, MidiError, MidiIn, MidiOut, midi_constants::*, MIDIMessageType};
use midir::MidiOutputConnection;
use rosc::{OscPacket, OscMessage as RoscOscMessage, OscType, OscBundle, OscTime};

use crate::clock::SyncTime;
use serde::{Deserialize, Serialize};

// Correctly import our Argument type
use crate::protocol::osc::Argument as BuboArgument;

pub mod midi;
pub mod osc;
pub mod log;

/// Charge utile unifiée pour transmettre n'importe quel message supporté par un protocole
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum ProtocolPayload {
    OSC(OSCMessage),
    MIDI(MIDIMessage),
    LOG(LogMessage),
}

impl Display for ProtocolPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolPayload::OSC(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::MIDI(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::LOG(m) => std::fmt::Display::fmt(m, f),
        }
    }
}

/// Message de protocole avec une cible (dispositif) et une charge utile
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub device: Arc<ProtocolDevice>,
    pub payload: ProtocolPayload
}

impl ProtocolMessage {
    /// Envoie le message au dispositif cible
    pub fn send(self, time: SyncTime) -> Result<(), ProtocolError> {
        self.device.send(self.payload, time)
    }
}

impl Display for ProtocolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] -> Dispositif : {}", self.payload, self.device)
    }
}

/// Types de dispositifs supportés par le système
#[derive(Serialize, Deserialize)]
pub enum ProtocolDevice {
    Log,
    OSCInDevice,
    MIDIInDevice(Arc<Mutex<MidiIn>>),
    MIDIOutDevice(Arc<Mutex<MidiOut>>),
    VirtualMIDIOutDevice { 
        name: String, 
        #[serde(skip)] connection: Arc<Mutex<Option<MidiOutputConnection>>> 
    },
    OSCOutputDevice {
        name: String,
        address: SocketAddr,
        latency: f64,
        #[serde(skip)] socket: Option<Arc<UdpSocket>>,
    },
}

impl Debug for ProtocolDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolDevice::Log => write!(f, "Log"),
            ProtocolDevice::OSCInDevice => write!(f, "OSCInDevice"),
            ProtocolDevice::MIDIInDevice(arg0_mutex) => {
                let guard = arg0_mutex.lock().map_err(|_| fmt::Error)?;
                f.debug_tuple("MIDIInDevice").field(&*guard).finish()
            }
            ProtocolDevice::MIDIOutDevice(arg0_mutex) => {
                let guard = arg0_mutex.lock().map_err(|_| fmt::Error)?;
                f.debug_tuple("MIDIOutDevice").field(&*guard).finish()
            }
            ProtocolDevice::VirtualMIDIOutDevice { name, connection: connection_arc_mutex } => {
                let connection_status = connection_arc_mutex.lock()
                    .map(|guard| guard.as_ref().map(|_| "<MidiOutputConnection>"))
                    .map_err(|_| fmt::Error)?;

                f.debug_struct("VirtualMIDIOutDevice")
                    .field("name", name)
                    .field("connection", &connection_status)
                    .finish()
            }
            ProtocolDevice::OSCOutputDevice { name, address, latency, socket } => {
                let socket_status = socket.as_ref().map(|_| "<UdpSocket>");
                f.debug_struct("OSCOutputDevice")
                    .field("name", name)
                    .field("address", address)
                    .field("latency", latency)
                    .field("socket", &socket_status)
                    .finish()
            }
        }
    }
}

impl Display for ProtocolDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolDevice::Log => write!(f, "Log"),
            ProtocolDevice::OSCInDevice => write!(f, "OSCInDevice"),
            ProtocolDevice::MIDIInDevice(midi_in_arc_mutex) => {
                midi_in_arc_mutex.lock()
                    .map_err(|_| fmt::Error)
                    .and_then(|guard| std::fmt::Display::fmt(&*guard, f))
            },
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                midi_out_arc_mutex.lock()
                    .map_err(|_| fmt::Error)
                    .and_then(|guard| std::fmt::Display::fmt(&*guard, f))
            },
            ProtocolDevice::VirtualMIDIOutDevice { name, connection: _ } => 
                write!(f, "VirtualMIDIOutDevice({})", name),
            ProtocolDevice::OSCOutputDevice { name, .. } =>
                write!(f, "OSCOutputDevice({})", name),
        }
    }
}

impl PartialEq for ProtocolDevice {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for ProtocolDevice {}

/// Erreur lors de l'utilisation d'un protocole
#[derive(Debug)]
pub struct ProtocolError(pub String);

impl From<MidiError> for ProtocolError {
    fn from(value: MidiError) -> Self {
        ProtocolError(value.0)
    }
}

impl From<std::io::Error> for ProtocolError {
    fn from(e: std::io::Error) -> Self {
        ProtocolError(format!("IO Error: {}", e))
    }
}

impl From<rosc::OscError> for ProtocolError {
    fn from(e: rosc::OscError) -> Self {
        ProtocolError(format!("OSC Error: {}", e))
    }
}

impl ProtocolDevice {
    /// Connecte le dispositif à son port par défaut
    pub fn connect(&mut self) -> Result<(), ProtocolError> {
        let device_address_for_log = match self {
            ProtocolDevice::VirtualMIDIOutDevice { name, .. } => name.clone(),
            _ => "".to_string(),
        };

        match self {
            ProtocolDevice::OSCInDevice => todo!(),
            ProtocolDevice::MIDIInDevice(midi_in_arc_mutex) => {
                println!("[~] ProtocolDevice::connect() called for MIDIInDevice '{}'. Connection is handled elsewhere.", midi_in_arc_mutex.lock().unwrap().name);
                Ok(())
            },
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                println!("[~] ProtocolDevice::connect() called for MIDIOutDevice '{}'. Connection is handled elsewhere.", midi_out_arc_mutex.lock().unwrap().name);
                Ok(())
            },
            ProtocolDevice::VirtualMIDIOutDevice { name: _, connection: connection_arc_mutex } => {
                println!("[~] ProtocolDevice::connect() called for VirtualMIDIOutDevice '{}'", 
                          device_address_for_log);
                let conn_opt_guard = connection_arc_mutex.lock()
                    .map_err(|_| ProtocolError("Virtual Connection Mutex poisoned".to_string()))?;
                
                if conn_opt_guard.is_some() {
                    println!("    Déjà connecté.");
                    Ok(())
                } else {
                    println!("    Pas connecté pour le moment.");
                    Ok(())
                }
            }
            ProtocolDevice::OSCOutputDevice { name, address, latency, socket } => {
                println!("[~] ProtocolDevice::connect() called for OSCOutputDevice '{}' @ {}", name, address);
                if socket.is_some() {
                    println!("    Already connected.");
                    Ok(())
                } else {
                    let local_addr: SocketAddr = "0.0.0.0:0".parse().expect("Failed to parse local address");
                    let udp_socket = UdpSocket::bind(local_addr)?;
                    println!("    Created UDP socket bound to {}", udp_socket.local_addr()?);
                    *socket = Some(Arc::new(udp_socket));
                    Ok(())
                }
            }
            _ => Ok(())
        }
    }

    /// Envoie un message via le dispositif
    pub fn send(&self, message: ProtocolPayload, time: SyncTime) -> Result<(), ProtocolError> {
        match self {
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                let ProtocolPayload::MIDI(midi_msg) = message else {
                    return Err(ProtocolError("Format de message invalide pour dispositif MIDI !".to_owned()));
                };
                
                let midi_out_guard = midi_out_arc_mutex.lock()
                    .map_err(|_| ProtocolError("MIDIOut Mutex poisoned".to_string()))?;
                midi_out_guard.send(midi_msg).map_err(ProtocolError::from)
            },
            ProtocolDevice::VirtualMIDIOutDevice { name: _, connection: connection_arc_mutex } => {
                let ProtocolPayload::MIDI(midi_msg) = message else {
                    return Err(ProtocolError("Format de message invalide pour dispositif MIDI virtuel !".to_owned()));
                };
                
                let mut conn_opt_guard = connection_arc_mutex.lock()
                    .map_err(|_| ProtocolError("Virtual Connection Mutex poisoned".to_string()))?;
                
                if let Some(conn) = conn_opt_guard.as_mut() {
                    let bytes = midi_msg.to_bytes()?;
                    conn.send(&bytes)
                        .map_err(|e| ProtocolError(format!("Échec d'envoi au MIDI virtuel : {}", e)))
                } else {
                    Err(ProtocolError("Dispositif MIDI virtuel non connecté.".to_string()))
                }
            }
            ProtocolDevice::OSCOutputDevice { name: _, address, latency, socket } => {
                let ProtocolPayload::OSC(crate_osc_msg) = message else {
                     return Err(ProtocolError("Invalid message format for OSC device!".to_owned()));
                 };

                if let Some(sock) = socket {
                    // Convert our OSCMessage args to rosc OscType args
                    let rosc_args = crate_osc_msg.args.into_iter().map(|arg| {
                        match arg {
                            BuboArgument::Int(i) => OscType::Int(i),
                            BuboArgument::Float(f) => OscType::Float(f),
                            BuboArgument::String(s) => OscType::String(s),
                            BuboArgument::Blob(b) => OscType::Blob(b),
                            BuboArgument::Timetag(t) => OscType::Time(OscTime{
                                seconds: (t >> 32) as u32,
                                fractional: (t & 0xFFFFFFFF) as u32,
                            }),
                            // Add other type conversions if needed
                        }
                    }).collect();

                    let rosc_msg = RoscOscMessage {
                        addr: crate_osc_msg.addr,
                        args: rosc_args,
                    };

                    // --- Calculate Timestamp based on Current Time + Latency ---
                    let now = SystemTime::now();
                    let since_epoch = now.duration_since(UNIX_EPOCH)
                        .map_err(|e| ProtocolError(format!("System time error: {}", e)))?;
                    
                    // Calculate total microseconds: current time + latency
                    let latency_micros = (latency * 1_000_000.0) as u64;
                    let target_time_micros = since_epoch.as_micros() as u64 + latency_micros;

                    // Convert target time (microseconds since UNIX epoch) to OscTime (NTP format)
                    const NTP_UNIX_OFFSET_SECS: u64 = 2_208_988_800;
                    let target_time_secs = target_time_micros / 1_000_000;
                    let target_micros_remainder = target_time_micros % 1_000_000;
                    let ntp_secs = target_time_secs + NTP_UNIX_OFFSET_SECS;
                    let ntp_frac = ((target_micros_remainder as f64 / 1_000_000.0) * (1u64 << 32) as f64) as u32;
                    let osc_time = OscTime { seconds: ntp_secs as u32, fractional: ntp_frac };
                    // --- End Timestamp Calculation ---

                    // Create the bundle
                    let bundle = OscBundle {
                        timetag: osc_time,
                        content: vec![OscPacket::Message(rosc_msg)],
                    };

                    // Encode the bundle using the generic encode function
                    match rosc::encoder::encode(&OscPacket::Bundle(bundle)) {
                        Ok(buf) => {
                            sock.send_to(&buf, *address)?;
                            Ok(())
                        }
                        Err(e) => Err(ProtocolError::from(e)),
                    }
                } else {
                    Err(ProtocolError("OSC socket not connected or available".to_string()))
                }
            },
            ProtocolDevice::Log => {
                let ProtocolPayload::LOG(log_msg) = message else {
                     return Err(ProtocolError("Invalid message format for Log device!".to_owned()));
                 };
                 // Simple stdout logging for now
                 println!("[LOG][{}] {}", log_msg.level, log_msg.msg);
                 // Potentially log event details if present
                 if let Some(event) = log_msg.event {
                     println!("    Associated Event: {:?}", event);
                 }
                 Ok(())
            },
            _ => Ok(())
        }
    }

    /// Vide toute file d'attente du dispositif
    pub fn flush(&self) {
        match self {
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                if let Ok(midi_out_guard) = midi_out_arc_mutex.lock() {
                    midi_out_guard.flush();
                } else {
                    eprintln!("[!] Échec de verrou du Mutex MIDIOut pour flush sur le dispositif : {}", 
                              self.address());
                }
            },
            ProtocolDevice::VirtualMIDIOutDevice { name: _, connection: connection_arc_mutex } => {
                if connection_arc_mutex.lock().map_or(false, |guard| guard.is_some()) {
                    println!("[~] Flush appelé sur dispositif VirtualMIDIOutDevice connecté '{}' (no-op pour connexion midir)", 
                             self.address());
                } else {
                    println!("[~] Flush appelé sur dispositif VirtualMIDIOutDevice déconnecté '{}'", 
                             self.address());
                }
            }
            ProtocolDevice::OSCOutputDevice { name, address, latency, socket } => {
                if socket.is_some() {
                    println!("[~] Flush called on connected OSCOutputDevice '{}' @ {} (no-op for UDP)",
                             name, address);
                } else {
                    println!("[~] Flush called on disconnected OSCOutputDevice '{}' @ {}",
                             name, address);
                }
            }
            _ => ()
        }
    }

    /// Obtient l'adresse ou identifiant du dispositif
    pub fn address(&self) -> String {
        match self {
            ProtocolDevice::Log => "log".to_string(),
            ProtocolDevice::OSCInDevice => todo!(),
            ProtocolDevice::MIDIInDevice(midi_in_arc_mutex) => {
                midi_in_arc_mutex.lock().map_or_else(
                    |_| "<MIDIIn Mutex Poisoned>".to_string(),
                    |guard| guard.name.clone()
                )
            },
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                midi_out_arc_mutex.lock().map_or_else(
                    |_| "<MIDIOut Mutex Poisoned>".to_string(),
                    |guard| guard.name.clone()
                )
            },
            ProtocolDevice::VirtualMIDIOutDevice { name, connection: _ } => name.clone(),
            ProtocolDevice::OSCOutputDevice { name, .. } => name.clone(),
        }
    }
}

impl From<MidiOut> for ProtocolDevice {
    fn from(value: MidiOut) -> Self {
        Self::MIDIOutDevice(Arc::new(Mutex::new(value)))
    }
}

impl From<MidiIn> for ProtocolDevice {
    fn from(value: MidiIn) -> Self {
        Self::MIDIInDevice(Arc::new(Mutex::new(value)))
    }
}

/// Message de protocole avec information temporelle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimedMessage {
    pub message: ProtocolMessage,
    pub time: SyncTime
}

impl Display for TimedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ Date : {}", self.message, self.time)
    }
}

impl ProtocolMessage {
    /// Ajoute une information temporelle à un ProtocolMessage pour créer un TimedMessage
    pub fn timed(self, time: SyncTime) -> TimedMessage {
        TimedMessage {
            message: self,
            time
        }
    }
}

impl TimedMessage {
    /// Décompose le TimedMessage en ses composants
    pub fn untimed(self) -> (ProtocolMessage, SyncTime) {
        (self.message, self.time)
    }
}

impl Eq for TimedMessage {}


/// Un TimedMessage est ordonné plus grand si son horodatage est inférieur (ordre inversé sur le temps)
impl Ord for TimedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

impl PartialOrd for TimedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<OSCMessage> for ProtocolPayload {
    fn from(value: OSCMessage) -> Self {
        Self::OSC(value)
    }
}

impl From<MIDIMessage> for ProtocolPayload {
    fn from(value: MIDIMessage) -> Self {
        Self::MIDI(value)
    }
}

impl From<LogMessage> for ProtocolPayload {
    fn from(value: LogMessage) -> Self {
        Self::LOG(value)
    }
}

impl MIDIMessage {
    /// Convertit un message MIDI en octets à envoyer
    pub fn to_bytes(&self) -> Result<Vec<u8>, MidiError> {
        let channel = self.channel;
        match self.payload {
            MIDIMessageType::NoteOn { note, velocity } => 
                Ok(vec![NOTE_ON_MSG + channel, note, velocity]),
                
            MIDIMessageType::NoteOff { note, velocity } => 
                Ok(vec![NOTE_OFF_MSG + channel, note, velocity]),
                
            MIDIMessageType::ControlChange { control, value } => 
                Ok(vec![CONTROL_CHANGE_MSG + channel, control, value]),
                
            MIDIMessageType::ProgramChange { program } => 
                Ok(vec![PROGRAM_CHANGE_MSG + channel, program]),
                
            MIDIMessageType::Aftertouch { note, value } => 
                Ok(vec![AFTERTOUCH_MSG + channel, note, value]),
                
            MIDIMessageType::ChannelPressure { value } => 
                Ok(vec![CHANNEL_PRESSURE_MSG + channel, value]),
                
            MIDIMessageType::PitchBend { value } => Ok(vec![
                PITCH_BEND_MSG + channel,
                (value & 0x7F) as u8,
                (value >> 7) as u8,
            ]),
            
            MIDIMessageType::Clock => Ok(vec![CLOCK_MSG]),
            MIDIMessageType::Continue => Ok(vec![CONTINUE_MSG]),
            MIDIMessageType::Reset => Ok(vec![RESET_MSG]),
            MIDIMessageType::Start => Ok(vec![START_MSG]),
            MIDIMessageType::Stop => Ok(vec![STOP_MSG]),
            
            MIDIMessageType::SystemExclusive { ref data } => {
                let mut message = vec![0xF0];
                message.extend(data);
                message.push(0xF7);
                Ok(message)
            }
            
            MIDIMessageType::Undefined(byte) => Ok(vec![byte]),
        }
    }
}
