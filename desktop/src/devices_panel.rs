use crate::server_panel::ServerResources;
use crate::widgets::{COLOR_MUTED, COLOR_OK};
use eframe::egui;
use sova_core::device_map::DeviceMap;
use sova_core::protocol::{DeviceInfo, DeviceKind};

pub struct DevicesPanel {
    pub open: bool,
    creating_midi: bool,
    new_midi_name: String,
    creating_osc: bool,
    osc_step: u8,
    osc_name: String,
    osc_ip: String,
    osc_port: String,
    editing_slot: Option<String>,
    slot_edit_value: String,
}

impl DevicesPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            creating_midi: false,
            new_midi_name: String::new(),
            creating_osc: false,
            osc_step: 0,
            osc_name: String::new(),
            osc_ip: String::new(),
            osc_port: String::new(),
            editing_slot: None,
            slot_edit_value: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, server_resources: Option<&ServerResources>) {
        let mut open = self.open;
        egui::Window::new("Devices")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(480.0)
            .vscroll(true)
            .show(ctx, |ui| {
                let Some(res) = server_resources else {
                    ui.colored_label(egui::Color32::GRAY, "Server not running");
                    return;
                };

                let devices = res.devices.device_list();
                self.show_device_table(ui, &devices, &res.devices);

                ui.add_space(8.0);
                self.show_creation_controls(ui, &res.devices);
            });
        self.open = open;
    }

    fn show_device_table(
        &mut self,
        ui: &mut egui::Ui,
        devices: &[DeviceInfo],
        device_map: &DeviceMap,
    ) {
        if devices.is_empty() {
            ui.label("No devices");
            return;
        }

        egui::Grid::new("devices_grid")
            .num_columns(6)
            .spacing([12.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Type");
                ui.strong("Slot");
                ui.strong("Status");
                ui.strong("Name");
                ui.strong("Address");
                ui.strong("Action");
                ui.end_row();

                for dev in devices {
                    self.show_device_row(ui, dev, device_map);
                    ui.end_row();
                }
            });
    }

    fn show_device_row(
        &mut self,
        ui: &mut egui::Ui,
        dev: &DeviceInfo,
        device_map: &DeviceMap,
    ) {
        let kind_label = match dev.kind {
            DeviceKind::Midi | DeviceKind::VirtualMidi => "MIDI",
            DeviceKind::Osc => "OSC",
            DeviceKind::AudioEngine => "AUDIO",
            DeviceKind::Log => "LOG",
            _ => "OTHER",
        };
        ui.label(kind_label);

        // Slot cell — click to edit
        let is_editing = self
            .editing_slot
            .as_ref()
            .is_some_and(|n| n == &dev.name);

        if is_editing {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.slot_edit_value)
                    .desired_width(30.0)
                    .hint_text("—"),
            );
            if resp.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.editing_slot = None;
                } else {
                    self.commit_slot_edit(dev, device_map);
                }
            }
            // Auto-focus on first frame
            resp.request_focus();
        } else {
            let slot_text = dev
                .slot_id
                .map(|s| s.to_string())
                .unwrap_or_else(|| "—".into());
            let resp = ui.add(
                egui::Label::new(&slot_text).sense(egui::Sense::click()),
            );
            if resp.clicked() {
                self.editing_slot = Some(dev.name.clone());
                self.slot_edit_value = dev
                    .slot_id
                    .map(|s| s.to_string())
                    .unwrap_or_default();
            }
        }

        // Status
        let (color, status_text) = if dev.is_connected {
            (COLOR_OK, "Connected")
        } else {
            match dev.kind {
                DeviceKind::AudioEngine => (COLOR_OK, "Active"),
                _ => (COLOR_MUTED, "Available"),
            }
        };
        ui.horizontal(|ui| {
            ui.colored_label(color, "●");
            ui.label(status_text);
        });

        // Name
        ui.label(&dev.name);

        // Address
        ui.label(dev.address.as_deref().unwrap_or(""));

        // Action
        match dev.kind {
            DeviceKind::Midi | DeviceKind::VirtualMidi => {
                if dev.is_connected {
                    if ui.button("Disconnect").clicked() {
                        let _ = device_map.disconnect_midi_by_name(&dev.name);
                    }
                } else if ui.button("Connect").clicked() {
                    let _ = device_map.connect_midi_by_name(&dev.name);
                }
            }
            DeviceKind::Osc => {
                if ui.button("Remove").clicked() {
                    let _ = device_map.remove_output_device(&dev.name);
                }
            }
            _ => {
                ui.label("");
            }
        }
    }

    fn commit_slot_edit(&mut self, dev: &DeviceInfo, device_map: &DeviceMap) {
        let val = self.slot_edit_value.trim();

        if val.is_empty() {
            if let Some(slot) = dev.slot_id {
                let _ = device_map.unassign_slot(slot);
            }
        } else if let Ok(slot) = val.parse::<usize>()
            && (1..=16).contains(&slot)
        {
            if let Some(old) = dev.slot_id
                && old != slot
            {
                let _ = device_map.unassign_slot(old);
            }
            let _ = device_map.assign_slot(slot, &dev.name);
        }

        self.editing_slot = None;
    }

    fn show_creation_controls(&mut self, ui: &mut egui::Ui, device_map: &DeviceMap) {
        if self.creating_midi {
            self.show_midi_creation(ui, device_map);
        } else if self.creating_osc {
            self.show_osc_creation(ui, device_map);
        } else {
            ui.horizontal(|ui| {
                if ui.button("+ Virtual MIDI").clicked() {
                    self.creating_midi = true;
                    self.new_midi_name.clear();
                }
                if ui.button("+ OSC Output").clicked() {
                    self.creating_osc = true;
                    self.osc_step = 0;
                    self.osc_name.clear();
                    self.osc_ip = "127.0.0.1".into();
                    self.osc_port = "9000".into();
                }
            });
        }
    }

    fn show_midi_creation(&mut self, ui: &mut egui::Ui, device_map: &DeviceMap) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            let resp = ui.text_edit_singleline(&mut self.new_midi_name);
            if ui.button("Create").clicked()
                || (resp.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                let name = self.new_midi_name.trim();
                if !name.is_empty() {
                    let _ = device_map.create_virtual_midi_port(name);
                }
                self.creating_midi = false;
            }
            if ui.button("Cancel").clicked() {
                self.creating_midi = false;
            }
        });
    }

    fn show_osc_creation(&mut self, ui: &mut egui::Ui, device_map: &DeviceMap) {
        ui.horizontal(|ui| {
            match self.osc_step {
                0 => {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.osc_name);
                }
                1 => {
                    ui.label("IP:");
                    ui.text_edit_singleline(&mut self.osc_ip);
                }
                _ => {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.osc_port);
                }
            }

            let advance = ui.button("Next").clicked();

            if advance {
                if self.osc_step < 2 {
                    self.osc_step += 1;
                } else {
                    let name = self.osc_name.trim();
                    let ip = self.osc_ip.trim();
                    if let Ok(port) = self.osc_port.trim().parse::<u16>()
                        && !name.is_empty()
                        && !ip.is_empty()
                    {
                        let _ = device_map.create_osc_output_device(name, ip, port);
                    }
                    self.creating_osc = false;
                }
            }
            if ui.button("Cancel").clicked() {
                self.creating_osc = false;
            }
        });
    }
}
