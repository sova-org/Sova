use sova_core::clock::{Clock, ClockServer};
use sova_core::lang::{bali::BaliCompiler, bob::BobCompiler, boinx::BoinxInterpreterFactory, forth::ForthInterpreterFactory};
use sova_core::schedule::ActionTiming;
use sova_core::vm::LanguageCenter;
use sova_core::vm::interpreter::InterpreterDirectory;
use sova_core::device_map::DeviceMap;
use sova_core::scene::{Line, Scene};
use sova_core::schedule::{SchedulerMessage, SovaNotification};
use sova_core::vm::Transcoder;
use sova_core::{log_eprintln, log_println, log_print, log_info};

use clap::Parser;
use doux_sova::AudioEngineState;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use thread_priority::{ThreadPriority, set_current_thread_priority};
use tokio::sync::Mutex;

use sova_server::{ServerState, SovaCoreServer};

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
    log_print!("{}", GREETER_LOGO);
    log_println!("Version: {}\n", env!("CARGO_PKG_VERSION"));
}

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
    #[arg(short, long, value_name = "IP_ADDRESS", default_value = "0.0.0.0")]
    ip: String,

    #[arg(short, long, value_name = "PORT", default_value_t = 8080)]
    port: u16,

    #[arg(short, long, value_name = "BPM", default_value_t = DEFAULT_TEMPO)]
    tempo: f64,

    #[arg(short, long, value_name = "BEATS", default_value_t = DEFAULT_QUANTUM)]
    quantum: f64,

    /// Disable audio engine (no Doux)
    #[arg(long, default_value_t = false)]
    no_audio: bool,

    /// Audio output device (name or index, uses system default if not specified)
    #[arg(long, value_name = "DEVICE")]
    audio_device: Option<String>,

    /// Audio input device (name or index, uses system default if not specified)
    #[arg(long, value_name = "DEVICE")]
    audio_input_device: Option<String>,

    /// Number of audio output channels (default: 2)
    #[arg(long, value_name = "CHANNELS", default_value_t = 2)]
    audio_channels: u16,

    /// Sample directory path (can be specified multiple times)
    #[arg(long = "sample-path", value_name = "PATH", action = clap::ArgAction::Append)]
    sample_paths: Vec<PathBuf>,
}

#[tokio::main]
async fn main() {
    match set_current_thread_priority(ThreadPriority::Max) {
        Ok(_) => eprintln!("[+] Real-time priority set successfully"),
        Err(e) => eprintln!("[!] Failed to set real-time priority: {:?}", e),
    }

    let cli = Cli::parse();

    sova_core::logger::init_standalone();

    let (update_sender, _) =
        tokio::sync::broadcast::channel::<SovaNotification>(256);
    sova_core::logger::set_full_mode(update_sender.clone());

    log_info!("Logger initialized in full mode - all logs will reach file, terminal, and clients");

    greeter();

    let clock_server = Arc::new(ClockServer::new(cli.tempo, cli.quantum));
    clock_server.link.enable(true);

    let devices = Arc::new(DeviceMap::new());
    let midi_name = DEFAULT_MIDI_OUTPUT.to_owned();
    if let Err(e) = devices.create_virtual_midi_port(&midi_name) {
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
        if let Err(e) = devices.assign_slot(1, &midi_name) {
            log_eprintln!("[!] Failed to assign '{}' to Slot 1: {}", midi_name, e);
        }
    }

    let audio_engine_state = Arc::new(StdMutex::new(AudioEngineState::default()));
    let doux_manager = if !cli.no_audio {
        let config = doux_sova::DouxConfig::default()
            .with_channels(cli.audio_channels);
        let config = if let Some(ref device) = cli.audio_device {
            config.with_output_device(device)
        } else {
            config
        };
        let config = if let Some(ref device) = cli.audio_input_device {
            config.with_input_device(device)
        } else {
            config
        };
        let config = cli.sample_paths.iter().fold(config, |c, p| c.with_sample_path(p));

        match doux_sova::DouxManager::new(config) {
            Ok(mut manager) => {
                let sync_time = Clock::from(&clock_server).micros();
                match manager.start(sync_time) {
                    Ok(proxy) => {
                        let audio_name = "Doux";
                        if let Err(e) = devices.connect_audio_engine(audio_name, proxy) {
                            log_eprintln!("[!] Failed to register Doux engine: {}", e);
                            None
                        } else {
                            log_println!("[+] Doux audio engine started successfully.");
                            if let Err(e) = devices.assign_slot(2, audio_name) {
                                log_eprintln!("[!] Failed to assign Doux to Slot 2: {}", e);
                            }
                            if let Ok(mut state) = audio_engine_state.lock() {
                                *state = manager.state();
                            }
                            Some(manager)
                        }
                    }
                    Err(e) => {
                        log_eprintln!("[!] Failed to start Doux audio engine: {:?}", e);
                        if let Ok(mut state) = audio_engine_state.lock() {
                            state.error = Some(format!("{:?}", e));
                        }
                        None
                    }
                }
            }
            Err(e) => {
                log_eprintln!("[!] Failed to create Doux manager: {:?}", e);
                if let Ok(mut state) = audio_engine_state.lock() {
                    state.error = Some(format!("{:?}", e));
                }
                None
            }
        }
    } else {
        log_println!("[~] Audio engine disabled (--no-audio flag).");
        None
    };

    let mut transcoder = Transcoder::default();
    transcoder.add_compiler(BaliCompiler);
    transcoder.add_compiler(BobCompiler);

    let mut interpreters = InterpreterDirectory::new();
    interpreters.add_factory(BoinxInterpreterFactory);
    interpreters.add_factory(ForthInterpreterFactory);

    let languages = Arc::new(LanguageCenter {
        transcoder,
        interpreters,
    });

    let (world_handle, sched_handle, sched_iface, sched_update) =
        sova_core::init::start_scheduler_and_world(clock_server.clone(), devices.clone(), languages.clone());

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
        audio_engine_state,
    );

    let server = SovaCoreServer::new(cli.ip, cli.port, server_state);
    log_println!(
        "[+] Starting Sova server on {}:{}...",
        server.ip,
        server.port
    );
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
                std::process::exit(1);
            } else {
                log_eprintln!("[!] Server failed to start: {}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(manager) = doux_manager {
        manager.hush();
    }

    devices.panic_all_midi_outputs();

    let _ = sched_iface.send(SchedulerMessage::Shutdown);

    let _ = sched_handle.join();
    let _ = world_handle.join();
}
