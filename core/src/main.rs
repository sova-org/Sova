use crate::clock::ClockServer;
use crate::compiler::{bali::BaliCompiler, dummylang::DummyCompiler};
use crate::lang::interpreter::InterpreterDirectory;
use crate::scene::script::Script;
use crate::schedule::notification::SchedulerNotification;
use crate::server::client::ClientMessage;
// TimingConfig import removed for now
use bubo_engine::{
    engine::AudioEngine,
    memory::{MemoryPool, SampleLibrary, VoiceMemory},
    registry::ModuleRegistry,
    server::ScheduledEngineMessage,
    types::{EngineLogMessage, EngineMessage},
};
use clap::Parser;
use crossbeam_channel::bounded;
use device_map::DeviceMap;
use scene::Scene;
use scene::Line;
use schedule::{Scheduler, message::SchedulerMessage};
use server::{SovaCoreServer, ServerState};
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{sync::Arc, thread};
use tokio::sync::Mutex;
use transcoder::Transcoder;
use world::World;

// Déclaration des modules
pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod lang;
pub mod logger;
pub mod protocol;
pub mod relay_client;
pub mod scene;
pub mod schedule;
pub mod server;
pub mod shared_types;
pub mod transcoder;
pub mod util;
pub mod world;

pub const DEFAULT_MIDI_OUTPUT: &str = "Sova";
pub const DEFAULT_TEMPO: f64 = 120.0;
pub const DEFAULT_QUANTUM: f64 = 4.0;
pub const GREETER_LOGO: &str = "
 ▗▄▄▖ ▄▄▄  ▄   ▄ ▗▞▀▜▌
▐▌   █   █ █   █ ▝▚▄▟▌
 ▝▀▚▖▀▄▄▄▀  ▀▄▀       
▗▄▄▞▘                 
";

fn greeter() {
    use crate::{log_print, log_println};
    log_print!("{}", GREETER_LOGO);
    log_println!("Version: {}\n", env!("CARGO_PKG_VERSION"));
}

fn initialize_sova_engine(
    cli: &Cli,
    registry: ModuleRegistry,
    osc_shutdown: Arc<AtomicBool>,
) -> (
    crossbeam_channel::Sender<ScheduledEngineMessage>,
    thread::JoinHandle<()>,
    ModuleRegistry,
    crossbeam_channel::Receiver<EngineLogMessage>,
) {
    use crate::log_println;
    log_println!("[+] Initializing Sova audio engine...");

    // Memory allocation calculations
    let memory_per_voice = cli.buffer_size * 8 * 4;
    let sample_memory = cli.max_audio_buffers * cli.buffer_size * 8;
    let dsp_memory = cli.max_voices * cli.buffer_size * 16 * 4;
    let base_memory = 16 * 1024 * 1024;
    let available_memory =
        base_memory + (cli.max_voices * memory_per_voice) + sample_memory + dsp_memory;

    log_println!(
        "   Memory allocation: {}MB total",
        available_memory / (1024 * 1024)
    );

    let global_pool = Arc::new(MemoryPool::new(available_memory));
    let voice_memory = Arc::new(VoiceMemory::new());

    // Sample library
    let sample_library = SampleLibrary::new(
        cli.max_audio_buffers,
        &cli.audio_files_location,
        cli.sample_rate,
    );
    sample_library.preload_all_samples();
    let sample_library = Arc::new(sample_library);

    log_println!(
        "   Engine config: {} voices | Sample rate: {} | Buffer: {}",
        cli.max_voices,
        cli.sample_rate,
        cli.buffer_size
    );

    // Clone registry for world usage
    let registry_for_world = registry.clone();
    let mut engine = AudioEngine::new_with_memory(
        cli.sample_rate as f32,
        cli.buffer_size,
        cli.max_voices,
        cli.block_size as usize,
        registry.clone(),
        global_pool,
        voice_memory.clone(),
        sample_library.clone(),
    );

    // Create log channel for engine-to-client communication
    let (log_tx, log_rx) = crossbeam_channel::unbounded::<EngineLogMessage>();
    engine.set_log_sender(log_tx);

    // Create message channels for engine communication
    let (engine_tx, engine_rx) = bounded(1024);

    // Start audio thread
    let audio_thread = AudioEngine::start_audio_thread(
        engine,
        cli.block_size,
        cli.max_voices,
        cli.sample_rate,
        cli.buffer_size,
        cli.output_device.clone(),
        engine_rx,
        None, // No status channel for sova
        cli.audio_priority,
    );

    // Start OSC server thread
    let osc_host = cli.osc_host.clone();
    let osc_port = cli.osc_port;
    let engine_tx_clone = engine_tx.clone();
    let osc_shutdown_clone = osc_shutdown.clone();
    let registry_clone = registry.clone();
    let voice_memory_clone = voice_memory.clone();
    let sample_library_clone = sample_library.clone();

    let _osc_thread = thread::Builder::new()
        .name("osc_server".to_string())
        .spawn(move || {
            let logger = bubo_engine::types::LoggerHandle::new_console();
            let mut osc_server = match bubo_engine::server::OscServer::new(
                &osc_host,
                osc_port,
                registry_clone,
                voice_memory_clone,
                sample_library_clone,
                osc_shutdown_clone,
                logger,
            ) {
                Ok(server) => server,
                Err(e) => {
                    use crate::log_eprintln;
                    log_eprintln!("Failed to create OSC server: {}", e);
                    return;
                }
            };
            osc_server.run_lockfree(engine_tx_clone);
        })
        .expect("Failed to spawn OSC thread");

    log_println!("   Audio engine ready ✓");
    (engine_tx, audio_thread, registry_for_world, log_rx)
}

// Define the CLI arguments struct
#[derive(Parser, Debug)]
#[clap(author = "Raphaël Forment <raphael.forment@gmail.com>")]
#[clap(author = "Loïg Jezequel <loig.jezequel@univ-nantes.fr>")]
#[clap(author = "Tanguy Dubois <tanguy.dubois@ls2n.fr>")]
#[command(
    version = "0.0.1",
    about = "Sova: A live coding environment server.",
    long_about = "Sova acts as the central server for a collaborative live coding environment.\n
    It manages connections from clients (like sovagui), handles MIDI devices,
    \nsynchronizes state, and processes scenes."
)]
struct Cli {
    /// IP address to bind the server to
    #[arg(short, long, value_name = "IP_ADDRESS", default_value = "0.0.0.0")]
    ip: String,

    /// Port to bind the server to
    #[arg(short, long, value_name = "PORT", default_value_t = 8080)]
    port: u16,

    /// Enable internal audio engine (Sova)
    #[arg(long)]
    audio_engine: bool,

    /// Audio engine sample rate
    #[arg(short, long, default_value_t = 44100)]
    sample_rate: u32,

    /// Audio engine block size
    #[arg(short, long, default_value_t = 512)]
    block_size: u32,

    /// Audio engine buffer size
    #[arg(short = 'B', long, default_value_t = 1024)]
    buffer_size: usize,

    /// Maximum audio buffers for sample library
    #[arg(short, long, default_value_t = 2048)]
    max_audio_buffers: usize,

    /// Maximum voices for audio engine
    #[arg(short = 'v', long, default_value_t = 128)]
    max_voices: usize,

    /// Audio output device name
    #[arg(short, long)]
    output_device: Option<String>,

    /// OSC server port for audio engine
    #[arg(long, default_value_t = 12345)]
    osc_port: u16,

    /// OSC server host for audio engine
    #[arg(long, default_value = "127.0.0.1")]
    osc_host: String,

    /// Timestamp tolerance in milliseconds for audio engine
    #[arg(long, default_value_t = 1000)]
    timestamp_tolerance_ms: u64,

    /// Location of audio files for sample library
    #[arg(long, default_value = "./samples")]
    audio_files_location: String,

    /// Audio thread priority (0-99, higher = more priority, 0 = disable, auto-mapped to platform ranges)
    #[arg(long, default_value_t = 80)]
    audio_priority: u8,

    /// List available audio output devices and exit
    #[arg(long)]
    list_devices: bool,

    /// Connect to relay server for remote collaboration
    #[arg(long, value_name = "RELAY_ADDRESS:PORT")]
    relay: Option<String>,

    /// Instance name for relay identification
    #[arg(long, value_name = "INSTANCE_NAME", default_value = "local")]
    instance_name: String,

    /// Authentication token for relay server (optional)
    #[arg(long, value_name = "TOKEN")]
    relay_token: Option<String>,
}

#[tokio::main]
async fn main() {
    // ======================================================================
    // Parse CLI arguments first
    let cli = Cli::parse();

    // ======================================================================
    // Initialize logger and immediately set up full mode for complete logging
    crate::logger::init_standalone();

    // Set up notification channel and switch to full mode IMMEDIATELY
    // This ensures ALL logs (including startup) reach file, terminal, and clients
    let (updater, update_notifier) = tokio::sync::watch::channel(
        crate::schedule::notification::SchedulerNotification::default(),
    );
    crate::logger::set_full_mode(updater.clone());

    // Test log to verify full mode works
    log_info!("Logger initialized in full mode - all logs will reach file, terminal, and clients");

    // Handle --list-devices flag before initialization
    if cli.list_devices {
        let logger = bubo_engine::types::LoggerHandle::new_console();
        bubo_engine::list_audio_devices(&logger);
        return;
    }

    // ======================================================================
    // Splash screen
    greeter();

    // ======================================================================
    // Initialize the clock
    let clock_server = Arc::new(ClockServer::new(DEFAULT_TEMPO, DEFAULT_QUANTUM));
    clock_server.link.enable(true);

    // ======================================================================
    // Initialize the list of devices
    let devices = Arc::new(DeviceMap::new());
    let midi_name = DEFAULT_MIDI_OUTPUT.to_owned();
    // Create the default virtual port
    if let Err(e) = devices.create_virtual_midi_port(&midi_name) {
        use crate::log_eprintln;
        log_eprintln!(
            "[!] Failed to create default virtual MIDI port '{}': {}",
            midi_name,
            e
        );
    } else {
        log_println!(
            "[+] Default virtual MIDI port '{}' created successfully.",
            midi_name
        );
        // Assign default MIDI port to Slot 1
        if let Err(e) = devices.assign_slot(1, &midi_name) {
            log_eprintln!("[!] Failed to assign '{}' to Slot 1: {}", midi_name, e);
        }
    }

    // Create and assign default OSC device (SuperDirt) to Slot 2
    let osc_name = "SuperDirt";
    let osc_ip = "127.0.0.1";
    let osc_port = 57120;
    if let Err(e) = devices.create_osc_output_device(osc_name, osc_ip, osc_port) {
        log_eprintln!(
            "[!] Failed to create default OSC device '{}': {}",
            osc_name,
            e
        );
    } else {
        log_println!(
            "[+] Default OSC device '{}' created successfully ({}:{}).",
            osc_name,
            osc_ip,
            osc_port
        );
        // Assign SuperDirt to Slot 2
        if let Err(e) = devices.assign_slot(2, osc_name) {
            log_eprintln!("[!] Failed to assign '{}' to Slot 2: {}", osc_name, e);
        }
    }

    // ======================================================================
    // Create module registry for both audio engine and world
    let mut registry = ModuleRegistry::new();
    registry.register_default_modules();

    // Conditionally initialize audio engine (Sova)
    let (audio_engine_components, registry_for_world, osc_shutdown_flag, engine_log_rx) =
        if cli.audio_engine {
            let osc_shutdown = Arc::new(AtomicBool::new(false));
            let (tx, thread_handle, registry_clone, log_rx) =
                initialize_sova_engine(&cli, registry, osc_shutdown.clone());
            (
                Some((tx, thread_handle)),
                registry_clone,
                Some(osc_shutdown),
                Some(log_rx),
            )
        } else {
            (None, registry, None, None)
        };

    // ======================================================================
    // Notification channels already created early for immediate dual mode logging

    // ======================================================================
    // Initialize the world (side effect performer)
    let audio_engine_tx = audio_engine_components.as_ref().map(|(tx, _)| tx.clone());
    let (world_handle, world_iface) = World::create(
        clock_server.clone(),
        audio_engine_tx,
        registry_for_world,
        engine_log_rx,
        Some(updater.clone()),
    );

    // ======================================================================
    // Extract status receiver and start monitoring thread if audio engine is enabled
    let audio_engine_components = if let Some((tx, thread_handle)) = audio_engine_components {
        Some((tx, thread_handle))
    } else {
        None
    };

    // ======================================================================
    // Initialize the transcoder (list of available compilers) and interpreter directory
    let mut transcoder = Transcoder::default();
    transcoder.add_compiler(BaliCompiler);
    transcoder.add_compiler(DummyCompiler);
    let _ = transcoder.set_active_compiler("bali");
    let transcoder = Arc::new(transcoder);

    let interpreter_directory = Arc::new(InterpreterDirectory::new());

    // Shared flag for transport state (playing/stopped)
    let shared_atomic_is_playing = Arc::new(AtomicBool::new(false));

    // ======================================================================
    // Initialize the scheduler (scene manager)
    let (sched_handle, sched_iface, sched_update) = Scheduler::create(
        clock_server.clone(),
        devices.clone(),
        interpreter_directory.clone(),
        transcoder.clone(),
        world_iface.clone(),
        shared_atomic_is_playing.clone(),
    );

    // ======================================================================
    // Initialize the default scene loaded when the server starts
    let initial_scene = Scene::new(vec![Line::new(vec![1.0])]);
    let scene_image: Arc<Mutex<Scene>> = Arc::new(Mutex::new(initial_scene.clone()));
    let scene_image_maintainer = Arc::clone(&scene_image);
    let updater_clone = updater.clone();

    thread::spawn(move || {
        loop {
            match sched_update.recv() {
                Ok(p) => {
                    let mut guard = scene_image_maintainer.blocking_lock();
                    match &p {
                        SchedulerNotification::UpdatedScene(scene) => {
                            *guard = scene.clone();
                        }
                        SchedulerNotification::UpdatedLine(i, line) => {
                            guard.set_line(*i, line.clone());
                        }
                        SchedulerNotification::FramePositionChanged(_positions) => {
                            // No update to scene needed for this notification
                        }
                        SchedulerNotification::EnableFrames(line_index, frame_indices) => {
                            guard
                                .line_mut(*line_index)
                                .map(|l| l.enable_frames(frame_indices));
                        }
                        SchedulerNotification::DisableFrames(line_index, frame_indices) => {
                            guard
                                .line_mut(*line_index)
                                .map(|l| l.disable_frames(frame_indices));
                        }
                        SchedulerNotification::UploadedScript(_, _, _script) => {}
                        SchedulerNotification::UpdatedLineFrames(frame_index, items) => {
                            guard
                                .line_mut(*frame_index)
                                .map(|l| l.set_frames(items.clone()));
                        }
                        SchedulerNotification::AddedLine(line) => {
                            guard.add_line(line.clone());
                        }
                        SchedulerNotification::RemovedLine(index) => {
                            guard.remove_line(*index);
                        }
                        _ => (),
                    };
                    drop(guard);
                    let _ = updater_clone.send(p);
                }
                Err(_) => break,
            }
        }
    });

    if let Err(e) = sched_iface.send(SchedulerMessage::UploadScene(initial_scene)) {
        log_eprintln!("[!] Failed to send initial scene to scheduler: {}", e);
        std::process::exit(1);
    }

    // ======================================================================
    // Initialize relay client if requested
    let relay_client = if let Some(relay_addr) = cli.relay {
        log_println!("[+] Initializing relay client...");

        let config = relay_client::RelayConfig {
            relay_address: relay_addr.clone(),
            instance_name: cli.instance_name.clone(),
            session_token: cli.relay_token.clone(),
        };

        let mut client = relay_client::RelayClient::new(config);

        match client.connect().await {
            Ok(_) => {
                log_println!("[+] Connected to relay server at {}", relay_addr);
                log_println!("[+] Relay client instance ID: {:?}", client.instance_id());
                Some(Arc::new(Mutex::new(client)))
            }
            Err(e) => {
                log_eprintln!("[!] Failed to connect to relay server: {}", e);
                log_eprintln!("    Continuing in local mode...");
                None
            }
        }
    } else {
        None
    };

    let server_state = ServerState::new(
        scene_image,
        clock_server,
        devices.clone(),
        world_iface,
        sched_iface.clone(),
        updater.clone(),
        update_notifier,
        transcoder,
        interpreter_directory,
        shared_atomic_is_playing.clone(),
    )
    .with_relay(relay_client.clone());

    // Start relay message handler if connected
    if let Some(relay) = relay_client {
        let sched_iface_relay = sched_iface.clone();
        let _updater_relay = updater.clone();

        tokio::spawn(async move {
            log_println!("[RELAY] Starting relay message handler task");
            loop {
                let (relay_msg, is_connected) = {
                    let mut client = relay.lock().await;
                    let msg = client.recv().await;
                    let connected = client.is_connected();
                    (msg, connected)
                };

                if !is_connected {
                    log_eprintln!("[RELAY] Connection lost, relay handler exiting");
                    break;
                }

                if let Some(relay_msg) = relay_msg {
                    use relay_client::RelayMessage;

                    match relay_msg {
                        RelayMessage::StateBroadcast {
                            source_instance_name,
                            timestamp: _,
                            update_data,
                        } => {
                            // Deserialize the client message
                            match rmp_serde::from_slice::<ClientMessage>(&update_data) {
                                Ok(client_msg) => {
                                    log_println!(
                                        "[RELAY] Received update from instance '{}': {:?}",
                                        source_instance_name,
                                        client_msg
                                    );

                                    // Process the message through the scheduler
                                    // This will update local state to match remote changes
                                    match client_msg {
                                        ClientMessage::SetScript(
                                            line_id,
                                            frame_id,
                                            content,
                                            timing,
                                        ) => {
                                            // For now, we'll compile with the default language
                                            // In the future, this should be included in the relay message
                                            if let Err(e) = sched_iface_relay.send(
                                                SchedulerMessage::UploadScript(
                                                    line_id,
                                                    frame_id,
                                                    Script::new(content, "bali".to_string()),
                                                    timing,
                                                ),
                                            ) {
                                                log_eprintln!(
                                                    "[RELAY] Failed to apply SetScript: {}",
                                                    e
                                                );
                                            }
                                        }
                                        ClientMessage::EnableFrames(line_id, frames, timing) => {
                                            let _ = sched_iface_relay.send(
                                                SchedulerMessage::EnableFrames(
                                                    line_id, frames, timing,
                                                ),
                                            );
                                        }
                                        ClientMessage::DisableFrames(line_id, frames, timing) => {
                                            let _ = sched_iface_relay.send(
                                                SchedulerMessage::DisableFrames(
                                                    line_id, frames, timing,
                                                ),
                                            );
                                        }
                                        ClientMessage::UpdateLineFrames(
                                            line_id,
                                            frames,
                                            timing,
                                        ) => {
                                            let _ = sched_iface_relay.send(
                                                SchedulerMessage::UpdateLineFrames(
                                                    line_id, frames, timing,
                                                ),
                                            );
                                        }
                                        ClientMessage::SetScene(scene, timing) => {
                                            let _ = sched_iface_relay
                                                .send(SchedulerMessage::SetScene(scene, timing));
                                        }
                                        // Add more message handlers as needed
                                        _ => {
                                            log_println!(
                                                "[RELAY] Unhandled message type from remote instance"
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    log_eprintln!(
                                        "[RELAY] Failed to deserialize client message: {}",
                                        e
                                    );
                                }
                            }
                        }
                        RelayMessage::InstanceDisconnected {
                            instance_id: _,
                            instance_name,
                        } => {
                            log_println!("[RELAY] Instance '{}' disconnected", instance_name);
                            // Could update UI to show disconnected instance
                        }
                        _ => {
                            // Handle other relay messages if needed
                        }
                    }
                } else {
                    // recv() returned None, channel is closed
                    log_eprintln!("[RELAY] Relay message channel closed, handler exiting");
                    break;
                }
            }
        });
    }

    // Use parsed arguments
    let server = SovaCoreServer::new(cli.ip, cli.port);
    log_println!(
        "[+] Starting Sova server on {}:{}...",
        server.ip,
        server.port
    );
    // Handle potential errors during server start
    match server.start(server_state).await {
        Ok(_) => {
            log_println!("[+] Server listening on {}:{}", server.ip, server.port);

            // Send a test log every 10 seconds to verify client log reception
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
                let mut counter = 1;
                loop {
                    interval.tick().await;
                    log_println!(
                        "[TEST] Periodic log message #{} - if you see this in GUI, logs are working!",
                        counter
                    );
                    counter += 1;
                }
            });
        }
        Err(e) => {
            if e.kind() == ErrorKind::AddrInUse {
                log_eprintln!(
                    "[!] Error: Address {}:{} is already in use.",
                    server.ip,
                    server.port
                );
                log_eprintln!(
                    "    Please check if another Sova instance or application is running on this port."
                );
                std::process::exit(1); // Exit with a non-zero code to indicate failure
            } else {
                // For other errors, print a generic message and the error details
                log_eprintln!("[!] Server failed to start: {}", e);
                std::process::exit(1);
            }
        }
    }

    devices.panic_all_midi_outputs();

    // Clean up audio engine if it was initialized
    if let Some((engine_tx, audio_thread)) = audio_engine_components {
        let _ = engine_tx.send(ScheduledEngineMessage::Immediate(EngineMessage::Stop));
        if let Some(osc_flag) = osc_shutdown_flag {
            osc_flag.store(true, Ordering::Relaxed);
        }
        let _ = audio_thread.join();
    }

    // Clean shutdown for scheduler and world threads
    let _ = sched_iface.send(SchedulerMessage::Shutdown);

    let _ = sched_handle.join();
    let _ = world_handle.join();
}
