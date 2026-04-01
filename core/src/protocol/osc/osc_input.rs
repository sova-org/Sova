use std::{
    collections::BTreeMap,
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use rosc::{OscPacket, OscTime};

use crate::{
    clock::{Clock, SyncTime},
    log_eprintln,
    protocol::{ProtocolError, osc::variable_from_osc},
    vm::variable::VariableValue,
};

#[derive(Debug)]
pub struct OSCDictionary {
    pub timetag: Option<OscTime>,
    pub args: Vec<VariableValue>,
}

#[derive(Debug)]
pub struct OSCIn {
    /// User-defined name to identify this device.
    pub name: String,
    /// The network port for destination OSC messages.
    pub port: u16,
    socket_handle: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    memory: Arc<Mutex<BTreeMap<String, OSCDictionary>>>,
}

fn process_packet(
    packet: OscPacket,
    memory: &mut BTreeMap<String, OSCDictionary>,
    timetag: Option<OscTime>,
) {
    match packet {
        OscPacket::Message(osc_message) => {
            let route = osc_message.addr;
            let args = osc_message
                .args
                .into_iter()
                .filter_map(variable_from_osc)
                .collect();
            let dic = OSCDictionary { timetag, args };
            memory.insert(route, dic);
        }
        OscPacket::Bundle(osc_bundle) => {
            let tag = osc_bundle.timetag;
            for msg in osc_bundle.content {
                process_packet(msg, memory, Some(tag));
            }
        }
    }
}

impl OSCIn {
    pub fn address(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }

    pub fn new(name: String, port: u16) -> Self {
        Self {
            name,
            port,
            socket_handle: None,
            shutdown: Arc::new(AtomicBool::new(false)),
            memory: Default::default(),
        }
    }

    pub fn is_connected(&self) -> bool {
        match &self.socket_handle {
            Some(s) => !s.is_finished(),
            None => false,
        }
    }

    pub fn connect(&mut self) -> Result<(), ProtocolError> {
        crate::log_println!(
            "[~] connect() called for OSCInDevice '{}' @ port {}",
            self.name,
            self.port
        );
        if self.socket_handle.is_some() {
            crate::log_println!("    Already connected.");
            return Ok(());
        }
        let addr = self.address().parse();
        if addr.is_err() {
            return Err(format!("Unable to bind socket on port {}", self.port).into());
        }
        let addr: SocketAddr = addr.unwrap();
        match UdpSocket::bind(addr) {
            Ok(socket) => {
                match socket.set_read_timeout(Some(Duration::from_millis(500))) {
                    Err(e) => {
                        log_eprintln!("[!] Unable to set read timeout for UDP socket !");
                        return Err(e.into());
                    }
                    _ => (),
                }
                let shutdown_signal = self.shutdown.clone();
                let mut buff = [0; 4096];
                let memory = self.memory.clone();
                let handle = thread::spawn(move || {
                    loop {
                        if shutdown_signal.load(Ordering::Relaxed) {
                            break;
                        }
                        match socket.recv(&mut buff) {
                            Ok(bytes) => match rosc::decoder::decode_udp(&buff[..bytes]) {
                                Ok((_, packet)) => {
                                    let mut mem_access = memory.lock().unwrap();
                                    process_packet(packet, &mut mem_access, None);
                                }
                                Err(e) => {
                                    log_eprintln!("[!] OSC input error : {e}");
                                }
                            },
                            Err(e) => {
                                log_eprintln!("[!] UDP socket error : {e} !");
                            }
                        }
                    }
                });
                self.socket_handle = Some(handle);
            }
            Err(e) => {
                crate::log_eprintln!(
                    "[!] Failed to bind UDP socket for OSCInDevice '{}': {}",
                    self.name,
                    e
                );
                return Err(ProtocolError::from(e));
            }
        }
        todo!()
    }

    pub fn values(&self, route: &str) -> Vec<VariableValue> {
        self.memory
            .lock()
            .unwrap()
            .get(route)
            .map(|dic| dic.args.clone())
            .unwrap_or_default()
    }

    pub fn timetag(&self, route: &str, clock: &Clock) -> Option<SyncTime> {
        self.memory
            .lock()
            .unwrap()
            .get(route)
            .and_then(|dic| dic.timetag)
            .map(|timetag| clock.from_system_time(timetag.into()))
    }

    pub fn get(&self, route: &str, clock: &Clock) -> (Vec<VariableValue>, Option<SyncTime>) {
        let guard = self.memory.lock().unwrap();
        let Some(dic) = guard.get(route) else {
            return Default::default();
        };
        let value = dic.args.clone();
        let timetag = dic.timetag.map(|t| clock.from_system_time(t.into()));
        (value, timetag)
    }

    pub fn disconnect(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    pub fn wait_disconnect(&mut self) {
        self.disconnect();
        match std::mem::take(&mut self.socket_handle) {
            Some(h) => {
                let _ = h.join();
            }
            None => (),
        }
    }
}

impl Drop for OSCIn {
    fn drop(&mut self) {
        self.disconnect();
    }
}
