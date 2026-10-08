use std::sync::Arc;

use sova_core::{clock::ClockServer, device_map::DeviceMap, schedule::SovaNotification};
use sova_server::{AudioEngineState, AudioRestartConfig, ClientRegistry, SovaCoreServer, audio::spawn_audio_thread};
use tokio::{io, sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use std::sync::Mutex as StdMutex;

pub struct ServerTask {
    handle: JoinHandle<io::Result<()>>,
    token: CancellationToken
}

pub fn start_server(
    port: u16, 
    password: Option<String>
) -> SovaCoreServer {
    sova_core::logger::init_standalone();

    let mut initial_audio_config = AudioRestartConfig::default();
    initial_audio_config.channels = 2;
    initial_audio_config.max_voices = 32;

    let (log_sender, _) = tokio::sync::broadcast::channel::<SovaNotification>(256);
    let client_registry = ClientRegistry::new();
    sova_core::logger::set_full_mode(log_sender.clone());

    let mut log_sub = log_sender.subscribe();
    let log_tx = self.log_tx.clone();
    let log_forwarder = tokio::spawn(async move {
        while let Ok(notif) = log_sub.recv().await {
            if let SovaNotification::Log(msg) = notif {
                let _ = log_tx.send(LogEntry {
                    source: LogSource::Server,
                    message: msg,
                });
            }
        }
    });

    let demo = sova_server::demos::random_demo();
    let clock_server = Arc::new(ClockServer::new(demo.tempo, demo.quantum));
    clock_server.link.enable(true);

    let devices = Arc::new(DeviceMap::new());
    if let Err(e) = devices.create_virtual_midi_port("Sova") {
        sova_core::log_eprintln!("Failed to create virtual MIDI port: {}", e);
    } else if let Err(e) = devices.assign_slot(1, "Sova") {
        sova_core::log_eprintln!("Failed to assign Sova to Slot 1: {}", e);
    }
    devices.register_host_proxy(host_tx);

    let languages = Arc::new(langs::create_language_center());

    let scene_image = Arc::new(Mutex::new(demo.scene));

    let frame_text = sova_server::FrameTextStore::new();
    {
        let initial = scene_image.blocking_lock();
        frame_text.rebuild_from_scene(&initial);
    }

    let audio_engine_state = Arc::new(StdMutex::new(AudioEngineState::default()));
    let audio_thread = spawn_audio_thread(
        initial_audio_config,
        Arc::clone(&audio_engine_state),
        Arc::clone(&devices),
        Arc::clone(&clock_server),
        client_registry.clone(),
    );
    let audio_restart_tx = Some(audio_thread.restart_tx.clone());
    let audio_cmd_tx = Some(audio_thread.cmd_tx.clone());
    let master_gain = Arc::clone(&audio_thread.master_gain);

    let mut server = SovaCoreServer::new(
        "0.0.0.0".to_owned(), 
        port, 
        scene_image,
        clock_server, 
        devices, 
        log_sender, 
        client_registry, 
        languages, 
        audio_engine_state, 
        audio_restart_tx, 
        audio_cmd_tx, 
        password, 
        master_gain, 
        frame_text, 
    );

    server
}