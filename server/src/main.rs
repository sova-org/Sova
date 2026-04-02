use sova_core::clock::ClockServer;
use sova_core::device_map::DeviceMap;
use sova_core::schedule::SovaNotification;

use clap::Parser;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use thread_priority::{ThreadPriority, set_current_thread_priority};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use sova_server::{AudioEngineState, AudioRestartConfig, ClientRegistry, SovaCoreServer};

#[cfg(feature = "audio")]
use sova_server::audio::spawn_audio_thread;

#[cfg(not(feature = "audio"))]
use sova_server::AudioRestartRequest;

#[cfg(feature = "audio")]
use std::path::PathBuf;

pub const DEFAULT_MIDI_OUTPUT: &str = "Sova";
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
    It manages connections from clients, handles MIDI devices,
    \nsynchronizes state, and processes scenes."
)]
struct Cli {
    #[arg(short, long, value_name = "IP_ADDRESS", default_value = "0.0.0.0")]
    ip: String,

    #[arg(short, long, value_name = "PORT", default_value_t = 8080)]
    port: u16,

    /// Require a password to connect (open access if not set)
    #[arg(long, value_name = "PASSWORD")]
    password: Option<String>,

    #[cfg(feature = "audio")]
    /// Disable audio engine (no Doux)
    #[arg(long, default_value_t = false)]
    no_audio: bool,

    #[cfg(feature = "audio")]
    /// Audio host/driver (e.g. coreaudio, jack, alsa, wasapi)
    #[arg(long, value_name = "HOST")]
    audio_host: Option<String>,

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

    #[cfg(feature = "audio")]
    /// Maximum polyphony (number of simultaneous voices)
    #[arg(long, value_name = "VOICES", default_value_t = 32)]
    max_voices: usize,
}

#[tokio::main]
async fn main() {
    match set_current_thread_priority(ThreadPriority::Max) {
        Ok(_) => eprintln!("Real-time priority set successfully"),
        Err(e) => eprintln!("Failed to set real-time priority: {:?}", e),
    }

    let cli = Cli::parse();

    sova_core::logger::init_standalone();

    let (log_sender, _) = tokio::sync::broadcast::channel::<SovaNotification>(256);
    let client_registry = ClientRegistry::new();
    sova_core::logger::set_full_mode(log_sender.clone());

    println!("Logger initialized in full mode.");

    greeter();

    let demo = sova_server::demos::random_demo();
    let clock_server = Arc::new(ClockServer::new(demo.tempo, demo.quantum));
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
    let (audio_restart_tx, audio_cmd_tx, audio_thread) = if !cli.no_audio {
        let initial_config = AudioRestartConfig {
            host: cli.audio_host.clone(),
            device: cli.audio_device.clone(),
            input_device: cli.audio_input_device.clone(),
            channels: cli.audio_channels,
            buffer_size: cli.audio_buffer_size,
            sample_paths: {
                #[cfg(feature = "default-samples")]
                let mut paths = vec![sova_server::audio::default_samples::ensure_default_samples()];
                #[cfg(not(feature = "default-samples"))]
                let mut paths = Vec::new();
                paths.extend(cli.sample_paths.clone());
                paths
            },
            max_voices: cli.max_voices,
        };

        let at = spawn_audio_thread(
            initial_config,
            Arc::clone(&audio_engine_state),
            Arc::clone(&devices),
            Arc::clone(&clock_server),
            client_registry.clone(),
        );

        let restart = at.restart_tx.clone();
        let cmd = at.cmd_tx.clone();
        (Some(restart), Some(cmd), Some(at))
    } else {
        println!("Audio engine disabled (--no-audio flag).");
        (None, None, None)
    };

    #[cfg(not(feature = "audio"))]
    let audio_restart_tx: Option<crossbeam_channel::Sender<AudioRestartRequest>> = None;

    #[cfg(not(feature = "audio"))]
    let audio_cmd_tx: Option<crossbeam_channel::Sender<sova_server::audio::AudioCommand>> = None;

    #[cfg(not(feature = "audio"))]
    println!("Audio engine not compiled (build without 'audio' feature).");

    let languages = Arc::new(langs::create_language_center());

    let scene_image = Arc::new(Mutex::new(demo.scene));

    #[cfg(feature = "audio")]
    let master_gain = audio_thread
        .as_ref()
        .map(|at| Arc::clone(&at.master_gain))
        .unwrap_or_else(|| Arc::new(AtomicU32::new(1.0f32.to_bits())));

    #[cfg(not(feature = "audio"))]
    let master_gain = Arc::new(AtomicU32::new(1.0f32.to_bits()));

    let mut server = SovaCoreServer::new(
        cli.ip,
        cli.port,
        Arc::clone(&scene_image),
        clock_server.clone(),
        devices.clone(),
        log_sender,
        client_registry.clone(),
        languages.clone(),
        audio_engine_state,
        audio_restart_tx,
        audio_cmd_tx,
        cli.password,
        master_gain,
    );

    println!("Starting Sova server on {}:{}...", server.ip, server.port);

    let token = CancellationToken::new();
    match server.start(token).await {
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
    if let Some(at) = audio_thread {
        at.running.store(false, Ordering::Relaxed);
        let _ = at.thread_handle.join();
    }

    devices.panic_all_midi_outputs();
}
