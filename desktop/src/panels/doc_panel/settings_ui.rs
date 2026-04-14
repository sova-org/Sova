use eframe::egui;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use crate::icons;
use crate::panels::server_panel::ServerAction;
use crate::theme::STROKE_EMPHASIS;

use super::{DocPanel, SettingsContext, SettingsTab, SETTINGS_TABS};

impl DocPanel {
    pub(crate) fn show_settings_content(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        settings: SettingsContext<'_>,
    ) -> (ServerAction, bool, bool) {
        let SettingsContext {
            server,
            audio,
            options,
            devices,
            logs,
            editor_settings,
            appearance,
            view_mode,
            show_phase_bar,
        } = settings;
        let mut server_action = ServerAction::None;
        let mut appearance_changed = false;
        let mut pick_sample_folder = false;

        egui::TopBottomPanel::top("settings_tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let selected = SettingsTab::from_u8(self.settings.settings_tab);
                for &tab in &SETTINGS_TABS {
                    let r = ui.selectable_label(
                        selected == tab,
                        icons::button_text(ui, tab.icon(), tab.label()),
                    );
                    if selected == tab {
                        let accent = ui.visuals().selection.bg_fill;
                        ui.painter().line_segment(
                            [r.rect.left_bottom(), r.rect.right_bottom()],
                            egui::Stroke::new(STROKE_EMPHASIS, accent),
                        );
                    }
                    if r.clicked() {
                        self.settings.settings_tab = tab as u8;
                    }
                }
            });
        });

        match SettingsTab::from_u8(self.settings.settings_tab) {
            SettingsTab::Logs => {
                logs.show_inside(ui);
            }
            tab => {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        match tab {
                            SettingsTab::Server => {
                                egui::CollapsingHeader::new(t!("config.server"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            server_action = server.show_actions(ui);
                                        });
                                        server.show_config(ui);
                                    });

                                egui::CollapsingHeader::new(t!("config.link"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        show_link_section(ui, bridge);
                                    });

                                egui::CollapsingHeader::new(t!("config.audio"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            audio.show_restart_button(ui, bridge);
                                        });
                                        pick_sample_folder |= audio.show_config(ui);
                                    });

                                if bridge.is_connected() && bridge.audio_state().running {
                                    egui::CollapsingHeader::new(t!("config.audio_status"))
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            audio.show_status_section(ui, bridge);
                                        });
                                }
                            }
                            SettingsTab::Appearance => {
                                appearance_changed = options.show_inside(
                                    ui,
                                    editor_settings,
                                    appearance,
                                    &mut self.settings,
                                    bridge.languages(),
                                    view_mode,
                                    show_phase_bar,
                                );
                            }
                            SettingsTab::Devices => {
                                devices.show_inside(ui, bridge);
                            }
                            SettingsTab::Logs => unreachable!(),
                        }
                    });
            }
        }

        (server_action, appearance_changed, pick_sample_folder)
    }
}

fn show_link_section(ui: &mut egui::Ui, bridge: &ClientBridge) {
    let ctx = ui.ctx().clone();
    let clock = bridge.clock();
    let connected = bridge.is_connected();

    egui::Grid::new("link_config")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            let label = ui.label(t!("link.enabled"));
            let mut link_on = clock.link_enabled;
            let toggle = ui.add_enabled(connected, egui::Checkbox::without_text(&mut link_on));
            if label.hovered() || toggle.hovered() {
                crate::widgets::hint::set(&ctx, t!("link.hint.enabled"));
            }
            if toggle.changed() {
                bridge.send(ClientMessage::SetLinkEnabled(link_on));
            }
            ui.end_row();

            let label = ui.label(t!("link.start_stop_sync"));
            let mut sss = clock.start_stop_sync;
            let toggle = ui.add_enabled(connected, egui::Checkbox::without_text(&mut sss));
            if label.hovered() || toggle.hovered() {
                crate::widgets::hint::set(&ctx, t!("link.hint.start_stop_sync"));
            }
            if toggle.changed() {
                bridge.send(ClientMessage::SetStartStopSync(sss));
            }
            ui.end_row();

            let label = ui.label(t!("link.peers"));
            let status = if !clock.link_enabled {
                t!("link.status.disabled").to_string()
            } else if clock.num_peers == 0 {
                t!("link.status.listening").to_string()
            } else {
                format!("{}", clock.num_peers)
            };
            let value = ui.monospace(&status);
            if label.hovered() || value.hovered() {
                crate::widgets::hint::set(&ctx, t!("link.hint.peers"));
            }
            ui.end_row();
        });
}
