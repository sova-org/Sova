use crate::clock::ClockServer;
use crate::compiler::{Compiler, CompilerCollection, bali::BaliCompiler, dummylang::DummyCompiler};
use clap::Parser;
use device_map::DeviceMap;
use scene::Scene;
use scene::line::Line;
use schedule::{Scheduler, message::SchedulerMessage, notification::SchedulerNotification};
use server::{BuboCoreServer, ServerState};
use std::io::ErrorKind;
use std::sync::atomic::AtomicBool;
use std::{collections::HashMap, sync::Arc, thread, sync::mpsc};
use tokio::sync::{Mutex, watch};
use transcoder::Transcoder;
use world::World;
use bubo_engine::{
    engine::AudioEngine, 
    server::ScheduledEngineMessage,
    memory::{MemoryPool, SampleLibrary, VoiceMemory},
    registry::ModuleRegistry,
    types::EngineMessage,
};

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

fn initialize_sova_engine(cli: &Cli) -> (Arc<std::sync::Mutex<AudioEngine>>, mpsc::Sender<ScheduledEngineMessage>, thread::JoinHandle<()>) {
    println!("[+] Initializing Sova audio engine...");
    
    // Memory allocation calculations
    let memory_per_voice = cli.buffer_size * 8 * 4;
    let sample_memory = cli.max_audio_buffers * cli.buffer_size * 8;
    let dsp_memory = cli.max_voices * cli.buffer_size * 16 * 4;
    let base_memory = 16 * 1024 * 1024;
    let available_memory = base_memory + 
        (cli.max_voices * memory_per_voice) + 
        sample_memory + 
        dsp_memory;

    println!("   Memory allocation: {}MB total", available_memory / (1024 * 1024));

    let global_pool = Arc::new(MemoryPool::new(available_memory));
    let voice_memory = Arc::new(VoiceMemory::new());
    
    // Sample library
    let mut sample_library = SampleLibrary::new(
        cli.max_audio_buffers, 
        &cli.audio_files_location, 
        cli.sample_rate
    );
    sample_library.preload_all_samples();
    let sample_library = Arc::new(std::sync::Mutex::new(sample_library));

    // Module registry
    let mut registry = ModuleRegistry::new();
    registry.register_default_modules();
    registry.set_timestamp_tolerance(cli.timestamp_tolerance_ms);

    println!("   Engine config: {} voices | Sample rate: {} | Buffer: {}", 
        cli.max_voices, cli.sample_rate, cli.buffer_size);

    // Create audio engine
    let audio_engine = Arc::new(std::sync::Mutex::new(AudioEngine::new_with_memory(
        cli.sample_rate as f32,
        cli.buffer_size,
        cli.max_voices,
        cli.block_size as usize,
        registry,
        global_pool,
        voice_memory,
        sample_library,
    )));

    // Create message channel for engine communication
    let (engine_tx, engine_rx) = mpsc::channel();

    // Start audio thread
    let engine_clone = Arc::clone(&audio_engine);
    let audio_thread = AudioEngine::start_audio_thread(
        engine_clone,
        cli.block_size,
        cli.max_voices,
        cli.sample_rate,
        cli.buffer_size,
        cli.output_device.clone(),
        engine_rx,
    );

    println!("   Audio engine ready ✓");
    (audio_engine, engine_tx, audio_thread)
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
}

#[tokio::main]
async fn main() {
    // ======================================================================
    // Splash screen
    greeter();

    // ======================================================================
    // Parse CLI arguments
    let cli = Cli::parse();

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
    // Conditionally initialize audio engine (Sova)
    let audio_engine_components = if cli.audio_engine {
        let (engine, tx, thread_handle) = initialize_sova_engine(&cli);
        Some((engine, tx, thread_handle))
    } else {
        None
    };

    // ======================================================================
    // Initialize the world (side effect performer)
    let audio_engine_tx = audio_engine_components.as_ref().map(|(_, tx, _)| tx.clone());
    let (world_handle, world_iface) = World::create(clock_server.clone(), audio_engine_tx);

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
        sched_iface,
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

    println!("\n[-] Stopping BuboCore...");
    // Send MIDI Panic (All Notes Off) before shutting down completely
    devices.panic_all_midi_outputs();
    
    // Clean up audio engine if it was initialized
    if let Some((_, engine_tx, audio_thread)) = audio_engine_components {
        println!("[-] Stopping audio engine...");
        // Send shutdown message to audio thread
        let shutdown_msg = ScheduledEngineMessage::Immediate(
            EngineMessage::Stop
        );
        let _ = engine_tx.send(shutdown_msg);
        // Now wait for thread to exit
        let _ = audio_thread.join();
    }
    
    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
