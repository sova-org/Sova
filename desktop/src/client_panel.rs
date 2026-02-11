use crate::widgets::{COLOR_ERROR, COLOR_OK};
use eframe::egui;
use sova_server::{ClientMessage, ServerMessage, SovaClient};
use std::sync::mpsc;
use tokio::sync::mpsc as tokio_mpsc;

use crate::log_panel::{LogEntry, LogSource};
use crate::settings::ClientSettings;

pub struct ClientInfo {
    pub connected: bool,
    pub username: Option<String>,
    pub address: String,
    pub peer_count: usize,
}

enum ClientCommand {
    Disconnect,
}

enum ClientEvent {
    Connected { username: String, peers: Vec<String> },
    PeersUpdated(Vec<String>),
    Disconnected { error: Option<String> },
}

enum ClientStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

pub struct ClientPanel {
    pub open: bool,
    ip: String,
    port: String,
    username: String,
    status: ClientStatus,
    runtime: tokio::runtime::Handle,
    cmd_tx: Option<tokio_mpsc::UnboundedSender<ClientCommand>>,
    event_rx: Option<mpsc::Receiver<ClientEvent>>,
    peers: Vec<String>,
    confirmed_username: Option<String>,
    ctx: egui::Context,
    log_tx: mpsc::Sender<LogEntry>,
}

impl ClientPanel {
    pub fn new(
        ctx: egui::Context,
        runtime: tokio::runtime::Handle,
        log_tx: mpsc::Sender<LogEntry>,
        settings: ClientSettings,
    ) -> Self {
        Self {
            open: true,
            ip: settings.ip,
            port: settings.port,
            username: settings.username,
            status: ClientStatus::Disconnected,
            runtime,
            cmd_tx: None,
            event_rx: None,
            peers: Vec::new(),
            confirmed_username: None,
            ctx,
            log_tx,
        }
    }

    pub fn settings(&self) -> ClientSettings {
        ClientSettings {
            ip: self.ip.clone(),
            port: self.port.clone(),
            username: self.username.clone(),
        }
    }

    pub fn info(&self) -> ClientInfo {
        ClientInfo {
            connected: matches!(self.status, ClientStatus::Connected),
            username: self.confirmed_username.clone(),
            address: format!("{}:{}", self.ip, self.port),
            peer_count: self.peers.len(),
        }
    }

    pub fn poll(&mut self) {
        let mut cleanup = false;
        if let Some(rx) = &self.event_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    ClientEvent::Connected { username, peers } => {
                        self.confirmed_username = Some(username);
                        self.peers = peers;
                        self.status = ClientStatus::Connected;
                    }
                    ClientEvent::PeersUpdated(peers) => {
                        self.peers = peers;
                    }
                    ClientEvent::Disconnected { error } => {
                        self.status = match error {
                            Some(e) => ClientStatus::Error(e),
                            None => ClientStatus::Disconnected,
                        };
                        self.confirmed_username = None;
                        self.peers.clear();
                        cleanup = true;
                    }
                }
            }
        }
        if cleanup {
            self.cmd_tx = None;
            self.event_rx = None;
        }
    }

    fn connect(&mut self) {
        let port: u16 = match self.port.parse() {
            Ok(p) => p,
            Err(_) => {
                self.status = ClientStatus::Error("Invalid port".into());
                return;
            }
        };

        let ip = self.ip.clone();
        let username = self.username.clone();
        let (cmd_tx, mut cmd_rx) = tokio_mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::channel();
        let ctx = self.ctx.clone();
        let log_tx = self.log_tx.clone();

        self.cmd_tx = Some(cmd_tx);
        self.event_rx = Some(event_rx);
        self.status = ClientStatus::Connecting;

        self.runtime.spawn(async move {
            let mut client = SovaClient::new(ip, port);

            if let Err(e) = client.connect().await {
                let _ = event_tx.send(ClientEvent::Disconnected {
                    error: Some(e.to_string()),
                });
                ctx.request_repaint();
                return;
            }

            if let Err(e) = client.send(ClientMessage::SetName(username)).await {
                let _ = event_tx.send(ClientEvent::Disconnected {
                    error: Some(e.to_string()),
                });
                ctx.request_repaint();
                return;
            }

            match client.read().await {
                Ok(ServerMessage::Hello {
                    username, peers, ..
                }) => {
                    let _ = event_tx.send(ClientEvent::Connected { username, peers });
                    ctx.request_repaint();
                }
                Ok(ServerMessage::ConnectionRefused(reason)) => {
                    let _ = event_tx.send(ClientEvent::Disconnected {
                        error: Some(reason),
                    });
                    ctx.request_repaint();
                    let _ = client.disconnect().await;
                    return;
                }
                Ok(_) => {
                    let _ = event_tx.send(ClientEvent::Disconnected {
                        error: Some("Unexpected server response".into()),
                    });
                    ctx.request_repaint();
                    let _ = client.disconnect().await;
                    return;
                }
                Err(e) => {
                    let _ = event_tx.send(ClientEvent::Disconnected {
                        error: Some(e.to_string()),
                    });
                    ctx.request_repaint();
                    return;
                }
            }

            loop {
                tokio::select! {
                    msg = client.read() => {
                        match msg {
                            Ok(ServerMessage::PeersUpdated(peers)) => {
                                let _ = event_tx.send(ClientEvent::PeersUpdated(peers));
                                ctx.request_repaint();
                            }
                            Ok(ServerMessage::Log(msg)) => {
                                let _ = log_tx.send(LogEntry {
                                    source: LogSource::Client,
                                    message: msg,
                                });
                                ctx.request_repaint();
                            }
                            Err(e) => {
                                let _ = event_tx.send(ClientEvent::Disconnected {
                                    error: Some(e.to_string()),
                                });
                                ctx.request_repaint();
                                break;
                            }
                            _ => {}
                        }
                    }
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(ClientCommand::Disconnect) | None => {
                                let _ = client.disconnect().await;
                                let _ = event_tx.send(ClientEvent::Disconnected { error: None });
                                ctx.request_repaint();
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    fn disconnect(&mut self) {
        if let Some(tx) = self.cmd_tx.take() {
            let _ = tx.send(ClientCommand::Disconnect);
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let mut open = self.open;
        egui::Window::new("Client")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                let active =
                    matches!(self.status, ClientStatus::Connected | ClientStatus::Connecting);

                ui.add_enabled_ui(!active, |ui| {
                    egui::Grid::new("client_config")
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("IP");
                            ui.text_edit_singleline(&mut self.ip);
                            ui.end_row();

                            ui.label("Port");
                            ui.text_edit_singleline(&mut self.port);
                            ui.end_row();

                            ui.label("Username");
                            ui.text_edit_singleline(&mut self.username);
                            ui.end_row();
                        });
                });

                ui.add_space(8.0);

                match self.status {
                    ClientStatus::Disconnected | ClientStatus::Error(_) => {
                        if ui
                            .add_enabled(
                                !self.username.is_empty(),
                                egui::Button::new("Connect"),
                            )
                            .clicked()
                        {
                            self.connect();
                        }
                    }
                    ClientStatus::Connecting => {
                        ui.add_enabled(false, egui::Button::new("Connecting..."));
                    }
                    ClientStatus::Connected => {
                        if ui.button("Disconnect").clicked() {
                            self.disconnect();
                        }
                    }
                }

                ui.add_space(4.0);
                match &self.status {
                    ClientStatus::Disconnected => {
                        ui.label("Disconnected");
                    }
                    ClientStatus::Connecting => {
                        ui.label("Connecting...");
                    }
                    ClientStatus::Connected => {
                        ui.colored_label(
                            COLOR_OK,
                            format!(
                                "Connected as {}",
                                self.confirmed_username.as_deref().unwrap_or("?")
                            ),
                        );
                    }
                    ClientStatus::Error(e) => {
                        ui.colored_label(COLOR_ERROR, e.as_str());
                    }
                }

                if !self.peers.is_empty() {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label("Peers:");
                    for peer in &self.peers {
                        ui.label(format!("  {}", peer));
                    }
                }
            });
        self.open = open;
    }
}
