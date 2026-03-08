use crate::widgets::{COLOR_ERROR, COLOR_OK};
use eframe::egui;
use std::sync::{Arc, Mutex as StdMutex, atomic::Ordering, mpsc};

use crossbeam_channel::Sender;
use sova_core::{
    clock::ClockServer,
    device_map::DeviceMap,
    scene::{Line, Scene},
    schedule::{ActionTiming, SchedulerMessage, SovaNotification},
};
use sova_server::audio::{AudioThread, spawn_audio_thread};
use sova_server::{AudioEngineState, ClientRegistry, ServerState, SovaCoreServer};
use tokio::sync::Mutex;

use crate::log_panel::{LogEntry, LogSource};
use crate::settings::ServerSettings;

pub enum ServerAction {
    None,
    Start,
    Stop,
}

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
}

impl ServerPanel {
    pub fn new(
        runtime: tokio::runtime::Handle,
        log_tx: mpsc::Sender<LogEntry>,
        ctx: egui::Context,
        settings: ServerSettings,
    ) -> Self {
        Self {
            open: false,
            ip: settings.ip,
            port: settings.port,
            tempo: settings.tempo,
            quantum: settings.quantum,
            status: ServerStatus::Stopped,
            runtime,
            embedded: None,
            log_tx,
            ctx,
        }
    }

    pub fn settings(&self) -> ServerSettings {
        ServerSettings {
            ip: self.ip.clone(),
            port: self.port.clone(),
            tempo: self.tempo.clone(),
            quantum: self.quantum.clone(),
        }
    }

    pub fn info(&self) -> ServerInfo {
        ServerInfo {
            running: matches!(self.status, ServerStatus::Running),
            address: format!("{}:{}", self.ip, self.port),
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, ServerStatus::Running)
    }

    pub fn poll(&mut self) {
        let crashed = self
            .embedded
            .as_ref()
            .is_some_and(|e| e.server_task.is_finished());
        if crashed {
            self.teardown_embedded();
            self.status = ServerStatus::Error(t!("server.error.unexpected_stop").into());
        }
    }

    pub fn start(&mut self, initial_audio_config: sova_server::AudioRestartConfig) {
        let port: u16 = match self.port.parse() {
            Ok(p) => p,
            Err(_) => {
                self.status = ServerStatus::Error(t!("server.error.invalid_port").into());
                return;
            }
        };
        let tempo: f64 = match self.tempo.parse() {
            Ok(t) => t,
            Err(_) => {
                self.status = ServerStatus::Error(t!("server.error.invalid_tempo").into());
                return;
            }
        };
        let quantum: f64 = match self.quantum.parse() {
            Ok(q) => q,
            Err(_) => {
                self.status = ServerStatus::Error(t!("server.error.invalid_quantum").into());
                return;
            }
        };

        sova_core::logger::init_standalone();

        let (update_sender, _) = tokio::sync::broadcast::channel::<SovaNotification>(256);
        let client_registry = ClientRegistry::new();
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

        let languages = Arc::new(langs::create_language_center());

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

        let audio_engine_state = Arc::new(StdMutex::new(AudioEngineState::default()));
        let audio_thread = spawn_audio_thread(
            initial_audio_config,
            Arc::clone(&audio_engine_state),
            Arc::clone(&devices),
            Arc::clone(&clock_server),
            client_registry.clone(),
        );
        let audio_restart_tx = Some(audio_thread.restart_tx.clone());

        let server_state = ServerState::new(
            Arc::clone(&scene_image),
            clock_server,
            devices.clone(),
            sched_iface.clone(),
            update_sender,
            client_registry,
            languages,
            audio_engine_state,
            audio_restart_tx,
        );

        let ip = self.ip.clone();
        let server = SovaCoreServer::new(ip, port, server_state);
        let server_task = self
            .runtime
            .spawn(async move { server.start(sched_update).await });

        self.embedded = Some(EmbeddedServer {
            server_task,
            log_forwarder,
            sched_iface,
            world_handle,
            sched_handle,
            devices,
            audio_thread: Some(audio_thread),
        });
        self.status = ServerStatus::Running;
    }

    pub fn stop(&mut self) {
        self.teardown_embedded();
        self.status = ServerStatus::Stopped;
    }

    fn teardown_embedded(&mut self) {
        if let Some(mut embedded) = self.embedded.take() {
            // Stop audio thread first
            if let Some(at) = embedded.audio_thread.take() {
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
    }

    pub fn show(&mut self, ctx: &egui::Context) -> ServerAction {
        let mut action = ServerAction::None;
        let mut open = self.open;
        egui::Window::new(t!("server.title"))
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
                            let label = ui.label(t!("server.ip"));
                            let field = ui.text_edit_singleline(&mut self.ip);
                            if label.hovered() || field.hovered() {
                                crate::widgets::hint::set(ctx, t!("server.hint.ip"));
                            }
                            ui.end_row();

                            let label = ui.label(t!("server.port"));
                            let field = ui.text_edit_singleline(&mut self.port);
                            if label.hovered() || field.hovered() {
                                crate::widgets::hint::set(ctx, t!("server.hint.port"));
                            }
                            ui.end_row();

                            let label = ui.label(t!("server.tempo"));
                            let field = ui.text_edit_singleline(&mut self.tempo);
                            if label.hovered() || field.hovered() {
                                crate::widgets::hint::set(ctx, t!("server.hint.tempo"));
                            }
                            ui.end_row();

                            let label = ui.label(t!("server.quantum"));
                            let field = ui.text_edit_singleline(&mut self.quantum);
                            if label.hovered() || field.hovered() {
                                crate::widgets::hint::set(ctx, t!("server.hint.quantum"));
                            }
                            ui.end_row();
                        });
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if running {
                        let r = ui.button(t!("common.stop"));
                        if r.hovered() {
                            crate::widgets::hint::set(ctx, t!("server.hint.stop"));
                        }
                        if r.clicked() {
                            action = ServerAction::Stop;
                        }
                    } else {
                        let r = ui.button(t!("common.start"));
                        if r.hovered() {
                            crate::widgets::hint::set(ctx, t!("server.hint.start"));
                        }
                        if r.clicked() {
                            action = ServerAction::Start;
                        }
                    }

                    let status_r = match &self.status {
                        ServerStatus::Stopped => ui.label(t!("server.stopped")),
                        ServerStatus::Running => ui.colored_label(COLOR_OK, t!("server.running")),
                        ServerStatus::Error(e) => ui.colored_label(COLOR_ERROR, e.as_str()),
                    };
                    if status_r.hovered() {
                        crate::widgets::hint::set(ctx, t!("server.hint.status"));
                    }
                });
            });
        self.open = open;
        action
    }
}

impl Drop for ServerPanel {
    fn drop(&mut self) {
        self.teardown_embedded();
    }
}
