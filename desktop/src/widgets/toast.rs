use std::collections::VecDeque;
use std::time::Instant;

use eframe::egui;

use crate::client_bridge::ChatMessage;

const MAX_VISIBLE: usize = 8;
const FADE_SECONDS: f32 = 2.0;
const MAX_WIDTH: f32 = 400.0;

#[allow(dead_code)]
pub enum ToastLevel {
    Error,
    Warning,
    Info,
    Success,
    Chat { user: String },
}

struct Toast {
    level: ToastLevel,
    message: String,
    created: Instant,
    duration: f32,
}

pub struct ToastStack {
    toasts: VecDeque<Toast>,
    chat_count: usize,
}

impl ToastStack {
    pub fn new() -> Self {
        Self {
            toasts: VecDeque::new(),
            chat_count: 0,
        }
    }

    pub fn push(&mut self, level: ToastLevel, message: impl Into<String>) {
        let duration = match &level {
            ToastLevel::Error => 8.0,
            ToastLevel::Warning => 6.0,
            ToastLevel::Info => 5.0,
            ToastLevel::Success => 5.0,
            ToastLevel::Chat { .. } => 6.0,
        };
        self.toasts.push_back(Toast {
            level,
            message: message.into(),
            created: Instant::now(),
            duration,
        });
        if self.toasts.len() > MAX_VISIBLE {
            self.toasts.pop_front();
        }
    }

    pub fn poll_chat(&mut self, chat_messages: &VecDeque<ChatMessage>) {
        for msg in chat_messages.iter().skip(self.chat_count) {
            if !msg.system {
                self.push(
                    ToastLevel::Chat {
                        user: msg.user.clone(),
                    },
                    msg.message.clone(),
                );
            }
        }
        self.chat_count = chat_messages.len();
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.toasts
            .retain(|t| t.created.elapsed().as_secs_f32() < t.duration);

        if self.toasts.is_empty() {
            return;
        }

        ctx.request_repaint();

        egui::Area::new(egui::Id::new("toast_stack"))
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -32.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                ui.set_max_width(MAX_WIDTH);
                ui.spacing_mut().item_spacing.y = 4.0;

                for toast in &self.toasts {
                    let elapsed = toast.created.elapsed().as_secs_f32();
                    let fade_start = toast.duration - FADE_SECONDS;
                    let alpha = if elapsed < fade_start {
                        1.0
                    } else {
                        ((toast.duration - elapsed) / FADE_SECONDS).clamp(0.0, 1.0)
                    };
                    let alpha_byte = (alpha * 255.0) as u8;

                    let (bg_r, bg_g, bg_b) = match &toast.level {
                        ToastLevel::Error => (40, 10, 10),
                        ToastLevel::Warning => (40, 30, 10),
                        ToastLevel::Info => (20, 20, 30),
                        ToastLevel::Success => (10, 30, 10),
                        ToastLevel::Chat { .. } => (15, 15, 20),
                    };
                    let bg = egui::Color32::from_rgba_unmultiplied(bg_r, bg_g, bg_b, alpha_byte);

                    let accent = match &toast.level {
                        ToastLevel::Error => super::COLOR_ERROR,
                        ToastLevel::Success => super::COLOR_OK,
                        ToastLevel::Warning => egui::Color32::from_rgb(200, 150, 50),
                        ToastLevel::Info => super::COLOR_MUTED,
                        ToastLevel::Chat { user } => super::username_color(user),
                    }
                    .gamma_multiply(alpha);

                    let frame = egui::Frame::NONE
                        .fill(bg)
                        .inner_margin(egui::Margin {
                            left: 10,
                            right: 8,
                            top: 6,
                            bottom: 6,
                        });

                    let resp = frame.show(ui, |ui| {
                        ui.set_max_width(MAX_WIDTH - 20.0);
                        match &toast.level {
                            ToastLevel::Chat { user } => {
                                let mut job = egui::text::LayoutJob {
                                    wrap: egui::text::TextWrapping {
                                        max_width: MAX_WIDTH - 20.0,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                };
                                let font = egui::FontId::proportional(13.0);
                                let name_color =
                                    super::username_color(user).gamma_multiply(alpha);
                                let text_color =
                                    egui::Color32::from_white_alpha(alpha_byte);
                                job.append(
                                    &format!("{user}: "),
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font.clone(),
                                        color: name_color,
                                        ..Default::default()
                                    },
                                );
                                job.append(
                                    &toast.message,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font,
                                        color: text_color,
                                        ..Default::default()
                                    },
                                );
                                ui.label(job);
                            }
                            _ => {
                                let text_color = match &toast.level {
                                    ToastLevel::Error => {
                                        egui::Color32::from_rgba_unmultiplied(
                                            255, 100, 100, alpha_byte,
                                        )
                                    }
                                    ToastLevel::Warning => {
                                        egui::Color32::from_rgba_unmultiplied(
                                            255, 200, 100, alpha_byte,
                                        )
                                    }
                                    _ => egui::Color32::from_white_alpha(alpha_byte),
                                };
                                ui.label(
                                    egui::RichText::new(&toast.message)
                                        .color(text_color)
                                        .small(),
                                );
                            }
                        }
                    });

                    // Left accent border
                    let rect = resp.response.rect;
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            rect.left_top(),
                            egui::vec2(2.0, rect.height()),
                        ),
                        egui::CornerRadius::ZERO,
                        accent,
                    );
                }
            });
    }
}
