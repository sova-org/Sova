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
    ip: String,
    port: String,
    username: String,
}

impl ClientPanel {
    pub fn new(settings: ClientSettings) -> Self {
        Self {
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

    pub fn show_centered(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &mut ClientBridge,
        server_running: bool,
    ) -> bool {
        let avail = ui.available_size();
        let mut start_server = false;

        ui.vertical_centered(|ui| {
            ui.add_space(avail.y * 0.3);
            ui.set_max_width(280.0);

            let status = bridge.status();
            let active = matches!(
                status,
                ConnectionStatus::Connected | ConnectionStatus::Connecting
            );

            ui.heading("Connect to Server");
            ui.add_space(12.0);

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
                        && let Ok(port) = self.port.parse::<u16>()
                    {
                        bridge.connect(&self.ip, port, &self.username);
                    }
                }
                ConnectionStatus::Connecting => {
                    ui.add_enabled(false, egui::Button::new("Connecting..."));
                }
                ConnectionStatus::Connected => {}
            }

            ui.add_space(4.0);
            match status {
                ConnectionStatus::Disconnected => {
                    ui.colored_label(egui::Color32::GRAY, "Disconnected");
                }
                ConnectionStatus::Connecting => {
                    ui.label("Connecting...");
                }
                ConnectionStatus::Connected => {}
                ConnectionStatus::Error => {
                    let msg = bridge.error_msg().unwrap_or("Connection error");
                    ui.colored_label(COLOR_ERROR, msg);
                }
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            if server_running {
                ui.colored_label(COLOR_OK, "Server Running");
            } else if ui.button("Start Server").clicked() {
                start_server = true;
            }
        });

        start_server
    }
}
