use crate::client_bridge::{ClientBridge, ConnectionStatus};
use crate::widgets::{COLOR_ERROR, COLOR_OK};
use eframe::egui;

use crate::settings::ClientSettings;

pub struct ClientInfo {
    pub connected: bool,
    pub username: Option<String>,
    pub address: String,
    pub peer_count: usize,
}

pub struct ClientPanel {
    pub open: bool,
    ip: String,
    port: String,
    username: String,
}

impl ClientPanel {
    pub fn new(settings: ClientSettings) -> Self {
        Self {
            open: true,
            ip: settings.ip,
            port: settings.port,
            username: settings.username,
        }
    }

    pub fn settings(&self) -> ClientSettings {
        ClientSettings {
            ip: self.ip.clone(),
            port: self.port.clone(),
            username: self.username.clone(),
        }
    }

    pub fn info(&self, bridge: &ClientBridge) -> ClientInfo {
        ClientInfo {
            connected: bridge.is_connected(),
            username: bridge.confirmed_username().map(str::to_owned),
            address: format!("{}:{}", self.ip, self.port),
            peer_count: bridge.peers().len(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, bridge: &mut ClientBridge) {
        let mut open = self.open;
        egui::Window::new("Client")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                let status = bridge.status();
                let active = matches!(
                    status,
                    ConnectionStatus::Connected | ConnectionStatus::Connecting
                );

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

                match status {
                    ConnectionStatus::Disconnected | ConnectionStatus::Error => {
                        if ui
                            .add_enabled(
                                !self.username.is_empty(),
                                egui::Button::new("Connect"),
                            )
                            .clicked()
                        {
                            let port: u16 = match self.port.parse() {
                                Ok(p) => p,
                                Err(_) => return,
                            };
                            bridge.connect(&self.ip, port, &self.username);
                        }
                    }
                    ConnectionStatus::Connecting => {
                        ui.add_enabled(false, egui::Button::new("Connecting..."));
                    }
                    ConnectionStatus::Connected => {
                        if ui.button("Disconnect").clicked() {
                            bridge.disconnect();
                        }
                    }
                }

                ui.add_space(4.0);
                match status {
                    ConnectionStatus::Disconnected => {
                        ui.label("Disconnected");
                    }
                    ConnectionStatus::Connecting => {
                        ui.label("Connecting...");
                    }
                    ConnectionStatus::Connected => {
                        ui.colored_label(
                            COLOR_OK,
                            format!(
                                "Connected as {}",
                                bridge.confirmed_username().unwrap_or("?")
                            ),
                        );
                    }
                    ConnectionStatus::Error => {
                        let msg = bridge.error_msg().unwrap_or("Connection error");
                        ui.colored_label(COLOR_ERROR, msg);
                    }
                }

                let peers = bridge.peers();
                if !peers.is_empty() {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label("Peers:");
                    for peer in peers {
                        ui.label(format!("  {}", peer));
                    }
                }
            });
        self.open = open;
    }
}
