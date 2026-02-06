use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType};
use std::fmt;
use std::net::{SocketAddr, UdpSocket};

use crate::clock::TimeSpan;
use crate::vm::variable::VariableValue;
use crate::protocol::error::ProtocolError;
use crate::util::decimal_operations::float64_from_decimal;

mod message;
pub use message::*;

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
            Ok(())
        } else {
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
    }

    pub fn send(&self, message: OSCMessage) -> Result<(), ProtocolError> {
        if let Some(sock) = &self.socket {
            // Convert our internal OSC Arguments to rosc::OscType arguments
            let rosc_args: Result<Vec<OscType>, rosc::OscError> = message
                .args
                .into_iter()
                .map(|arg| {
                    match arg {
                        VariableValue::Integer(i) => Ok(OscType::Int(i as i32)),
                        VariableValue::Float(f) => Ok(OscType::Float(f as f32)),
                        VariableValue::Decimal(sign, num, den) => {
                            let f = float64_from_decimal(sign, num, den);
                            Ok(OscType::Float(f as f32))
                        }
                        VariableValue::Str(s) => Ok(OscType::String(s)),
                        VariableValue::Blob(b) => Ok(OscType::Blob(b)),
                        VariableValue::Dur(t) => {
                            let TimeSpan::Micros(t) = t else {
                                return Err(rosc::OscError::Unimplemented);
                            };
                            Ok(OscType::Time(OscTime {
                                seconds: (t >> 32) as u32,
                                fractional: (t & 0xFFFFFFFF) as u32,
                            }))
                        },
                        _ => Err(rosc::OscError::Unimplemented)
                        // ... etc.
                    }
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