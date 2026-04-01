use crate::client_bridge::ClientBridge;
use crate::widgets::{COLOR_MUTED, COLOR_OK};
use eframe::egui;
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
    editing_latency: Option<String>,
    slot_edit_value: String,
    latency_edit_value: String,
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
            editing_latency: None,
            slot_edit_value: String::new(),
            latency_edit_value: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, bridge: &ClientBridge) {
        let mut open = self.open;
        egui::Window::new(t!("devices.title"))
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(480.0)
            .vscroll(true)
            .show(ctx, |ui| {
                self.show_inside(ui, bridge);
            });
        self.open = open;
    }

    pub fn show_inside(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if !bridge.is_connected() {
            ui.colored_label(egui::Color32::GRAY, t!("common.not_connected"));
            return;
        }

        let devices = bridge.devices();
        self.show_device_table(ui, devices, bridge);

        ui.add_space(8.0);
        self.show_creation_controls(ui, bridge);
    }

    fn show_device_table(
        &mut self,
        ui: &mut egui::Ui,
        devices: &[DeviceInfo],
        bridge: &ClientBridge,
    ) {
        if devices.is_empty() {
            ui.label(t!("devices.no_devices"));
            return;
        }

        egui::Grid::new("devices_grid")
            .num_columns(6)
            .spacing([12.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                use crate::widgets::hint;

                let r = ui.strong(t!("devices.type"));
                hint::on_hover(ui.ctx(), &r, t!("devices.hint.type"));
                let r = ui.strong(t!("devices.slot"));
                hint::on_hover(ui.ctx(), &r, t!("devices.hint.latency"));
                let r = ui.strong(t!("devices.latency"));
                hint::on_hover(ui.ctx(), &r, t!("devices.hint.slot"));
                let r = ui.strong(t!("devices.status"));
                hint::on_hover(ui.ctx(), &r, t!("devices.hint.status"));
                let r = ui.strong(t!("devices.name"));
                hint::on_hover(ui.ctx(), &r, t!("devices.hint.name"));
                let r = ui.strong(t!("devices.address"));
                hint::on_hover(ui.ctx(), &r, t!("devices.hint.address"));
                let r = ui.strong(t!("devices.action"));
                hint::on_hover(ui.ctx(), &r, t!("devices.hint.action"));
                ui.end_row();

                for dev in devices {
                    self.show_device_row(ui, dev, bridge);
                    ui.end_row();
                }
            });
    }

    fn show_device_row(&mut self, ui: &mut egui::Ui, dev: &DeviceInfo, bridge: &ClientBridge) {
        let kind_label = match dev.kind {
            DeviceKind::Midi | DeviceKind::VirtualMidi => "MIDI",
            DeviceKind::Osc => "OSC",
            DeviceKind::AudioEngine => "AUDIO",
            DeviceKind::Log => "LOG",
            _ => "OTHER",
        };
        ui.label(kind_label);

        let is_editing_slot = self.editing_slot.as_ref().is_some_and(|n| n == &dev.name);
        if is_editing_slot {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.slot_edit_value)
                    .desired_width(30.0)
                    .hint_text("\u{2014}"),
            );
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.slot_edit"));
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
                .unwrap_or_else(|| "\u{2014}".into());
            let resp = ui.add(egui::Label::new(&slot_text).sense(egui::Sense::click()));
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.slot_click"));
            }
            if resp.clicked() {
                self.editing_slot = Some(dev.name.clone());
                self.slot_edit_value = dev.slot_id.map(|s| s.to_string()).unwrap_or_default();
            }
        }

        let is_editing_latency = self
            .editing_latency
            .as_ref()
            .is_some_and(|n| n == &dev.name);
        if is_editing_latency {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.latency_edit_value)
                    .desired_width(30.0)
                    .hint_text("\u{2014}"),
            );
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.latency"));
            }
            if resp.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.editing_latency = None;
                } else {
                    self.commit_latency_edit(dev, bridge);
                }
            }
            resp.request_focus();
        } else {
            let latency_text = dev.latency.to_string();
            let resp = ui.add(egui::Label::new(&latency_text).sense(egui::Sense::click()));
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.latency"));
            }
            if resp.clicked() {
                self.editing_latency = Some(dev.name.clone());
                self.latency_edit_value = dev.latency.to_string();
            }
        }

        let (color, status_text) = if dev.is_connected {
            (COLOR_OK, t!("devices.connected"))
        } else {
            match dev.kind {
                DeviceKind::AudioEngine => (COLOR_OK, t!("devices.active")),
                _ => (COLOR_MUTED, t!("devices.available")),
            }
        };
        ui.horizontal(|ui| {
            ui.colored_label(color, "\u{25cf}");
            ui.label(status_text);
        });

        ui.label(&dev.name);
        ui.label(dev.address.as_deref().unwrap_or(""));

        match dev.kind {
            DeviceKind::Midi | DeviceKind::VirtualMidi => {
                if dev.is_connected {
                    let r = ui.button(t!("common.disconnect"));
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.disconnect_midi"));
                    }
                    if r.clicked() {
                        bridge.disconnect_midi(&dev.name);
                    }
                } else {
                    let r = ui.button(t!("common.connect"));
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.connect_midi"));
                    }
                    if r.clicked() {
                        bridge.connect_midi(&dev.name);
                    }
                }
            }
            DeviceKind::Osc => {
                let r = ui.button(t!("common.remove"));
                if r.hovered() {
                    crate::widgets::hint::set(ui.ctx(), t!("devices.hint.remove_osc"));
                }
                if r.clicked() {
                    bridge.remove_osc(&dev.name);
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
                bridge.unassign_slot(slot);
            }
        } else if let Ok(slot) = val.parse::<usize>()
            && (1..=16).contains(&slot)
        {
            if let Some(old) = dev.slot_id
                && old != slot
            {
                bridge.unassign_slot(old);
            }
            bridge.assign_slot(slot, &dev.name);
        }

        self.editing_slot = None;
    }

    fn commit_latency_edit(&mut self, dev: &DeviceInfo, bridge: &ClientBridge) {
        let val = self.latency_edit_value.trim();

        if val.is_empty() {
            bridge.set_latency(&dev.name, 0.2);
        } else if let Ok(latency) = val.parse::<f64>() {
            bridge.set_latency(&dev.name, latency);
        }

        self.editing_latency = None;
    }

    fn show_creation_controls(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if self.creating_midi {
            self.show_midi_creation(ui, bridge);
        } else if self.creating_osc {
            self.show_osc_creation(ui, bridge);
        } else {
            ui.horizontal(|ui| {
                let r = ui.button(t!("devices.add_virtual_midi"));
                if r.hovered() {
                    crate::widgets::hint::set(ui.ctx(), t!("devices.hint.new_midi"));
                }
                if r.clicked() {
                    self.creating_midi = true;
                    self.new_midi_name.clear();
                }
                let r = ui.button(t!("devices.add_osc"));
                if r.hovered() {
                    crate::widgets::hint::set(ui.ctx(), t!("devices.hint.new_osc"));
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
            ui.label(t!("devices.label.name"));
            let resp = ui.text_edit_singleline(&mut self.new_midi_name);
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.midi_name"));
            }
            if ui.button(t!("common.create")).clicked()
                || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                let name = self.new_midi_name.trim();
                if !name.is_empty() {
                    bridge.create_virtual_midi(name);
                }
                self.creating_midi = false;
            }
            if ui.button(t!("common.cancel")).clicked() {
                self.creating_midi = false;
            }
        });
    }

    fn show_osc_creation(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        ui.horizontal(|ui| {
            match self.osc_step {
                0 => {
                    ui.label(t!("devices.label.name"));
                    let r = ui.text_edit_singleline(&mut self.osc_name);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.osc_name"));
                    }
                }
                1 => {
                    ui.label(t!("devices.label.ip"));
                    let r = ui.text_edit_singleline(&mut self.osc_ip);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.osc_ip"));
                    }
                }
                _ => {
                    ui.label(t!("devices.label.port"));
                    let r = ui.text_edit_singleline(&mut self.osc_port);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.osc_port"));
                    }
                }
            }

            let advance = ui.button(t!("common.next")).clicked();

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
                        bridge.create_osc(name, ip, port);
                    }
                    self.creating_osc = false;
                }
            }
            if ui.button(t!("common.cancel")).clicked() {
                self.creating_osc = false;
            }
        });
    }
}
