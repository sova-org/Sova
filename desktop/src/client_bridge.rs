mod connection;
mod presence;
mod scene_cache;
mod server_messages;

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;
use sova_core::error::SovaError;
use sova_core::protocol::DeviceInfo;
use sova_core::scene::Scene;
use sova_core::vm::interpreter::Annotation;
use sova_core::vm::language::LanguageDefinition;
use sova_server::{AudioEngineState, ClientMessage, FrameTextId, ServerMessage};
use tokio::sync::mpsc as tokio_mpsc;

use crate::feedback_engine::FeedbackEngine;
use crate::panels::log_panel::LogEntry;
use crate::widgets::syntax_highlight::CompiledSyntax;

const MAX_CHAT_MESSAGES: usize = 500;
const COMPILATION_FLASH_SECS: f32 = 1.0;
const MUTATION_FLASH_SECS: f32 = 1.2;
const SCENE_HISTORY_CAP: usize = 50;

pub struct ChatMessage {
    pub user: String,
    pub message: String,
    pub time: String,
    pub system: bool,
}

fn now_hhmm() -> String {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("system clock is before UNIX epoch")
        .as_secs();
    let time = epoch as libc::time_t;
    // SAFETY: `tm` is a freshly zero-initialised stack value and we pass
    // stable references to owned locals. Both `localtime_r` and
    // `localtime_s` are documented to accept any valid non-null pointer
    // pair and to not retain the pointers past the call.
    let tm = unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        #[cfg(target_os = "windows")]
        libc::localtime_s(&mut tm, &time);
        #[cfg(not(target_os = "windows"))]
        libc::localtime_r(&time, &mut tm);
        tm
    };
    format!("{:02}:{:02}", tm.tm_hour, tm.tm_min)
}

fn scope_signature(buf: &[f32]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    buf.len().hash(&mut h);
    if let Some(&first) = buf.first() {
        first.to_bits().hash(&mut h);
    }
    if let Some(&last) = buf.last() {
        last.to_bits().hash(&mut h);
    }
    if !buf.is_empty() {
        buf[buf.len() / 2].to_bits().hash(&mut h);
    }
    h.finish()
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
    pub num_peers: u32,
    pub start_stop_sync: bool,
    pub link_enabled: bool,
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            tempo: 120.0,
            beat: 0.0,
            phase: 0.0,
            quantum: 4.0,
            playing: false,
            num_peers: 0,
            start_stop_sync: true,
            link_enabled: true,
        }
    }
}

enum OutgoingMessage {
    Send(Box<ClientMessage>),
    Disconnect,
}

enum BridgeEvent {
    Server(Box<ServerMessage>),
    LocalDisconnected,
}

// Thin wrapper around mpsc::Sender<BridgeEvent> that hides the wrapping so
// call sites in `connect()` can still write `event_tx.send(ServerMessage::X)`.
#[derive(Clone)]
struct EventSender(mpsc::Sender<BridgeEvent>);

impl EventSender {
    fn send(&self, msg: ServerMessage) -> Result<(), mpsc::SendError<BridgeEvent>> {
        self.0.send(BridgeEvent::Server(Box::new(msg)))
    }

    fn send_local_disconnect(&self) {
        let _ = self.0.send(BridgeEvent::LocalDisconnected);
    }
}

pub struct ClientBridge {
    status: ConnectionStatus,
    error_msg: Option<String>,
    pub just_connected: bool,

    // State from server
    scene: Option<Scene>,
    positions: Vec<Vec<(usize, usize)>>,
    position_start_beat: Vec<Vec<f64>>,
    devices: Vec<DeviceInfo>,
    clock: ClockState,
    audio_state: AudioEngineState,
    scope_data: Vec<f32>,
    scope_generation: u64,
    peak_data: Vec<f32>,
    peers: Vec<String>,
    confirmed_username: Option<String>,
    languages: Vec<LanguageDefinition>,
    pub syntax_map: HashMap<String, CompiledSyntax>,
    peer_editing: HashMap<(usize, usize), Vec<String>>,
    chat_messages: VecDeque<ChatMessage>,
    pub errors: HashMap<(usize, usize), SovaError>,
    annotations: Vec<Vec<Vec<Annotation>>>,

    // Remote Hydra code from peers
    remote_hydra: Option<(String, String)>,

    // Visual flashes for multiplayer liveness
    pub compilation_flashes: HashMap<(usize, usize), (bool, Instant)>,
    pub mutation_flashes: HashMap<(usize, usize), Instant>,

    // Latest error (compile or runtime) for toast display
    pub last_error: Option<(String, Instant)>,

    // Loro CRDT state
    pub peer_id: Option<u64>,
    pub frame_text_layout: HashMap<(usize, usize), FrameTextId>,
    pub frame_docs: HashMap<FrameTextId, (loro::LoroDoc, loro::Subscription)>,
    pub presence: loro::awareness::EphemeralStore,
    pub presence_subscription: Option<loro::Subscription>,
    pub(super) last_presence_gc: Instant,

    // Scene history for undo/redo
    scene_history: VecDeque<Scene>,
    history_index: usize,
    skip_next_history_push: bool,
    scene_dirty: bool,

    // Local audio feedback
    feedback_engine: Option<FeedbackEngine>,

    // Communication channels
    send_tx: Option<tokio_mpsc::UnboundedSender<OutgoingMessage>>,
    event_rx: Option<mpsc::Receiver<BridgeEvent>>,

    runtime: tokio::runtime::Handle,
    ctx: egui::Context,
    log_tx: mpsc::Sender<LogEntry>,
}

impl ClientBridge {
    pub fn frame_text_id_at(&self, li: usize, fi: usize) -> Option<FrameTextId> {
        self.frame_text_layout.get(&(li, fi)).copied()
    }

    pub fn frame_doc(&self, id: FrameTextId) -> Option<&loro::LoroDoc> {
        self.frame_docs.get(&id).map(|(doc, _)| doc)
    }

    pub fn frame_doc_text(&self, id: FrameTextId) -> Option<String> {
        self.frame_docs.get(&id).map(|(doc, _)| {
            doc.get_text(sova_server::FrameTextStore::CONTENT_CONTAINER)
                .to_string()
        })
    }

    pub(crate) fn install_frame_doc(&mut self, id: FrameTextId, doc: loro::LoroDoc) {
        let send = self.send_tx.clone();
        let id_copy = id;
        let sub = doc.subscribe_local_update(Box::new(move |bytes: &Vec<u8>| {
            if let Some(tx) = &send {
                let _ = tx.send(OutgoingMessage::Send(Box::new(ClientMessage::ScriptEdit {
                    frame_text_id: id_copy,
                    update: bytes.clone(),
                })));
            }
            true
        }));
        self.frame_docs.insert(id, (doc, sub));
    }

    /// Build a `LoroDoc` from a snapshot blob, set the peer id (if known) and
    /// install it under `id`, wiring up the local-update subscription.
    pub(crate) fn install_frame_doc_from_snapshot(&mut self, id: FrameTextId, blob: &[u8]) {
        if let Ok(doc) = loro::LoroDoc::from_snapshot(blob) {
            if let Some(p) = self.peer_id {
                let _ = doc.set_peer_id(p);
            }
            self.install_frame_doc(id, doc);
        }
    }

    pub(crate) fn install_presence_wire(&mut self) {
        let send = self.send_tx.clone();
        self.presence_subscription =
            Some(self.presence.subscribe_local_updates(Box::new(move |bytes| {
                if let Some(tx) = &send {
                    let _ = tx.send(OutgoingMessage::Send(Box::new(ClientMessage::Presence {
                        update: bytes.clone(),
                    })));
                }
                true
            })));
    }

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
            position_start_beat: Vec::new(),
            devices: Vec::new(),
            clock: ClockState::default(),
            audio_state: AudioEngineState::default(),
            scope_data: Vec::new(),
            scope_generation: 0,
            peak_data: Vec::new(),
            peers: Vec::new(),
            confirmed_username: None,
            languages: Vec::new(),
            syntax_map: HashMap::new(),
            peer_editing: HashMap::new(),
            chat_messages: VecDeque::new(),
            errors: HashMap::new(),
            annotations: Vec::new(),
            remote_hydra: None,
            compilation_flashes: HashMap::new(),
            mutation_flashes: HashMap::new(),
            last_error: None,
            peer_id: None,
            frame_text_layout: HashMap::new(),
            frame_docs: HashMap::new(),
            presence: loro::awareness::EphemeralStore::new(30_000),
            presence_subscription: None,
            last_presence_gc: Instant::now(),
            scene_history: VecDeque::new(),
            history_index: 0,
            skip_next_history_push: false,
            scene_dirty: false,
            feedback_engine: None,
            send_tx: None,
            event_rx: None,
            runtime,
            ctx,
            log_tx,
        }
    }

}
