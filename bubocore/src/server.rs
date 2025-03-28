use std::{
    net::SocketAddrV4,
    sync::{Arc, mpsc::Sender},
};

use client::ClientMessage;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::watch,
};

use crate::{
    clock::{Clock, ClockServer, SyncTime},
    pattern::Pattern,
    protocol::TimedMessage,
    schedule::{SchedulerMessage, SchedulerNotification},
};

pub mod client;

pub const ENDING_BYTE: u8 = 0x07;

#[derive(Clone)]
pub struct ServerState {
    pub clock_server: Arc<ClockServer>,
    pub world_iface: Sender<TimedMessage>,
    pub sched_iface: Sender<SchedulerMessage>,
    pub update_notifier: watch::Receiver<SchedulerNotification>,
}

pub struct BuboCoreServer {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    LogMessage(TimedMessage),
    StepPosition(Vec<usize>),
    PatternValue(Pattern),
    PatternLayout(Vec<Vec<(f64, bool)>>),
    ClockState(f64, f64, SyncTime, f64),
    Success,
    InternalError,
}

async fn on_message(msg: ClientMessage, state: ServerState) -> ServerMessage {
    match msg {
        ClientMessage::SchedulerControl(sched_msg) => {
            println!("[📅] Sending scheduler message");
            if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success
            } else {
                ServerMessage::InternalError
            }
        }
        ClientMessage::SetTempo(tempo) => {
            println!("[🕒] Setting tempo to {}", tempo);
            let mut clock = Clock::from(state.clock_server);
            clock.set_tempo(tempo);
            ServerMessage::Success
        }
        ClientMessage::GetClock => {
            println!("[🕒] Sending clock state");
            let clock = Clock::from(state.clock_server);
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        }
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

async fn process_client(mut socket: TcpStream, mut state: ServerState) -> io::Result<()> {
    let mut buff = Vec::new();
    let mut ready_check = [0];
    loop {
        select! {
            a = state.update_notifier.changed() => {
                if a.is_err() {
                    return Ok(())
                }
                let res = generate_update_message(&state.update_notifier.borrow());
                let Ok(mut res) = serde_json::to_vec(&res) else {
                    continue;
                };
                res.push(ENDING_BYTE);
                socket.write_all(&res).await?;
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
                    let Ok(mut res) = serde_json::to_vec(&res) else {
                        continue;
                    };
                    res.push(ENDING_BYTE);
                    socket.write_all(&res).await?;
                }
                buff.clear();
            }
        };
    }
}

impl BuboCoreServer {
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
            tokio::spawn(async move {
                let _ = process_client(socket, client_state).await;
                println!("[👋] Client disconnected {}", c_addr);
            });
        }
    }
}
