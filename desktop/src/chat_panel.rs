use eframe::egui;
use egui::text::TextWrapping;

use crate::client_bridge::ClientBridge;
use crate::settings::AppearanceSettings;
use crate::widgets::{self, COLOR_MUTED, username_color};

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
        // Input bar pinned to bottom — claims fixed space
        egui::TopBottomPanel::bottom(ui.id().with("chat_input_bar"))
            .show_separator_line(true)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if show_popout {
                        let r = ui
                            .button(crate::icons::POPOUT)
                            .on_hover_text(t!("common.pop_out"));
                        if r.hovered() {
                            crate::widgets::hint::set(ui.ctx(), t!("chat.hint.detach"));
                        }
                        if r.clicked() {
                            self.detached = true;
                        }
                    }

                    // Right-to-left: send button sizes itself naturally,
                    // TextEdit fills the exact remainder. No measurement,
                    // no fractional-pixel overflow, no Resize ratchet trigger.
                    let input_id = ui.id().with("chat_input");
                    let (resp, send_btn) = ui
                        .with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let send_btn = ui.button(t!("common.send"));
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.input)
                                    .id(input_id)
                                    .hint_text(t!("chat.type_message"))
                                    .desired_width(ui.available_width()),
                            );
                            (resp, send_btn)
                        })
                        .inner;

                    if resp.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("chat.hint.input"));
                    }
                    if send_btn.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("chat.hint.send"));
                    }

                    let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if (enter || send_btn.clicked()) && !self.input.trim().is_empty() {
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
            });

        // Messages fill remaining space via CentralPanel — eliminates the
        // available_height() circular dependency with the Resize ratchet.
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        let messages = bridge.chat_messages();
                        if messages.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    egui::RichText::new(t!("chat.no_messages")).color(COLOR_MUTED),
                                );
                            });
                        } else {
                            for msg in messages {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(&msg.time).small().color(COLOR_MUTED),
                                    );
                                    if msg.system {
                                        ui.label(
                                            egui::RichText::new(&msg.message)
                                                .italics()
                                                .color(COLOR_MUTED),
                                        );
                                    } else {
                                        ui.label(
                                            egui::RichText::new(&msg.user)
                                                .strong()
                                                .color(username_color(&msg.user)),
                                        );
                                        let mut job = egui::text::LayoutJob {
                                            wrap: TextWrapping {
                                                max_width: ui.available_width(),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        };
                                        widgets::append_inline_markdown(
                                            &mut job,
                                            &msg.message,
                                            &egui::TextFormat {
                                                font_id: egui::FontId::proportional(14.0),
                                                color: ui.visuals().text_color(),
                                                ..Default::default()
                                            },
                                            ui.visuals().strong_text_color(),
                                            ui.visuals().code_bg_color,
                                        );
                                        ui.label(job);
                                    }
                                });
                            }
                        }

                        if self.scroll_to_bottom {
                            ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                            self.scroll_to_bottom = false;
                        }
                    });
            });
    }

    fn show_embedded(&mut self, ctx: &egui::Context, bridge: &mut ClientBridge) {
        let mut open = self.open;
        egui::Window::new(t!("chat.title"))
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
            &t!("chat.detached_title"),
            [360.0, 400.0],
            appearance,
            |ui| self.chat_content(ui, bridge, false),
        );
        self.open = open;
        self.detached = detached;
    }
}
