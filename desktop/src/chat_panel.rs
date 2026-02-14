use eframe::egui;

use crate::client_bridge::ClientBridge;
use crate::settings::AppearanceSettings;
use crate::widgets::{self, COLOR_MUTED};

pub struct ChatPanel {
    pub open: bool,
    pub detached: bool,
    input: String,
    prev_count: usize,
    scroll_to_bottom: bool,
}

impl ChatPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            detached: false,
            input: String::new(),
            prev_count: 0,
            scroll_to_bottom: true,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        bridge: &mut ClientBridge,
        appearance: &AppearanceSettings,
    ) {
        if !self.open {
            return;
        }

        let count = bridge.chat_messages().len();
        if count != self.prev_count {
            self.prev_count = count;
            self.scroll_to_bottom = true;
        }

        if self.detached {
            self.show_detached(ctx, bridge, appearance);
        } else {
            self.show_embedded(ctx, bridge);
        }
    }

    fn chat_content(&mut self, ui: &mut egui::Ui, bridge: &mut ClientBridge, show_popout: bool) {
        let input_height = 28.0;
        let separator_height = 6.0;
        let scroll_height = (ui.available_height() - input_height - separator_height).max(40.0);

        ui.allocate_ui(egui::vec2(ui.available_width(), scroll_height), |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let messages = bridge.chat_messages();
                    if messages.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                egui::RichText::new("No messages yet").color(COLOR_MUTED),
                            );
                        });
                    } else {
                        for msg in messages {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&msg.time)
                                        .small()
                                        .color(COLOR_MUTED),
                                );
                                ui.label(
                                    egui::RichText::new(&msg.user)
                                        .strong()
                                        .color(username_color(&msg.user)),
                                );
                                ui.label(&msg.message);
                            });
                        }
                    }

                    if self.scroll_to_bottom {
                        ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                        self.scroll_to_bottom = false;
                    }
                });
        });

        ui.separator();

        ui.horizontal(|ui| {
            if show_popout && ui.button(crate::icons::POPOUT).on_hover_text("Pop out").clicked() {
                self.detached = true;
            }
            let input_id = ui.id().with("chat_input");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .id(input_id)
                    .hint_text("Type a message...")
                    .desired_width(ui.available_width() - 50.0),
            );

            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let send_clicked = ui.button("Send").clicked();

            if (enter || send_clicked) && !self.input.trim().is_empty() {
                let text = self.input.trim().to_owned();
                bridge.send_chat(&text);
                if let Some(name) = bridge.confirmed_username() {
                    let name = name.to_owned();
                    bridge.push_chat(name, text);
                }
                self.input.clear();
                resp.request_focus();
            }
        });
    }

    fn show_embedded(&mut self, ctx: &egui::Context, bridge: &mut ClientBridge) {
        let mut open = self.open;
        egui::Window::new("Chat")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([360.0, 400.0])
            .show(ctx, |ui| {
                self.chat_content(ui, bridge, true);
            });
        self.open = open;
    }

    fn show_detached(
        &mut self,
        ctx: &egui::Context,
        bridge: &mut ClientBridge,
        appearance: &AppearanceSettings,
    ) {
        let mut open = self.open;
        let mut detached = self.detached;
        widgets::show_detached_viewport(
            ctx,
            &mut open,
            &mut detached,
            "chat_viewport",
            "Sova - Chat",
            [360.0, 400.0],
            appearance,
            |ui| self.chat_content(ui, bridge, false),
        );
        self.open = open;
        self.detached = detached;
    }
}

fn username_color(name: &str) -> egui::Color32 {
    let mut hash: u32 = 0;
    for b in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    let hue = (hash % 360) as f32;
    let (r, g, b) = hsl_to_rgb(hue, 0.65, 0.55);
    egui::Color32::from_rgb(r, g, b)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h as u32) / 60 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
