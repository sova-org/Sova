use crate::client_bridge::{ClientBridge, ConnectionStatus};
use crate::widgets::{COLOR_ERROR, COLOR_OK};
use eframe::egui;

use crate::settings::ClientSettings;

pub struct ClientInfo {
    pub connected: bool,
    pub username: Option<String>,
    pub address: String,
    pub peers: Vec<String>,
    pub has_feedback: bool,
}

pub struct SplashAction {
    pub start_server: bool,
    pub stop_server: bool,
    pub open_server_config: bool,
    pub start_feedback: bool,
}

pub struct ClientPanel {
    ip: String,
    port: String,
    username: String,
    password: String,
    feedback: bool,
}

impl ClientPanel {
    pub fn new(settings: ClientSettings) -> Self {
        Self {
            ip: settings.ip,
            port: settings.port,
            username: settings.username,
            password: String::new(),
            feedback: settings.feedback,
        }
    }

    pub fn settings(&self) -> ClientSettings {
        ClientSettings {
            ip: self.ip.clone(),
            port: self.port.clone(),
            username: self.username.clone(),
            feedback: self.feedback,
        }
    }

    pub fn info(&self, bridge: &ClientBridge) -> ClientInfo {
        ClientInfo {
            connected: bridge.is_connected(),
            username: bridge.confirmed_username().map(str::to_owned),
            address: format!("{}:{}", self.ip, self.port),
            peers: bridge.peers().to_vec(),
            has_feedback: bridge.has_feedback(),
        }
    }

    pub fn show_centered(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &mut ClientBridge,
        server_running: bool,
    ) -> SplashAction {
        let avail = ui.available_size();
        let mut start_server = false;
        let mut stop_server = false;
        let mut open_server_config = false;

        ui.vertical_centered(|ui| {
            ui.add_space((avail.y - 350.0).max(40.0) / 2.0);
            ui.set_max_width(320.0);

            let status = bridge.status();
            let active = matches!(
                status,
                ConnectionStatus::Connected | ConnectionStatus::Connecting
            );

            // Icon
            ui.add(
                egui::Image::new(egui::include_image!("../assets/icon.png"))
                    .max_size(egui::vec2(96.0, 96.0)),
            );
            ui.add_space(8.0);

            ui.heading(
                egui::RichText::new(t!("client.heading"))
                    .size(28.0)
                    .strong(),
            );
            ui.add_space(16.0);

            // Form
            let form_width = 280.0;
            let form_offset = (ui.available_width() - form_width).max(0.0) / 2.0;
            ui.add_enabled_ui(!active, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(form_offset);
                    ui.vertical(|ui| {
                        ui.set_width(form_width);
                        egui::Grid::new("client_config")
                            .num_columns(2)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                let label = ui.label(t!("client.ip"));
                                let field = ui.text_edit_singleline(&mut self.ip);
                                if label.hovered() || field.hovered() {
                                    crate::widgets::hint::set(ui.ctx(), t!("client.hint.ip"));
                                }
                                ui.end_row();

                                let label = ui.label(t!("client.port"));
                                let field = ui.text_edit_singleline(&mut self.port);
                                if label.hovered() || field.hovered() {
                                    crate::widgets::hint::set(ui.ctx(), t!("client.hint.port"));
                                }
                                ui.end_row();

                                let label = ui.label(t!("client.user"));
                                let field = ui.text_edit_singleline(&mut self.username);
                                if label.hovered() || field.hovered() {
                                    crate::widgets::hint::set(ui.ctx(), t!("client.hint.user"));
                                }
                                ui.end_row();

                                let label = ui.label(t!("client.password"));
                                let field = ui.add(
                                    egui::TextEdit::singleline(&mut self.password).password(true),
                                );
                                if label.hovered() || field.hovered() {
                                    crate::widgets::hint::set(ui.ctx(), t!("client.hint.password"));
                                }
                                ui.end_row();

                                if server_running {
                                    self.feedback = false;
                                } else {
                                    ui.label("");
                                    let r = ui
                                        .checkbox(&mut self.feedback, t!("client.audio_feedback"));
                                    if r.hovered() {
                                        crate::widgets::hint::set(
                                            ui.ctx(),
                                            t!("client.hint.audio_feedback"),
                                        );
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                });
            });

            ui.add_space(12.0);

            // Buttons side-by-side
            let button_width = 130.0;
            let btn_height = ui.spacing().interact_size.y;
            let total_width = button_width * 2.0 + btn_height + 16.0;
            let offset = (ui.available_width() - total_width).max(0.0) / 2.0;

            ui.horizontal(|ui| {
                ui.add_space(offset);

                if server_running {
                    let r = ui.add(
                        egui::Button::new(
                            egui::RichText::new(t!("client.server_running")).color(COLOR_OK),
                        )
                        .min_size(egui::vec2(button_width, 0.0)),
                    );
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("client.hint.stop_server"));
                    }
                    if r.clicked() {
                        stop_server = true;
                    }
                } else {
                    let r = ui.add(
                        egui::Button::new(t!("client.start_server"))
                            .min_size(egui::vec2(button_width, 0.0)),
                    );
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("client.hint.start_server"));
                    }
                    if r.clicked() {
                        start_server = true;
                    }
                }

                let r = ui.add(
                    egui::Button::new(crate::icons::GEAR)
                        .min_size(egui::vec2(btn_height, btn_height)),
                );
                if r.hovered() {
                    crate::widgets::hint::set(ui.ctx(), t!("client.hint.server_config"));
                }
                if r.clicked() {
                    open_server_config = true;
                }

                match status {
                    ConnectionStatus::Disconnected | ConnectionStatus::Error => {
                        let r = ui.add_enabled(
                            !self.username.is_empty(),
                            egui::Button::new(t!("common.connect"))
                                .min_size(egui::vec2(button_width, 0.0)),
                        );
                        if r.hovered() {
                            crate::widgets::hint::set(ui.ctx(), t!("client.hint.connect"));
                        }
                        if r.clicked()
                            && let Ok(port) = self.port.parse::<u16>()
                        {
                            bridge.connect(
                                &self.ip,
                                port,
                                &self.username,
                                &self.password,
                                self.feedback,
                            );
                        }
                    }
                    ConnectionStatus::Connecting => {
                        if ui
                            .add(
                                egui::Button::new(t!("common.cancel"))
                                    .min_size(egui::vec2(button_width, 0.0)),
                            )
                            .clicked()
                        {
                            bridge.disconnect();
                        }
                    }
                    ConnectionStatus::Connected => {}
                }
            });

            ui.add_space(8.0);

            // Status line
            match status {
                ConnectionStatus::Disconnected => {
                    ui.colored_label(
                        egui::Color32::GRAY,
                        format!(
                            "{} {}",
                            crate::icons::CIRCLE_FILLED,
                            t!("client.disconnected")
                        ),
                    );
                }
                ConnectionStatus::Connecting => {
                    ui.label(t!("client.connecting"));
                }
                ConnectionStatus::Connected => {}
                ConnectionStatus::Error => {
                    let msg = bridge
                        .error_msg()
                        .map(|s| s.to_owned())
                        .unwrap_or_else(|| t!("client.connection_error").into());
                    ui.colored_label(COLOR_ERROR, msg);
                }
            }
        });

        SplashAction {
            start_server,
            stop_server,
            open_server_config,
            start_feedback: self.feedback,
        }
    }
}
