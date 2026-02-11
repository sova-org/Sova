use crate::server_panel::ServerResources;
use crate::widgets::{COLOR_ERROR, COLOR_OK};
use eframe::egui;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex as StdMutex};

use crate::settings::AudioSettings;
use sova_server::audio::doux_audio::{self, AudioDeviceInfo};
use sova_server::audio::{AudioThread, ScopeCapture, spawn_audio_thread};
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
    pub audio_engine_state: Arc<StdMutex<AudioEngineState>>,
    audio_thread: Option<AudioThread>,
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
            audio_engine_state: Arc::new(StdMutex::new(AudioEngineState::default())),
            audio_thread: None,
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

    fn config(&self) -> AudioRestartConfig {
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

    pub fn start(&mut self, res: &ServerResources) {
        self.audio_thread = Some(spawn_audio_thread(
            self.config(),
            Arc::clone(&self.audio_engine_state),
            Arc::clone(&res.devices),
            Arc::clone(&res.clock_server),
            res.update_sender.clone(),
        ));
    }

    pub fn stop(&mut self) {
        if let Some(at) = self.audio_thread.take() {
            at.running.store(false, Ordering::Relaxed);
            let _ = at.thread_handle.join();
        }
        if let Ok(mut state) = self.audio_engine_state.lock() {
            *state = AudioEngineState::default();
        }
    }

    pub fn is_running(&self) -> bool {
        self.audio_thread.is_some()
    }

    pub fn scope_capture(&self) -> Option<Arc<ScopeCapture>> {
        self.audio_thread
            .as_ref()
            .and_then(|at| at.scope.lock().ok())
            .and_then(|guard| guard.clone())
    }

    fn refresh_devices(&mut self) {
        self.output_devices = doux_audio::list_output_devices();
        self.input_devices = doux_audio::list_input_devices();
    }

    pub fn show(&mut self, ctx: &egui::Context, server_resources: Option<&ServerResources>) {
        let mut open = self.open;
        egui::Window::new("Audio")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                self.show_config(ui);
                ui.separator();

                let server_running = server_resources.is_some();
                let audio_running = self.audio_thread.is_some();

                if server_running && !audio_running {
                    if ui.button("Start Audio").clicked() {
                        self.start(server_resources.unwrap());
                    }
                } else if audio_running {
                    ui.horizontal(|ui| {
                        if ui.button("Stop").clicked() {
                            self.stop();
                        }
                        if ui.button("Restart").clicked()
                            && let Some(ref at) = self.audio_thread
                        {
                            let (resp_tx, _) = crossbeam_channel::bounded(1);
                            let _ = at.restart_tx.send(AudioRestartRequest {
                                config: self.config(),
                                response_tx: resp_tx,
                            });
                        }
                    });
                } else {
                    ui.colored_label(egui::Color32::GRAY, "Server not running");
                }

                let state = self
                    .audio_engine_state
                    .lock()
                    .map(|s| s.clone())
                    .unwrap_or_default();

                if state.running || state.error.is_some() {
                    ui.separator();
                    self.show_status(ui, &state);
                }
            });
        self.open = open;
    }

    fn show_config(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("audio_config")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
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

                ui.label("Channels");
                ui.add(egui::DragValue::new(&mut self.channels).range(1..=64));
                ui.end_row();

                ui.label("Voices");
                ui.add(egui::DragValue::new(&mut self.max_voices).range(1..=128));
                ui.end_row();

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
    }

    fn show_status(&self, ui: &mut egui::Ui, state: &AudioEngineState) {
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
        }
    }
}

impl Drop for AudioPanel {
    fn drop(&mut self) {
        self.stop();
    }
}
