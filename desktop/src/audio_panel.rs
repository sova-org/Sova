use crate::client_bridge::ClientBridge;
use crate::widgets::{COLOR_ERROR, COLOR_OK};
use eframe::egui;
use std::path::PathBuf;

use crate::settings::AudioSettings;
use sova_server::audio::doux_audio::AudioDeviceInfo;
use sova_server::{AudioRestartConfig, ClientMessage};

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
    output_device: String,
    input_device: String,
    channels: u16,
    buffer_size: Option<u32>,
    max_voices: usize,
    sample_paths: Vec<PathBuf>,
    output_devices: Vec<AudioDeviceInfo>,
    input_devices: Vec<AudioDeviceInfo>,
}

impl AudioPanel {
    pub fn new(settings: AudioSettings) -> Self {
        let mut panel = Self {
            output_device: settings.output_device,
            input_device: settings.input_device,
            channels: settings.channels,
            buffer_size: settings.buffer_size,
            max_voices: settings.max_voices,
            sample_paths: settings.sample_paths,
            output_devices: Vec::new(),
            input_devices: Vec::new(),
        };
        panel.refresh_devices();
        panel
    }

    pub fn sample_paths(&self) -> &[PathBuf] {
        &self.sample_paths
    }

    pub fn settings(&self) -> AudioSettings {
        AudioSettings {
            output_device: self.output_device.clone(),
            input_device: self.input_device.clone(),
            channels: self.channels,
            buffer_size: self.buffer_size,
            max_voices: self.max_voices,
            sample_paths: self.sample_paths.clone(),
            ..Default::default()
        }
    }

    fn refresh_devices(&mut self) {
        self.output_devices = sova_server::audio::doux_audio::list_output_devices();
        self.input_devices = sova_server::audio::doux_audio::list_input_devices();
    }

    pub fn generate_audio_config(&self) -> AudioRestartConfig {
        AudioRestartConfig {
            device: if self.output_device.is_empty() {
                None
            } else {
                Some(self.output_device.clone())
            },
            input_device: if self.input_device.is_empty() {
                None
            } else {
                Some(self.input_device.clone())
            },
            channels: self.channels,
            buffer_size: self.buffer_size,
            sample_paths: self.sample_paths.iter().map(|p| p.into()).collect(),
            max_voices: self.max_voices,
        }
    }

    pub fn restart_message(&self) -> ClientMessage {
        ClientMessage::RestartAudioEngine(self.generate_audio_config())
    }

    pub fn show_inside(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        self.show_config(ui);
        ui.separator();

        if !bridge.is_connected() {
            ui.colored_label(egui::Color32::GRAY, t!("common.not_connected"));
        } else {
            ui.horizontal(|ui| {
                let r = ui.button(t!("audio.restart"));
                if r.hovered() {
                    crate::widgets::hint::set(ui.ctx(), t!("audio.hint.restart"));
                }
                if r.clicked() {
                    bridge.send(self.restart_message());
                }
            });

            let state = bridge.audio_state();
            if state.running || state.error.is_some() {
                ui.separator();
                self.show_status(ui, state);
            }
        }
    }

    fn show_config(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("audio_config")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                use crate::widgets::hint;

                let r = ui.label(t!("audio.output"));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.output"));
                let r = egui::ComboBox::from_id_salt("audio_output_device")
                    .selected_text(if self.output_device.is_empty() {
                        t!("audio.system_default")
                    } else {
                        self.output_device.clone().into()
                    })
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.output_device,
                            String::new(),
                            t!("audio.system_default"),
                        );
                        for dev in &self.output_devices {
                            ui.selectable_value(
                                &mut self.output_device,
                                dev.name.clone(),
                                &dev.name,
                            );
                        }
                    });
                hint::on_hover(ui.ctx(), &r.response, t!("audio.hint.output"));
                ui.end_row();

                let r = ui.label(t!("audio.input"));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.input"));
                let r = egui::ComboBox::from_id_salt("audio_input_device")
                    .selected_text(if self.input_device.is_empty() {
                        t!("audio.system_default")
                    } else {
                        self.input_device.clone().into()
                    })
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.input_device,
                            String::new(),
                            t!("audio.system_default"),
                        );
                        for dev in &self.input_devices {
                            ui.selectable_value(
                                &mut self.input_device,
                                dev.name.clone(),
                                &dev.name,
                            );
                        }
                    });
                hint::on_hover(ui.ctx(), &r.response, t!("audio.hint.input"));
                ui.end_row();

                let r = ui.label(t!("audio.channels"));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.channels"));
                let r = ui.add(egui::DragValue::new(&mut self.channels).range(1..=64));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.channels"));
                ui.end_row();

                let r = ui.label(t!("audio.voices"));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.voices"));
                let r = ui.add(egui::DragValue::new(&mut self.max_voices).range(1..=128));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.voices"));
                ui.end_row();

                let r = ui.label(t!("audio.buffer"));
                hint::on_hover(ui.ctx(), &r, t!("audio.hint.buffer"));
                let buf_label = match self.buffer_size {
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
                            ui.selectable_value(&mut self.buffer_size, opt, label);
                        }
                    });
                hint::on_hover(ui.ctx(), &r.response, t!("audio.hint.buffer"));
                ui.end_row();
            });

        ui.add_space(4.0);
        ui.label(t!("audio.sample_paths"));

        let mut remove_idx = None;
        for (i, path) in self.sample_paths.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.monospace(path.display().to_string());
                if ui.small_button("x").clicked() {
                    remove_idx = Some(i);
                }
            });
        }
        if let Some(idx) = remove_idx {
            self.sample_paths.remove(idx);
        }

        let r = ui.button(t!("audio.add_folder"));
        if r.hovered() {
            crate::widgets::hint::set(ui.ctx(), t!("audio.hint.add_folder"));
        }
        if r.clicked() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                self.sample_paths.push(folder);
            }
        }
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
                    ui.add(
                        egui::ProgressBar::new(state.cpu_load)
                            .text(format!("{:.1}%", state.cpu_load * 100.0)),
                    );
                    ui.end_row();
                });
        }
    }
}
