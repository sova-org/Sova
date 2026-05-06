use eframe::egui;

use crate::client_bridge::ClientBridge;
use crate::theme::{
    COLOR_ERROR, COLOR_MUTED, STROKE_NORMAL, accent_fill_med, accent_fill_soft,
    accent_fill_strong, tile_label_font, username_color,
};
use crate::widgets::syntax_highlight::SyntaxTheme;

use super::preview::show_frame_preview;
use super::{compact_duration_label, TILE_H, TILE_W};
use crate::scene_panel::SceneOpacity;
use crate::theme::COLOR_OK;

impl crate::scene_panel::ScenePanel {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn show_tile(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        fi: usize,
        frame: &sova_core::scene::Frame,
        current_rep: Option<usize>,
        is_playing: bool,
        progress: f32,
        is_cursor: bool,
        is_selected: bool,
        is_in_range: bool,
        accent: egui::Color32,
        opacity: &SceneOpacity,
        bridge: &ClientBridge,
        theme: &SyntaxTheme,
    ) -> egui::Response {
        let (rect, mut resp) =
            ui.allocate_exact_size(egui::vec2(TILE_W, TILE_H), egui::Sense::click());

        if !ui.is_rect_visible(rect) {
            return resp;
        }

        let painter = ui.painter().clone();

        let bg = if is_playing && frame.enabled {
            accent_fill_strong(accent)
        } else if is_cursor && frame.enabled {
            accent_fill_med(accent)
        } else if is_selected && frame.enabled {
            accent_fill_soft(accent)
        } else if !frame.enabled {
            opacity.fill(egui::Color32::from_gray(20), 1.0)
        } else if !is_in_range {
            opacity.fill(egui::Color32::from_gray(28), 1.0)
        } else {
            opacity.fill(ui.visuals().faint_bg_color, 1.0)
        };
        painter.rect_filled(rect, 0.0, bg);

        if is_playing && frame.enabled {
            let progress_w = rect.width() * progress;
            let progress_rect =
                egui::Rect::from_min_size(rect.min, egui::vec2(progress_w, rect.height()));
            painter.rect_filled(progress_rect, 0.0, accent_fill_soft(accent));
            ui.ctx().request_repaint();
        }

        if is_playing && frame.enabled {
            let strip = egui::Rect::from_min_size(rect.min, egui::vec2(2.0, rect.height()));
            painter.rect_filled(strip, 0.0, accent);
        }

        if is_cursor {
            let border_color = if frame.enabled {
                accent
            } else {
                accent_fill_strong(accent)
            };
            let s = egui::Stroke::new(STROKE_NORMAL, border_color);
            painter.line_segment([rect.left_top(), rect.right_top()], s);
            painter.line_segment([rect.right_top(), rect.right_bottom()], s);
            painter.line_segment([rect.right_bottom(), rect.left_bottom()], s);
            painter.line_segment([rect.left_bottom(), rect.left_top()], s);
        }

        if is_selected && !is_cursor {
            let sel_color = if frame.enabled {
                accent_fill_soft(accent)
            } else {
                egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 25)
            };
            painter.rect_filled(rect, 0.0, sel_color);
        }

        let editing_peers = bridge.editing_peers_for_frame(li, fi);
        let editing_count = editing_peers.len();
        let mut editing_names: Vec<&str> = editing_peers.iter().map(String::as_str).collect();
        editing_names.sort_unstable();

        let shows_inline_editor = self.show_inline_tile_editor(ui, rect, li, fi, frame, bridge);
        if !shows_inline_editor {
            let has_content = !frame.script().content().is_empty();
            let is_dirty = self
                .frame_states
                .get(&(li, fi))
                .is_some_and(|state| state.dirty);
            let text_color = if !frame.enabled || !is_in_range {
                COLOR_MUTED
            } else if is_playing {
                egui::Color32::WHITE
            } else if has_content {
                ui.visuals().text_color()
            } else {
                COLOR_MUTED
            };
            let step_label = format!("{:02}", fi + 1);
            let step_font = tile_label_font(ui.ctx());
            let step_galley = painter.layout_no_wrap(step_label, step_font, text_color);
            let step_pos = egui::pos2(
                rect.left() + if editing_count > 0 { 28.0 } else { 8.0 },
                rect.top() + 6.0,
            );
            painter.galley(step_pos, step_galley, text_color);

            if editing_count > 0 {
                let badge_color = COLOR_OK;
                if editing_count == 1 {
                    painter.circle_filled(
                        egui::pos2(rect.left() + 12.0, rect.top() + 14.0),
                        4.0,
                        badge_color,
                    );
                } else {
                    let count_label = editing_count.to_string();
                    let count_font = egui::TextStyle::Small.resolve(ui.style());
                    let count_galley =
                        painter.layout_no_wrap(count_label, count_font, egui::Color32::WHITE);
                    let indicator_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.left() + 6.0, rect.top() + 6.0),
                        egui::vec2(count_galley.size().x + 8.0, count_galley.size().y + 4.0),
                    );
                    painter.rect_filled(indicator_rect, 3.0, badge_color);
                    painter.galley(
                        egui::pos2(
                            indicator_rect.center().x - count_galley.size().x / 2.0,
                            indicator_rect.center().y - count_galley.size().y / 2.0,
                        ),
                        count_galley,
                        egui::Color32::WHITE,
                    );
                }
            }

            let meta_font = egui::TextStyle::Small.resolve(ui.style());
            let duration_label = compact_duration_label(frame.duration);
            let duration_color = if frame.enabled && is_in_range {
                ui.visuals().weak_text_color()
            } else {
                COLOR_MUTED
            };
            let duration_galley =
                painter.layout_no_wrap(duration_label, meta_font.clone(), duration_color);
            let duration_pos = egui::pos2(
                rect.left() + 8.0,
                rect.bottom() - duration_galley.size().y - 8.0,
            );
            painter.galley(duration_pos, duration_galley, duration_color);

            let reps_label = if is_playing && frame.repetitions > 1 {
                match current_rep {
                    Some(rep) => format!("{}/{}", rep + 1, frame.repetitions),
                    None => format!("1/{}", frame.repetitions),
                }
            } else {
                format!("x{}", frame.repetitions)
            };
            let reps_color = if frame.enabled && is_in_range {
                text_color
            } else {
                COLOR_MUTED
            };
            let reps_galley = painter.layout_no_wrap(reps_label, meta_font.clone(), reps_color);
            let badge_size = egui::vec2(reps_galley.size().x + 10.0, reps_galley.size().y + 4.0);
            let badge_rect = egui::Rect::from_min_size(
                egui::pos2(rect.right() - badge_size.x - 6.0, rect.top() + 6.0),
                badge_size,
            );
            let badge_fill = if frame.enabled && is_in_range {
                opacity.fill(ui.visuals().widgets.inactive.bg_fill, 0.95)
            } else {
                opacity.fill(egui::Color32::from_gray(32), 1.0)
            };
            painter.rect_filled(badge_rect, 2.0, badge_fill);
            painter.galley(
                egui::pos2(
                    badge_rect.center().x - reps_galley.size().x / 2.0,
                    badge_rect.center().y - reps_galley.size().y / 2.0,
                ),
                reps_galley,
                reps_color,
            );

            if has_content {
                let dot_center = egui::pos2(rect.right() - 8.0, rect.bottom() - 8.0);
                let dot_color = if !frame.enabled {
                    COLOR_MUTED
                } else if is_playing {
                    accent
                } else {
                    ui.visuals().text_color()
                };
                painter.circle_filled(dot_center, 2.5, dot_color);
            }

            if is_dirty {
                let dirty_icon = painter.layout_no_wrap(
                    crate::icons::MODIFIED.to_owned(),
                    egui::FontId::new(meta_font.size, crate::icons::family()),
                    COLOR_ERROR,
                );
                let dirty_pos = egui::pos2(
                    rect.right() - dirty_icon.size().x - 16.0,
                    rect.bottom() - dirty_icon.size().y - 5.0,
                );
                painter.galley(dirty_pos, dirty_icon, COLOR_ERROR);
            }
        }

        crate::scene_panel::prelude::draw_feedback_flashes(ui, li, fi, rect, bridge, 80.0, 50.0);

        for name in bridge.editing_peers_for_frame(li, fi) {
            let color = username_color(name);
            let bottom_strip = egui::Rect::from_min_size(
                egui::pos2(rect.left(), rect.bottom() - 2.0),
                egui::vec2(rect.width(), 2.0),
            );
            painter.rect_filled(bottom_strip, 0.0, color);
        }

        let has_script = !frame.script().content().is_empty()
            || bridge.frame_text_id_at(li, fi).is_some();
        if has_script || !editing_names.is_empty() {
            resp = resp.on_hover_ui(|ui| {
                show_frame_preview(ui, li, fi, frame, bridge, theme, &editing_names);
            });

            if is_cursor && !resp.hovered()
                && let Some(deadline) = self.cursor_preview_deadline
            {
                let now = std::time::Instant::now();
                if now < deadline {
                    egui::Area::new(egui::Id::new(("seq_kb_preview", li, fi)))
                        .order(egui::Order::Tooltip)
                        .fixed_pos(rect.right_bottom() + egui::vec2(6.0, 4.0))
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                show_frame_preview(
                                    ui,
                                    li,
                                    fi,
                                    frame,
                                    bridge,
                                    theme,
                                    &editing_names,
                                );
                            });
                        });
                    ui.ctx().request_repaint_after(deadline - now);
                }
            }
        }

        resp
    }
}
