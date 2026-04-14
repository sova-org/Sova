use crate::theme::{COLOR_ERROR, COLOR_OK};
use eframe::egui;
use std::sync::{Arc, Mutex as StdMutex, atomic::Ordering, mpsc};
use tokio_util::sync::CancellationToken;

use sova_core::{clock::ClockServer, device_map::DeviceMap, schedule::SovaNotification};
use sova_server::audio::{AudioThread, spawn_audio_thread};
use sova_server::{AudioEngineState, ClientRegistry, SovaCoreServer};
use tokio::sync::Mutex;

use crate::panels::log_panel::{LogEntry, LogSource};
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
    cancel_token: CancellationToken,
    log_forwarder: tokio::task::JoinHandle<()>,
    devices: Arc<DeviceMap>,
    audio_thread: Option<AudioThread>,
}

pub struct ServerPanel {
    ip: String,
    port: String,
    password: String,
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
            ip: settings.ip,
            port: settings.port,
            password: String::new(),
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
        sova_core::logger::init_standalone();

        let (log_sender, _) = tokio::sync::broadcast::channel::<SovaNotification>(256);
        let client_registry = ClientRegistry::new();
        sova_core::logger::set_full_mode(log_sender.clone());

        let mut log_sub = log_sender.subscribe();
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

        let demo = sova_server::demos::random_demo();
        let clock_server = Arc::new(ClockServer::new(demo.tempo, demo.quantum));
        clock_server.link.enable(true);

        let devices = Arc::new(DeviceMap::new());
        if let Err(e) = devices.create_virtual_midi_port("Sova") {
            sova_core::log_eprintln!("Failed to create virtual MIDI port: {}", e);
        } else if let Err(e) = devices.assign_slot(1, "Sova") {
            sova_core::log_eprintln!("Failed to assign Sova to Slot 1: {}", e);
        }

        let languages = Arc::new(langs::create_language_center());

        let scene_image = Arc::new(Mutex::new(demo.scene));

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

        let password = if self.password.is_empty() {
            None
        } else {
            Some(self.password.clone())
        };

        let mut server = SovaCoreServer::new(
            self.ip.clone(),
            port,
            Arc::clone(&scene_image),
            clock_server.clone(),
            devices.clone(),
            log_sender,
            client_registry.clone(),
            languages.clone(),
            audio_engine_state,
            audio_restart_tx,
            audio_cmd_tx,
            password,
            master_gain,
        );

        let cancel_token = CancellationToken::new();

        let server_token = cancel_token.clone();
        let server_task = self
            .runtime
            .spawn(async move { server.start(server_token).await });

        self.embedded = Some(EmbeddedServer {
            server_task,
            log_forwarder,
            devices,
            cancel_token,
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
            if let Some(at) = embedded.audio_thread.take() {
                at.running.store(false, Ordering::Relaxed);
                let _ = at.thread_handle.join();
            }

            embedded.cancel_token.cancel();
            embedded.log_forwarder.abort();
            // Orchestrator thread will handle scheduler/world shutdown
            // when the core_restart_tx channel drops
            embedded.devices.panic_all_midi_outputs();
        }
    }

    pub fn show_actions(&mut self, ui: &mut egui::Ui) -> ServerAction {
        let mut action = ServerAction::None;
        let running = matches!(self.status, ServerStatus::Running);

        if running {
            let r = ui.button(crate::icons::button_text(
                ui,
                crate::icons::STOP,
                t!("common.stop"),
            ));
            crate::widgets::hint::on_hover(ui.ctx(), &r, t!("server.hint.stop"));
            if r.clicked() {
                action = ServerAction::Stop;
            }
        } else {
            let r = ui.button(crate::icons::button_text(
                ui,
                crate::icons::PLAY,
                t!("common.start"),
            ));
            crate::widgets::hint::on_hover(ui.ctx(), &r, t!("server.hint.start"));
            if r.clicked() {
                action = ServerAction::Start;
            }
        }

        let status_r = match &self.status {
            ServerStatus::Stopped => ui.label(t!("server.stopped")),
            ServerStatus::Running => ui.colored_label(COLOR_OK, t!("server.running")),
            ServerStatus::Error(e) => ui.colored_label(COLOR_ERROR, e.as_str()),
        };
        crate::widgets::hint::on_hover(ui.ctx(), &status_r, t!("server.hint.status"));

        action
    }

    pub fn show_config(&mut self, ui: &mut egui::Ui) {
        let running = matches!(self.status, ServerStatus::Running);

        ui.add_enabled_ui(!running, |ui| {
            egui::Grid::new("server_config")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    crate::widgets::hint::labeled(
                        ui, t!("server.ip"), t!("server.hint.ip"),
                        |ui| ui.text_edit_singleline(&mut self.ip),
                    );
                    ui.end_row();

                    crate::widgets::hint::labeled(
                        ui, t!("server.port"), t!("server.hint.port"),
                        |ui| ui.text_edit_singleline(&mut self.port),
                    );
                    ui.end_row();

                    crate::widgets::hint::labeled(
                        ui, t!("server.password"), t!("server.hint.password"),
                        |ui| ui.add(egui::TextEdit::singleline(&mut self.password).password(true)),
                    );
                    ui.end_row();
                });
        });
    }
}

impl Drop for ServerPanel {
    fn drop(&mut self) {
        self.teardown_embedded();
    }
}
