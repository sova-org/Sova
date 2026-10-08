use std::{io::ErrorKind, sync::Arc};

use sova_core::{clock::ClockServer, device_map::DeviceMap, schedule::SovaNotification};
use sova_server::{AudioEngineState, AudioRestartConfig, ClientRegistry, SovaCoreServer, audio::spawn_audio_thread};
use tokio::{io, sync::{Mutex, broadcast}, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use std::sync::Mutex as StdMutex;

pub struct ServerTask {
    handle: JoinHandle<io::Result<()>>,
    token: CancellationToken
}

impl ServerTask {
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished()
    }

    pub async fn stop(self) -> io::Result<()> {
        self.token.cancel();
        self.handle
            .await
            .unwrap_or(Err(io::Error::new(ErrorKind::Other, "Unable to stop server task!")))
    }
}

pub fn start_server(
    port: u16, 
    password: Option<String>,
    log_sender: broadcast::Sender<SovaNotification>
) -> ServerTask {
    sova_core::logger::init_network(log_sender.clone());

    let mut initial_audio_config = AudioRestartConfig::default();
    initial_audio_config.channels = 2;
    initial_audio_config.max_voices = 32;

    let client_registry = ClientRegistry::new();

    // let mut log_sub = log_sender.subscribe();
    // let log_tx = log_tx.clone();
    // let log_forwarder = tokio::spawn(async move {
    //     while let Ok(notif) = log_sub.recv().await {
    //         if let SovaNotification::Log(msg) = notif {
    //             let _ = log_tx.send(LogEntry {
    //                 source: LogSource::Server,
    //                 message: msg,
    //             });
    //         }
    //     }
    // });

    let demo = sova_server::demos::random_demo();
    let clock_server = Arc::new(ClockServer::new(demo.tempo, demo.quantum));
    clock_server.link.enable(true);

    let devices = Arc::new(DeviceMap::new());
    if let Err(e) = devices.create_virtual_midi_port("Sova") {
        sova_core::log_eprintln!("Failed to create virtual MIDI port: {}", e);
    } else if let Err(e) = devices.assign_slot(1, "Sova") {
        sova_core::log_eprintln!("Failed to assign Sova to Slot 1: {}", e);
    }
    //devices.register_host_proxy(host_tx);

    let languages = Arc::new(langs::create_language_center());

    let scene_image = Arc::new(Mutex::new(demo.scene));

    let audio_engine_state = Arc::new(StdMutex::new(AudioEngineState::default()));
    // let audio_thread = spawn_audio_thread(
    //     initial_audio_config,
    //     Arc::clone(&audio_engine_state),
    //     Arc::clone(&devices),
    //     Arc::clone(&clock_server),
    //     client_registry.clone(),
    // );
    // let audio_restart_tx = Some(audio_thread.restart_tx.clone());
    // let audio_cmd_tx = Some(audio_thread.cmd_tx.clone());
    // let master_gain = Arc::clone(&audio_thread.master_gain);

    let token = CancellationToken::new();
    let task_token = token.clone();

    let task = tokio::spawn(async move {
        let frame_text = sova_server::FrameTextStore::new();
        {
            let initial = scene_image.lock().await;
            frame_text.rebuild_from_scene(&initial);
        }

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
            None, 
            None, 
            password, 
            Default::default(), 
            frame_text, 
        );
    
        server.start(task_token).await
    });

    ServerTask {
        handle: task,
        token,
    }
}