use crate::audio::{AudioCommand, AudioEngineState};
use crate::client::{ClientMessage, serialize_to_wire_frame};
use crate::server::image_maintainer::start_image_maintainer;
use crate::server::message_processing::on_message;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use socket2::SockRef;
use sova_core::{Scene, vm::LanguageCenter};
use std::sync::OnceLock;
use std::thread::JoinHandle;
use std::{
    io::ErrorKind,
    path::PathBuf,
    sync::{
        Arc, Mutex as StdMutex, RwLock,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};
use tokio::time::{self, Duration, timeout};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::{Mutex, broadcast, mpsc},
};
use tokio_util::sync::CancellationToken;

use sova_core::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    schedule::{ActionTiming, SchedulerMessage, SovaNotification},
};

pub type TokioReceiver<T> = tokio::sync::mpsc::Receiver<T>;
pub type TokioSender<T> = tokio::sync::mpsc::Sender<T>;

use crate::message::ServerMessage;

mod image_maintainer;
mod message_processing;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioRestartConfig {
    pub host: Option<String>,
    pub device: Option<String>,
    pub input_device: Option<String>,
    pub channels: u16,
    pub buffer_size: Option<u32>,
    pub sample_paths: Vec<PathBuf>,
    pub max_voices: usize,
}

pub struct AudioRestartRequest {
    pub config: AudioRestartConfig,
    pub response_tx: crossbeam_channel::Sender<Result<AudioEngineState, String>>,
}

pub struct CoreRestartRequest {
    pub response_tx: TokioSender<Result<(), String>>,
}

pub const DEFAULT_CLIENT_NAME: &str = "Unknown musician";
const POSITION_BROADCAST_INTERVAL_MS: u64 = 33;
const CLIENT_CHANNEL_CAPACITY: usize = 512;
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub enum BroadcastItem {
    Raw {
        bytes: Arc<Vec<u8>>,
        droppable: bool,
    },
    Filtered(String, ServerMessage),
    Feedback(SchedulerMessage),
}

impl BroadcastItem {
    fn is_droppable(&self) -> bool {
        match self {
            BroadcastItem::Raw { droppable, .. } => *droppable,
            BroadcastItem::Feedback(_) => false,
            BroadcastItem::Filtered(_, msg) => matches!(msg, ServerMessage::PeerCursorMoved(..)),
        }
    }
}

struct ClientHandle {
    tx: mpsc::Sender<BroadcastItem>,
    needs_resync: Arc<AtomicBool>,
}

#[derive(Clone, Default)]
pub struct ClientRegistry {
    handles: Arc<StdMutex<Vec<ClientHandle>>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        ClientRegistry {
            handles: Arc::new(StdMutex::new(Vec::new())),
        }
    }

    pub fn register(&self) -> (mpsc::Receiver<BroadcastItem>, Arc<AtomicBool>) {
        let (tx, rx) = mpsc::channel(CLIENT_CHANNEL_CAPACITY);
        let needs_resync = Arc::new(AtomicBool::new(false));
        self.handles.lock().unwrap().push(ClientHandle {
            tx,
            needs_resync: Arc::clone(&needs_resync),
        });
        (rx, needs_resync)
    }

    pub fn broadcast(&self, item: BroadcastItem) {
        let mut handles = self.handles.lock().unwrap();
        handles.retain(|handle| match handle.tx.try_send(item.clone()) {
            Ok(()) => true,
            Err(mpsc::error::TrySendError::Full(_)) => {
                if !item.is_droppable() {
                    handle.needs_resync.store(true, Ordering::Relaxed);
                }
                true
            }
            Err(mpsc::error::TrySendError::Closed(_)) => false,
        });
    }
}

#[derive(Clone)]
pub struct ServerState {
    pub clock_server: Arc<ClockServer>,
    pub devices: Arc<DeviceMap>,
    pub sched_iface: Arc<RwLock<Sender<SchedulerMessage>>>,
    pub client_registry: ClientRegistry,
    pub clients: Arc<Mutex<Vec<String>>>,
    pub scene_image: Arc<Mutex<Scene>>,
    pub languages: Arc<LanguageCenter>,
    pub is_playing: Arc<AtomicBool>,
    pub audio_engine_state: Arc<StdMutex<AudioEngineState>>,
    pub audio_restart_tx: Option<Sender<AudioRestartRequest>>,
    pub audio_cmd_tx: Option<Sender<AudioCommand>>,
    pub core_restart_tx: TokioSender<CoreRestartRequest>,
    pub password: Option<String>,
    pub master_gain: Arc<AtomicU32>,
}

impl ServerState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scene_image: Arc<Mutex<Scene>>,
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        sched_iface: Arc<RwLock<Sender<SchedulerMessage>>>,
        client_registry: ClientRegistry,
        clients: Arc<Mutex<Vec<String>>>,
        languages: Arc<LanguageCenter>,
        is_playing: Arc<AtomicBool>,
        audio_engine_state: Arc<StdMutex<AudioEngineState>>,
        audio_restart_tx: Option<Sender<AudioRestartRequest>>,
        audio_cmd_tx: Option<Sender<AudioCommand>>,
        core_restart_tx: TokioSender<CoreRestartRequest>,
        password: Option<String>,
        master_gain: Arc<AtomicU32>,
    ) -> Self {
        ServerState {
            clock_server,
            devices,
            sched_iface,
            client_registry,
            clients,
            scene_image,
            languages,
            is_playing,
            audio_engine_state,
            audio_restart_tx,
            audio_cmd_tx,
            core_restart_tx,
            password,
            master_gain,
        }
    }

    pub fn get_audio_engine_state(&self) -> AudioEngineState {
        self.audio_engine_state
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub scene: Scene,
    pub tempo: f64,
    pub beat: f64,
    pub micros: SyncTime,
    pub quantum: f64,
    #[serde(default)]
    pub devices: Vec<sova_core::protocol::DeviceInfo>,
}

async fn send_msg<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: ServerMessage) -> io::Result<()> {
    let frame = serialize_to_wire_frame(&msg)?;
    writer.write_all(&frame).await?;
    writer.flush().await?;
    Ok(())
}

fn broadcast_raw(registry: &ClientRegistry, msg: &ServerMessage, droppable: bool) {
    if let Ok(bytes) = serialize_to_wire_frame(msg) {
        registry.broadcast(BroadcastItem::Raw {
            bytes: Arc::new(bytes),
            droppable,
        });
    }
}

pub struct SovaCoreServer {
    pub ip: String,
    pub port: u16,
    pub clock_server: Arc<ClockServer>,
    pub devices: Arc<DeviceMap>,
    pub sched_iface: OnceLock<Arc<RwLock<Sender<SchedulerMessage>>>>,
    pub log_sender: broadcast::Sender<SovaNotification>,
    pub client_registry: ClientRegistry,
    pub clients: Arc<Mutex<Vec<String>>>,
    pub scene_image: Arc<Mutex<Scene>>,
    pub languages: Arc<LanguageCenter>,
    pub is_playing: Arc<AtomicBool>,
    pub audio_engine_state: Arc<StdMutex<AudioEngineState>>,
    pub audio_restart_tx: Option<Sender<AudioRestartRequest>>,
    pub audio_cmd_tx: Option<Sender<AudioCommand>>,
    pub core_restart_rx: TokioReceiver<CoreRestartRequest>,
    pub core_restart_tx: TokioSender<CoreRestartRequest>,
    pub password: Option<String>,
    pub master_gain: Arc<AtomicU32>,
}

impl SovaCoreServer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ip: String,
        port: u16,
        scene_image: Arc<Mutex<Scene>>,
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        log_sender: broadcast::Sender<SovaNotification>,
        client_registry: ClientRegistry,
        languages: Arc<LanguageCenter>,
        audio_engine_state: Arc<StdMutex<AudioEngineState>>,
        audio_restart_tx: Option<Sender<AudioRestartRequest>>,
        audio_cmd_tx: Option<Sender<AudioCommand>>,
        password: Option<String>,
        master_gain: Arc<AtomicU32>,
    ) -> Self {
        let (core_restart_tx, core_restart_rx) = tokio::sync::mpsc::channel(128);
        SovaCoreServer {
            ip,
            port,
            clock_server,
            devices,
            sched_iface: OnceLock::new(),
            log_sender,
            client_registry,
            clients: Default::default(),
            scene_image,
            languages,
            is_playing: Arc::new(AtomicBool::new(false)),
            audio_engine_state,
            audio_restart_tx,
            audio_cmd_tx,
            core_restart_rx,
            core_restart_tx,
            password,
            master_gain,
        }
    }

    pub fn state(&self) -> ServerState {
        ServerState::new(
            self.scene_image.clone(),
            self.clock_server.clone(),
            self.devices.clone(),
            self.sched_iface.get().unwrap().clone(),
            self.client_registry.clone(),
            self.clients.clone(),
            self.languages.clone(),
            self.is_playing.clone(),
            self.audio_engine_state.clone(),
            self.audio_restart_tx.clone(),
            self.audio_cmd_tx.clone(),
            self.core_restart_tx.clone(),
            self.password.clone(),
            self.master_gain.clone(),
        )
    }

    pub fn stop_core(&self) {
        if let Some(iface) = &self.sched_iface.get() {
            let _ = iface.read().unwrap().send(SchedulerMessage::Shutdown);
            self.is_playing.store(false, Ordering::Relaxed);
        }
    }

    pub async fn start_core(&self) -> (JoinHandle<()>, JoinHandle<()>, Option<String>) {
        let (new_world, new_sched, new_iface, new_update) =
            sova_core::init::start_scheduler_and_world(
                self.clock_server.clone(),
                self.devices.clone(),
                self.languages.clone(),
            );
        let scene = self.scene_image.lock().await.clone();
        if let Err(e) = new_iface.send(SchedulerMessage::SetScene(
            scene.clone(),
            ActionTiming::Immediate,
        )) {
            return (
                new_world,
                new_sched,
                Some(format!("Failed to set scene: {e}")),
            );
        }
        self.set_scheduler_connection(new_iface, new_update);
        broadcast_raw(&self.client_registry, &ServerMessage::CoreRestarted, false);
        broadcast_raw(
            &self.client_registry,
            &ServerMessage::Notification(SovaNotification::UpdatedScene(scene)),
            false,
        );
        (new_world, new_sched, None)
    }

    pub fn set_scheduler_connection(
        &self,
        sched_iface: Sender<SchedulerMessage>,
        sched_update: Receiver<SovaNotification>,
    ) {
        self.start_image_maintainer(sched_update);
        match self.sched_iface.get() {
            Some(lock) => {
                let mut iface = lock.write().unwrap();
                *iface = sched_iface;
            }
            None => {
                let _ = self.sched_iface.set(Arc::new(RwLock::new(sched_iface)));
            }
        }
    }

    pub async fn start(&mut self, token: CancellationToken) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Server listening on {}", addr);

        let (mut world_handle, mut sched_handle, _) = self.start_core().await;

        // Bridge logger notifications (from core) to per-client channels
        let mut log_rx = self.log_sender.subscribe();
        let bridge_registry = self.client_registry.clone();
        tokio::spawn(async move {
            loop {
                match log_rx.recv().await {
                    Ok(notif @ SovaNotification::Log(_)) => {
                        broadcast_raw(&bridge_registry, &ServerMessage::Notification(notif), false);
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) | Ok(_) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        let mut annotations_interval = time::interval(Duration::from_millis(33));
        let annotations_token = token.child_token();
        let sched_iface = self.sched_iface.clone();
        tokio::task::spawn(async move {
            loop {
                select! {
                    _ = annotations_token.cancelled() => {
                        break;
                    }
                    _ = annotations_interval.tick() => {
                        let Some(iface) = sched_iface.get() else {
                            continue;
                        };
                        let _ = iface.read().unwrap().send(SchedulerMessage::GetAnnotations);
                    }
                }
            }
        });

        let mut tick_interval = time::interval(Duration::from_millis(20));
        let tick_token = token.child_token();
        let mut clock = Clock::from(Arc::clone(&self.clock_server));
        let tick_registry = self.client_registry.clone();
        tokio::task::spawn(async move {
            loop {
                select! {
                    _ = tick_token.cancelled() => {
                        break;
                    }
                    _ = tick_interval.tick() => {
                        clock.capture_app_state();
                        let msg = ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum());
                        broadcast_raw(&tick_registry, &msg, true);
                    }
                }
            }
        });

        loop {
            select! {
                Ok((socket, client_addr)) = listener.accept() => {
                    println!("New connection from {}", client_addr);
                    let client_state = self.state();
                    tokio::spawn(async move {
                        match process_client(socket, client_state).await {
                            Ok(client_name) => {
                                println!("Client '{}' disconnected.", client_name);
                            },
                            Err(e) => {
                                eprintln!("Error handling client {}: {}", client_addr, e);
                            }
                        }
                    });
                }
                Some(req) = self.core_restart_rx.recv() => {
                    let mut requestors = vec![req];
                    while let Ok(extra) = self.core_restart_rx.try_recv() {
                        requestors.push(extra);
                    }

                    self.stop_core();
                    let _ = sched_handle.join();
                    let _ = world_handle.join();

                    let (new_world, new_sched, err) = self.start_core().await;
                    world_handle = new_world;
                    sched_handle = new_sched;

                    if let Some(e) = err {
                        for r in requestors {
                            let _ = r.response_tx.send(Err(e.clone()));
                        }
                        continue;
                    }
                    for r in requestors {
                        let _ = r.response_tx.send(Ok(()));
                    }
                }
                _ = token.cancelled() => {
                    println!("\n[!] Server task cancelled, shutting down server...");
                    break;
                }
                _ = signal::ctrl_c() => {
                    token.cancel();
                    println!("\n[!] Ctrl+C received, shutting down server...");
                    break;
                }
            }
        }

        self.stop_core();
        let _ = world_handle.join();
        let _ = sched_handle.join();
        Ok(())
    }

    pub fn start_image_maintainer(&self, scheduler_notifications: Receiver<SovaNotification>) {
        start_image_maintainer(
            scheduler_notifications,
            self.scene_image.clone(),
            self.client_registry.clone(),
            self.is_playing.clone(),
            Clock::from(Arc::clone(&self.clock_server)),
        );
    }
}

async fn process_client(socket: TcpStream, state: ServerState) -> io::Result<String> {
    socket.set_nodelay(true)?;
    let keepalive = socket2::TcpKeepalive::new()
        .with_time(std::time::Duration::from_secs(60))
        .with_interval(std::time::Duration::from_secs(10));
    let _ = SockRef::from(&socket).set_tcp_keepalive(&keepalive);
    let client_addr = socket.peer_addr()?;
    let client_addr_str = client_addr.to_string();
    let (reader, writer) = socket.into_split();
    let mut reader = BufReader::with_capacity(32 * 1024, reader);
    let mut writer = BufWriter::with_capacity(32 * 1024, writer);
    let mut client_name = DEFAULT_CLIENT_NAME.to_string();

    let clock = Clock::from(&state.clock_server);

    let hello_msg: ServerMessage;

    match read_message_internal(&mut reader, &client_addr_str).await {
        Ok(Some(ClientMessage::SetName {
            name: new_name,
            password,
        })) => {
            if let Some(required) = &state.password {
                if password.as_deref() != Some(required.as_str()) {
                    eprintln!(
                        "Connection rejected: Invalid password from {}",
                        client_addr_str
                    );
                    let refuse_msg =
                        ServerMessage::ConnectionRefused("Invalid password.".to_string());
                    let _ = send_msg(&mut writer, refuse_msg).await;
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Invalid password",
                    ));
                }
            }

            if new_name.is_empty() || new_name == DEFAULT_CLIENT_NAME {
                eprintln!(
                    "Connection rejected: Invalid username '{}' from {}",
                    new_name, client_addr_str
                );
                let refuse_msg = ServerMessage::ConnectionRefused(
                    "Invalid username (empty or reserved).".to_string(),
                );
                let _ = send_msg(&mut writer, refuse_msg).await;
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid username",
                ));
            }

            let mut clients_guard = state.clients.lock().await;
            if clients_guard.iter().any(|name| name == &new_name) {
                eprintln!(
                    "Connection rejected: Username '{}' already taken by {}",
                    new_name, client_addr_str
                );
                let refuse_msg = ServerMessage::ConnectionRefused(format!(
                    "Username '{}' is already taken.",
                    new_name
                ));
                let _ = send_msg(&mut writer, refuse_msg).await;
                drop(clients_guard);
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Username taken",
                ));
            }

            client_name = new_name;
            println!("Client {} identified as: {}", client_addr_str, client_name);
            clients_guard.push(client_name.clone());

            let initial_scene = state.scene_image.lock().await.clone();
            let initial_devices = state.devices.device_list();
            let initial_peers = clients_guard.clone();
            let updated_peers_for_broadcast = initial_peers.clone();

            drop(clients_guard);

            broadcast_raw(
                &state.client_registry,
                &ServerMessage::PeersUpdated(updated_peers_for_broadcast),
                false,
            );

            let initial_link_state = (
                clock.tempo(),
                clock.beat(),
                clock.beat() % clock.quantum(),
                state.clock_server.link.num_peers() as u32,
                state.clock_server.link.is_start_stop_sync_enabled(),
            );
            let initial_is_playing = state.is_playing.load(Ordering::Relaxed);

            let mut available_languages: Vec<_> = state.languages.definitions().collect();
            #[cfg(feature = "audio")]
            enrich_with_sound_docs(&mut available_languages);

            println!(
                "[ handshake ] Sending Hello to {} ({}). Initial is_playing state: {}",
                client_addr_str, client_name, initial_is_playing
            );
            hello_msg = ServerMessage::Hello {
                username: client_name.clone(),
                scene: initial_scene,
                devices: initial_devices,
                peers: initial_peers,
                link_state: initial_link_state,
                is_playing: initial_is_playing,
                languages: available_languages,
                audio_engine_state: state.get_audio_engine_state(),
                link_enabled: state.clock_server.link.is_enabled(),
            };

            if !matches!(
                timeout(WRITE_TIMEOUT, send_msg(&mut writer, hello_msg)).await,
                Ok(Ok(()))
            ) {
                eprintln!("Failed to send Hello to {}", client_name);
                let mut clients_guard = state.clients.lock().await;
                if let Some(i) = clients_guard.iter().position(|x| *x == client_name) {
                    clients_guard.remove(i);
                }
                let updated = clients_guard.clone();
                drop(clients_guard);
                broadcast_raw(
                    &state.client_registry,
                    &ServerMessage::PeersUpdated(updated),
                    false,
                );
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "Failed to send Hello message",
                ));
            }
        }
        Ok(Some(other_msg)) => {
            eprintln!(
                "Connection rejected: Expected SetName, received {:?} from {}",
                other_msg, client_addr_str
            );
            let refuse_msg =
                ServerMessage::ConnectionRefused("Invalid handshake sequence.".to_string());
            let _ = send_msg(&mut writer, refuse_msg).await;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid handshake sequence",
            ));
        }
        Ok(None) => {
            println!("Connection closed by {} during handshake.", client_addr_str);
            return Ok(client_name);
        }
        Err(e) => {
            eprintln!(
                "Read error during handshake with {}: {}",
                client_addr_str, e
            );
            return Err(e);
        }
    }

    let (mut update_receiver, needs_resync) = state.client_registry.register();
    let mut feedback_enabled = false;

    // Dedicated reader task — never cancelled, so read_wire_frame
    // can't lose partial reads (which would desync the TCP stream).
    enum ClientRead {
        Message(Box<ClientMessage>),
        Closed,
        Error(io::Error),
    }
    let (client_msg_tx, mut client_msg_rx) = mpsc::unbounded_channel::<ClientRead>();
    let reader_client_name = client_name.clone();
    let reader_task = tokio::spawn(async move {
        loop {
            match read_message_internal(&mut reader, &reader_client_name).await {
                Ok(Some(msg)) => {
                    if client_msg_tx
                        .send(ClientRead::Message(Box::new(msg)))
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(None) => {
                    let _ = client_msg_tx.send(ClientRead::Closed);
                    break;
                }
                Err(e) if e.kind() == ErrorKind::InvalidData => {
                    eprintln!("Bad frame from {}: {}. Skipping.", reader_client_name, e);
                }
                Err(e) => {
                    let _ = client_msg_tx.send(ClientRead::Error(e));
                    break;
                }
            }
        }
    });

    loop {
        select! {
            biased;

            read_result = client_msg_rx.recv() => {
                match read_result {
                    Some(ClientRead::Message(msg)) => {
                        if matches!(*msg, ClientMessage::EnableFeedback) {
                            feedback_enabled = true;
                        }
                        let response = on_message(*msg, &state, &mut client_name).await;

                        if !matches!(
                            timeout(WRITE_TIMEOUT, send_msg(&mut writer, response)).await,
                            Ok(Ok(()))
                        ) {
                            eprintln!("Failed write direct response to {}", client_name);
                            break;
                        }
                    },
                    Some(ClientRead::Closed) => {
                        println!("Connection closed cleanly by {}.", client_name);
                        break;
                    },
                    Some(ClientRead::Error(_e)) => {
                        eprintln!("Read error for client {}. Closing connection.", client_name);
                        break;
                    }
                    None => {
                        eprintln!("Reader task ended for {}. Closing connection.", client_name);
                        break;
                    }
                }
            }

            update_result = update_receiver.recv() => {
                if needs_resync.swap(false, Ordering::Relaxed) {
                    let scene = state.scene_image.lock().await.clone();
                    let c = Clock::from(&state.clock_server);
                    let devices = state.devices.create_device_snapshot();
                    let snapshot = Snapshot {
                        scene,
                        tempo: c.tempo(),
                        beat: c.beat(),
                        micros: c.micros(),
                        quantum: c.quantum(),
                        devices,
                    };
                    if !matches!(
                        timeout(WRITE_TIMEOUT, send_msg(&mut writer, ServerMessage::Snapshot(snapshot))).await,
                        Ok(Ok(()))
                    ) {
                        eprintln!("Resync write failed for {}, evicting", client_name);
                        break;
                    }
                    while update_receiver.try_recv().is_ok() {}
                    continue;
                }

                let Some(item) = update_result else { break; };

                match item {
                    BroadcastItem::Raw { bytes, .. } => {
                        if !matches!(
                            timeout(WRITE_TIMEOUT, async {
                                writer.write_all(&bytes).await?;
                                writer.flush().await
                            }).await,
                            Ok(Ok(()))
                        ) {
                            break;
                        }
                    }
                    BroadcastItem::Feedback(msg) => {
                        if feedback_enabled
                            && !matches!(
                                timeout(WRITE_TIMEOUT, send_msg(&mut writer, ServerMessage::Feedback(msg))).await,
                                Ok(Ok(()))
                            )
                        {
                            break;
                        }
                    }
                    BroadcastItem::Filtered(sender_name, msg) => {
                        if *sender_name != *client_name {
                            if !matches!(
                                    timeout(WRITE_TIMEOUT, send_msg(&mut writer, msg)).await,
                                    Ok(Ok(()))
                                )
                            {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    reader_task.abort();

    println!("Cleaning up connection for client: {}", client_name);
    if client_name != DEFAULT_CLIENT_NAME {
        let mut clients_guard = state.clients.lock().await;
        if let Some(i) = clients_guard.iter().position(|x| *x == client_name) {
            clients_guard.remove(i);
            println!("Removed {} from client list.", client_name);
            let updated_clients = clients_guard.clone();
            drop(clients_guard);
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::PeersUpdated(updated_clients),
                false,
            );
        } else {
            eprintln!(
                "Client '{}' not found in list during cleanup, though name was set.",
                client_name
            );
        }
    } else {
        println!(
            "Client disconnected before setting a name (still '{}'). No list removal needed.",
            DEFAULT_CLIENT_NAME
        );
    }

    Ok(client_name)
}

async fn read_message_internal<R: AsyncReadExt + Unpin>(
    reader: &mut R,
    client_id_for_logging: &str,
) -> io::Result<Option<ClientMessage>> {
    use crate::client::read_wire_frame;

    let payload = match read_wire_frame(reader).await {
        Ok(buf) => buf,
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
            println!("Connection closed by {} (EOF).", client_id_for_logging);
            return Ok(None);
        }
        Err(e) => return Err(e),
    };

    ClientMessage::deserialize(&payload)
}

#[cfg(feature = "audio")]
fn enrich_with_sound_docs(languages: &mut [sova_core::vm::language::LanguageDefinition]) {
    use sova_core::vm::language::{LanguageElement, ReferenceEntry};

    let sources = doux_sova::types::Source::all_source_docs();
    let gm_presets = doux_sova::soundfont::gm_preset_docs();

    for lang in languages.iter_mut() {
        let ref_map = &mut lang.documentation.reference;

        for src in &sources {
            let sig = if src.params.is_empty() {
                None
            } else {
                Some(
                    src.params
                        .iter()
                        .map(|(name, desc)| format!("{name}: {desc}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            };
            let mut entry = ReferenceEntry::new(src.description)
                .with_category(format!("Sound: {}", src.category));
            if let Some(s) = sig {
                entry = entry.with_signature(s);
            }
            if !src.aliases.is_empty() {
                entry = entry.with_aliases(src.aliases);
            }
            ref_map.insert(LanguageElement::Word(src.name.to_string()), entry);
        }

        for preset in &gm_presets {
            let desc = format!("GM {} (program {})", preset.family, preset.program);
            let mut entry = ReferenceEntry::new(desc).with_category("Sound: GM");
            if !preset.aliases.is_empty() {
                let aliases: Vec<&str> = preset.aliases.iter().map(String::as_str).collect();
                entry = entry.with_aliases(&aliases);
            }
            ref_map.insert(LanguageElement::Word(preset.name.to_string()), entry);
        }
    }
}
