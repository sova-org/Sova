use crate::client_panel::ClientInfo;
use crate::server_panel::ServerInfo;
use crate::widgets::COLOR_OK;
use eframe::egui;

pub struct BottomBarResponse {
    pub disconnect: bool,
    pub open_palette: bool,
}

pub fn bottom_bar(ui: &mut egui::Ui, server: &ServerInfo, client: &ClientInfo) -> BottomBarResponse {
    let mut disconnect = false;
    let mut open_palette = false;
    ui.horizontal(|ui| {
        if server.running {
            ui.colored_label(COLOR_OK, crate::icons::CIRCLE_FILLED);
            ui.label(t!("bottom.server", addr = &server.address));
        } else {
            ui.colored_label(egui::Color32::GRAY, crate::icons::CIRCLE_FILLED);
            ui.label(t!("bottom.server_stopped"));
        }

        ui.separator();

        if client.connected {
            ui.colored_label(COLOR_OK, crate::icons::CIRCLE_FILLED);
            if let Some(ref name) = client.username {
                ui.label(format!("{} @ {}", name, client.address));
            }
            if client.has_feedback {
                ui.label(t!("bottom.local_audio"));
            }
            if !client.peers.is_empty() {
                let tooltip = client.peers.join(", ");
                ui.label(t!("bottom.peers", count = client.peers.len()))
                    .on_hover_text(tooltip);
            }
            if ui.small_button(t!("common.disconnect")).clicked() {
                disconnect = true;
            }
        } else {
            ui.colored_label(egui::Color32::GRAY, crate::icons::CIRCLE_FILLED);
            ui.label(t!("bottom.disconnected"));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let shortcut = if cfg!(target_os = "macos") { "\u{2318}K" } else { "Ctrl+K" };
            let btn = ui.weak(shortcut);
            if btn.clicked() {
                open_palette = true;
            }
            if btn.hovered() {
                super::hint::set(ui.ctx(), t!("bottom.command_palette").to_string());
            }

            if let Some(hint) = super::hint::current(ui.ctx()) {
                ui.weak(hint);
            }
        });
    });
    BottomBarResponse { disconnect, open_palette }
}
