use std::sync::Arc;
use std::thread;
use crossbeam_channel::Sender;
use sova_core::{
    clock::ClockServer,
    device_map::DeviceMap,
    lang::{LanguageCenter, Transcoder, interpreter::InterpreterDirectory},
    compiler::{bali::BaliCompiler, dummylang::DummyCompiler, ExternalCompiler},
    lang::interpreter::{boinx::BoinxInterpreterFactory, external::ExternalInterpreterFactory},
    init,
    scene::{Scene, Line},
    schedule::{SchedulerMessage, ActionTiming},
    server::{SovaCoreServer, ServerState},
};
use tokio::sync::{Mutex, watch};
use tauri::{AppHandle, Emitter};

pub struct ServerManager {
    world_handle: Option<thread::JoinHandle<()>>,
    sched_handle: Option<thread::JoinHandle<()>>,
    server_task: Option<tokio::task::JoinHandle<()>>,
    sched_iface: Option<Sender<SchedulerMessage>>,
    clock_server: Option<Arc<ClockServer>>,
    devices: Option<Arc<DeviceMap>>,
    app_handle: AppHandle,
}

impl ServerManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            world_handle: None,
            sched_handle: None,
            server_task: None,
            sched_iface: None,
            clock_server: None,
            devices: None,
            app_handle,
        }
    }

    pub async fn start_server(&mut self, port: u16) -> Result<(), String> {
        if self.is_running() {
            return Err("Server already running".to_string());
        }

        let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
        clock_server.link.enable(true);

        let devices = Arc::new(DeviceMap::new());
        let _ = devices.create_virtual_midi_port("Sova");

        let mut transcoder = Transcoder::default();
        transcoder.add_compiler(BaliCompiler);
        transcoder.add_compiler(DummyCompiler);
        transcoder.add_compiler(ExternalCompiler);

        let mut interpreters = InterpreterDirectory::new();
        interpreters.add_factory(BoinxInterpreterFactory);
        interpreters.add_factory(ExternalInterpreterFactory);

        let languages = Arc::new(LanguageCenter { transcoder, interpreters });

        let (world_handle, sched_handle, sched_iface, sched_update) =
            init::start_scheduler_and_world(
                clock_server.clone(),
                devices.clone(),
                languages.clone(),
            );

        let initial_scene = Scene::new(vec![Line::default()]);
        let scene_image = Arc::new(Mutex::new(initial_scene.clone()));
        let _ = sched_iface.send(SchedulerMessage::SetScene(
            initial_scene,
            ActionTiming::Immediate,
        ));

        let (update_sender, update_receiver) = watch::channel(
            sova_core::schedule::SovaNotification::default()
        );

        // Initialize Sova logger in Full mode (logs to file + terminal + sends notifications)
        sova_core::logger::set_full_mode(update_sender.clone());

        // Spawn task to forward logs to GUI
        let app_handle_clone = self.app_handle.clone();
        let mut log_receiver = update_receiver.clone();
        tokio::spawn(async move {
            loop {
                if log_receiver.changed().await.is_ok() {
                    let notification = log_receiver.borrow().clone();
                    if let sova_core::schedule::SovaNotification::Log(log_msg) = notification {
                        let _ = app_handle_clone.emit("server:log", log_msg.to_string());
                    }
                }
            }
        });

        let server_state = ServerState::new(
            scene_image,
            clock_server.clone(),
            devices.clone(),
            sched_iface.clone(),
            update_sender,
            update_receiver,
            languages,
        );

        let server = SovaCoreServer {
            ip: "127.0.0.1".to_string(),
            port,
            state: server_state,
        };

        let server_task = tokio::spawn(async move {
            if let Err(e) = server.start(sched_update).await {
                sova_core::log_error!("Server error: {}", e);
            }
        });

        self.world_handle = Some(world_handle);
        self.sched_handle = Some(sched_handle);
        self.server_task = Some(server_task);
        self.sched_iface = Some(sched_iface);
        self.clock_server = Some(clock_server);
        self.devices = Some(devices);

        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<(), String> {
        if !self.is_running() {
            return Err("Server not running".to_string());
        }

        if let Some(devices) = &self.devices {
            devices.panic_all_midi_outputs();
        }

        if let Some(sched_iface) = &self.sched_iface {
            let _ = sched_iface.send(SchedulerMessage::Shutdown);
        }

        if let Some(task) = self.server_task.take() {
            task.abort();
        }

        if let Some(handle) = self.world_handle.take() {
            let _ = handle.join();
        }

        if let Some(handle) = self.sched_handle.take() {
            let _ = handle.join();
        }

        self.sched_iface = None;
        self.clock_server = None;
        self.devices = None;

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.sched_iface.is_some()
    }
}
