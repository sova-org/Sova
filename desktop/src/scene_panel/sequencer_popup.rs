mod header;
mod inline_edit;
mod preview;
mod tile;

use eframe::egui;
use sova_core::scene::{Line, Scene};
use sova_core::schedule::{ActionTiming, SchedulerMessage};

use super::prelude::{HeadPlayback, current_repetition, frame_playback};
use super::{
    ContextTarget, SceneOpacity, new_frame,
};
use crate::client_bridge::ClientBridge;
use crate::theme::{
    COLOR_MUTED,
};
use crate::widgets::syntax_highlight::SyntaxTheme;

const TILE_W: f32 = 112.0;
const TILE_H: f32 = 72.0;
const TILE_GAP: f32 = 4.0;
const GROUP_GAP: f32 = 14.0;
const TILES_PER_GROUP: usize = 4;
const LINE_HEADER_W: f32 = 176.0;
const GRID_PAD: f32 = 6.0;
const ROW_SPACING: f32 = 4.0;

pub(super) fn compact_duration_label(duration: f64) -> String {
    let mut s = format!("{duration:.2}");
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    format!("{s}b")
}

pub(super) fn row_separator(ui: &mut egui::Ui) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.add(egui::Separator::default().spacing(ROW_SPACING));
    });
}

pub(super) fn draw_rect_stroke(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    painter.line_segment([rect.left_top(), rect.right_top()], stroke);
    painter.line_segment([rect.right_top(), rect.right_bottom()], stroke);
    painter.line_segment([rect.right_bottom(), rect.left_bottom()], stroke);
    painter.line_segment([rect.left_bottom(), rect.left_top()], stroke);
}

impl super::ScenePanel {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn show_tile_grid(
        &mut self,
        ui: &mut egui::Ui,
        scene: &Scene,
        head_progress: &[Vec<HeadPlayback>],
        accent: egui::Color32,
        opacity: &SceneOpacity,
        bridge: &ClientBridge,
        theme: &SyntaxTheme,
        default_lang: &str,
    ) {
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(GRID_PAD as i8))
            .show(ui, |ui| {
                egui::ScrollArea::both()
                    .id_salt("seq_grid")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, ROW_SPACING);

                        // Prelude row (top)
                        ui.horizontal(|ui| {
                            egui::Frame::NONE
                                .inner_margin(egui::Margin::symmetric(6, 4))
                                .show(ui, |ui| {
                                    ui.set_width(LINE_HEADER_W - 12.0);
                                    ui.set_height(TILE_H - 8.0);
                                    ui.label(
                                        egui::RichText::new("Prelude").small().color(COLOR_MUTED),
                                    );
                                });

                            for idx in 0..self.prelude_states.len() {
                                if idx > 0 {
                                    ui.add_space(TILE_GAP);
                                }
                                let is_selected = self.state.selected_prelude() == Some(idx);
                                let resp =
                                    self.show_prelude_tile(ui, idx, is_selected, accent, opacity);
                                if resp.clicked() {
                                    self.navigate_to_prelude(idx);
                                }
                            }

                            // Add prelude script button
                            ui.add_space(TILE_GAP);
                            let add_fill = opacity.fill(ui.visuals().widgets.inactive.bg_fill, 0.5);
                            if ui
                                .add(
                                    egui::Button::new(egui::RichText::new("+").small().strong())
                                        .fill(add_fill)
                                        .min_size(egui::vec2(TILE_H, TILE_H)),
                                )
                                .clicked()
                            {
                                let mut scripts: Vec<sova_core::scene::script::Script> = bridge
                                    .scene()
                                    .map(|s| s.prelude.clone())
                                    .unwrap_or_default();
                                scripts.push(sova_core::scene::script::Script::new(
                                    String::new(),
                                    default_lang.to_string(),
                                ));
                                bridge.send(sova_server::ClientMessage::SchedulerControl(
                                    SchedulerMessage::SetScenePrelude(scripts),
                                ));
                            }
                        });

                        row_separator(ui);

                        // Lines
                        for li in 0..scene.lines.len() {
                            let line = &scene.lines[li];
                            let line_heads = head_progress.get(li);
                            let line_accent = crate::theme::cycled_accent(accent, li);

                            ui.horizontal(|ui| {
                                self.show_compact_line_header(
                                    ui,
                                    li,
                                    line,
                                    line_accent,
                                    opacity,
                                    bridge,
                                );

                                for fi in 0..line.frames.len() {
                                    if fi > 0 && fi % TILES_PER_GROUP == 0 {
                                        ui.add_space(GROUP_GAP);
                                    } else if fi > 0 {
                                        ui.add_space(TILE_GAP);
                                    }

                                    let frame = &line.frames[fi];
                                    let (is_playing, progress) = frame_playback(line_heads, fi);
                                    let current_rep = current_repetition(line_heads, fi);
                                    let is_cursor = self.state.cursor() == Some((li, fi));
                                    let is_selected = self.selection.contains(&(li, fi));
                                    let has_range =
                                        line.start_frame.is_some() || line.end_frame.is_some();
                                    let is_in_range = !has_range
                                        || (fi >= line.get_effective_start_frame()
                                            && fi <= line.get_effective_end_frame());

                                    let resp = self.show_tile(
                                        ui,
                                        li,
                                        fi,
                                        frame,
                                        current_rep,
                                        is_playing,
                                        progress,
                                        is_cursor,
                                        is_selected,
                                        is_in_range,
                                        line_accent,
                                        opacity,
                                        bridge,
                                        theme,
                                    );
                                    let is_inline_editing = self
                                        .sequencer_inline_edit
                                        .as_ref()
                                        .is_some_and(|edit| edit.target == (li, fi));

                                    if is_cursor && self.scroll_to_cursor {
                                        resp.scroll_to_me(Some(egui::Align::Center));
                                    }

                                    // Tooltip for disabled frames
                                    if !frame.enabled {
                                        resp.clone().on_hover_text(t!("scene.hint.enable"));
                                    }

                                    if !is_inline_editing && resp.clicked() {
                                        let alt = ui.input(|i| i.modifiers.alt);
                                        let shift = ui.input(|i| i.modifiers.shift);
                                        if alt {
                                            // Alt+Click: toggle enabled
                                            self.toggle_enabled(li, fi, bridge);
                                        } else if shift
                                            && self.anchor.is_some_and(|(al, _)| al == li)
                                        {
                                            self.move_cursor((li, fi), bridge);
                                            self.extend_selection((li, fi));
                                        } else {
                                            self.navigate_to_frame((li, fi), bridge);
                                        }
                                    }

                                    if !is_inline_editing && resp.double_clicked() {
                                        self.enter_frame_edit((li, fi));
                                    }

                                    if !is_inline_editing && resp.secondary_clicked() {
                                        self.update_cursor((li, fi), bridge);
                                        if !self.selection.contains(&(li, fi)) {
                                            self.selection.clear();
                                            self.selection.insert((li, fi));
                                            self.anchor = Some((li, fi));
                                        }
                                        let pos = ui
                                            .input(|i| i.pointer.interact_pos())
                                            .unwrap_or(resp.rect.center());
                                        self.open_context_menu(ContextTarget::Cell(li, fi), pos);
                                    }
                                }

                                // Add frame button
                                ui.add_space(TILE_GAP);
                                let add_fill =
                                    opacity.fill(ui.visuals().widgets.inactive.bg_fill, 0.5);
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("+").small().strong(),
                                        )
                                        .fill(add_fill)
                                        .min_size(egui::vec2(TILE_H, TILE_H)),
                                    )
                                    .on_hover_text(t!("scene.hint.add_frame"))
                                    .clicked()
                                {
                                    let new_fi = line.frames.len();
                                    bridge.send(SchedulerMessage::AddFrame(
                                        li,
                                        new_fi,
                                        new_frame(default_lang),
                                        ActionTiming::Immediate,
                                    ));
                                    self.navigate_to_frame((li, new_fi), bridge);
                                }
                            });

                            if li + 1 < scene.lines.len() {
                                row_separator(ui);
                            }
                        }

                        row_separator(ui);

                        // Add line button (in the header column)
                        ui.horizontal(|ui| {
                            let add_fill = opacity.fill(ui.visuals().widgets.inactive.bg_fill, 0.4);
                            if ui
                                .add(
                                    egui::Button::new(
                                        crate::icons::small(crate::icons::ADD).color(COLOR_MUTED),
                                    )
                                    .fill(add_fill)
                                    .min_size(egui::vec2(LINE_HEADER_W, TILE_H * 0.7)),
                                )
                                .on_hover_text(t!("scene.hint.add_line"))
                                .clicked()
                            {
                                let new_li = scene.lines.len();
                                bridge.send(SchedulerMessage::AddLine(
                                    new_li,
                                    Line::new(vec![1.0]),
                                    ActionTiming::Immediate,
                                ));
                                self.navigate_to_frame((new_li, 0), bridge);
                            }
                        });
                    });
            });
    }

}
