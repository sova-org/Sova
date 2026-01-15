use langs::{
    bali::BaliCompiler, bob::BobCompiler, boinx::BoinxInterpreterFactory,
    forth::ForthInterpreterFactory,
};
#[cfg(feature = "audio")]
use sova_core::clock::Clock;
use sova_core::clock::ClockServer;
use sova_core::device_map::DeviceMap;
use sova_core::scene::{Line, Scene};
use sova_core::schedule::ActionTiming;
use sova_core::schedule::{SchedulerMessage, SovaNotification};
use sova_core::vm::LanguageCenter;
use sova_core::vm::Transcoder;
use sova_core::vm::interpreter::InterpreterDirectory;
use sova_core::{log_eprintln, log_info, log_print, log_println};

use clap::Parser;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use thread_priority::{ThreadPriority, set_current_thread_priority};
use tokio::sync::Mutex;

use sova_server::{AudioEngineState, ServerState, SovaCoreServer};

#[cfg(feature = "audio")]
struct AudioRuntime {
    manager: sova_server::audio::DouxManager,
    telemetry_handle: std::thread::JoinHandle<()>,
    telemetry_running: Arc<AtomicBool>,
}

#[cfg(feature = "audio")]
use std::path::PathBuf;

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

    #[cfg(feature = "audio")]
    /// Disable audio engine (no Doux)
    #[arg(long, default_value_t = false)]
    no_audio: bool,

    #[cfg(feature = "audio")]
    /// Audio output device (name or index, uses system default if not specified)
    #[arg(long, value_name = "DEVICE")]
    audio_device: Option<String>,

    #[cfg(feature = "audio")]
    /// Audio input device (name or index, uses system default if not specified)
    #[arg(long, value_name = "DEVICE")]
    audio_input_device: Option<String>,

    #[cfg(feature = "audio")]
    /// Number of audio output channels (default: 2)
    #[arg(long, value_name = "CHANNELS", default_value_t = 2)]
    audio_channels: u16,

    #[cfg(feature = "audio")]
    /// Sample directory path (can be specified multiple times)
    #[arg(long = "sample-path", value_name = "PATH", action = clap::ArgAction::Append)]
    sample_paths: Vec<PathBuf>,
}

#[tokio::main]
async fn main() {
    match set_current_thread_priority(ThreadPriority::Max) {
        Ok(_) => eprintln!("Real-time priority set successfully"),
        Err(e) => eprintln!("Failed to set real-time priority: {:?}", e),
    }

    let cli = Cli::parse();

    sova_core::logger::init_standalone();

    let (update_sender, _) = tokio::sync::broadcast::channel::<SovaNotification>(256);
    sova_core::logger::set_full_mode(update_sender.clone());

    log_info!("Logger initialized in full mode - all logs will reach file, terminal, and clients");

    greeter();

    let clock_server = Arc::new(ClockServer::new(cli.tempo, cli.quantum));
    clock_server.link.enable(true);

    let devices = Arc::new(DeviceMap::new());
    let midi_name = DEFAULT_MIDI_OUTPUT.to_owned();
    if let Err(e) = devices.create_virtual_midi_port(&midi_name) {
        log_eprintln!(
            "Failed to create default virtual MIDI port '{}': {}",
            midi_name,
            e
        );
    } else {
        log_println!(
            "Default virtual MIDI port '{}' created successfully.",
            midi_name
        );
        if let Err(e) = devices.assign_slot(1, &midi_name) {
            log_eprintln!("Failed to assign '{}' to Slot 1: {}", midi_name, e);
        }
    }

    let audio_engine_state = Arc::new(StdMutex::new(AudioEngineState::default()));

    #[cfg(feature = "audio")]
    let audio_runtime = if !cli.no_audio {
        use sova_server::audio::{DouxConfig, DouxManager};
        let config = DouxConfig::default().with_channels(cli.audio_channels);
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
        let config = cli
            .sample_paths
            .iter()
            .fold(config, |c, p| c.with_sample_path(p));

        match DouxManager::new(config) {
            Ok(mut manager) => {
                let sync_time = Clock::from(&clock_server).micros();
                match manager.start(sync_time) {
                    Ok(proxy) => {
                        let audio_name = "Doux";
                        if let Err(e) = devices.connect_audio_engine(audio_name, proxy) {
                            log_eprintln!("Failed to register Doux engine: {}", e);
                            None
                        } else {
                            log_println!("Doux audio engine started successfully.");
                            if let Err(e) = devices.assign_slot(2, audio_name) {
                                log_eprintln!("Failed to assign Doux to Slot 2: {}", e);
                            }
                            if let Ok(mut state) = audio_engine_state.lock() {
                                *state = manager.state();
                            }
                            let engine_handle = manager.engine_handle();
                            let state_cache = Arc::clone(&audio_engine_state);
                            let telemetry_running = Arc::new(AtomicBool::new(true));
                            let running_flag = Arc::clone(&telemetry_running);
                            let telemetry_handle = std::thread::spawn(move || {
                                while running_flag.load(Ordering::Relaxed) {
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                    if let Ok(engine) = engine_handle.lock() {
                                        if let Ok(mut cache) = state_cache.lock() {
                                            cache.cpu_load = engine.metrics.load.get_load();
                                            cache.active_voices = engine.active_voices;
                                            cache.peak_voices =
                                                engine.metrics.peak_voices.load(Ordering::Relaxed)
                                                    as usize;
                                            cache.schedule_depth = engine
                                                .metrics
                                                .schedule_depth
                                                .load(Ordering::Relaxed)
                                                as usize;
                                            cache.sample_pool_mb = engine.metrics.sample_pool_mb();
                                        }
                                    }
                                }
                            });
                            Some(AudioRuntime {
                                manager,
                                telemetry_handle,
                                telemetry_running,
                            })
                        }
                    }
                    Err(e) => {
                        log_eprintln!("Failed to start Doux audio engine: {:?}", e);
                        if let Ok(mut state) = audio_engine_state.lock() {
                            state.error = Some(format!("{:?}", e));
                        }
                        None
                    }
                }
            }
            Err(e) => {
                log_eprintln!("Failed to create Doux manager: {:?}", e);
                if let Ok(mut state) = audio_engine_state.lock() {
                    state.error = Some(format!("{:?}", e));
                }
                None
            }
        }
    } else {
        log_println!("Audio engine disabled (--no-audio flag).");
        None
    };

    #[cfg(not(feature = "audio"))]
    log_println!("Audio engine not compiled (build without 'audio' feature).");

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
        sova_core::init::start_scheduler_and_world(
            clock_server.clone(),
            devices.clone(),
            languages.clone(),
        );

    let initial_scene = Scene::new(vec![Line::new(vec![1.0])]);
    let scene_image = Arc::new(Mutex::new(initial_scene.clone()));

    if let Err(e) = sched_iface.send(SchedulerMessage::SetScene(
        initial_scene,
        ActionTiming::Immediate,
    )) {
        log_eprintln!("Failed to send initial scene to scheduler: {}", e);
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
    log_println!("Starting Sova server on {}:{}...", server.ip, server.port);
    match server.start(sched_update).await {
        Ok(_) => {
            log_println!("Server listening on {}:{}", server.ip, server.port);
        }
        Err(e) => {
            if e.kind() == ErrorKind::AddrInUse {
                log_eprintln!(
                    "Error: Address {}:{} is already in use.",
                    server.ip,
                    server.port
                );
                log_eprintln!(
                    "    Please check if another Sova instance or application is running on this port."
                );
                std::process::exit(1);
            } else {
                log_eprintln!("Server failed to start: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(feature = "audio")]
    if let Some(runtime) = audio_runtime {
        runtime.telemetry_running.store(false, Ordering::Relaxed);
        runtime.manager.hush();
        let _ = runtime.telemetry_handle.join();
    }

    devices.panic_all_midi_outputs();

    let _ = sched_iface.send(SchedulerMessage::Shutdown);

    let _ = sched_handle.join();
    let _ = world_handle.join();
}
