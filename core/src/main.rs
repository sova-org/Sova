use crate::clock::ClockServer;
use crate::lang::{bali::BaliCompiler, boinx::BoinxInterpreterFactory, imp::ImpCompiler};
use crate::logger::get_logger;
use crate::schedule::ActionTiming;
use crate::vm::LanguageCenter;
use crate::vm::interpreter::InterpreterDirectory;
use clap::Parser;
use device_map::DeviceMap;
use scene::Line;
use scene::Scene;
use schedule::SchedulerMessage;
use server::{ServerState, SovaCoreServer};
use std::io::ErrorKind;
use std::sync::Arc;
use thread_priority::{ThreadPriority, set_current_thread_priority};
use tokio::sync::Mutex;
use vm::Transcoder;

// Déclaration des modules
pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod init;
pub mod lang;
pub mod logger;
pub mod protocol;
pub mod scene;
pub mod schedule;
pub mod server;
pub mod util;
pub mod vm;
pub mod world;

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

    /// Initial tempo in BPM
    #[arg(short, long, value_name = "BPM", default_value_t = DEFAULT_TEMPO)]
    tempo: f64,

    /// Initial quantum in beats
    #[arg(short, long, value_name = "BEATS", default_value_t = DEFAULT_QUANTUM)]
    quantum: f64,
}

#[tokio::main]
async fn main() {
    // ======================================================================
    // Set real-time priority for the main thread (critical for musical timing)
    match set_current_thread_priority(ThreadPriority::Max) {
        Ok(_) => eprintln!("[+] Real-time priority set successfully"),
        Err(e) => eprintln!("[!] Failed to set real-time priority: {:?}", e),
    }

    // ======================================================================
    // Parse CLI arguments first
    let cli = Cli::parse();

    // ======================================================================
    // Initialize logger and immediately set up full mode for complete logging
    crate::logger::init_standalone();

    // Set up notification channel and switch to full mode IMMEDIATELY
    // This ensures ALL logs (including startup) reach file, terminal, and clients
    let (update_sender, _) =
        tokio::sync::broadcast::channel::<crate::schedule::SovaNotification>(256);
    crate::logger::set_full_mode(update_sender.clone());

    // Test log to verify full mode works
    log_info!("Logger initialized in full mode - all logs will reach file, terminal, and clients");

    // ======================================================================
    // Splash screen
    greeter();

    // ======================================================================
    // Initialize the clock
    let clock_server = Arc::new(ClockServer::new(cli.tempo, cli.quantum));
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
    // Initialize the transcoder (list of available compilers) and interpreter directory
    let mut transcoder = Transcoder::default();
    transcoder.add_compiler(BaliCompiler);
    transcoder.add_compiler(ImpCompiler);

    let mut interpreters = InterpreterDirectory::new();
    interpreters.add_factory(BoinxInterpreterFactory);

    let languages = Arc::new(LanguageCenter {
        transcoder,
        interpreters,
    });

    // ======================================================================
    // Initialize the scheduler (scene manager)
    let (world_handle, sched_handle, sched_iface, sched_update) =
        init::start_scheduler_and_world(clock_server.clone(), devices.clone(), languages.clone());

    // ======================================================================
    // Initialize the default scene loaded when the server starts
    let initial_scene = Scene::new(vec![Line::new(vec![1.0])]);
    let scene_image = Arc::new(Mutex::new(initial_scene.clone()));

    if let Err(e) = sched_iface.send(SchedulerMessage::SetScene(
        initial_scene,
        ActionTiming::Immediate,
    )) {
        log_eprintln!("[!] Failed to send initial scene to scheduler: {}", e);
        std::process::exit(1);
    }

    let server_state = ServerState::new(
        scene_image,
        clock_server,
        devices.clone(),
        sched_iface.clone(),
        update_sender.clone(),
        languages,
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

    // Clean shutdown for scheduler and world threads
    let _ = sched_iface.send(SchedulerMessage::Shutdown);

    let _ = sched_handle.join();
    let _ = world_handle.join();
}
