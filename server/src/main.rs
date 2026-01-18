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

use clap::Parser;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use thread_priority::{ThreadPriority, set_current_thread_priority};
use tokio::sync::Mutex;

use sova_server::{AudioEngineState, AudioRestartConfig, AudioRestartRequest, ServerState, SovaCoreServer};

#[cfg(feature = "audio")]
struct AudioRuntime {
    audio_thread_handle: std::thread::JoinHandle<()>,
    running: Arc<AtomicBool>,
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
    print!("{}", GREETER_LOGO);
    println!("Version: {}\n", env!("CARGO_PKG_VERSION"));
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
    /// Audio buffer size in samples (lower = less latency, higher = more stable)
    #[arg(long, value_name = "SAMPLES")]
    audio_buffer_size: Option<u32>,

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

    println!("Logger initialized in full mode.");

    greeter();

    let clock_server = Arc::new(ClockServer::new(cli.tempo, cli.quantum));
    clock_server.link.enable(true);

    let devices = Arc::new(DeviceMap::new());
    let midi_name = DEFAULT_MIDI_OUTPUT.to_owned();
    if let Err(e) = devices.create_virtual_midi_port(&midi_name) {
        eprintln!(
            "Failed to create default virtual MIDI port '{}': {}",
            midi_name, e
        );
    } else {
        println!(
            "Default virtual MIDI port '{}' created successfully.",
            midi_name
        );
        if let Err(e) = devices.assign_slot(1, &midi_name) {
            eprintln!("Failed to assign '{}' to Slot 1: {}", midi_name, e);
        }
    }

    let audio_engine_state = Arc::new(StdMutex::new(AudioEngineState::default()));

    #[cfg(feature = "audio")]
    let (audio_restart_tx, audio_runtime) = if !cli.no_audio {
        use sova_server::audio::{DouxConfig, DouxManager};

        let initial_config = AudioRestartConfig {
            device: cli.audio_device.clone(),
            input_device: cli.audio_input_device.clone(),
            channels: cli.audio_channels,
            buffer_size: cli.audio_buffer_size,
            sample_paths: cli.sample_paths.clone(),
        };

        let (restart_tx, restart_rx) = crossbeam_channel::unbounded::<AudioRestartRequest>();
        let running = Arc::new(AtomicBool::new(true));
        let running_flag = Arc::clone(&running);
        let state_cache = Arc::clone(&audio_engine_state);
        let scope_sender = update_sender.clone();
        let devices_clone = Arc::clone(&devices);
        let clock_server_clone = Arc::clone(&clock_server);

        let audio_thread_handle = std::thread::spawn(move || {
            fn build_doux_config(cfg: &AudioRestartConfig) -> DouxConfig {
                let mut config = DouxConfig::default().with_channels(cfg.channels);
                if let Some(ref device) = cfg.device {
                    config = config.with_output_device(device);
                }
                if let Some(ref device) = cfg.input_device {
                    config = config.with_input_device(device);
                }
                for path in &cfg.sample_paths {
                    config = config.with_sample_path(path);
                }
                if let Some(size) = cfg.buffer_size {
                    config = config.with_buffer_size(size);
                }
                config
            }

            let doux_config = build_doux_config(&initial_config);
            let mut manager: Option<DouxManager> = match DouxManager::new(doux_config) {
                Ok(mut mgr) => {
                    let sync_time = Clock::from(&clock_server_clone).micros();
                    match mgr.start(sync_time) {
                        Ok(proxy) => {
                            let audio_name = "Doux";
                            if let Err(e) = devices_clone.connect_audio_engine(audio_name, proxy) {
                                eprintln!("Failed to register Doux engine: {}", e);
                                if let Ok(mut state) = state_cache.lock() {
                                    state.error = Some(format!("Failed to register: {}", e));
                                }
                                None
                            } else {
                                println!("Doux audio engine started successfully.");
                                if let Err(e) = devices_clone.assign_slot(2, audio_name) {
                                    eprintln!("Failed to assign Doux to Slot 2: {}", e);
                                }
                                if let Ok(mut state) = state_cache.lock() {
                                    *state = mgr.state();
                                }
                                Some(mgr)
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to start Doux audio engine: {:?}", e);
                            if let Ok(mut state) = state_cache.lock() {
                                state.error = Some(format!("{:?}", e));
                            }
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create Doux manager: {:?}", e);
                    if let Ok(mut state) = state_cache.lock() {
                        state.error = Some(format!("{:?}", e));
                    }
                    None
                }
            };

            let mut frame_counter = 0u32;

            while running_flag.load(Ordering::Relaxed) {
                // Check for restart requests (non-blocking)
                if let Ok(request) = restart_rx.try_recv() {
                    println!("[ audio ] Received restart request");

                    // Stop current audio engine
                    if let Some(ref mut mgr) = manager {
                        mgr.hush();
                        let _ = devices_clone.remove_output_device("Doux");
                        mgr.stop();
                    }

                    // Build new config and restart
                    let new_config = build_doux_config(&request.config);
                    let result = match DouxManager::new(new_config) {
                        Ok(mut new_mgr) => {
                            let sync_time = Clock::from(&clock_server_clone).micros();
                            match new_mgr.start(sync_time) {
                                Ok(proxy) => {
                                    if let Err(e) = devices_clone.connect_audio_engine("Doux", proxy) {
                                        manager = None;
                                        if let Ok(mut state) = state_cache.lock() {
                                            state.running = false;
                                            state.error = Some(format!("Failed to register: {}", e));
                                        }
                                        Err(format!("Failed to register audio engine: {}", e))
                                    } else {
                                        if let Err(e) = devices_clone.assign_slot(2, "Doux") {
                                            eprintln!("Failed to assign Doux to Slot 2: {}", e);
                                        }
                                        let new_state = new_mgr.state();
                                        if let Ok(mut state) = state_cache.lock() {
                                            *state = new_state.clone();
                                        }
                                        manager = Some(new_mgr);
                                        println!("[ audio ] Restart successful");
                                        Ok(new_state)
                                    }
                                }
                                Err(e) => {
                                    manager = None;
                                    if let Ok(mut state) = state_cache.lock() {
                                        state.running = false;
                                        state.error = Some(format!("{:?}", e));
                                    }
                                    Err(format!("Failed to start audio engine: {:?}", e))
                                }
                            }
                        }
                        Err(e) => {
                            manager = None;
                            if let Ok(mut state) = state_cache.lock() {
                                state.running = false;
                                state.error = Some(format!("{:?}", e));
                            }
                            Err(format!("Failed to create audio manager: {:?}", e))
                        }
                    };

                    let _ = request.response_tx.send(result);
                }

                // Telemetry updates
                std::thread::sleep(std::time::Duration::from_millis(16));
                frame_counter += 1;

                if let Some(ref mgr) = manager {
                    // Stream scope data every frame (~60fps)
                    if let Some(scope) = mgr.scope_capture() {
                        let peaks = scope.read_peaks(256);
                        let _ = scope_sender.send(SovaNotification::ScopeData(peaks));
                    }

                    // Update telemetry every 6 frames (~100ms)
                    if frame_counter % 6 == 0 {
                        if let Ok(engine) = mgr.engine_handle().lock() {
                            if let Ok(mut cache) = state_cache.lock() {
                                cache.cpu_load = engine.metrics.load.get_load();
                                cache.active_voices = engine.active_voices;
                                cache.peak_voices = engine.metrics.peak_voices.load(Ordering::Relaxed) as usize;
                                cache.schedule_depth = engine.metrics.schedule_depth.load(Ordering::Relaxed) as usize;
                                cache.sample_pool_mb = engine.metrics.sample_pool_mb();
                            }
                        }
                    }
                }
            }

            // Cleanup on shutdown
            if let Some(mut mgr) = manager {
                mgr.hush();
                let _ = devices_clone.remove_output_device("Doux");
                mgr.stop();
            }
        });

        (
            Some(restart_tx),
            Some(AudioRuntime {
                audio_thread_handle,
                running,
            }),
        )
    } else {
        println!("Audio engine disabled (--no-audio flag).");
        (None, None)
    };

    #[cfg(not(feature = "audio"))]
    let audio_restart_tx: Option<crossbeam_channel::Sender<AudioRestartRequest>> = None;

    #[cfg(not(feature = "audio"))]
    println!("Audio engine not compiled (build without 'audio' feature).");

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
        eprintln!("Failed to send initial scene to scheduler: {}", e);
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
        audio_restart_tx,
    );

    let server = SovaCoreServer::new(cli.ip, cli.port, server_state);
    println!("Starting Sova server on {}:{}...", server.ip, server.port);
    match server.start(sched_update).await {
        Ok(_) => {}
        Err(e) => {
            if e.kind() == ErrorKind::AddrInUse {
                eprintln!(
                    "Error: Address {}:{} is already in use.",
                    server.ip, server.port
                );
                eprintln!(
                    "    Please check if another Sova instance or application is running on this port."
                );
                std::process::exit(1);
            } else {
                eprintln!("Server failed to start: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(feature = "audio")]
    if let Some(runtime) = audio_runtime {
        runtime.running.store(false, Ordering::Relaxed);
        let _ = runtime.audio_thread_handle.join();
    }

    devices.panic_all_midi_outputs();

    let _ = sched_iface.send(SchedulerMessage::Shutdown);

    let _ = sched_handle.join();
    let _ = world_handle.join();
}
