use std::collections::VecDeque;
use std::time::Instant;

use eframe::egui;

use crate::client_bridge::ChatMessage;
use crate::widgets;

const MAX_VISIBLE: usize = 5;
const FADE_DURATION: f32 = 6.0;
const FADE_START: f32 = 4.0;

pub struct ChatOverlay {
    messages: VecDeque<(String, String, Instant)>,
    last_count: usize,
}

impl ChatOverlay {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            last_count: 0,
        }
    }

    pub fn poll(&mut self, chat_messages: &[ChatMessage]) {
        let now = Instant::now();

        for msg in chat_messages.iter().skip(self.last_count) {
            if !msg.system {
                self.messages
                    .push_back((msg.user.clone(), msg.message.clone(), now));
                if self.messages.len() > MAX_VISIBLE {
                    self.messages.pop_front();
                }
            }
        }
        self.last_count = chat_messages.len();

        while let Some(front) = self.messages.front() {
            if front.2.elapsed().as_secs_f32() > FADE_DURATION {
                self.messages.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn show(&self, ctx: &egui::Context) {
        if self.messages.is_empty() {
            return;
        }

        ctx.request_repaint();

        let screen = ctx.viewport_rect();
        let x = screen.right() - 360.0 - 16.0;

        egui::Area::new(egui::Id::new("chat_overlay"))
            .fixed_pos(egui::pos2(x, 80.0))
            .order(egui::Order::Background)
            .interactable(false)
            .show(ctx, |ui| {
                for (user, message, arrived) in &self.messages {
                    let elapsed = arrived.elapsed().as_secs_f32();
                    let alpha = if elapsed < FADE_START {
                        1.0
                    } else {
                        1.0 - ((elapsed - FADE_START) / (FADE_DURATION - FADE_START))
                            .clamp(0.0, 1.0)
                    };
                    let alpha_byte = (alpha * 255.0) as u8;

                    let name_color = widgets::username_color(user).gamma_multiply(alpha);
                    let text_color = egui::Color32::from_white_alpha(alpha_byte);
                    let shadow_color = egui::Color32::from_black_alpha((alpha * 180.0) as u8);

                    let mut job = egui::text::LayoutJob {
                        halign: egui::Align::RIGHT,
                        wrap: egui::text::TextWrapping {
                            max_width: 360.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    let font = egui::FontId::proportional(13.0);
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
                        message,
                        0.0,
                        egui::TextFormat {
                            font_id: font,
                            color: text_color,
                            ..Default::default()
                        },
                    );

                    let galley = ctx.fonts_mut(|f| f.layout_job(job.clone()));
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(360.0, galley.size().y),
                        egui::Sense::hover(),
                    );

                    // Shadow offset for readability
                    let mut shadow_job = job;
                    for section in &mut shadow_job.sections {
                        section.format.color = shadow_color;
                    }
                    let shadow_galley = ctx.fonts_mut(|f| f.layout_job(shadow_job));
                    ui.painter().galley(
                        rect.min + egui::vec2(1.0, 1.0),
                        shadow_galley,
                        shadow_color,
                    );

                    ui.painter().galley(rect.min, galley, text_color);
                }
            });
    }
}
