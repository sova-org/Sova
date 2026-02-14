use crate::client_panel::ClientInfo;
use crate::server_panel::ServerInfo;
use crate::widgets::COLOR_OK;
use eframe::egui;

pub fn bottom_bar(ui: &mut egui::Ui, server: &ServerInfo, client: &ClientInfo) -> bool {
    let mut disconnect = false;
    ui.horizontal(|ui| {
        if server.running {
            ui.colored_label(COLOR_OK, crate::icons::CIRCLE_FILLED);
            ui.label(format!("Server {}", server.address));
        } else {
            ui.colored_label(egui::Color32::GRAY, crate::icons::CIRCLE_FILLED);
            ui.label("Server stopped");
        }

        ui.separator();

        if client.connected {
            ui.colored_label(COLOR_OK, crate::icons::CIRCLE_FILLED);
            if let Some(ref name) = client.username {
                ui.label(format!("{} @ {}", name, client.address));
            }
            if client.peer_count > 0 {
                ui.label(format!("({} peers)", client.peer_count));
            }
            if ui.small_button("Disconnect").clicked() {
                disconnect = true;
            }
        } else {
            ui.colored_label(egui::Color32::GRAY, crate::icons::CIRCLE_FILLED);
            ui.label("Disconnected");
        }
    });
    disconnect
}
