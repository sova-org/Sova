use eframe::egui;
use sova_core::scene::ExecutionMode;
use sova_core::schedule::ActionTiming;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;

pub struct TransportBar {
    editing_tempo: bool,
    tempo_buf: String,
}

impl TransportBar {
    pub fn new() -> Self {
        Self {
            editing_tempo: false,
            tempo_buf: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, bridge: &ClientBridge) {
        egui::TopBottomPanel::top("transport_bar").show(ctx, |ui| {
            if !bridge.is_connected() {
                ui.add_enabled_ui(false, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Not connected");
                    });
                });
                return;
            }

            let clock = bridge.clock();
            let accent = ui.visuals().selection.bg_fill;

            ui.horizontal(|ui| {
                let (label, msg) = if clock.playing {
                    (
                        "\u{23F8}",
                        ClientMessage::TransportStop(ActionTiming::Immediate),
                    )
                } else {
                    (
                        "\u{25B6}",
                        ClientMessage::TransportStart(ActionTiming::Immediate),
                    )
                };
                if ui.button(label).clicked() {
                    bridge.send(msg);
                }

                ui.separator();

                ui.monospace(format!("Beat: {:.2}", clock.beat));

                ui.separator();

                if self.editing_tempo {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.tempo_buf)
                            .desired_width(50.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if resp.lost_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && let Ok(t) = self.tempo_buf.parse::<f64>()
                        {
                            let t = t.clamp(20.0, 300.0);
                            bridge.send(ClientMessage::SetTempo(t, ActionTiming::Immediate));
                        }
                        self.editing_tempo = false;
                    }
                } else {
                    let resp = ui.monospace(format!("{:.1} BPM", clock.tempo));
                    if resp.clicked() {
                        self.editing_tempo = true;
                        self.tempo_buf = format!("{:.1}", clock.tempo);
                    }
                    resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                }

                ui.separator();

                let phase_frac = if clock.quantum > 0.0 {
                    (clock.phase / clock.quantum) as f32
                } else {
                    0.0
                };
                let bar_color = if clock.playing {
                    accent
                } else {
                    ui.visuals().widgets.inactive.bg_fill
                };
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(80.0, ui.text_style_height(&egui::TextStyle::Body)),
                    egui::Sense::hover(),
                );
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
                let mut fill = rect;
                fill.set_right(rect.left() + rect.width() * phase_frac);
                painter.rect_filled(fill, 0.0, bar_color);

                ui.separator();

                ui.monospace(format!("Q: {}", clock.quantum as u32));

                ui.separator();

                let mode = bridge.scene().map(|s| s.mode).unwrap_or_default();
                let mode_color = if mode.is_free() {
                    ui.visuals().text_color()
                } else {
                    accent
                };
                let resp = ui.colored_label(mode_color, format!("{mode}"));
                if resp.clicked() {
                    let next = match mode {
                        ExecutionMode::Free => ExecutionMode::AtQuantum,
                        ExecutionMode::AtQuantum => ExecutionMode::LongestLine,
                        ExecutionMode::LongestLine => ExecutionMode::Free,
                    };
                    bridge.send(ClientMessage::SetSceneMode(next, ActionTiming::Immediate));
                }
                resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            });

            if clock.playing {
                ctx.request_repaint();
            }
        });
    }
}
