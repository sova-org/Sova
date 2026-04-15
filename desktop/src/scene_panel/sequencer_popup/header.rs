use eframe::egui;
use sova_core::scene::Line;

use crate::client_bridge::ClientBridge;
use crate::scene_panel::{ContextTarget, SceneOpacity};
use crate::theme::{COLOR_OK, STROKE_HAIRLINE, STROKE_NORMAL, accent_fill_soft, tile_label_font};

use super::{LINE_HEADER_W, TILE_H, TILE_W, draw_rect_stroke};

impl crate::scene_panel::ScenePanel {
    pub(super) fn show_prelude_tile(
        &self,
        ui: &mut egui::Ui,
        idx: usize,
        is_selected: bool,
        accent: egui::Color32,
        opacity: &SceneOpacity,
    ) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(TILE_W, TILE_H), egui::Sense::click());

        if !ui.is_rect_visible(rect) {
            return resp;
        }

        let hovered = resp.hovered();
        let fill = if is_selected {
            accent_fill_soft(accent)
        } else if hovered {
            opacity.fill(ui.visuals().widgets.hovered.weak_bg_fill, 1.0)
        } else {
            opacity.fill(ui.visuals().faint_bg_color, 1.0)
        };

        ui.painter().rect_filled(rect, 0.0, fill);

        let stroke = if is_selected {
            egui::Stroke::new(STROKE_NORMAL, accent)
        } else if hovered {
            egui::Stroke::new(STROKE_NORMAL, ui.visuals().widgets.hovered.bg_stroke.color)
        } else {
            egui::Stroke::new(
                STROKE_HAIRLINE,
                opacity.fill(ui.visuals().widgets.noninteractive.bg_stroke.color, 0.5),
            )
        };
        draw_rect_stroke(ui.painter(), rect, stroke);

        let text_color = if is_selected {
            ui.visuals().strong_text_color()
        } else {
            ui.visuals().text_color()
        };
        let label = format!("P{}", idx + 1);
        let galley = ui
            .painter()
            .layout_no_wrap(label, tile_label_font(ui.ctx()), text_color);
        let text_pos = egui::pos2(
            rect.center().x - galley.size().x * 0.5,
            rect.center().y - galley.size().y * 0.5,
        );
        ui.painter().galley(text_pos, galley, text_color);

        resp
    }

    pub(super) fn show_compact_line_header(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        line: &Line,
        accent: egui::Color32,
        opacity: &SceneOpacity,
        bridge: &ClientBridge,
    ) {
        let resp = egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.set_width(LINE_HEADER_W - 12.0);
                ui.set_height(TILE_H - 8.0);
                opacity.override_widget_visuals(ui);
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    ui.label(
                        egui::RichText::new(format!("{}", li + 1))
                            .strong()
                            .color(ui.visuals().text_color()),
                    );

                    self.line_flag_button(
                        ui,
                        li,
                        line.looping,
                        accent,
                        bridge,
                        t!("scene.toggle_looping"),
                        |_, c| crate::icons::small(crate::icons::LOOPING).color(c).into(),
                        |l| l.looping = !l.looping,
                    );
                    self.line_flag_button(
                        ui,
                        li,
                        line.trailing,
                        accent,
                        bridge,
                        t!("scene.toggle_trailing"),
                        |_, c| crate::icons::small(crate::icons::TRAILING).color(c).into(),
                        |l| l.trailing = !l.trailing,
                    );
                    self.line_flag_button(
                        ui,
                        li,
                        line.manual,
                        accent,
                        bridge,
                        t!("scene.toggle_manual"),
                        |_, c| crate::icons::small(crate::icons::MANUAL).color(c).into(),
                        |l| l.manual = !l.manual,
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut speed = line.speed_factor;
                        if self.sequencer_line_speed_focus == Some(li) {
                            self.sequencer_line_speed_focus = None;
                            let id = ui.next_auto_id();
                            ui.memory_mut(|m| m.request_focus(id));
                        }
                        let speed_resp = ui.add(
                            egui::DragValue::new(&mut speed)
                                .range(0.01..=f64::MAX)
                                .speed(0.05)
                                .custom_formatter(|v, _| format!("x{v:.1}")),
                        );
                        if speed_resp.lost_focus() {
                            let _ = crate::widgets::consume_key_on_lost_focus(
                                ui,
                                &speed_resp,
                                egui::Key::Escape,
                            );
                        }
                        if speed_resp.changed() {
                            self.toggle_line_field(li, bridge, |l| l.speed_factor = speed);
                        }

                        let any_peer = bridge.peer_cursors().iter().any(|(name, &(pli, _, _))| {
                            pli == li
                                && bridge
                                    .confirmed_username()
                                    .is_none_or(|my| my != name.as_str())
                        });
                        if any_peer {
                            ui.add(
                                egui::Label::new(
                                    crate::icons::small(crate::icons::CIRCLE_LARGE_FILLED)
                                        .color(COLOR_OK),
                                )
                                .selectable(false),
                            );
                        }
                    });
                });
            });

        if resp.response.secondary_clicked() {
            let pos = ui
                .input(|i| i.pointer.interact_pos())
                .unwrap_or(resp.response.rect.center());
            self.open_context_menu(ContextTarget::Header(li), pos);
        }
    }
}
