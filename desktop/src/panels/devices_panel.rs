use crate::client_bridge::ClientBridge;
use crate::theme::{COLOR_MUTED, COLOR_OK};
use eframe::egui;
use sova_core::protocol::{DeviceInfo, DeviceKind};
use sova_server::ClientMessage;

#[derive(Copy, Clone, PartialEq, Eq)]
enum OscStep {
    Name,
    Ip,
    Port,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum OscInputStep {
    Name,
    Port,
}

enum Creation {
    None,
    Midi {
        name: String,
    },
    Osc {
        step: OscStep,
        name: String,
        ip: String,
        port: String,
    },
    OscInput {
        step: OscInputStep,
        name: String,
        port: String,
    },
}

enum EditingField {
    None,
    Slot { device: String, buf: String },
    Latency { device: String, buf: String },
}

pub struct DevicesPanel {
    pub open: bool,
    creation: Creation,
    editing: EditingField,
    last_device_poll: Option<std::time::Instant>,
}

impl DevicesPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            creation: Creation::None,
            editing: EditingField::None,
            last_device_poll: None,
        }
    }

    fn poll_devices_if_needed(&mut self, bridge: &ClientBridge) {
        if bridge.has_feedback() || !bridge.is_connected() {
            return;
        }
        const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);
        let should_poll = match self.last_device_poll {
            None => true,
            Some(last) => last.elapsed() >= POLL_INTERVAL,
        };
        if should_poll {
            bridge.send(ClientMessage::RequestDeviceList);
            self.last_device_poll = Some(std::time::Instant::now());
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, bridge: &ClientBridge) {
        if self.open {
            self.poll_devices_if_needed(bridge);
        } else {
            self.last_device_poll = None;
        }
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
        self.poll_devices_if_needed(bridge);
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

        if let EditingField::Slot { device, buf } = &mut self.editing
            && device == &dev.name
        {
            let resp = ui.add(
                egui::TextEdit::singleline(buf)
                    .desired_width(30.0)
                    .hint_text("\u{2014}"),
            );
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.slot_edit"));
            }
            if resp.lost_focus() {
                if crate::widgets::consume_key_on_lost_focus(ui, &resp, egui::Key::Escape) {
                    self.editing = EditingField::None;
                } else {
                    crate::widgets::consume_key_on_lost_focus(ui, &resp, egui::Key::Enter);
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
                self.editing = EditingField::Slot {
                    device: dev.name.clone(),
                    buf: dev.slot_id.map(|s| s.to_string()).unwrap_or_default(),
                };
            }
        }

        if let EditingField::Latency { device, buf } = &mut self.editing
            && device == &dev.name
        {
            let resp = ui.add(
                egui::TextEdit::singleline(buf)
                    .desired_width(30.0)
                    .hint_text("\u{2014}"),
            );
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.latency"));
            }
            if resp.lost_focus() {
                if crate::widgets::consume_key_on_lost_focus(ui, &resp, egui::Key::Escape) {
                    self.editing = EditingField::None;
                } else {
                    crate::widgets::consume_key_on_lost_focus(ui, &resp, egui::Key::Enter);
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
                self.editing = EditingField::Latency {
                    device: dev.name.clone(),
                    buf: dev.latency.to_string(),
                };
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
                    let r = ui.button(crate::icons::button_text(
                        ui,
                        crate::icons::DISCONNECT,
                        t!("common.disconnect"),
                    ));
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.disconnect_midi"));
                    }
                    if r.clicked() {
                        bridge.disconnect_midi(&dev.name);
                    }
                } else {
                    let r = ui.button(crate::icons::button_text(
                        ui,
                        crate::icons::CONNECT,
                        t!("common.connect"),
                    ));
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.connect_midi"));
                    }
                    if r.clicked() {
                        bridge.connect_midi(&dev.name);
                    }
                }
            }
            DeviceKind::Osc => {
                let r = ui.button(crate::icons::button_text(
                    ui,
                    crate::icons::TRASH,
                    t!("common.remove"),
                ));
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
        let EditingField::Slot { buf, .. } = &self.editing else {
            return;
        };
        let val = buf.trim();

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

        self.editing = EditingField::None;
    }

    fn commit_latency_edit(&mut self, dev: &DeviceInfo, bridge: &ClientBridge) {
        let EditingField::Latency { buf, .. } = &self.editing else {
            return;
        };
        let val = buf.trim();

        if val.is_empty() {
            bridge.set_latency(&dev.name, 0.2);
        } else if let Ok(latency) = val.parse::<f64>() {
            bridge.set_latency(&dev.name, latency);
        }

        self.editing = EditingField::None;
    }

    fn show_creation_controls(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        match &mut self.creation {
            Creation::None => self.show_creation_buttons(ui),
            Creation::Midi { .. } => self.show_midi_creation(ui, bridge),
            Creation::Osc { .. } => self.show_osc_creation(ui, bridge),
            Creation::OscInput { .. } => self.show_osc_input_creation(ui, bridge),
        }
    }

    fn show_creation_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let r = ui.button(crate::icons::button_text(
                ui,
                crate::icons::ADD,
                t!("devices.add_virtual_midi"),
            ));
            if r.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.new_midi"));
            }
            if r.clicked() {
                self.creation = Creation::Midi {
                    name: String::new(),
                };
            }
            let r = ui.button(crate::icons::button_text(
                ui,
                crate::icons::ADD,
                t!("devices.add_osc"),
            ));
            if r.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.new_osc"));
            }
            if r.clicked() {
                self.creation = Creation::Osc {
                    step: OscStep::Name,
                    name: String::new(),
                    ip: "127.0.0.1".into(),
                    port: "9000".into(),
                };
            }
            let r = ui.button(crate::icons::button_text(
                ui,
                crate::icons::ADD,
                t!("devices.add_osc_input"),
            ));
            if r.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.new_osc"));
            }
            if r.clicked() {
                self.creation = Creation::OscInput {
                    step: OscInputStep::Name,
                    name: String::new(),
                    port: "9000".into(),
                };
            }
        });
    }

    fn show_midi_creation(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let Creation::Midi { name } = &mut self.creation else {
            return;
        };
        let mut commit = false;
        let mut cancel = false;
        ui.horizontal(|ui| {
            ui.label(t!("devices.label.name"));
            let resp = ui.text_edit_singleline(name);
            if resp.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("devices.hint.midi_name"));
            }
            if ui.button(t!("common.create")).clicked()
                || crate::widgets::consume_key_on_lost_focus(ui, &resp, egui::Key::Enter)
            {
                commit = true;
            }
            if ui.button(t!("common.cancel")).clicked() {
                cancel = true;
            }
        });
        if commit {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                bridge.create_virtual_midi(trimmed);
            }
            self.creation = Creation::None;
        } else if cancel {
            self.creation = Creation::None;
        }
    }

    fn show_osc_creation(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let Creation::Osc {
            step,
            name,
            ip,
            port,
        } = &mut self.creation
        else {
            return;
        };
        let mut advance = false;
        let mut cancel = false;
        ui.horizontal(|ui| {
            match step {
                OscStep::Name => {
                    ui.label(t!("devices.label.name"));
                    let r = ui.text_edit_singleline(name);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.osc_name"));
                    }
                }
                OscStep::Ip => {
                    ui.label(t!("devices.label.ip"));
                    let r = ui.text_edit_singleline(ip);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.osc_ip"));
                    }
                }
                OscStep::Port => {
                    ui.label(t!("devices.label.port"));
                    let r = ui.text_edit_singleline(port);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.osc_port"));
                    }
                }
            }
            if ui.button(t!("common.next")).clicked() {
                advance = true;
            }
            if ui.button(t!("common.cancel")).clicked() {
                cancel = true;
            }
        });
        if cancel {
            self.creation = Creation::None;
            return;
        }
        if !advance {
            return;
        }
        match step {
            OscStep::Name => *step = OscStep::Ip,
            OscStep::Ip => *step = OscStep::Port,
            OscStep::Port => {
                let trimmed_name = name.trim();
                let trimmed_ip = ip.trim();
                if let Ok(port_num) = port.trim().parse::<u16>()
                    && !trimmed_name.is_empty()
                    && !trimmed_ip.is_empty()
                {
                    bridge.create_osc(trimmed_name, trimmed_ip, port_num);
                }
                self.creation = Creation::None;
            }
        }
    }

    fn show_osc_input_creation(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let Creation::OscInput { step, name, port } = &mut self.creation else {
            return;
        };
        let mut advance = false;
        let mut cancel = false;
        ui.horizontal(|ui| {
            match step {
                OscInputStep::Name => {
                    ui.label(t!("devices.label.name"));
                    let r = ui.text_edit_singleline(name);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.osc_name"));
                    }
                }
                OscInputStep::Port => {
                    ui.label(t!("devices.label.port"));
                    let r = ui.text_edit_singleline(port);
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("devices.hint.osc_port"));
                    }
                }
            }
            if ui.button(t!("common.next")).clicked() {
                advance = true;
            }
            if ui.button(t!("common.cancel")).clicked() {
                cancel = true;
            }
        });
        if cancel {
            self.creation = Creation::None;
            return;
        }
        if !advance {
            return;
        }
        match step {
            OscInputStep::Name => *step = OscInputStep::Port,
            OscInputStep::Port => {
                let trimmed_name = name.trim();
                if let Ok(port_num) = port.trim().parse::<u16>()
                    && !trimmed_name.is_empty()
                {
                    bridge.create_osc_input(trimmed_name, port_num);
                }
                self.creation = Creation::None;
            }
        }
    }
}
