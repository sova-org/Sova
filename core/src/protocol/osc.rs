use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType};
use std::collections::HashMap;
use std::fmt;
use std::net::{SocketAddr, UdpSocket};

use crate::clock::TimeSpan;
use crate::vm::variable::VariableValue;
use crate::protocol::error::ProtocolError;

mod message;
pub use message::*;

mod osc_input;
pub use osc_input::*;

pub fn variable_from_osc(value: OscType) -> Option<VariableValue> {
    match value {
        OscType::Int(i) => Some(VariableValue::Integer(i as i64)),
        OscType::Float(f) => Some(VariableValue::Float(f as f64)),
        OscType::String(s) => Some(VariableValue::Str(s)),
        OscType::Blob(items) => Some(VariableValue::Blob(items)),
        OscType::Time(_osc_time) => {
            None
        }
        OscType::Long(i) => Some(VariableValue::Integer(i)),
        OscType::Double(d) => Some(VariableValue::Float(d)),
        OscType::Char(c) => Some(VariableValue::Str(c.to_string())),
        OscType::Midi(midi) => {
            let mut map = HashMap::new();
            map.insert("port".to_owned(), (midi.port as i64).into());
            map.insert("data1".to_owned(), (midi.data1 as i64).into());
            map.insert("data2".to_owned(), (midi.data2 as i64).into());
            Some(VariableValue::Map(map))
        },
        OscType::Bool(b) => Some(VariableValue::Bool(b)),
        OscType::Array(osc_array) => {
            let values = osc_array.content.into_iter().filter_map(|x| variable_from_osc(x)).collect();
            Some(VariableValue::Vec(values))
        }
        _ => None
    }
}

pub fn osc_from_variable(value: VariableValue) -> Option<OscType> {
    match value {
        VariableValue::Integer(i) => Some(OscType::Int(i as i32)),
        VariableValue::Float(f) => Some(OscType::Float(f as f32)),
        VariableValue::Decimal(d) => {
            let f = f64::from(d);
            Some(OscType::Float(f as f32))
        }
        VariableValue::Str(s) => Some(OscType::String(s)),
        VariableValue::Blob(b) => Some(OscType::Blob(b)),
        VariableValue::Dur(t) => {
            let TimeSpan::Micros(t) = t else {
                return None
            };
            let secs = t / 1_000_000;
            Some(OscType::Time(OscTime {
                seconds: secs as u32,
                fractional: (t - secs) as u32,
            }))
        },
        _ => None
    }
}

pub struct OSCOut {
    /// User-defined name to identify this device.
    pub name: String,
    /// The network address (IP and port) for destination OSC messages.
    pub address: SocketAddr,
    /// The UDP socket used for sending, managed in a thread-safe manner.
    pub socket: Option<UdpSocket>,
}

impl OSCOut {

    pub fn connect(&mut self) -> Result<(), ProtocolError> {
        crate::log_println!(
            "[~] connect() called for OSCOutDevice '{}' @ {}",
            self.name, self.address
        );
        if self.socket.is_some() {
            crate::log_println!("    Already connected.");
            return Ok(());
        } 
        // Bind to any available local port for sending
        let local_addr: SocketAddr = "0.0.0.0:0"
            .parse()
            .expect("Failed to parse local UDP bind address");
        match UdpSocket::bind(local_addr) {
            Ok(udp_socket) => {
                crate::log_println!(
                    "    Created UDP socket bound to {}",
                    udp_socket.local_addr()?
                );
                self.socket = Some(udp_socket);
                Ok(())
            }
            Err(e) => {
                crate::log_eprintln!(
                    "[!] Failed to bind UDP socket for OSCOutDevice '{}': {}",
                    self.name, e
                );
                Err(ProtocolError::from(e))
            }
        }
    }

    pub fn send(&self, message: OSCMessage) -> Result<(), ProtocolError> {
        if let Some(sock) = &self.socket {
            // Convert our internal OSC Arguments to rosc::OscType arguments
            let rosc_args: Result<Vec<OscType>, rosc::OscError> = message
                .args
                .into_iter()
                .map(|arg| {
                    osc_from_variable(arg)
                        .map(|x| Ok(x))
                        .unwrap_or(Err(rosc::OscError::Unimplemented))
                })
                .collect();
            let rosc_args = rosc_args?; // Propagate potential conversion errors

            let rosc_msg = OscMessage {
                addr: message.addr,
                args: rosc_args,
            };
            let rosc_msg = OscPacket::Message(rosc_msg);

            let packet = if let Some(timetag) = message.timetag {

                // Create an OSC bundle containing the single message with the calculated timetag
                OscPacket::Bundle(OscBundle {
                    timetag: timetag.into(),
                    content: vec![rosc_msg],
                })
            } else {
                rosc_msg
            };

            match rosc::encoder::encode(&packet) {
                Ok(buf) => {
                    // Send the encoded buffer to the target address
                    sock.send_to(&buf, self.address).map_err(ProtocolError::from)?; // Convert IO error
                    Ok(())
                }
                Err(e) => Err(ProtocolError::from(e)), // Convert OSC encoding error
            }
        } else {
            Err(ProtocolError(format!(
                "OSC device '{}' socket not connected.",
                self.name
            )))
        }
    }

}

impl fmt::Debug for OSCOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show socket status (bound/unbound) rather than the object itself
        let socket_status = if self.socket.is_some() {
            "<Bound>"
        } else {
            "<Unbound>"
        };
        f.debug_struct("OSCOutDevice")
            .field("name", &self.name)
            .field("address", &self.address)
            .field("socket", &socket_status)
            .finish()
    }
}