use crate::clock::ClockServer;
use crate::compiler::ExternalCompiler;
use crate::compiler::{bali::BaliCompiler, dummylang::DummyCompiler};
use crate::lang::interpreter::boinx::BoinxInterpreterFactory;
use crate::lang::interpreter::external::ExternalInterpreterFactory;
use crate::lang::interpreter::InterpreterDirectory;
use crate::lang::LanguageCenter;
use crate::logger::get_logger;
use crate::protocol::audio_engine_proxy::AudioEngineProxy;
use crate::schedule::ActionTiming;
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
use schedule::SchedulerMessage;
use server::{SovaCoreServer, ServerState};
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{sync::Arc, thread};
use tokio::sync::Mutex;
use lang::Transcoder;

// Déclaration des modules
pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod lang;
pub mod logger;
pub mod protocol;
pub mod scene;
pub mod schedule;
pub mod server;
pub mod util;
pub mod world;
pub mod init;

pub use protocol::log::{LogMessage, Severity};

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
    let (update_sender, update_receiver) = tokio::sync::watch::channel(
        crate::schedule::SovaNotification::default(),
    );
    crate::logger::set_full_mode(update_sender.clone());

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
    
    // Conditionally initialize audio engine (Sova)
    let (audio_engine_components, osc_shutdown_flag) =
        if cli.audio_engine {
            let mut registry = ModuleRegistry::new();
            registry.register_default_modules();
            let osc_shutdown = Arc::new(AtomicBool::new(false));
            let (tx, thread_handle, registry_clone, log_rx) =
                initialize_sova_engine(&cli, registry, osc_shutdown.clone());
            let mut proxy = AudioEngineProxy::new(tx.clone(), registry_clone);
            proxy.log_callback(log_rx, |msg| {
                get_logger().log_message(msg);
            });
            let _ = devices.connect_audio_engine("SovaEngine", proxy);
            (
                Some((tx, thread_handle)),
                Some(osc_shutdown),
            )
        } else {
            (None, None)
        };

    // ======================================================================
    // Initialize the transcoder (list of available compilers) and interpreter directory
    let mut transcoder = Transcoder::default();
    transcoder.add_compiler(BaliCompiler);
    transcoder.add_compiler(DummyCompiler);
    transcoder.add_compiler(ExternalCompiler);

    let mut interpreters = InterpreterDirectory::new();
    interpreters.add_factory(BoinxInterpreterFactory);
    interpreters.add_factory(ExternalInterpreterFactory);

    let languages = Arc::new(LanguageCenter { transcoder, interpreters });

    // ======================================================================
    // Initialize the scheduler (scene manager)
    let (world_handle, sched_handle, sched_iface, sched_update) = init::start_scheduler_and_world(
        clock_server.clone(),
        devices.clone(),
        languages.clone(),
    );

    // ======================================================================
    // Initialize the default scene loaded when the server starts
    let initial_scene = Scene::new(vec![Line::new(vec![1.0])]);
    let scene_image = Arc::new(Mutex::new(initial_scene.clone()));

    if let Err(e) = sched_iface.send(SchedulerMessage::SetScene(initial_scene, ActionTiming::Immediate)) {
        log_eprintln!("[!] Failed to send initial scene to scheduler: {}", e);
        std::process::exit(1);
    }

    let server_state = ServerState::new(
        scene_image,
        clock_server,
        devices.clone(),
        sched_iface.clone(),
        update_sender.clone(),
        update_receiver,
        languages
    );

    // Use parsed arguments
    let server = SovaCoreServer::new(cli.ip, cli.port, server_state);
    log_println!(
        "[+] Starting Sova server on {}:{}...",
        server.ip,
        server.port
    );
    // Handle potential errors during server start
    match server.start(sched_update).await {
        Ok(_) => {
            log_println!("[+] Server listening on {}:{}", server.ip, server.port);
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
        log_println!("[~] Exiting audio engine...");
        let _ = engine_tx.send(ScheduledEngineMessage::Immediate(EngineMessage::Stop));
        if let Some(osc_flag) = osc_shutdown_flag {
            osc_flag.store(true, Ordering::Relaxed);
        }
        let _ = audio_thread.join();
        log_println!("[+] Audio engine quitted...");
    }

    // Clean shutdown for scheduler and world threads
    let _ = sched_iface.send(SchedulerMessage::Shutdown);

    let _ = sched_handle.join();
    let _ = world_handle.join();
}
