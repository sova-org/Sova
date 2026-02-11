use crate::widgets::{COLOR_ERROR, COLOR_OK};
use eframe::egui;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex as StdMutex, mpsc};

use crossbeam_channel::Sender;
use langs::{
    bali::BaliCompiler, bob::BobCompiler, boinx::BoinxInterpreterFactory,
    forth::ForthInterpreterFactory,
};
use sova_core::{
    clock::ClockServer,
    device_map::DeviceMap,
    scene::{Line, Scene},
    schedule::{ActionTiming, SchedulerMessage, SovaNotification},
    vm::{LanguageCenter, Transcoder, interpreter::InterpreterDirectory},
};
use sova_server::audio::{AudioThread, spawn_audio_thread};
use sova_server::{AudioEngineState, AudioRestartConfig, AudioRestartRequest, ServerState, SovaCoreServer};
use tokio::sync::Mutex;

use crate::log_panel::{LogEntry, LogSource};

pub struct ServerInfo {
    pub running: bool,
    pub address: String,
}

enum ServerStatus {
    Stopped,
    Running,
    Error(String),
}

struct EmbeddedServer {
    server_task: tokio::task::JoinHandle<std::io::Result<()>>,
    log_forwarder: tokio::task::JoinHandle<()>,
    sched_iface: Sender<SchedulerMessage>,
    world_handle: std::thread::JoinHandle<()>,
    sched_handle: std::thread::JoinHandle<()>,
    devices: Arc<DeviceMap>,
    audio_thread: Option<AudioThread>,
}

pub struct ServerPanel {
    pub open: bool,
    ip: String,
    port: String,
    tempo: String,
    quantum: String,
    status: ServerStatus,
    runtime: tokio::runtime::Handle,
    embedded: Option<EmbeddedServer>,
    log_tx: mpsc::Sender<LogEntry>,
    ctx: egui::Context,
    pub audio_engine_state: Arc<StdMutex<AudioEngineState>>,
    pub audio_restart_tx: Option<Sender<AudioRestartRequest>>,
}

impl ServerPanel {
    pub fn new(
        runtime: tokio::runtime::Handle,
        log_tx: mpsc::Sender<LogEntry>,
        ctx: egui::Context,
    ) -> Self {
        Self {
            open: true,
            ip: "127.0.0.1".into(),
            port: "8080".into(),
            tempo: "120".into(),
            quantum: "4".into(),
            status: ServerStatus::Stopped,
            runtime,
            embedded: None,
            log_tx,
            ctx,
            audio_engine_state: Arc::new(StdMutex::new(AudioEngineState::default())),
            audio_restart_tx: None,
        }
    }

    pub fn info(&self) -> ServerInfo {
        ServerInfo {
            running: matches!(self.status, ServerStatus::Running),
            address: format!("{}:{}", self.ip, self.port),
        }
    }

    pub fn poll(&mut self) {
        let crashed = self
            .embedded
            .as_ref()
            .is_some_and(|e| e.server_task.is_finished());
        if crashed {
            self.teardown_embedded();
            self.status = ServerStatus::Error("Server stopped unexpectedly".into());
        }
    }

    pub fn start(&mut self, audio_config: AudioRestartConfig) {
        let port: u16 = match self.port.parse() {
            Ok(p) => p,
            Err(_) => {
                self.status = ServerStatus::Error("Invalid port".into());
                return;
            }
        };
        let tempo: f64 = match self.tempo.parse() {
            Ok(t) => t,
            Err(_) => {
                self.status = ServerStatus::Error("Invalid tempo".into());
                return;
            }
        };
        let quantum: f64 = match self.quantum.parse() {
            Ok(q) => q,
            Err(_) => {
                self.status = ServerStatus::Error("Invalid quantum".into());
                return;
            }
        };

        sova_core::logger::init_standalone();

        let (update_sender, _) = tokio::sync::broadcast::channel::<SovaNotification>(256);
        sova_core::logger::set_full_mode(update_sender.clone());

        let mut log_sub = update_sender.subscribe();
        let log_tx = self.log_tx.clone();
        let ctx = self.ctx.clone();
        let log_forwarder = self.runtime.spawn(async move {
            while let Ok(notif) = log_sub.recv().await {
                if let SovaNotification::Log(msg) = notif {
                    let _ = log_tx.send(LogEntry {
                        source: LogSource::Server,
                        message: msg,
                    });
                    ctx.request_repaint();
                }
            }
        });

        let clock_server = Arc::new(ClockServer::new(tempo, quantum));
        clock_server.link.enable(true);

        let devices = Arc::new(DeviceMap::new());
        if let Err(e) = devices.create_virtual_midi_port("Sova") {
            eprintln!("Failed to create virtual MIDI port: {}", e);
        } else if let Err(e) = devices.assign_slot(1, "Sova") {
            eprintln!("Failed to assign Sova to Slot 1: {}", e);
        }

        // Spawn audio thread
        let audio_engine_state = Arc::new(StdMutex::new(AudioEngineState::default()));
        let at = spawn_audio_thread(
            audio_config,
            Arc::clone(&audio_engine_state),
            Arc::clone(&devices),
            Arc::clone(&clock_server),
            update_sender.clone(),
        );
        let restart_tx = at.restart_tx.clone();
        self.audio_engine_state = audio_engine_state.clone();
        self.audio_restart_tx = Some(restart_tx.clone());

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
            self.status = ServerStatus::Error(format!("Failed to send initial scene: {}", e));
            return;
        }

        let server_state = ServerState::new(
            scene_image,
            clock_server,
            devices.clone(),
            sched_iface.clone(),
            update_sender,
            languages,
            audio_engine_state,
            Some(restart_tx),
        );

        let ip = self.ip.clone();
        let server = SovaCoreServer::new(ip, port, server_state);
        let server_task = self.runtime.spawn(async move {
            server.start(sched_update).await
        });

        self.embedded = Some(EmbeddedServer {
            server_task,
            log_forwarder,
            sched_iface,
            world_handle,
            sched_handle,
            devices,
            audio_thread: Some(at),
        });
        self.status = ServerStatus::Running;
    }

    pub fn stop(&mut self) {
        self.teardown_embedded();
        self.status = ServerStatus::Stopped;
    }

    fn teardown_embedded(&mut self) {
        if let Some(embedded) = self.embedded.take() {
            // Stop audio thread first
            if let Some(at) = embedded.audio_thread {
                at.running.store(false, Ordering::Relaxed);
                let _ = at.thread_handle.join();
            }

            embedded.server_task.abort();
            embedded.log_forwarder.abort();
            let _ = embedded.sched_iface.send(SchedulerMessage::Shutdown);
            if let Err(e) = embedded.sched_handle.join() {
                eprintln!("scheduler thread panicked: {:?}", e);
            }
            if let Err(e) = embedded.world_handle.join() {
                eprintln!("world thread panicked: {:?}", e);
            }
            embedded.devices.panic_all_midi_outputs();
        }
        self.audio_restart_tx = None;
    }

    pub fn show(&mut self, ctx: &egui::Context, audio_config: AudioRestartConfig) {
        let mut open = self.open;
        egui::Window::new("Server")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                let running = matches!(self.status, ServerStatus::Running);

                ui.add_enabled_ui(!running, |ui| {
                    egui::Grid::new("server_config")
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("IP");
                            ui.text_edit_singleline(&mut self.ip);
                            ui.end_row();

                            ui.label("Port");
                            ui.text_edit_singleline(&mut self.port);
                            ui.end_row();

                            ui.label("Tempo");
                            ui.text_edit_singleline(&mut self.tempo);
                            ui.end_row();

                            ui.label("Quantum");
                            ui.text_edit_singleline(&mut self.quantum);
                            ui.end_row();
                        });
                });

                ui.add_space(8.0);

                if running {
                    if ui.button("Stop").clicked() {
                        self.stop();
                    }
                } else if ui.button("Start").clicked() {
                    self.start(audio_config);
                }

                ui.add_space(4.0);
                match &self.status {
                    ServerStatus::Stopped => {
                        ui.label("Stopped");
                    }
                    ServerStatus::Running => {
                        ui.colored_label(COLOR_OK, "Running");
                    }
                    ServerStatus::Error(e) => {
                        ui.colored_label(COLOR_ERROR, e.as_str());
                    }
                }
            });
        self.open = open;
    }
}

impl Drop for ServerPanel {
    fn drop(&mut self) {
        self.teardown_embedded();
    }
}
