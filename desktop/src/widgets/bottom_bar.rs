use crate::panels::client_panel::ClientInfo;
use crate::icons;
use crate::panels::server_panel::ServerInfo;
use crate::theme::COLOR_OK;
use crate::widgets::shortcut::{self, Key, Shortcut};
use eframe::egui;

pub struct BottomBarResponse {
    pub disconnect: bool,
    pub open_palette: bool,
    pub toggle_chat: bool,
    pub toggle_sample_browser: bool,
}

pub fn bottom_bar(
    ui: &mut egui::Ui,
    server: &ServerInfo,
    client: &ClientInfo,
    chat_active: bool,
    sample_browser_active: bool,
    sample_browser_available: bool,
) -> BottomBarResponse {
    let mut disconnect = false;
    let mut open_palette = false;
    let mut toggle_chat = false;
    let mut toggle_sample_browser = false;
    ui.horizontal(|ui| {
        if server.running {
            ui.colored_label(COLOR_OK, icons::rich(icons::CIRCLE_FILLED));
            ui.label(t!("bottom.server", addr = &server.address));
        } else {
            ui.colored_label(egui::Color32::GRAY, icons::rich(icons::CIRCLE_FILLED));
            ui.label(t!("bottom.server_stopped"));
        }

        ui.separator();

        if client.connected {
            ui.colored_label(COLOR_OK, icons::rich(icons::CIRCLE_FILLED));
            if let Some(ref name) = client.username {
                ui.label(format!("{} @ {}", name, client.address));
            }
            if client.has_feedback {
                ui.label(t!("bottom.audio_feedback"));
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
            ui.colored_label(egui::Color32::GRAY, icons::rich(icons::CIRCLE_FILLED));
            ui.label(t!("bottom.disconnected"));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Command palette (rightmost)
            let glyph = shortcut::format(ui.ctx(), &Shortcut::cmd(Key::Char('K')));
            let palette_btn = ui.weak(glyph);
            if palette_btn.clicked() {
                open_palette = true;
            }
            if palette_btn.hovered() {
                super::hint::set(ui.ctx(), t!("bottom.command_palette").to_string());
            }

            ui.separator();

            // Sample browser toggle
            ui.add_enabled_ui(sample_browser_available, |ui| {
                let sb_btn =
                    ui.selectable_label(sample_browser_active, icons::rich(icons::MUSIC_NOTE));
                if sb_btn.clicked() {
                    toggle_sample_browser = true;
                }
                if sb_btn.hovered() {
                    super::hint::set(ui.ctx(), t!("bottom.sample_browser").to_string());
                }
            });

            // Chat toggle
            let chat_btn = ui.selectable_label(chat_active, icons::rich(icons::CHAT));
            if chat_btn.clicked() {
                toggle_chat = true;
            }
            if chat_btn.hovered() {
                super::hint::set(ui.ctx(), t!("bottom.chat").to_string());
            }

            if let Some(hint) = super::hint::current(ui.ctx()) {
                ui.weak(hint);
            }
        });
    });
    BottomBarResponse {
        disconnect,
        open_palette,
        toggle_chat,
        toggle_sample_browser,
    }
}
