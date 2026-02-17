use crate::client_bridge::ClientBridge;
use crate::widgets::{COLOR_MUTED, COLOR_OK};
use eframe::egui;
use sova_core::protocol::{DeviceInfo, DeviceKind};
use sova_server::ClientMessage;

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

    pub fn show(&mut self, ctx: &egui::Context, bridge: &ClientBridge) {
        let mut open = self.open;
        egui::Window::new("Devices")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(480.0)
            .vscroll(true)
            .show(ctx, |ui| {
                if !bridge.is_connected() {
                    ui.colored_label(egui::Color32::GRAY, "Not connected");
                    return;
                }

                let devices = bridge.devices();
                self.show_device_table(ui, devices, bridge);

                ui.add_space(8.0);
                self.show_creation_controls(ui, bridge);
            });
        self.open = open;
    }

    fn show_device_table(
        &mut self,
        ui: &mut egui::Ui,
        devices: &[DeviceInfo],
        bridge: &ClientBridge,
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
                let ctx = ui.ctx().clone();
                let hint = |r: &egui::Response, text: &'static str| {
                    if r.hovered() { crate::widgets::hint::set(&ctx, text); }
                };

                hint(&ui.strong("Type"), "Device protocol (MIDI, OSC, Audio, Log)");
                hint(&ui.strong("Slot"), "Routing slot (1-16) — click to reassign");
                hint(&ui.strong("Status"), "Connection state of the device");
                hint(&ui.strong("Name"), "Device identifier");
                hint(&ui.strong("Address"), "Network or system address");
                hint(&ui.strong("Action"), "Connect, disconnect, or remove the device");
                ui.end_row();

                for dev in devices {
                    self.show_device_row(ui, dev, bridge);
                    ui.end_row();
                }
            });
    }

    fn show_device_row(
        &mut self,
        ui: &mut egui::Ui,
        dev: &DeviceInfo,
        bridge: &ClientBridge,
    ) {
        let kind_label = match dev.kind {
            DeviceKind::Midi | DeviceKind::VirtualMidi => "MIDI",
            DeviceKind::Osc => "OSC",
            DeviceKind::AudioEngine => "AUDIO",
            DeviceKind::Log => "LOG",
            _ => "OTHER",
        };
        ui.label(kind_label);

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
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), "Enter slot number (1-16), empty to unassign");
            }
            if resp.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.editing_slot = None;
                } else {
                    self.commit_slot_edit(dev, bridge);
                }
            }
            resp.request_focus();
        } else {
            let slot_text = dev
                .slot_id
                .map(|s| s.to_string())
                .unwrap_or_else(|| "—".into());
            let resp = ui.add(
                egui::Label::new(&slot_text).sense(egui::Sense::click()),
            );
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), "Click to assign a routing slot");
            }
            if resp.clicked() {
                self.editing_slot = Some(dev.name.clone());
                self.slot_edit_value = dev
                    .slot_id
                    .map(|s| s.to_string())
                    .unwrap_or_default();
            }
        }

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

        ui.label(&dev.name);
        ui.label(dev.address.as_deref().unwrap_or(""));

        match dev.kind {
            DeviceKind::Midi | DeviceKind::VirtualMidi => {
                if dev.is_connected {
                    let r = ui.button("Disconnect");
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), "Close MIDI connection to this device");
                    }
                    if r.clicked() {
                        bridge.send(ClientMessage::DisconnectMidiDeviceByName(
                            dev.name.clone(),
                        ));
                    }
                } else {
                    let r = ui.button("Connect");
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), "Open MIDI connection to this device");
                    }
                    if r.clicked() {
                        bridge.send(ClientMessage::ConnectMidiDeviceByName(dev.name.clone()));
                    }
                }
            }
            DeviceKind::Osc => {
                let r = ui.button("Remove");
                if r.hovered() {
                    crate::widgets::hint::set(ui.ctx(), "Remove this OSC device");
                }
                if r.clicked() {
                    bridge.send(ClientMessage::RemoveOscDevice(dev.name.clone()));
                }
            }
            _ => {
                ui.label("");
            }
        }
    }

    fn commit_slot_edit(&mut self, dev: &DeviceInfo, bridge: &ClientBridge) {
        let val = self.slot_edit_value.trim();

        if val.is_empty() {
            if let Some(slot) = dev.slot_id {
                bridge.send(ClientMessage::UnassignDeviceFromSlot(slot));
            }
        } else if let Ok(slot) = val.parse::<usize>()
            && (1..=16).contains(&slot)
        {
            if let Some(old) = dev.slot_id
                && old != slot
            {
                bridge.send(ClientMessage::UnassignDeviceFromSlot(old));
            }
            bridge.send(ClientMessage::AssignDeviceToSlot(slot, dev.name.clone()));
        }

        self.editing_slot = None;
    }

    fn show_creation_controls(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if self.creating_midi {
            self.show_midi_creation(ui, bridge);
        } else if self.creating_osc {
            self.show_osc_creation(ui, bridge);
        } else {
            ui.horizontal(|ui| {
                let r = ui.button("+ Virtual MIDI");
                if r.hovered() {
                    crate::widgets::hint::set(ui.ctx(), "Create a new virtual MIDI output port");
                }
                if r.clicked() {
                    self.creating_midi = true;
                    self.new_midi_name.clear();
                }
                let r = ui.button("+ OSC Output");
                if r.hovered() {
                    crate::widgets::hint::set(ui.ctx(), "Create a new OSC output device");
                }
                if r.clicked() {
                    self.creating_osc = true;
                    self.osc_step = 0;
                    self.osc_name.clear();
                    self.osc_ip = "127.0.0.1".into();
                    self.osc_port = "9000".into();
                }
            });
        }
    }

    fn show_midi_creation(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            let resp = ui.text_edit_singleline(&mut self.new_midi_name);
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), "Name for the virtual MIDI port");
            }
            if ui.button("Create").clicked()
                || (resp.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                let name = self.new_midi_name.trim();
                if !name.is_empty() {
                    bridge.send(ClientMessage::CreateVirtualMidiOutput(name.to_owned()));
                }
                self.creating_midi = false;
            }
            if ui.button("Cancel").clicked() {
                self.creating_midi = false;
            }
        });
    }

    fn show_osc_creation(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        ui.horizontal(|ui| {
            match self.osc_step {
                0 => {
                    ui.label("Name:");
                    let r = ui.text_edit_singleline(&mut self.osc_name);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), "Identifier for this OSC device");
                    }
                }
                1 => {
                    ui.label("IP:");
                    let r = ui.text_edit_singleline(&mut self.osc_ip);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), "Target IP address for OSC messages");
                    }
                }
                _ => {
                    ui.label("Port:");
                    let r = ui.text_edit_singleline(&mut self.osc_port);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), "Target UDP port for OSC messages");
                    }
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
                        bridge.send(ClientMessage::CreateOscDevice(
                            name.to_owned(),
                            ip.to_owned(),
                            port,
                        ));
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
