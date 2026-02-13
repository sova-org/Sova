use std::collections::HashMap;
use std::sync::mpsc;

use eframe::egui;
use sova_core::compiler::CompilationState;
use sova_core::protocol::DeviceInfo;
use sova_core::scene::Scene;
use sova_core::schedule::playback::PlaybackState;
use sova_server::{AudioEngineState, ClientMessage, ServerMessage, Snapshot, SovaClient};
use tokio::sync::mpsc as tokio_mpsc;

use crate::log_panel::{LogEntry, LogSource};

pub struct ChatMessage {
    pub user: String,
    pub message: String,
    pub time: String,
}

fn now_hhmm() -> String {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    unsafe {
        let time = epoch as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&time, &mut tm);
        format!("{:02}:{:02}", tm.tm_hour, tm.tm_min)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

pub struct ClockState {
    pub tempo: f64,
    pub beat: f64,
    pub phase: f64,
    pub quantum: f64,
    pub playing: bool,
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            tempo: 120.0,
            beat: 0.0,
            phase: 0.0,
            quantum: 4.0,
            playing: false,
        }
    }
}

enum OutgoingMessage {
    Send(Box<ClientMessage>),
    Disconnect,
}

pub struct ClientBridge {
    status: ConnectionStatus,
    error_msg: Option<String>,

    // State from server
    scene: Option<Scene>,
    positions: Vec<Vec<(usize, usize)>>,
    devices: Vec<DeviceInfo>,
    clock: ClockState,
    audio_state: AudioEngineState,
    scope_data: Vec<f32>,
    peers: Vec<String>,
    confirmed_username: Option<String>,
    languages: Vec<String>,
    compilation_states: HashMap<(usize, usize), CompilationState>,
    chat_messages: Vec<ChatMessage>,

    // Communication channels
    send_tx: Option<tokio_mpsc::UnboundedSender<OutgoingMessage>>,
    event_rx: Option<mpsc::Receiver<ServerMessage>>,

    runtime: tokio::runtime::Handle,
    ctx: egui::Context,
    log_tx: mpsc::Sender<LogEntry>,
}

impl ClientBridge {
    pub fn new(
        runtime: tokio::runtime::Handle,
        ctx: egui::Context,
        log_tx: mpsc::Sender<LogEntry>,
    ) -> Self {
        Self {
            status: ConnectionStatus::Disconnected,
            error_msg: None,
            scene: None,
            positions: Vec::new(),
            devices: Vec::new(),
            clock: ClockState::default(),
            audio_state: AudioEngineState::default(),
            scope_data: Vec::new(),
            peers: Vec::new(),
            confirmed_username: None,
            languages: Vec::new(),
            compilation_states: HashMap::new(),
            chat_messages: Vec::new(),
            send_tx: None,
            event_rx: None,
            runtime,
            ctx,
            log_tx,
        }
    }

    pub fn connect(&mut self, ip: &str, port: u16, username: &str) {
        if matches!(self.status, ConnectionStatus::Connecting | ConnectionStatus::Connected) {
            return;
        }

        let ip = ip.to_owned();
        let username = username.to_owned();
        let (send_tx, mut send_rx) = tokio_mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::channel();
        let ctx = self.ctx.clone();

        self.send_tx = Some(send_tx);
        self.event_rx = Some(event_rx);
        self.status = ConnectionStatus::Connecting;
        self.error_msg = None;

        self.runtime.spawn(async move {
            let mut client = SovaClient::new(ip, port);

            if let Err(e) = client.connect().await {
                let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                ctx.request_repaint();
                return;
            }

            if let Err(e) = client.send(ClientMessage::SetName(username)).await {
                let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                ctx.request_repaint();
                return;
            }

            match client.read().await {
                Ok(msg @ ServerMessage::Hello { .. }) => {
                    let _ = event_tx.send(msg);
                    ctx.request_repaint();
                }
                Ok(ServerMessage::ConnectionRefused(reason)) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(reason));
                    ctx.request_repaint();
                    let _ = client.disconnect().await;
                    return;
                }
                Ok(_) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(
                        "Unexpected server response".into(),
                    ));
                    ctx.request_repaint();
                    let _ = client.disconnect().await;
                    return;
                }
                Err(e) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                    ctx.request_repaint();
                    return;
                }
            }

            loop {
                tokio::select! {
                    msg = client.read() => {
                        match msg {
                            Ok(server_msg) => {
                                let _ = event_tx.send(server_msg);
                                ctx.request_repaint();
                            }
                            Err(e) => {
                                let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                                ctx.request_repaint();
                                break;
                            }
                        }
                    }
                    cmd = send_rx.recv() => {
                        match cmd {
                            Some(OutgoingMessage::Send(client_msg)) => {
                                if let Err(e) = client.send(*client_msg).await {
                                    let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                                    ctx.request_repaint();
                                    break;
                                }
                            }
                            Some(OutgoingMessage::Disconnect) | None => {
                                let _ = client.disconnect().await;
                                let _ = event_tx.send(ServerMessage::ConnectionRefused(String::new()));
                                ctx.request_repaint();
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn disconnect(&mut self) {
        if let Some(tx) = self.send_tx.take() {
            let _ = tx.send(OutgoingMessage::Disconnect);
        }
    }

    pub fn send(&self, msg: ClientMessage) {
        if let Some(tx) = &self.send_tx {
            let _ = tx.send(OutgoingMessage::Send(Box::new(msg)));
        }
    }

    pub fn poll(&mut self) {
        let Some(rx) = &self.event_rx else { return };

        while let Ok(msg) = rx.try_recv() {
            match msg {
                ServerMessage::Hello {
                    username,
                    scene,
                    devices,
                    peers,
                    link_state,
                    is_playing,
                    available_languages,
                    audio_engine_state,
                } => {
                    self.confirmed_username = Some(username);
                    self.scene = Some(scene);
                    self.devices = devices;
                    self.peers = peers;
                    self.languages = available_languages;
                    self.clock = ClockState {
                        tempo: link_state.0,
                        beat: link_state.1,
                        phase: 0.0,
                        quantum: link_state.2,
                        playing: is_playing,
                    };
                    self.audio_state = audio_engine_state;
                    self.status = ConnectionStatus::Connected;
                }
                ServerMessage::ConnectionRefused(reason) => {
                    self.clear_state();
                    if reason.is_empty() {
                        self.status = ConnectionStatus::Disconnected;
                        self.error_msg = None;
                    } else {
                        self.status = ConnectionStatus::Error;
                        self.error_msg = Some(reason);
                    }
                    self.send_tx = None;
                    self.event_rx = None;
                    return;
                }
                ServerMessage::SceneValue(s) => {
                    self.scene = Some(s);
                }
                ServerMessage::AddLine(idx, line) => {
                    if let Some(scene) = &mut self.scene
                        && idx <= scene.lines.len()
                    {
                        scene.lines.insert(idx, line);
                    }
                }
                ServerMessage::RemoveLine(idx) => {
                    if let Some(scene) = &mut self.scene
                        && idx < scene.lines.len()
                    {
                        scene.lines.remove(idx);
                    }
                }
                ServerMessage::AddFrame(li, fi, frame) => {
                    if let Some(line) = self.scene.as_mut().and_then(|s| s.lines.get_mut(li))
                        && fi <= line.frames.len()
                    {
                        line.frames.insert(fi, frame);
                    }
                }
                ServerMessage::RemoveFrame(li, fi) => {
                    if let Some(line) = self.scene.as_mut().and_then(|s| s.lines.get_mut(li))
                        && fi < line.frames.len()
                    {
                        line.frames.remove(fi);
                    }
                }
                ServerMessage::FrameValues(items) => {
                    if let Some(scene) = &mut self.scene {
                        for (li, fi, frame) in items {
                            if let Some(f) = scene
                                .lines
                                .get_mut(li)
                                .and_then(|l| l.frames.get_mut(fi))
                            {
                                *f = frame;
                            }
                        }
                    }
                }
                ServerMessage::LineValues(items) | ServerMessage::LineConfigurations(items) => {
                    if let Some(scene) = &mut self.scene {
                        for (li, line) in items {
                            if let Some(l) = scene.lines.get_mut(li) {
                                *l = line;
                            }
                        }
                    }
                }
                ServerMessage::SceneMode(mode) => {
                    if let Some(scene) = &mut self.scene {
                        scene.mode = mode;
                    }
                }
                ServerMessage::FramePosition(p) => {
                    self.positions = p;
                }
                ServerMessage::ClockState(tempo, beat, _micros, quantum) => {
                    self.clock.tempo = tempo;
                    self.clock.beat = beat;
                    self.clock.phase = if quantum > 0.0 { beat % quantum } else { 0.0 };
                    self.clock.quantum = quantum;
                }
                ServerMessage::PlaybackStateChanged(state) => {
                    self.clock.playing = !matches!(state, PlaybackState::Stopped);
                }
                ServerMessage::DeviceList(devices) => {
                    self.devices = devices;
                }
                ServerMessage::AudioEngineState(state) => {
                    self.audio_state = state;
                }
                ServerMessage::ScopeData(data) => {
                    self.scope_data = data;
                }
                ServerMessage::PeersUpdated(peers) => {
                    self.peers = peers;
                }
                ServerMessage::CompilationUpdate(li, fi, _id, state) => {
                    self.compilation_states.insert((li, fi), state);
                }
                ServerMessage::Log(msg) => {
                    let _ = self.log_tx.send(LogEntry {
                        source: LogSource::Client,
                        message: msg,
                    });
                }
                ServerMessage::Chat(user, message) => {
                    self.chat_messages.push(ChatMessage {
                        time: now_hhmm(),
                        user,
                        message,
                    });
                }
                _ => {}
            }
        }
    }

    fn clear_state(&mut self) {
        self.scene = None;
        self.positions.clear();
        self.devices.clear();
        self.clock = ClockState::default();
        self.audio_state = AudioEngineState::default();
        self.scope_data.clear();
        self.peers.clear();
        self.confirmed_username = None;
        self.languages.clear();
        self.compilation_states.clear();
        self.chat_messages.clear();
    }

    pub fn status(&self) -> ConnectionStatus {
        self.status
    }

    pub fn error_msg(&self) -> Option<&str> {
        self.error_msg.as_deref()
    }

    pub fn is_connected(&self) -> bool {
        self.status == ConnectionStatus::Connected
    }

    pub fn scene(&self) -> Option<&Scene> {
        self.scene.as_ref()
    }

    pub fn positions(&self) -> &[Vec<(usize, usize)>] {
        &self.positions
    }

    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }

    pub fn clock(&self) -> &ClockState {
        &self.clock
    }

    pub fn audio_state(&self) -> &AudioEngineState {
        &self.audio_state
    }

    pub fn scope_data(&self) -> &[f32] {
        &self.scope_data
    }

    pub fn peers(&self) -> &[String] {
        &self.peers
    }

    pub fn confirmed_username(&self) -> Option<&str> {
        self.confirmed_username.as_deref()
    }

    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    pub fn compilation_state(&self, li: usize, fi: usize) -> Option<&CompilationState> {
        self.compilation_states.get(&(li, fi))
    }

    pub fn chat_messages(&self) -> &[ChatMessage] {
        &self.chat_messages
    }

    pub fn push_chat(&mut self, user: String, message: String) {
        self.chat_messages.push(ChatMessage {
            time: now_hhmm(),
            user,
            message,
        });
    }

    pub fn send_chat(&self, msg: &str) {
        self.send(ClientMessage::Chat(msg.to_owned()));
    }

    pub fn build_snapshot(&self) -> Option<Snapshot> {
        let scene = self.scene.as_ref()?.clone();
        Some(Snapshot {
            scene,
            tempo: self.clock.tempo,
            beat: self.clock.beat,
            micros: 0,
            quantum: self.clock.quantum,
            devices: self.devices.clone(),
        })
    }
}
