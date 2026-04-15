use eframe::egui;
use sova_core::scene::ExecutionMode;
use sova_core::schedule::ActionTiming;
use sova_core::schedule::SchedulerMessage;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;

pub enum TransportAction {
    Panic,
}

enum Editing {
    None,
    Tempo { buf: String, request_focus: bool },
    Quantum { buf: String, request_focus: bool },
}

pub struct TransportBar {
    editing: Editing,
    pub show_phase_bar: bool,
}

impl TransportBar {
    pub fn new() -> Self {
        Self {
            editing: Editing::None,
            show_phase_bar: true,
        }
    }

    pub fn show_inline(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        bridge: &ClientBridge,
    ) -> Option<TransportAction> {
        let mut action = None;
        let clock = bridge.clock();
        let accent = ui.visuals().selection.bg_fill;
        let text_color = ui.visuals().widgets.inactive.fg_stroke.color;

        // Play/Pause
        if clock.playing {
            let msg = SchedulerMessage::TransportStop(ActionTiming::Immediate);
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(
                    accent.r(),
                    accent.g(),
                    accent.b(),
                    30,
                ))
                .inner_margin(egui::Margin::symmetric(2, 1))
                .show(ui, |ui| {
                    let r = ui.button(crate::icons::rich(crate::icons::PAUSE));
                    if r.hovered() {
                        crate::widgets::hint::set(ctx, t!("transport.hint.stop"));
                    }
                    if r.clicked() {
                        bridge.send(msg);
                    }
                });
        } else {
            let r = ui.button(crate::icons::rich(crate::icons::PLAY));
            if r.hovered() {
                crate::widgets::hint::set(ctx, t!("transport.hint.play"));
            }
            if r.clicked() {
                bridge.send(SchedulerMessage::TransportStart(ActionTiming::Immediate));
            }
        }

        let r = ui.button(crate::icons::rich(crate::icons::HUSH));
        if r.hovered() {
            crate::widgets::hint::set(ctx, t!("transport.hint.hush"));
        }
        if r.clicked() {
            bridge.send(ClientMessage::Hush);
        }

        let r = ui.button(crate::icons::rich(crate::icons::PANIC));
        if r.hovered() {
            crate::widgets::hint::set(ctx, t!("transport.hint.panic"));
        }
        if r.clicked() {
            bridge.send(ClientMessage::Panic);
            action = Some(TransportAction::Panic);
        }

        self.show_tempo(ui, ctx, bridge, clock.tempo, text_color);
        self.show_quantum(ui, ctx, bridge, clock.quantum, text_color);

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
            bridge.send(SchedulerMessage::SetSceneMode(
                next,
                ActionTiming::Immediate,
            ));
        }

        if self.show_phase_bar {
            self.show_phase_ring(ui, ctx, bridge);
        }

        if clock.playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        action
    }

    fn show_tempo(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        tempo: f64,
        text_color: egui::Color32,
    ) {
        if let Editing::Tempo { buf, request_focus } = &mut self.editing {
            let resp = ui.add(
                egui::TextEdit::singleline(buf)
                    .desired_width(50.0)
                    .font(egui::TextStyle::Monospace),
            );
            if *request_focus {
                *request_focus = false;
                resp.request_focus();
            }
            if resp.hovered() {
                crate::widgets::hint::set(ctx, t!("transport.hint.tempo_edit"));
            }
            if resp.lost_focus() {
                if crate::widgets::consume_key_on_lost_focus(ui, &resp, egui::Key::Enter)
                    && let Ok(t) = buf.parse::<f64>()
                {
                    let t = t.clamp(20.0, 300.0);
                    bridge.send(SchedulerMessage::SetTempo(t, ActionTiming::Immediate));
                }
                self.editing = Editing::None;
            }
            return;
        }
        let wv = &mut ui.style_mut().visuals.widgets;
        wv.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        wv.inactive.bg_stroke = egui::Stroke::NONE;
        let resp = ui.add(egui::Button::new(
            egui::RichText::new(
                t!("transport.tempo_value", val = format!("{:.1}", tempo)).to_string(),
            )
            .monospace()
            .color(text_color),
        ));
        if resp.hovered() {
            crate::widgets::hint::set(ctx, t!("transport.hint.tempo"));
        }
        if resp.clicked() {
            self.editing = Editing::Tempo {
                buf: format!("{:.1}", tempo),
                request_focus: true,
            };
        }
    }

    fn show_quantum(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        quantum: f64,
        text_color: egui::Color32,
    ) {
        if let Editing::Quantum { buf, request_focus } = &mut self.editing {
            let resp = ui.add(
                egui::TextEdit::singleline(buf)
                    .desired_width(30.0)
                    .font(egui::TextStyle::Monospace),
            );
            if *request_focus {
                *request_focus = false;
                resp.request_focus();
            }
            if resp.hovered() {
                crate::widgets::hint::set(ctx, t!("transport.hint.quantum_edit"));
            }
            if resp.lost_focus() {
                if crate::widgets::consume_key_on_lost_focus(ui, &resp, egui::Key::Enter)
                    && let Ok(q) = buf.parse::<u32>()
                {
                    let q = q.clamp(1, 16);
                    bridge.send(SchedulerMessage::SetQuantum(
                        q as f64,
                        ActionTiming::Immediate,
                    ));
                }
                self.editing = Editing::None;
            }
            return;
        }
        let wv = &mut ui.style_mut().visuals.widgets;
        wv.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        wv.inactive.bg_stroke = egui::Stroke::NONE;
        let resp = ui.add(egui::Button::new(
            egui::RichText::new(t!("transport.quantum_value", val = quantum as u32).to_string())
                .monospace()
                .color(text_color),
        ));
        if resp.hovered() {
            crate::widgets::hint::set(ctx, t!("transport.hint.quantum"));
        }
        if resp.clicked() {
            self.editing = Editing::Quantum {
                buf: format!("{}", quantum as u32),
                request_focus: true,
            };
        }
    }

    pub fn show_phase_ring(&self, ui: &mut egui::Ui, ctx: &egui::Context, bridge: &ClientBridge) {
        let clock = bridge.clock();
        let accent = ui.visuals().selection.bg_fill;
        let bar_color = if clock.playing {
            accent
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };
        let bg_color = ui.visuals().extreme_bg_color;

        let size = ui.available_height().clamp(20.0, 24.0);
        let (rect, phase_r) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
        if phase_r.hovered() {
            crate::widgets::hint::set(
                ctx,
                format!(
                    "beat {:.2}  ·  bar {}",
                    clock.beat,
                    (clock.beat / clock.quantum.max(1.0)).floor() as u64 + 1
                ),
            );
        }
        let painter = ui.painter_at(rect);
        let center = rect.center();
        let radius_outer = size * 0.5 - 1.0;
        let radius_inner = radius_outer * 0.68;
        let thickness = radius_outer - radius_inner;
        let radius_mid = (radius_inner + radius_outer) * 0.5;

        use std::f32::consts::{PI, TAU};
        let start_angle = -PI * 0.5;

        let quantum_int = clock.quantum as u32;
        let use_segments =
            (1..=16).contains(&quantum_int) && (clock.quantum - quantum_int as f64).abs() < 0.001;

        if use_segments {
            let current_beat = clock.phase.floor() as u32;
            let beat_frac = clock.phase.fract() as f32;
            let pulse = (1.0 - beat_frac).powi(3);

            let slot = TAU / quantum_int as f32;
            let gap_rad: f32 = (slot * 0.15).min(0.08);
            let seg_span = slot - gap_rad;

            for i in 0..quantum_int {
                let s = start_angle + i as f32 * slot + gap_rad * 0.5;
                let e = s + seg_span;
                draw_arc(&painter, center, radius_mid, s, e, thickness, bg_color);
                if i < current_beat {
                    draw_arc(&painter, center, radius_mid, s, e, thickness, bar_color);
                } else if i == current_beat {
                    let e_frac = s + seg_span * beat_frac;
                    let boost = if i == 0 { pulse * 80.0 } else { pulse * 50.0 };
                    let pulsed = egui::Color32::from_rgb(
                        bar_color.r().saturating_add(boost as u8),
                        bar_color.g().saturating_add(boost as u8),
                        bar_color.b().saturating_add(boost as u8),
                    );
                    draw_arc(&painter, center, radius_mid, s, e_frac, thickness, pulsed);
                }
            }
        } else {
            let phase_frac = if clock.quantum > 0.0 {
                (clock.phase / clock.quantum) as f32
            } else {
                0.0
            };
            draw_arc(
                &painter,
                center,
                radius_mid,
                start_angle,
                start_angle + TAU,
                thickness,
                bg_color,
            );
            draw_arc(
                &painter,
                center,
                radius_mid,
                start_angle,
                start_angle + TAU * phase_frac,
                thickness,
                bar_color,
            );
        }

        let beat_in_bar = clock.phase.floor() as u32 + 1;
        let text_color = if clock.playing {
            bar_color
        } else {
            ui.visuals().widgets.inactive.fg_stroke.color
        };
        let font_size = if beat_in_bar >= 10 {
            (radius_inner * 0.95).clamp(7.0, 10.0)
        } else {
            (radius_inner * 1.35).clamp(9.0, 13.0)
        };
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("{beat_in_bar}"),
            egui::FontId::monospace(font_size),
            text_color,
        );

        if clock.playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }
    }
}

fn draw_arc(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    start: f32,
    end: f32,
    thickness: f32,
    color: egui::Color32,
) {
    let span = (end - start).abs();
    if span <= f32::EPSILON {
        return;
    }
    let steps = (span * radius * 0.8).ceil().max(4.0) as usize;
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = start + (end - start) * i as f32 / steps as f32;
        points.push(center + egui::vec2(t.cos() * radius, t.sin() * radius));
    }
    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(thickness, color),
    ));
}
