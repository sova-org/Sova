use eframe::egui;
use sova_core::scene::ExecutionMode;
use sova_core::schedule::ActionTiming;
use sova_core::schedule::SchedulerMessage;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;

pub enum TransportAction {
    Panic,
}

pub struct TransportBar {
    editing_tempo: bool,
    tempo_buf: String,
    editing_quantum: bool,
    quantum_buf: String,
}

impl TransportBar {
    pub fn new() -> Self {
        Self {
            editing_tempo: false,
            tempo_buf: String::new(),
            editing_quantum: false,
            quantum_buf: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, bridge: &ClientBridge) -> Option<TransportAction> {
        let mut action = None;
        egui::TopBottomPanel::top("transport_bar").show(ctx, |ui| {
            if !bridge.is_connected() {
                ui.add_enabled_ui(false, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(t!("transport.not_connected"));
                    });
                });
                return;
            }

            let clock = bridge.clock();
            let accent = ui.visuals().selection.bg_fill;

            ui.horizontal(|ui| {
                // Play/Pause
                if clock.playing {
                    let msg = ClientMessage::TransportStop(ActionTiming::Immediate);
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgba_unmultiplied(
                            accent.r(),
                            accent.g(),
                            accent.b(),
                            30,
                        ))
                        .inner_margin(egui::Margin::symmetric(2, 1))
                        .show(ui, |ui| {
                            let r = ui.button(crate::icons::PAUSE);
                            if r.hovered() {
                                crate::widgets::hint::set(ctx, t!("transport.hint.stop"));
                            }
                            if r.clicked() {
                                bridge.send(msg);
                            }
                        });
                } else {
                    let r = ui.button(crate::icons::PLAY);
                    if r.hovered() {
                        crate::widgets::hint::set(ctx, t!("transport.hint.play"));
                    }
                    if r.clicked() {
                        bridge.send(ClientMessage::TransportStart(
                            ActionTiming::Immediate,
                        ));
                    }
                }

                let r = ui.button(crate::icons::HUSH);
                if r.hovered() {
                    crate::widgets::hint::set(ctx, t!("transport.hint.hush"));
                }
                if r.clicked() {
                    bridge.send(ClientMessage::Hush);
                }

                let r = ui.button(crate::icons::PANIC);
                if r.hovered() {
                    crate::widgets::hint::set(ctx, t!("transport.hint.panic"));
                }
                if r.clicked() {
                    bridge.send(ClientMessage::Panic);
                    action = Some(TransportAction::Panic);
                }

                ui.separator();

                let r = ui.monospace(
                    t!("transport.beat_value", val = format!("{:.2}", clock.beat))
                        .to_string(),
                );
                if r.hovered() {
                    crate::widgets::hint::set(ctx, t!("transport.hint.beat"));
                }

                ui.separator();

                if self.editing_tempo {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.tempo_buf)
                            .desired_width(50.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if resp.hovered() {
                        crate::widgets::hint::set(ctx, t!("transport.hint.tempo_edit"));
                    }
                    if resp.lost_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && let Ok(t) = self.tempo_buf.parse::<f64>()
                        {
                            let t = t.clamp(20.0, 300.0);
                            bridge.send(ClientMessage::SetTempo(
                                t,
                                ActionTiming::Immediate,
                            ));
                        }
                        self.editing_tempo = false;
                    }
                } else {
                    let resp = ui.monospace(
                        t!(
                            "transport.tempo_value",
                            val = format!("{:.1}", clock.tempo)
                        )
                        .to_string(),
                    );
                    if resp.hovered() {
                        crate::widgets::hint::set(ctx, t!("transport.hint.tempo"));
                    }
                    if resp.clicked() {
                        self.editing_tempo = true;
                        self.tempo_buf = format!("{:.1}", clock.tempo);
                    }
                    resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                }

                ui.separator();

                // Segmented phase bar — fills remaining center space
                let right_reserve = 120.0;
                let bar_width = (ui.available_width() - right_reserve).max(40.0);
                let bar_height = ui.text_style_height(&egui::TextStyle::Body);
                let bar_color = if clock.playing {
                    accent
                } else {
                    ui.visuals().widgets.inactive.bg_fill
                };

                let quantum_int = clock.quantum as u32;
                let use_segments = (1..=16).contains(&quantum_int)
                    && (clock.quantum - quantum_int as f64).abs() < 0.001;

                if use_segments {
                    let gap: f32 = 2.0;
                    let seg_w =
                        (bar_width - (quantum_int.saturating_sub(1)) as f32 * gap)
                            / quantum_int as f32;

                    let (rect, phase_r) = ui.allocate_exact_size(
                        egui::vec2(bar_width, bar_height),
                        egui::Sense::hover(),
                    );
                    if phase_r.hovered() {
                        crate::widgets::hint::set(ctx, t!("transport.hint.phase"));
                    }
                    let painter = ui.painter_at(rect);
                    let bg_color = ui.visuals().extreme_bg_color;
                    let current_beat = clock.phase.floor() as u32;
                    let beat_frac = clock.phase.fract() as f32;
                    let pulse = (1.0 - clock.phase.fract() as f32).powi(3);

                    for i in 0..quantum_int {
                        let x = rect.left() + i as f32 * (seg_w + gap);
                        let seg = egui::Rect::from_min_size(
                            egui::pos2(x, rect.top()),
                            egui::vec2(seg_w, bar_height),
                        );
                        painter.rect_filled(seg, 0.0, bg_color);

                        if i < current_beat {
                            painter.rect_filled(seg, 0.0, bar_color);
                        } else if i == current_beat {
                            let mut fill = seg;
                            fill.set_right(seg.left() + seg_w * beat_frac);
                            let boost =
                                if i == 0 { pulse * 80.0 } else { pulse * 50.0 };
                            let pulsed = egui::Color32::from_rgb(
                                bar_color.r().saturating_add(boost as u8),
                                bar_color.g().saturating_add(boost as u8),
                                bar_color.b().saturating_add(boost as u8),
                            );
                            painter.rect_filled(fill, 0.0, pulsed);
                        }
                    }
                } else {
                    let phase_frac = if clock.quantum > 0.0 {
                        (clock.phase / clock.quantum) as f32
                    } else {
                        0.0
                    };
                    let (rect, phase_r) = ui.allocate_exact_size(
                        egui::vec2(bar_width, bar_height),
                        egui::Sense::hover(),
                    );
                    if phase_r.hovered() {
                        crate::widgets::hint::set(ctx, t!("transport.hint.phase"));
                    }
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
                    let mut fill = rect;
                    fill.set_right(rect.left() + rect.width() * phase_frac);
                    painter.rect_filled(fill, 0.0, bar_color);
                }

                ui.separator();

                if self.editing_quantum {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.quantum_buf)
                            .desired_width(30.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if resp.hovered() {
                        crate::widgets::hint::set(ctx, t!("transport.hint.quantum_edit"));
                    }
                    if resp.lost_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && let Ok(q) = self.quantum_buf.parse::<u32>()
                        {
                            let q = q.clamp(1, 16);
                            bridge.send(ClientMessage::SchedulerControl(
                                SchedulerMessage::SetQuantum(
                                    q as f64,
                                    ActionTiming::Immediate,
                                ),
                            ));
                        }
                        self.editing_quantum = false;
                    }
                } else {
                    let resp = ui.monospace(
                        t!("transport.quantum_value", val = clock.quantum as u32)
                            .to_string(),
                    );
                    if resp.hovered() {
                        crate::widgets::hint::set(ctx, t!("transport.hint.quantum"));
                    }
                    if resp.clicked() {
                        self.editing_quantum = true;
                        self.quantum_buf = format!("{}", clock.quantum as u32);
                    }
                    resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                }

                ui.separator();

                // Execution mode — selectable label so it looks interactive
                let mode = bridge.scene().map(|s| s.mode).unwrap_or_default();
                let resp = ui.selectable_label(!mode.is_free(), format!("{mode}"));
                if resp.hovered() {
                    crate::widgets::hint::set(ctx, t!("transport.hint.mode"));
                }
                if resp.clicked() {
                    let next = match mode {
                        ExecutionMode::Free => ExecutionMode::AtQuantum,
                        ExecutionMode::AtQuantum => ExecutionMode::LongestLine,
                        ExecutionMode::LongestLine => ExecutionMode::Free,
                    };
                    bridge.send(ClientMessage::SetSceneMode(
                        next,
                        ActionTiming::Immediate,
                    ));
                }

            });

            if clock.playing {
                ctx.request_repaint();
            }
        });
        action
    }
}
