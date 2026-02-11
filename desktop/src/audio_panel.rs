use crate::widgets::{COLOR_ERROR, COLOR_OK};
use eframe::egui;
use std::path::PathBuf;

use crossbeam_channel::Sender;
use sova_server::audio::doux_audio::{self, AudioDeviceInfo};
use sova_server::{AudioEngineState, AudioRestartConfig, AudioRestartRequest};

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
    // Config (editable when server stopped)
    output_device: String,
    input_device: String,
    channels: u16,
    buffer_size: Option<u32>,
    max_voices: usize,
    sample_paths: Vec<PathBuf>,
    // Device cache
    output_devices: Vec<AudioDeviceInfo>,
    input_devices: Vec<AudioDeviceInfo>,
}

impl AudioPanel {
    pub fn new() -> Self {
        let mut panel = Self {
            open: false,
            output_device: String::new(),
            input_device: String::new(),
            channels: 2,
            buffer_size: None,
            max_voices: 32,
            sample_paths: Vec::new(),
            output_devices: Vec::new(),
            input_devices: Vec::new(),
        };
        panel.refresh_devices();
        panel
    }

    pub fn config(&self) -> AudioRestartConfig {
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
            max_voices: self.max_voices,
            sample_paths: self.sample_paths.clone(),
        }
    }

    pub fn refresh_devices(&mut self) {
        self.output_devices = doux_audio::list_output_devices();
        self.input_devices = doux_audio::list_input_devices();
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        audio_state: &AudioEngineState,
        restart_tx: &Option<Sender<AudioRestartRequest>>,
    ) {
        let mut open = self.open;
        egui::Window::new("Audio")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                let running = audio_state.running;

                self.show_config(ui, running);
                ui.separator();
                self.show_status(ui, audio_state, restart_tx);
            });
        self.open = open;
    }

    fn show_config(&mut self, ui: &mut egui::Ui, running: bool) {
        ui.add_enabled_ui(!running, |ui| {
            egui::Grid::new("audio_config")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    // Output device
                    ui.label("Output");
                    egui::ComboBox::from_id_salt("audio_output_device")
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
                    ui.end_row();

                    // Input device
                    ui.label("Input");
                    egui::ComboBox::from_id_salt("audio_input_device")
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
                    ui.end_row();

                    // Channels
                    ui.label("Channels");
                    ui.add(egui::DragValue::new(&mut self.channels).range(1..=64));
                    ui.end_row();

                    // Max voices
                    ui.label("Voices");
                    ui.add(egui::DragValue::new(&mut self.max_voices).range(1..=128));
                    ui.end_row();

                    // Buffer size
                    ui.label("Buffer");
                    let buf_label = match self.buffer_size {
                        None => "Default".to_string(),
                        Some(s) => s.to_string(),
                    };
                    egui::ComboBox::from_id_salt("audio_buffer_size")
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
                    ui.end_row();
                });

            // Sample paths
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

            if ui.button("Add folder...").clicked()
                && let Some(folder) = rfd::FileDialog::new().pick_folder()
            {
                self.sample_paths.push(folder);
            }
        });
    }

    fn show_status(
        &mut self,
        ui: &mut egui::Ui,
        state: &AudioEngineState,
        restart_tx: &Option<Sender<AudioRestartRequest>>,
    ) {
        if !state.running && state.error.is_none() {
            ui.colored_label(egui::Color32::GRAY, "Audio stopped");
            return;
        }

        // Status indicator
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

                    ui.label("Samples");
                    ui.label(format!("{:.1} MB", state.sample_pool_mb));
                    ui.end_row();
                });

            ui.add_space(4.0);
            if let Some(tx) = restart_tx
                && ui.button("Restart").clicked()
            {
                let (resp_tx, _resp_rx) = crossbeam_channel::bounded(1);
                let _ = tx.send(AudioRestartRequest {
                    config: self.config(),
                    response_tx: resp_tx,
                });
            }
        }
    }
}
