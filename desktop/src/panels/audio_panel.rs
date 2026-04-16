use crate::client_bridge::ClientBridge;
use crate::theme::{COLOR_ERROR, COLOR_OK};
use eframe::egui;
use std::path::PathBuf;

use crate::settings::AudioSettings;
use sova_server::AudioRestartConfig;
use sova_server::audio::doux_audio::{self, AudioDeviceInfo, AudioHostInfo};

const BUFFER_SIZE_OPTIONS: &[Option<u32>] = &[
    None,
    Some(64),
    Some(128),
    Some(256),
    Some(512),
    Some(1024),
    Some(2048),
];

pub struct AudioPanel {
    persisted: AudioSettings,
    #[cfg(feature = "default-samples")]
    default_samples_path: PathBuf,
    available_hosts: Vec<AudioHostInfo>,
    output_devices: Vec<AudioDeviceInfo>,
    input_devices: Vec<AudioDeviceInfo>,
    host_controls_buffer: bool,
}

impl AudioPanel {
    pub fn new(settings: AudioSettings) -> Self {
        let mut panel = Self {
            persisted: settings,
            #[cfg(feature = "default-samples")]
            default_samples_path: sova_server::audio::default_samples::ensure_default_samples(),
            available_hosts: Vec::new(),
            output_devices: Vec::new(),
            input_devices: Vec::new(),
            host_controls_buffer: false,
        };
        panel.refresh_devices();
        panel
    }

    pub fn sample_paths(&self) -> &[PathBuf] {
        &self.persisted.sample_paths
    }

    #[cfg(feature = "default-samples")]
    pub fn default_samples_path(&self) -> &std::path::Path {
        &self.default_samples_path
    }

    pub fn settings(&self) -> AudioSettings {
        self.persisted.clone()
    }

    pub fn refresh_devices(&mut self) {
        self.available_hosts = doux_audio::list_hosts();
        let selection = if self.persisted.host.is_empty() {
            doux_audio::HostSelection::Auto
        } else {
            doux_audio::HostSelection::Named(self.persisted.host.to_lowercase())
        };
        if let Ok(host) = doux_audio::get_host(selection) {
            self.output_devices = doux_audio::list_output_devices_for(&host);
            self.input_devices = doux_audio::list_input_devices_for(&host);
            self.host_controls_buffer = doux_audio::host_controls_buffer_size(&host);
        } else {
            self.output_devices.clear();
            self.input_devices.clear();
            self.host_controls_buffer = false;
        }
    }

    pub fn generate_audio_config(&self) -> AudioRestartConfig {
        AudioRestartConfig {
            host: if self.persisted.host.is_empty() {
                None
            } else {
                Some(self.persisted.host.clone())
            },
            device: if self.persisted.output_device.is_empty() {
                None
            } else {
                Some(self.persisted.output_device.clone())
            },
            input_device: if self.persisted.input_device.is_empty() {
                None
            } else {
                Some(self.persisted.input_device.clone())
            },
            channels: self.persisted.channels,
            buffer_size: self.persisted.buffer_size,
            sample_paths: {
                #[cfg(feature = "default-samples")]
                let mut paths = vec![self.default_samples_path.clone()];
                #[cfg(not(feature = "default-samples"))]
                let mut paths = Vec::new();
                paths.extend(self.persisted.sample_paths.iter().map(PathBuf::from));
                paths
            },
            max_voices: self.persisted.max_voices,
        }
    }

    pub fn show_restart_button(&self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if !bridge.is_connected() {
            return;
        }
        let r = ui.button(crate::icons::button_text(
            ui,
            crate::icons::REFRESH,
            t!("audio.restart"),
        ));
        if r.hovered() {
            crate::widgets::hint::set(ui.ctx(), t!("audio.hint.restart"));
        }
        if r.clicked() {
            bridge.restart_audio(self.generate_audio_config());
        }
    }

    pub fn show_status_section(&self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if !bridge.is_connected() {
            return;
        }
        let state = bridge.audio_state();
        if state.running || state.error.is_some() {
            self.show_status(ui, state);
        }
    }

    pub fn show_config(&mut self, ui: &mut egui::Ui) -> bool {
        let mut pick_folder_requested = false;
        egui::Grid::new("audio_config")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                use crate::widgets::hint;

                // Host selector — only shown when multiple hosts are available
                if self.available_hosts.len() > 1 {
                    let r = ui.label(t!("audio.host"));
                    hint::on_hover(ui.ctx(), &r, t!("audio.hint.host"));
                    ui.horizontal(|ui| {
                        let prev_host = self.persisted.host.clone();
                        let default = t!("audio.system_default").to_string();
                        let names: Vec<&str> = self
                            .available_hosts
                            .iter()
                            .filter(|h| h.available)
                            .map(|h| h.name.as_str())
                            .collect();
                        crate::widgets::combo_string_list(
                            ui,
                            "audio_host",
                            &mut self.persisted.host,
                            Some(&default),
                            &names,
                        );
                        let refresh = ui.small_button("\u{21bb}");
                        hint::on_hover(ui.ctx(), &refresh, t!("hint.refresh_devices"));
                        if refresh.clicked() {
                            self.refresh_devices();
                        }
                        if self.persisted.host != prev_host {
                            self.persisted.output_device.clear();
                            self.persisted.input_device.clear();
                            self.refresh_devices();
                        }
                    });
                    ui.end_row();
                }

                let r = ui.label(t!("audio.output"));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.output"));
                ui.horizontal(|ui| {
                    let default = t!("audio.system_default").to_string();
                    let names: Vec<&str> = self
                        .output_devices
                        .iter()
                        .map(|d| d.name.as_str())
                        .collect();
                    crate::widgets::combo_string_list(
                        ui,
                        "audio_output_device",
                        &mut self.persisted.output_device,
                        Some(&default),
                        &names,
                    );
                    // Refresh button here only when host selector is hidden
                    if self.available_hosts.len() <= 1 {
                        let refresh = ui.small_button("\u{21bb}");
                        hint::on_hover(ui.ctx(), &refresh, t!("hint.refresh_devices"));
                        if refresh.clicked() {
                            self.refresh_devices();
                        }
                    }
                });
                ui.end_row();

                let r = ui.label(t!("audio.input"));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.input"));
                ui.horizontal(|ui| {
                    let default = t!("audio.system_default").to_string();
                    let names: Vec<&str> =
                        self.input_devices.iter().map(|d| d.name.as_str()).collect();
                    crate::widgets::combo_string_list(
                        ui,
                        "audio_input_device",
                        &mut self.persisted.input_device,
                        Some(&default),
                        &names,
                    );
                });
                ui.end_row();

                hint::labeled(ui, t!("audio.channels"), t!("audio.hint.channels"), |ui| {
                    ui.add(egui::DragValue::new(&mut self.persisted.channels).range(1..=64))
                });
                ui.end_row();

                hint::labeled(ui, t!("audio.voices"), t!("audio.hint.voices"), |ui| {
                    ui.add(egui::DragValue::new(&mut self.persisted.max_voices).range(1..=2048))
                });
                ui.end_row();

                let r = ui.label(t!("audio.buffer"));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.buffer"));
                if self.host_controls_buffer {
                    ui.add_enabled(false, egui::Label::new(t!("audio.host_managed")));
                } else {
                    let buf_label = match self.persisted.buffer_size {
                        None => t!("audio.default").to_string(),
                        Some(s) => s.to_string(),
                    };
                    let r = egui::ComboBox::from_id_salt("audio_buffer_size")
                        .selected_text(buf_label)
                        .show_ui(ui, |ui| {
                            for &opt in BUFFER_SIZE_OPTIONS {
                                let label = match opt {
                                    None => t!("audio.default").to_string(),
                                    Some(s) => s.to_string(),
                                };
                                ui.selectable_value(&mut self.persisted.buffer_size, opt, label);
                            }
                        });
                    hint::on_hover(ui.ctx(), &r.response, t!("audio.hint.buffer"));
                }
                ui.end_row();
            });

        ui.add_space(4.0);
        ui.label(t!("audio.sample_paths"));

        #[cfg(feature = "default-samples")]
        {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(self.default_samples_path.display().to_string())
                        .color(egui::Color32::GRAY),
                )
                .wrap_mode(egui::TextWrapMode::Wrap),
            );
            ui.weak("(built-in)");
        }

        let mut remove_idx = None;
        for (i, path) in self.persisted.sample_paths.iter().enumerate() {
            ui.add(
                egui::Label::new(egui::RichText::new(path.display().to_string()).monospace())
                    .wrap_mode(egui::TextWrapMode::Wrap),
            );
            if ui.small_button("x").clicked() {
                remove_idx = Some(i);
            }
        }
        if let Some(idx) = remove_idx {
            self.persisted.sample_paths.remove(idx);
        }

        let r = ui.button(t!("audio.add_folder"));
        if r.hovered() {
            crate::widgets::hint::set(ui.ctx(), t!("audio.hint.add_folder"));
        }
        if r.clicked() {
            pick_folder_requested = true;
        }
        pick_folder_requested
    }

    pub fn add_sample_path(&mut self, path: PathBuf) {
        self.persisted.sample_paths.push(path);
    }

    fn show_status(&self, ui: &mut egui::Ui, state: &sova_server::AudioEngineState) {
        if state.running {
            ui.horizontal(|ui| {
                ui.colored_label(COLOR_OK, "\u{25cf}");
                ui.label(t!("audio.running"));
            });
        }

        if let Some(ref err) = state.error {
            ui.colored_label(COLOR_ERROR, err.as_str());
        }

        if state.running {
            egui::Grid::new("audio_status")
                .num_columns(2)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    if let Some(ref dev) = state.device {
                        ui.label(t!("audio.device"));
                        ui.label(dev);
                        ui.end_row();
                    }

                    ui.label(t!("audio.sample_rate"));
                    ui.label(format!("{} Hz", state.sample_rate));
                    ui.end_row();

                    ui.label(t!("audio.channels"));
                    ui.label(state.channels.to_string());
                    ui.end_row();

                    if let Some(buf) = state.buffer_size {
                        ui.label(t!("audio.buffer"));
                        ui.label(buf.to_string());
                        ui.end_row();
                    }

                    ui.label(t!("audio.voices"));
                    ui.label(format!(
                        "{} / {} (peak {})",
                        state.active_voices, state.max_voices, state.peak_voices
                    ));
                    ui.end_row();

                    ui.label(t!("audio.cpu"));
                    ui.add_sized(
                        [120.0, ui.spacing().interact_size.y],
                        egui::ProgressBar::new(state.cpu_load)
                            .text(format!("{:.1}%", state.cpu_load * 100.0))
                            .corner_radius(egui::CornerRadius::ZERO),
                    );
                    ui.end_row();
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let original = AudioSettings {
            host: "CoreAudio".into(),
            output_device: "Built-in Output".into(),
            input_device: "Built-in Microphone".into(),
            channels: 2,
            buffer_size: Some(256),
            max_voices: 64,
            sample_paths: vec![PathBuf::from("/tmp/samples")],
            master_volume: 0.8,
        };
        let panel = AudioPanel::new(original.clone());
        assert!(panel.settings() == original);
    }
}
