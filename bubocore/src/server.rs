use std::{
    io::ErrorKind, net::SocketAddrV4, sync::{mpsc::Sender, Arc}
};

use client::ClientMessage;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::{watch, Mutex},
};

use crate::{
    clock::{Clock, ClockServer, SyncTime}, device_map::DeviceMap, pattern::Pattern, protocol::TimedMessage, schedule::{SchedulerMessage, SchedulerNotification}
};

pub mod client;

pub const ENDING_BYTE: u8 = 0x07;
pub const DEFAULT_CLIENT_NAME: &str = "Unkown musician";

#[derive(Clone)]
pub struct ServerState {
    pub clock_server: Arc<ClockServer>,
    pub devices: Arc<DeviceMap>,
    pub world_iface: Sender<TimedMessage>,
    pub sched_iface: Sender<SchedulerMessage>,
    pub update_notifier: watch::Receiver<SchedulerNotification>,
    pub clients: Arc<Mutex<Vec<String>>>,
    pub pattern_image: Arc<Mutex<Pattern>>,
    pub client_name: String,
}

impl ServerState {

    pub fn new(
        pattern_image : Arc<Mutex<Pattern>>,
        clock_server : Arc<ClockServer>, 
        devices : Arc<DeviceMap>, 
        world_iface : Sender<TimedMessage>,
        sched_iface : Sender<SchedulerMessage>,
        update_notifier : watch::Receiver<SchedulerNotification>,
    ) -> Self {
        Self {
            pattern_image,
            clock_server,
            devices,
            world_iface,
            sched_iface,
            update_notifier,
            clients: Default::default(),
            client_name: DEFAULT_CLIENT_NAME.to_owned(),
        }
    }

}

pub struct BuboCoreServer {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    LogMessage(TimedMessage),
    StepPosition(Vec<usize>),
    Hello { pattern : Pattern, devices : Vec<(String, String)>, clients : Vec<String> },
    PatternValue(Pattern),
    PatternLayout(Vec<Vec<(f64, bool)>>),
    ClockState(f64, f64, SyncTime, f64),
    Success,
    InternalError,
}

async fn generate_hello(state : &ServerState) -> ServerMessage {
    ServerMessage::Hello { 
        pattern: state.pattern_image.lock().await.clone(), 
        devices: state.devices.device_list(), 
        clients: state.clients.lock().await.clone(),
    }
}

async fn on_message(msg: ClientMessage, mut state: ServerState) -> ServerMessage {
    match msg {
        ClientMessage::SetName(name) => {
            let mut guard = state.clients.lock().await;
            let Some(i) = guard.iter().position(|x| *x == state.client_name) else {
                return ServerMessage::InternalError;
            };
            guard[i] = name.clone();
            state.client_name = name;
            ServerMessage::Success
        },
        ClientMessage::SchedulerControl(sched_msg) => {
            println!("[📅] Sending scheduler message");
            if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success
            } else {
                ServerMessage::InternalError
            }
        },
        ClientMessage::SetTempo(tempo) => {
            println!("[🕒] Setting tempo to {}", tempo);
            let mut clock = Clock::from(state.clock_server);
            clock.set_tempo(tempo);
            ServerMessage::Success
        },
        ClientMessage::GetClock => {
            println!("[🕒] Sending clock state");
            let clock = Clock::from(state.clock_server);
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        },
        _ => ServerMessage::Success,
    }
}

fn generate_update_message(pattern: &SchedulerNotification) -> ServerMessage {
    match pattern {
        SchedulerNotification::Log(msg) => ServerMessage::LogMessage(msg.clone()),
        // TODO: implement more responses (see schedule.rs)
        _ => todo!(),
    }
}

async fn send_msg(socket: &mut TcpStream, msg : ServerMessage) -> io::Result<()> {
    let Ok(mut res) = serde_json::to_vec(&msg) else {
        return Err(ErrorKind::InvalidData.into());
    };
    res.push(ENDING_BYTE);
    socket.write_all(&res).await?;
    Ok(())
}

async fn process_client(mut socket: TcpStream, mut state: ServerState) -> io::Result<()> {
    let mut buff = Vec::new();
    let mut ready_check = [0];
    send_msg(&mut socket, generate_hello(&state).await).await?;
    loop {
        select! {
            a = state.update_notifier.changed() => {
                if a.is_err() {
                    return Ok(())
                }
                let res = generate_update_message(&state.update_notifier.borrow());
                send_msg(&mut socket, res).await?;
            },
            _ = socket.peek(&mut ready_check) => {
                let mut buf_reader = BufReader::new(&mut socket);
                let n = buf_reader.read_until(ENDING_BYTE, &mut buff).await?;
                if n == 0 {
                    return Ok(());
                }
                buff.pop();
                if let Ok(msg) = serde_json::from_slice::<ClientMessage>(&buff) {
                    let res = on_message(msg, state.clone()).await;
                    send_msg(&mut socket, res).await?;
                }
                buff.clear();
            }
        };
    }
}

impl BuboCoreServer {

    pub fn new(ip : String, port : u16) -> Self {
        Self { ip, port }
    }

    pub async fn start(&self, state: ServerState) -> io::Result<()> {
        println!("[↕] Starting server");
        let addr = SocketAddrV4::new(self.ip.parse().unwrap(), self.port);
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (socket, c_addr) = tokio::select! {
                _ = signal::ctrl_c() => return Ok(()),
                res = listener.accept() => res.unwrap()
            };
            println!("[🎺] New client connected {}", c_addr);
            let client_state = state.clone();

            state.clients.lock().await.push(state.client_name.clone());

            tokio::spawn(async move {
                let _ = process_client(socket, client_state).await;
                println!("[👋] Client disconnected {}", c_addr);
            });
        }
    }
}
