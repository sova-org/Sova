use crate::clock::ClockServer;
use crate::compiler::{Compiler, CompilerCollection, bali::BaliCompiler, dummylang::DummyCompiler};
// TimingConfig import removed for now
use bubo_engine::{
    engine::AudioEngine,
    memory::{MemoryPool, SampleLibrary, VoiceMemory},
    registry::ModuleRegistry,
    server::ScheduledEngineMessage,
    types::EngineMessage,
};
use clap::Parser;
use device_map::DeviceMap;
use scene::Scene;
use scene::line::Line;
use schedule::{Scheduler, message::SchedulerMessage, notification::SchedulerNotification};
use server::{BuboCoreServer, ServerState};
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam_channel::bounded;
use std::{collections::HashMap, sync::Arc, thread};
use tokio::sync::{Mutex, watch};
use transcoder::Transcoder;
use world::World;

// Déclaration des modules
pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod lang;
pub mod protocol;
pub mod scene;
pub mod schedule;
pub mod server;
pub mod shared_types;
pub mod transcoder;
pub mod util;
pub mod world;

pub const DEFAULT_MIDI_OUTPUT: &str = "BuboCore";
pub const DEFAULT_TEMPO: f64 = 120.0;
pub const DEFAULT_QUANTUM: f64 = 4.0;
pub const GREETER_LOGO: &str = "
▗▄▄▖ █  ▐▌▗▖    ▄▄▄   ▗▄▄▖▄▄▄   ▄▄▄ ▗▞▀▚▖
▐▌ ▐▌▀▄▄▞▘▐▌   █   █ ▐▌  █   █ █    ▐▛▀▀▘
▐▛▀▚▖     ▐▛▀▚▖▀▄▄▄▀ ▐▌  ▀▄▄▄▀ █    ▝▚▄▄▖
▐▙▄▞▘     ▐▙▄▞▘      ▝▚▄▄▖

";

fn greeter() {
    print!("{}", GREETER_LOGO);
    println!("Version: {}\n", env!("CARGO_PKG_VERSION"));
}


fn initialize_sova_engine(
    cli: &Cli,
    registry: ModuleRegistry,
    osc_shutdown: Arc<AtomicBool>,
) -> (
    crossbeam_channel::Sender<ScheduledEngineMessage>,
    thread::JoinHandle<()>,
    ModuleRegistry,
) {
    println!("[+] Initializing Sova audio engine...");

    // Memory allocation calculations
    let memory_per_voice = cli.buffer_size * 8 * 4;
    let sample_memory = cli.max_audio_buffers * cli.buffer_size * 8;
    let dsp_memory = cli.max_voices * cli.buffer_size * 16 * 4;
    let base_memory = 16 * 1024 * 1024;
    let available_memory =
        base_memory + (cli.max_voices * memory_per_voice) + sample_memory + dsp_memory;

    println!(
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

    println!(
        "   Engine config: {} voices | Sample rate: {} | Buffer: {}",
        cli.max_voices, cli.sample_rate, cli.buffer_size
    );

    // Clone registry for world usage
    let registry_for_world = registry.clone();
    let engine = AudioEngine::new_with_memory(
        cli.sample_rate as f32,
        cli.buffer_size,
        cli.max_voices,
        cli.block_size as usize,
        registry.clone(),
        global_pool,
        voice_memory.clone(),
        sample_library.clone(),
    );

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
        None, // No status channel for bubocore
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
            let mut osc_server = match bubo_engine::server::OscServer::new(
                &osc_host,
                osc_port,
                registry_clone,
                voice_memory_clone,
                sample_library_clone,
                osc_shutdown_clone,
            ) {
                Ok(server) => server,
                Err(e) => {
                    eprintln!("Failed to create OSC server: {}", e);
                    return;
                }
            };
            osc_server.run_lockfree(engine_tx_clone);
        })
        .expect("Failed to spawn OSC thread");

    println!("   Audio engine ready ✓");
    (engine_tx, audio_thread, registry_for_world)
}

// Define the CLI arguments struct
#[derive(Parser, Debug)]
#[clap(author = "Raphaël Forment <raphael.forment@gmail.com>")]
#[clap(author = "Loïg Jezequel <loig.jezequel@univ-nantes.fr>")]
#[clap(author = "Tanguy Dubois <email@address.com>")]
#[command(
    version = "0.0.1",
    about = "BuboCore: A live coding environment server.",
    long_about = "BuboCore acts as the central server for a collaborative live coding environment.\n
    It manages connections from clients (like bubocoretui), handles MIDI devices,
    \nsynchronizes state, and processes scenes."
)]
struct Cli {
    /// IP address to bind the server to
    #[arg(short, long, value_name = "IP_ADDRESS", default_value = "127.0.0.1")]
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
}

#[tokio::main]
async fn main() {
    // ======================================================================
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Handle --list-devices flag before initialization
    if cli.list_devices {
        bubo_engine::list_audio_devices();
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
        eprintln!(
            "[!] Failed to create default virtual MIDI port '{}': {}",
            midi_name, e
        );
    } else {
        println!(
            "[+] Default virtual MIDI port '{}' created successfully.",
            midi_name
        );
        // Assign default MIDI port to Slot 1
        if let Err(e) = devices.assign_slot(1, &midi_name) {
            eprintln!("[!] Failed to assign '{}' to Slot 1: {}", midi_name, e);
        }
    }

    // Create and assign default OSC device (SuperDirt) to Slot 2
    let osc_name = "SuperDirt";
    let osc_ip = "127.0.0.1";
    let osc_port = 57120;
    if let Err(e) = devices.create_osc_output_device(osc_name, osc_ip, osc_port) {
        eprintln!(
            "[!] Failed to create default OSC device '{}': {}",
            osc_name, e
        );
    } else {
        println!(
            "[+] Default OSC device '{}' created successfully ({}:{}).",
            osc_name, osc_ip, osc_port
        );
        // Assign SuperDirt to Slot 2
        if let Err(e) = devices.assign_slot(2, osc_name) {
            eprintln!("[!] Failed to assign '{}' to Slot 2: {}", osc_name, e);
        }
    }

    // ======================================================================
    // Create module registry for both audio engine and world
    let mut registry = ModuleRegistry::new();
    registry.register_default_modules();

    // Conditionally initialize audio engine (Sova)
    let (audio_engine_components, registry_for_world, osc_shutdown_flag) = if cli.audio_engine {
        let osc_shutdown = Arc::new(AtomicBool::new(false));
        let (tx, thread_handle, registry_clone) = initialize_sova_engine(&cli, registry, osc_shutdown.clone());
        (Some((tx, thread_handle)), registry_clone, Some(osc_shutdown))
    } else {
        (None, registry, None)
    };

    // ======================================================================
    // Initialize the world (side effect performer)
    let audio_engine_tx = audio_engine_components.as_ref().map(|(tx, _)| tx.clone());
    let (world_handle, world_iface) =
        World::create(clock_server.clone(), audio_engine_tx, registry_for_world);

    // ======================================================================
    // Extract status receiver and start monitoring thread if audio engine is enabled
    let audio_engine_components = if let Some((tx, thread_handle)) = audio_engine_components {
        Some((tx, thread_handle))
    } else {
        None
    };

    // ======================================================================
    // Initialize the transcoder (list of available compilers)
    let mut compilers: CompilerCollection = HashMap::new();
    // 1) The BaLi compiler
    let bali_compiler = BaliCompiler;
    let dummy_compiler = DummyCompiler;
    compilers.insert(bali_compiler.name(), Box::new(bali_compiler));
    compilers.insert(dummy_compiler.name(), Box::new(dummy_compiler));
    let transcoder = Arc::new(tokio::sync::Mutex::new(Transcoder::new(
        compilers,
        Some("bali".to_string()),
    )));

    // Shared flag for transport state (playing/stopped)
    let shared_atomic_is_playing = Arc::new(AtomicBool::new(false));

    // ======================================================================
    // Initialize the scheduler (scene manager)
    let (sched_handle, sched_iface, sched_update) = Scheduler::create(
        clock_server.clone(),
        devices.clone(),
        world_iface.clone(),
        shared_atomic_is_playing.clone(),
    );
    let (updater, update_notifier) = watch::channel(SchedulerNotification::default());

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
                            *guard.mut_line(*i) = line.clone()
                        }
                        SchedulerNotification::FramePositionChanged(_positions) => {
                            // No update to scene needed for this notification
                        }
                        SchedulerNotification::EnableFrames(line_index, frame_indices) => {
                            guard.mut_line(*line_index).enable_frames(frame_indices);
                        }
                        SchedulerNotification::DisableFrames(line_index, frame_indices) => {
                            guard.mut_line(*line_index).disable_frames(frame_indices);
                        }
                        SchedulerNotification::UploadedScript(_, _, _script) => {}
                        SchedulerNotification::UpdatedLineFrames(frame_index, items) => {
                            guard.mut_line(*frame_index).set_frames(items.clone());
                        }
                        SchedulerNotification::AddedLine(line) => {
                            guard.add_line(line.clone());
                        }
                        SchedulerNotification::RemovedLine(index) => {
                            guard.remove_line(*index);
                        }
                        SchedulerNotification::SceneLengthChanged(length) => {
                            guard.set_length(*length);
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
        eprintln!("[!] Failed to send initial scene to scheduler: {}", e);
        std::process::exit(1);
    }

    let server_state = ServerState::new(
        scene_image,
        clock_server,
        devices.clone(),
        world_iface,
        sched_iface.clone(),
        updater,
        update_notifier,
        transcoder,
        shared_atomic_is_playing.clone(),
    );

    // Use parsed arguments
    let server = BuboCoreServer::new(cli.ip, cli.port);
    println!(
        "[+] Starting BuboCore server on {}:{}...",
        server.ip, server.port
    );
    // Handle potential errors during server start
    match server.start(server_state).await {
        Ok(_) => {}
        Err(e) => {
            if e.kind() == ErrorKind::AddrInUse {
                eprintln!(
                    "[!] Error: Address {}:{} is already in use.",
                    server.ip, server.port
                );
                eprintln!(
                    "    Please check if another BuboCore instance or application is running on this port."
                );
                std::process::exit(1); // Exit with a non-zero code to indicate failure
            } else {
                // For other errors, print a generic message and the error details
                eprintln!("[!] Server failed to start: {}", e);
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
