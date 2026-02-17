use crate::client_bridge::ClientBridge;
use crate::widgets::{COLOR_ERROR, COLOR_OK};
use eframe::egui;
use std::path::PathBuf;

use crate::settings::AudioSettings;
use sova_server::{AudioRestartConfig, ClientMessage};
use sova_server::audio::doux_audio::AudioDeviceInfo;

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
    pub open: bool,
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
            open: false,
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
        }
    }

    fn refresh_devices(&mut self) {
        self.output_devices = sova_server::audio::doux_audio::list_output_devices();
        self.input_devices = sova_server::audio::doux_audio::list_input_devices();
    }

    pub fn initial_audio_config(&self) -> AudioRestartConfig {
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
            sample_paths: self
                .sample_paths
                .iter()
                .map(|p| p.into())
                .collect(),
            max_voices: self.max_voices,
        }
    }

    pub fn restart_message(&self) -> ClientMessage {
        ClientMessage::RestartAudioEngine {
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
            sample_paths: self
                .sample_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, bridge: &ClientBridge) {
        let mut open = self.open;
        egui::Window::new("Audio")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                self.show_config(ui);
                ui.separator();

                if !bridge.is_connected() {
                    ui.colored_label(egui::Color32::GRAY, "Not connected");
                } else {
                    ui.horizontal(|ui| {
                        let r = ui.button("Restart Audio");
                        if r.hovered() {
                            crate::widgets::hint::set(ctx, "Restart the audio engine with current settings");
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
            });
        self.open = open;
    }

    fn show_config(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("audio_config")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                let ctx = ui.ctx().clone();
                let hint = |ctx: &egui::Context, r: &egui::Response, text: &'static str| {
                    if r.hovered() { crate::widgets::hint::set(ctx, text); }
                };

                let h = "Audio output device for playback";
                hint(&ctx, &ui.label("Output"), h);
                let r = egui::ComboBox::from_id_salt("audio_output_device")
                    .selected_text(if self.output_device.is_empty() {
                        "System Default"
                    } else {
                        &self.output_device
                    })
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.output_device,
                            String::new(),
                            "System Default",
                        );
                        for dev in &self.output_devices {
                            ui.selectable_value(
                                &mut self.output_device,
                                dev.name.clone(),
                                &dev.name,
                            );
                        }
                    });
                hint(&ctx, &r.response, h);
                ui.end_row();

                let h = "Audio input device for recording or analysis";
                hint(&ctx, &ui.label("Input"), h);
                let r = egui::ComboBox::from_id_salt("audio_input_device")
                    .selected_text(if self.input_device.is_empty() {
                        "System Default"
                    } else {
                        &self.input_device
                    })
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.input_device,
                            String::new(),
                            "System Default",
                        );
                        for dev in &self.input_devices {
                            ui.selectable_value(
                                &mut self.input_device,
                                dev.name.clone(),
                                &dev.name,
                            );
                        }
                    });
                hint(&ctx, &r.response, h);
                ui.end_row();

                let h = "Number of output channels (1-64)";
                hint(&ctx, &ui.label("Channels"), h);
                let r = ui.add(egui::DragValue::new(&mut self.channels).range(1..=64));
                hint(&ctx, &r, h);
                ui.end_row();

                let h = "Maximum polyphony for the audio engine (1-128)";
                hint(&ctx, &ui.label("Voices"), h);
                let r = ui.add(egui::DragValue::new(&mut self.max_voices).range(1..=128));
                hint(&ctx, &r, h);
                ui.end_row();

                let h = "Audio buffer size — lower is less latency, higher is more stable";
                hint(&ctx, &ui.label("Buffer"), h);
                let buf_label = match self.buffer_size {
                    None => "Default".to_string(),
                    Some(s) => s.to_string(),
                };
                let r = egui::ComboBox::from_id_salt("audio_buffer_size")
                    .selected_text(buf_label)
                    .show_ui(ui, |ui| {
                        for &opt in BUFFER_SIZE_OPTIONS {
                            let label = match opt {
                                None => "Default".to_string(),
                                Some(s) => s.to_string(),
                            };
                            ui.selectable_value(&mut self.buffer_size, opt, label);
                        }
                    });
                hint(&ctx, &r.response, h);
                ui.end_row();
            });

        ui.add_space(4.0);
        ui.label("Sample Paths");

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

        let r = ui.button("Add folder...");
        if r.hovered() {
            crate::widgets::hint::set(ui.ctx(), "Add a folder containing audio samples");
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
                ui.colored_label(COLOR_OK, "●");
                ui.label("Running");
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
                        ui.label("Device");
                        ui.label(dev);
                        ui.end_row();
                    }

                    ui.label("Sample Rate");
                    ui.label(format!("{} Hz", state.sample_rate));
                    ui.end_row();

                    ui.label("Channels");
                    ui.label(state.channels.to_string());
                    ui.end_row();

                    if let Some(buf) = state.buffer_size {
                        ui.label("Buffer");
                        ui.label(buf.to_string());
                        ui.end_row();
                    }

                    ui.label("Voices");
                    ui.label(format!(
                        "{} / {} (peak {})",
                        state.active_voices, state.max_voices, state.peak_voices
                    ));
                    ui.end_row();

                    ui.label("CPU");
                    ui.add(
                        egui::ProgressBar::new(state.cpu_load)
                            .text(format!("{:.1}%", state.cpu_load * 100.0)),
                    );
                    ui.end_row();
                });
        }
    }
}
