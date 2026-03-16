use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;
use sova_core::error::SovaError;
use sova_core::{compiler::CompilationState, vm::language::LanguageDefinition};
use sova_core::protocol::DeviceInfo;
use sova_core::scene::Scene;
use sova_core::schedule::{SchedulerMessage, playback::PlaybackState};
use sova_server::{AudioEngineState, AudioRestartConfig, ClientMessage, ServerMessage, Snapshot, SovaClient};
use tokio::sync::mpsc as tokio_mpsc;

use crate::feedback_engine::FeedbackEngine;
use crate::log_panel::{LogEntry, LogSource};
use crate::widgets::syntax_highlight::CompiledSyntax;

const MAX_CHAT_MESSAGES: usize = 500;

pub struct ChatMessage {
    pub user: String,
    pub message: String,
    pub time: String,
    pub system: bool,
}

fn now_hhmm() -> String {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    unsafe {
        let time = epoch as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        #[cfg(target_os = "windows")] { 
            libc::localtime_s(&mut tm, &time);
        }
        #[cfg(not(target_os = "windows"))] {
            libc::localtime_r(&time, &mut tm);
        }
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
    pub just_connected: bool,

    // State from server
    scene: Option<Scene>,
    positions: Vec<Vec<(usize, usize)>>,
    position_start: Vec<Instant>,
    devices: Vec<DeviceInfo>,
    clock: ClockState,
    audio_state: AudioEngineState,
    scope_data: Vec<f32>,
    peers: Vec<String>,
    confirmed_username: Option<String>,
    languages: Vec<LanguageDefinition>,
    pub syntax_map: HashMap<String, CompiledSyntax>,
    peer_editing: HashMap<(usize, usize), Vec<String>>,
    peer_cursors: HashMap<String, (usize, usize, Option<(usize, usize)>)>,
    chat_messages: Vec<ChatMessage>,
    pub errors: HashMap<(usize, usize), SovaError>,

    // Remote Hydra code from peers
    remote_hydra: Option<(String, String)>,

    // Visual flashes for multiplayer liveness
    pub compilation_flashes: HashMap<(usize, usize), (bool, Instant)>,
    pub mutation_flashes: HashMap<(usize, usize), Instant>,

    // Latest compilation error for toast display
    pub last_error: Option<(String, Instant)>,

    // Incoming script edits from peers
    pub pending_script_edits: Vec<(usize, usize, Vec<sova_server::TextOp>)>,

    // Local audio feedback
    feedback_engine: Option<FeedbackEngine>,

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
            just_connected: false,
            scene: None,
            positions: Vec::new(),
            position_start: Vec::new(),
            devices: Vec::new(),
            clock: ClockState::default(),
            audio_state: AudioEngineState::default(),
            scope_data: Vec::new(),
            peers: Vec::new(),
            confirmed_username: None,
            languages: Vec::new(),
            syntax_map: HashMap::new(),
            peer_editing: HashMap::new(),
            peer_cursors: HashMap::new(),
            chat_messages: Vec::new(),
            errors: HashMap::new(),
            remote_hydra: None,
            compilation_flashes: HashMap::new(),
            mutation_flashes: HashMap::new(),
            last_error: None,
            pending_script_edits: Vec::new(),
            feedback_engine: None,
            send_tx: None,
            event_rx: None,
            runtime,
            ctx,
            log_tx,
        }
    }

    pub fn connect(&mut self, ip: &str, port: u16, username: &str, password: &str, feedback: bool) {
        if matches!(
            self.status,
            ConnectionStatus::Connecting | ConnectionStatus::Connected
        ) {
            return;
        }

        let ip = ip.to_owned();
        let username = username.to_owned();
        let password = if password.is_empty() { None } else { Some(password.to_owned()) };
        let (send_tx, mut send_rx) = tokio_mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::channel();
        let ctx = self.ctx.clone();

        self.send_tx = Some(send_tx);
        self.event_rx = Some(event_rx);
        self.status = ConnectionStatus::Connecting;
        self.error_msg = None;

        self.runtime.spawn(async move {
            let mut client = SovaClient::new(ip, port);

            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                client.connect(),
            )
            .await
            {
                Ok(Err(e)) => {
                    let _ =
                        event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                    ctx.request_repaint();
                    return;
                }
                Err(_) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(
                        "Connection timed out".to_string(),
                    ));
                    ctx.request_repaint();
                    return;
                }
                Ok(Ok(())) => {}
            }

            if let Err(e) = client.send(ClientMessage::SetName { name: username, password }).await {
                let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                ctx.request_repaint();
                return;
            }

            match client.read().await {
                Ok(Some(msg @ ServerMessage::Hello { .. })) => {
                    let _ = event_tx.send(msg);
                    ctx.request_repaint();
                    if feedback
                        && let Err(e) = client.send(ClientMessage::EnableFeedback).await
                    {
                        let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                        ctx.request_repaint();
                        return;
                    }
                }
                Ok(Some(ServerMessage::ConnectionRefused(reason))) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(reason));
                    ctx.request_repaint();
                    let _ = client.disconnect().await;
                    return;
                }
                Ok(Some(_)) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(
                        "Unexpected server response".into(),
                    ));
                    ctx.request_repaint();
                    let _ = client.disconnect().await;
                    return;
                }
                Ok(None) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(
                        "Failed to deserialize handshake".into(),
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

            // Dedicated reader task — never cancelled, so read_wire_frame
            // can't lose partial reads (which would desync the TCP stream).
            let mut reader = client.take_reader().unwrap();
            let read_event_tx = event_tx.clone();
            let read_ctx = ctx.clone();

            let read_task = tokio::spawn(async move {
                loop {
                    match sova_server::read_server_message(&mut reader).await {
                        Ok(msg) => {
                            if read_event_tx.send(msg).is_err() {
                                break;
                            }
                            read_ctx.request_repaint();
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => continue,
                        Err(e) => {
                            let _ = read_event_tx.send(
                                ServerMessage::ConnectionRefused(e.to_string()),
                            );
                            read_ctx.request_repaint();
                            break;
                        }
                    }
                }
            });

            loop {
                match send_rx.recv().await {
                    Some(OutgoingMessage::Send(client_msg)) => {
                        if let Err(e) = client.send(*client_msg).await {
                            let _ = event_tx.send(
                                ServerMessage::ConnectionRefused(e.to_string()),
                            );
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

            read_task.abort();
        });
    }

    pub fn disconnect(&mut self) {
        if let Some(tx) = self.send_tx.take() {
            let _ = tx.send(OutgoingMessage::Disconnect);
        }
        self.event_rx = None;
        self.status = ConnectionStatus::Disconnected;
    }

    pub fn send(&self, msg: ClientMessage) {
        if let Some(tx) = &self.send_tx {
            let _ = tx.send(OutgoingMessage::Send(Box::new(msg)));
        }
    }

    pub fn poll(&mut self) {
        let Some(rx) = &self.event_rx else { return };

        self.compilation_flashes.retain(|_, (_, t)| t.elapsed().as_secs_f32() < 1.0);
        self.mutation_flashes.retain(|_, t| t.elapsed().as_secs_f32() < 1.2);

        while let Ok(msg) = rx.try_recv() {
            match msg {
                ServerMessage::Hello {
                    username,
                    scene,
                    devices,
                    peers,
                    link_state,
                    is_playing,
                    languages,
                    audio_engine_state,
                } => {
                    self.confirmed_username = Some(username);
                    self.scene = Some(scene);
                    self.devices = devices;
                    self.peers = peers;
                    self.languages = languages;
                    for lang_def in self.languages.iter() {
                        if let Some(syn) = &lang_def.syntax
                            && let Some(compiled) = CompiledSyntax::new(syn)
                        {
                            self.syntax_map.insert(lang_def.name.to_owned(), compiled);
                        }
                    }
                    self.clock = ClockState {
                        tempo: link_state.0,
                        beat: link_state.1,
                        phase: 0.0,
                        quantum: link_state.2,
                        playing: is_playing,
                    };
                    self.audio_state = audio_engine_state;
                    self.status = ConnectionStatus::Connected;
                    self.just_connected = true;
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
                    self.errors.clear();
                    self.compilation_flashes.clear();
                    self.mutation_flashes.clear();
                }
                ServerMessage::AddLine(idx, line) => {
                    if let Some(scene) = &mut self.scene
                        && idx <= scene.lines.len()
                    {
                        let now = Instant::now();
                        for fi in 0..line.frames.len() {
                            self.mutation_flashes.insert((idx, fi), now);
                        }
                        scene.lines.insert(idx, line);
                    }
                }
                ServerMessage::RemoveLine(idx) => {
                    if let Some(scene) = &mut self.scene
                        && idx < scene.lines.len()
                    {
                        scene.lines.remove(idx);
                    }
                    self.mutation_flashes.retain(|&(li, _), _| li != idx);
                }
                ServerMessage::AddFrame(li, fi, frame) => {
                    if let Some(line) = self.scene.as_mut().and_then(|s| s.lines.get_mut(li))
                        && fi <= line.frames.len()
                    {
                        line.frames.insert(fi, frame);
                        self.mutation_flashes.insert((li, fi), Instant::now());
                    }
                }
                ServerMessage::RemoveFrame(li, fi) => {
                    if let Some(line) = self.scene.as_mut().and_then(|s| s.lines.get_mut(li))
                        && fi < line.frames.len()
                    {
                        line.frames.remove(fi);
                    }
                    self.errors.remove(&(li, fi));
                    self.mutation_flashes.insert((li, fi), Instant::now());
                }
                ServerMessage::FrameValues(items) => {
                    if let Some(scene) = &mut self.scene {
                        let now = Instant::now();
                        for (li, fi, frame) in items {
                            if let Some(f) =
                                scene.lines.get_mut(li).and_then(|l| l.frames.get_mut(fi))
                            {
                                *f = frame;
                                self.mutation_flashes.insert((li, fi), now);
                            }
                        }
                    }
                }
                ServerMessage::LineValues(items) | ServerMessage::LineConfigurations(items) => {
                    if let Some(scene) = &mut self.scene {
                        let now = Instant::now();
                        for (li, line) in items {
                            for fi in 0..line.frames.len() {
                                self.mutation_flashes.insert((li, fi), now);
                            }
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
                    let now = Instant::now();
                    self.position_start.resize(p.len(), now);
                    for (li, new_pos) in p.iter().enumerate() {
                        if self.positions.get(li) != Some(new_pos) {
                            self.position_start[li] = now;
                        }
                    }
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
                ServerMessage::PeersUpdated(new_peers) => {
                    let time = now_hhmm();
                    for p in &new_peers {
                        if !self.peers.contains(p) {
                            self.chat_messages.push(ChatMessage {
                                time: time.clone(),
                                user: String::new(),
                                message: format!("{p} joined"),
                                system: true,
                            });
                        }
                    }
                    for p in &self.peers {
                        if !new_peers.contains(p) {
                            self.chat_messages.push(ChatMessage {
                                time: time.clone(),
                                user: String::new(),
                                message: format!("{p} left"),
                                system: true,
                            });
                            self.peer_editing.retain(|_, names| {
                                names.retain(|n| n != p);
                                !names.is_empty()
                            });
                            self.peer_cursors.remove(p);
                        }
                    }
                    self.peers = new_peers;
                }
                ServerMessage::CompilationUpdate(li, fi, _id, state) => {
                    self.errors.remove(&(li, fi));
                    match &state {
                        CompilationState::Compiled(_) | CompilationState::Parsed(_) => {
                            self.compilation_flashes.insert((li, fi), (true, Instant::now()));
                            self.last_error = None;
                        }
                        CompilationState::Error(e) => {
                            self.compilation_flashes.insert((li, fi), (false, Instant::now()));
                            self.last_error = Some((
                                format!("L{}:F{} — {}", li, fi, e.info),
                                Instant::now(),
                            ));
                        }
                        _ => {}
                    }
                    if let Some(scene) = &mut self.scene {
                        *scene.get_frame_mut(li, fi).compilation_state_mut() = state;
                    }
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
                        system: false,
                    });
                }
                ServerMessage::PeerStartedEditing(name, li, fi) => {
                    self.peer_editing.entry((li, fi)).or_default().push(name);
                }
                ServerMessage::PeerStoppedEditing(name, li, fi) => {
                    if let Some(names) = self.peer_editing.get_mut(&(li, fi)) {
                        names.retain(|n| n != &name);
                        if names.is_empty() {
                            self.peer_editing.remove(&(li, fi));
                        }
                    }
                }
                ServerMessage::PeerCursorMoved(name, li, fi, tc) => {
                    self.peer_cursors.insert(name, (li, fi, tc));
                }
                ServerMessage::Error(e) => {
                    self.errors.insert((e.line, e.frame), e);
                }
                ServerMessage::FeedbackEnabled { scene, tempo, quantum, is_playing } => {
                    if let Some(engine) = &self.feedback_engine {
                        use sova_core::schedule::ActionTiming;
                        engine.send(SchedulerMessage::SetScene(scene, ActionTiming::Immediate));
                        engine.send(SchedulerMessage::SetTempo(tempo, ActionTiming::Immediate));
                        engine.send(SchedulerMessage::SetQuantum(quantum, ActionTiming::Immediate));
                        if is_playing {
                            engine.send(SchedulerMessage::TransportStart(ActionTiming::Immediate));
                        }
                    }
                }
                ServerMessage::Feedback(msg) => {
                    if let Some(engine) = &self.feedback_engine {
                        engine.send(msg);
                    }
                }
                ServerMessage::HydraCode(sender, code) => {
                    if self.confirmed_username.as_deref() != Some(&sender) {
                        self.remote_hydra = Some((sender, code));
                    }
                }
                ServerMessage::ScriptEdit { sender, li, fi, ops } => {
                    if self.confirmed_username.as_deref() != Some(&sender) {
                        self.pending_script_edits.push((li, fi, ops));
                    }
                }
                ServerMessage::CoreRestarted => {
                    self.errors.clear();
                    self.compilation_flashes.clear();
                    self.mutation_flashes.clear();
                    self.positions.clear();
                    self.position_start.clear();
                }
                ServerMessage::Snapshot(snapshot) => {
                    self.scene = Some(snapshot.scene);
                    self.clock.tempo = snapshot.tempo;
                    self.clock.beat = snapshot.beat;
                    self.clock.quantum = snapshot.quantum;
                    self.devices = snapshot.devices;
                    self.errors.clear();
                    self.compilation_flashes.clear();
                    self.mutation_flashes.clear();
                    self.positions.clear();
                    self.position_start.clear();
                }
                _ => {}
            }
        }
        self.cap_chat();

        if let Some(engine) = &self.feedback_engine {
            self.devices = engine.devices().device_list();
            self.audio_state = engine.audio_state();
            let data = engine.scope_data();
            if !data.is_empty() {
                self.scope_data = data;
            }
        }
    }

    fn clear_state(&mut self) {
        self.scene = None;
        self.positions.clear();
        self.position_start.clear();
        self.devices.clear();
        self.clock = ClockState::default();
        self.audio_state = AudioEngineState::default();
        self.scope_data.clear();
        self.peers.clear();
        self.confirmed_username = None;
        self.languages.clear();
        self.peer_editing.clear();
        self.peer_cursors.clear();
        self.chat_messages.clear();
        self.remote_hydra = None;
        self.compilation_flashes.clear();
        self.mutation_flashes.clear();
        self.feedback_engine = None;
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

    pub fn position_start(&self) -> &[Instant] {
        &self.position_start
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

    pub fn start_feedback(&mut self, audio_config: AudioRestartConfig) {
        match FeedbackEngine::start(audio_config) {
            Ok(engine) => self.feedback_engine = Some(engine),
            Err(e) => eprintln!("Failed to start feedback engine: {}", e),
        }
    }

    pub fn has_feedback(&self) -> bool {
        self.feedback_engine.is_some()
    }

    pub fn restart_audio(&self, config: AudioRestartConfig) {
        if let Some(engine) = &self.feedback_engine {
            engine.restart_audio(config);
        } else {
            self.send(ClientMessage::RestartAudioEngine(config));
        }
    }

    pub fn peers(&self) -> &[String] {
        &self.peers
    }

    pub fn confirmed_username(&self) -> Option<&str> {
        self.confirmed_username.as_deref()
    }

    pub fn set_confirmed_username(&mut self, name: String) {
        self.confirmed_username = Some(name);
    }

    pub fn languages(&self) -> &[LanguageDefinition] {
        &self.languages
    }

    pub fn peer_editing(&self) -> &HashMap<(usize, usize), Vec<String>> {
        &self.peer_editing
    }

    pub fn peer_cursors(&self) -> &HashMap<String, (usize, usize, Option<(usize, usize)>)> {
        &self.peer_cursors
    }

    pub fn text_cursors_for_frame(&self, li: usize, fi: usize) -> Vec<(&str, usize, usize)> {
        self.peer_cursors
            .iter()
            .filter_map(|(name, &(pli, pfi, ref tc))| {
                if pli == li && pfi == fi {
                    tc.map(|(line, col)| (name.as_str(), line, col))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn compilation_flashes(&self) -> &HashMap<(usize, usize), (bool, Instant)> {
        &self.compilation_flashes
    }

    pub fn mutation_flashes(&self) -> &HashMap<(usize, usize), Instant> {
        &self.mutation_flashes
    }

    pub fn compilation_state(&self, li: usize, fi: usize) -> Option<&CompilationState> {
        self.scene().and_then(|s| s.get_frame(li, fi)).map(|f| f.script().compilation_state())
    }

    pub fn chat_messages(&self) -> &[ChatMessage] {
        &self.chat_messages
    }

    pub fn push_chat(&mut self, user: String, message: String) {
        self.chat_messages.push(ChatMessage {
            time: now_hhmm(),
            user,
            message,
            system: false,
        });
        self.cap_chat();
    }

    pub fn send_chat(&self, msg: &str) {
        self.send(ClientMessage::Chat(msg.to_owned()));
    }

    fn cap_chat(&mut self) {
        if self.chat_messages.len() > MAX_CHAT_MESSAGES {
            self.chat_messages
                .drain(..self.chat_messages.len() - MAX_CHAT_MESSAGES);
        }
    }

    pub fn connect_midi(&self, name: &str) {
        if let Some(engine) = &self.feedback_engine {
            let _ = engine.devices().connect_midi_by_name(name);
        } else {
            self.send(ClientMessage::ConnectMidiDeviceByName(name.to_owned()));
        }
    }

    pub fn disconnect_midi(&self, name: &str) {
        if let Some(engine) = &self.feedback_engine {
            let _ = engine.devices().disconnect_midi_by_name(name);
        } else {
            self.send(ClientMessage::DisconnectMidiDeviceByName(name.to_owned()));
        }
    }

    pub fn create_virtual_midi(&self, name: &str) {
        if let Some(engine) = &self.feedback_engine {
            let _ = engine.devices().create_virtual_midi_port(name);
        } else {
            self.send(ClientMessage::CreateVirtualMidiOutput(name.to_owned()));
        }
    }

    pub fn assign_slot(&self, slot: usize, name: &str) {
        if let Some(engine) = &self.feedback_engine {
            let _ = engine.devices().assign_slot(slot, name);
        } else {
            self.send(ClientMessage::AssignDeviceToSlot(slot, name.to_owned()));
        }
    }

    pub fn unassign_slot(&self, slot: usize) {
        if let Some(engine) = &self.feedback_engine {
            let _ = engine.devices().unassign_slot(slot);
        } else {
            self.send(ClientMessage::UnassignDeviceFromSlot(slot));
        }
    }

    pub fn create_osc(&self, name: &str, ip: &str, port: u16) {
        if let Some(engine) = &self.feedback_engine {
            let _ = engine.devices().create_osc_output_device(name, ip, port);
        } else {
            self.send(ClientMessage::CreateOscDevice(name.to_owned(), ip.to_owned(), port));
        }
    }

    pub fn remove_osc(&self, name: &str) {
        if let Some(engine) = &self.feedback_engine {
            let _ = engine.devices().remove_output_device(name);
        } else {
            self.send(ClientMessage::RemoveOscDevice(name.to_owned()));
        }
    }

    pub fn set_latency(&self, name: &str, latency: f64) {
        if let Some(engine) = &self.feedback_engine {
            engine.devices().set_latency(name.to_owned(), latency);
        } else {
            self.send(ClientMessage::SetDeviceLatency(name.to_owned(), latency));
        }
    }

    pub fn take_remote_hydra(&mut self) -> Option<(String, String)> {
        self.remote_hydra.take()
    }

    pub fn send_hydra_code(&self, code: &str) {
        self.send(ClientMessage::HydraCode(code.to_owned()));
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
