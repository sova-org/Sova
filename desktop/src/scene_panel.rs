use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

use eframe::egui;
use sova_core::scene::{Frame, Line};
use sova_core::schedule::ActionTiming;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use crate::widgets::syntax_highlight::SyntaxTheme;
use crate::widgets::{
    EditorSettings, PeerCursor, username_color, COLOR_MUTED, COLOR_OK,
};
use crate::widgets::inline_scene_view::InlineFrameState;

const MIN_COL_WIDTH: f32 = 120.0;
const MAX_COL_WIDTH: f32 = 800.0;
const DEFAULT_COL_WIDTH: f32 = 450.0;
const CELL_HEIGHT: f32 = 180.0;
const HEADER_HEIGHT: f32 = 26.0;
const LINE_HEADER_HEIGHT: f32 = 26.0;
const GAP: f32 = 1.0;
const DRAG_HANDLE_WIDTH: f32 = 6.0;

#[derive(Clone, Copy)]
pub struct SceneOpacity {
    base: f32,
    active: bool,
}

impl SceneOpacity {
    pub fn new(visuals_enabled: bool, opacity: f32) -> Self {
        Self {
            base: opacity,
            active: visuals_enabled,
        }
    }

    pub fn alpha(&self, scale: f32) -> u8 {
        if !self.active {
            return 255;
        }
        ((self.base * scale).clamp(0.0, 1.0) * 255.0) as u8
    }

    pub fn fill(&self, c: egui::Color32, scale: f32) -> egui::Color32 {
        if !self.active {
            return c;
        }
        egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), self.alpha(scale))
    }

    pub fn override_widget_visuals(&self, ui: &mut egui::Ui) {
        if !self.active {
            return;
        }
        let v = ui.visuals_mut();
        v.extreme_bg_color = egui::Color32::TRANSPARENT;
        v.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        v.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        v.widgets.hovered.bg_fill = egui::Color32::from_white_alpha(self.alpha(0.3));
        v.widgets.hovered.weak_bg_fill = egui::Color32::from_white_alpha(self.alpha(0.3));
        v.widgets.active.bg_fill = egui::Color32::from_white_alpha(self.alpha(0.4));
        v.widgets.active.weak_bg_fill = egui::Color32::from_white_alpha(self.alpha(0.4));
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum ContextTarget {
    Cell(usize, usize),
    Header(usize),
    Void,
}

pub struct PanelVisibility {
    pub sidebar: bool,
    pub devices: bool,
    pub scope: bool,
    pub spectrum: bool,
    pub vu_meter: bool,
    pub scope_bar: bool,
    pub logs: bool,
    pub debug: bool,
}

#[derive(Default)]
pub struct ScenePanel {
    cursor: Option<(usize, usize)>,
    anchor: Option<(usize, usize)>,
    selection: BTreeSet<(usize, usize)>,
    clipboard: Vec<Frame>,
    context_target: Option<ContextTarget>,
    frame_states: HashMap<(usize, usize), InlineFrameState>,
    column_widths: Vec<f32>,
    currently_editing: Option<(usize, usize)>,
    last_line_count: usize,
    last_frame_counts: Vec<usize>,
}

impl ScenePanel {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        panels: &mut PanelVisibility,
        visuals_enabled: bool,
        scene_opacity: f32,
        editor_settings: &EditorSettings,
    ) {
        let Some(scene) = bridge.scene() else {
            ui.colored_label(egui::Color32::GRAY, t!("scene.no_scene"));
            return;
        };

        let has_positions = bridge.positions().iter().any(|p| !p.is_empty());
        let accent = ui.visuals().selection.bg_fill;
        let opacity = SceneOpacity::new(visuals_enabled, scene_opacity);

        // Compute per-line progress for playing indicators
        let progress: Vec<f32> = {
            let now = Instant::now();
            let secs_per_beat = 60.0 / bridge.clock().tempo;
            let positions = bridge.positions();
            let starts = bridge.position_start();
            (0..scene.lines.len())
                .map(|li| {
                    let Some(&(fi, _rep)) = positions.get(li).and_then(|p| p.first()) else {
                        return 0.0;
                    };
                    let line = &scene.lines[li];
                    let Some(frame) = line.frames.get(fi) else {
                        return 0.0;
                    };
                    let start = starts.get(li).copied().unwrap_or(now);
                    let elapsed = now.duration_since(start).as_secs_f64();
                    let dur = (frame.duration / line.speed_factor) * secs_per_beat;
                    if dur <= 0.0 {
                        return 0.0;
                    }
                    (elapsed / dur).clamp(0.0, 1.0) as f32
                })
                .collect()
        };

        // Sync frame state lifecycle
        self.sync_frame_states(scene, bridge);

        // Ensure column widths match line count
        while self.column_widths.len() < scene.lines.len() {
            self.column_widths.push(DEFAULT_COL_WIDTH);
        }
        self.column_widths.truncate(scene.lines.len());

        let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
        let available_height = ui.available_height();

        // Track which frame has editor focus for StartedEditingFrame/StoppedEditingFrame
        let mut new_editing: Option<(usize, usize)> = None;

        egui::ScrollArea::horizontal()
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    for li in 0..scene.lines.len() {
                        let col_width = self.column_widths[li];
                        let line = &scene.lines[li];

                        ui.allocate_ui(egui::vec2(col_width, available_height), |ui| {
                            ui.vertical(|ui| {
                                // Line header
                                self.show_line_header(ui, li, line, accent, &opacity, bridge);

                                // Independent vertical scroll for frames
                                egui::ScrollArea::vertical()
                                    .id_salt(("line_scroll", li))
                                    .auto_shrink(false)
                                    .show(ui, |ui| {
                                        for fi in 0..line.frames.len() {
                                            let frame = &line.frames[fi];
                                            let is_playing = bridge
                                                .positions()
                                                .get(li)
                                                .is_some_and(|p| p.iter().any(|&(pf, _)| pf == fi));
                                            let line_progress = if is_playing {
                                                progress.get(li).copied().unwrap_or(0.0)
                                            } else {
                                                0.0
                                            };
                                            let is_selected = self.selection.contains(&(li, fi));
                                            let is_cursor = self.cursor == Some((li, fi));

                                            // Ensure frame state exists
                                            let state_key = (li, fi);
                                            self.frame_states
                                                .entry(state_key)
                                                .or_insert_with(|| InlineFrameState::new(frame));

                                            let cell_resp = self.show_frame_cell(
                                                ui,
                                                li,
                                                fi,
                                                frame,
                                                is_playing,
                                                line_progress,
                                                is_selected,
                                                is_cursor,
                                                accent,
                                                &opacity,
                                                editor_settings,
                                                &theme,
                                                bridge,
                                            );

                                            // Track editor focus
                                            if self.frame_states
                                                .get(&(li, fi))
                                                .is_some_and(|s| s.editor_has_focus)
                                            {
                                                new_editing = Some((li, fi));
                                            }

                                            // Handle click on cell
                                            if cell_resp.clicked() {
                                                let shift = ui.input(|i| i.modifiers.shift);
                                                if shift {
                                                    self.extend_selection((li, fi));
                                                } else {
                                                    self.update_cursor((li, fi), bridge);
                                                    self.anchor = Some((li, fi));
                                                    self.selection.clear();
                                                    self.selection.insert((li, fi));
                                                }
                                            }

                                            // Right-click on cell
                                            if cell_resp.secondary_clicked() {
                                                self.context_target =
                                                    Some(ContextTarget::Cell(li, fi));
                                                if !self.selection.contains(&(li, fi)) {
                                                    self.update_cursor((li, fi), bridge);
                                                    self.selection.clear();
                                                    self.selection.insert((li, fi));
                                                    self.anchor = Some((li, fi));
                                                }
                                            }

                                            // Context menu
                                            cell_resp.context_menu(|ui| {
                                                self.show_context_menu(
                                                    ui,
                                                    Some(ContextTarget::Cell(li, fi)),
                                                    bridge,
                                                    panels,
                                                );
                                            });

                                            ui.add_space(GAP);
                                        }

                                        // Add frame button
                                        ui.add_space(4.0);
                                        let add_btn_fill = opacity.fill(
                                            ui.visuals().widgets.inactive.bg_fill, 0.5,
                                        );
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("+").strong(),
                                                )
                                                .fill(add_btn_fill)
                                                .min_size(egui::vec2(ui.available_width(), 22.0)),
                                            )
                                            .clicked()
                                        {
                                            let fi = line.frames.len();
                                            bridge.send(ClientMessage::AddFrame(
                                                li,
                                                fi,
                                                Frame::default(),
                                                ActionTiming::Immediate,
                                            ));
                                        }
                                    });
                            });
                        });

                        // Drag handle between columns
                        if li + 1 < scene.lines.len() {
                            let (handle_rect, handle_resp) = ui.allocate_exact_size(
                                egui::vec2(DRAG_HANDLE_WIDTH, available_height),
                                egui::Sense::drag(),
                            );
                            if handle_resp.dragged() {
                                let delta = handle_resp.drag_delta().x;
                                self.column_widths[li] =
                                    (self.column_widths[li] + delta).clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
                            }
                            if handle_resp.hovered() || handle_resp.dragged() {
                                let center_x = handle_rect.center().x;
                                ui.painter().vline(
                                    center_x,
                                    handle_rect.y_range(),
                                    egui::Stroke::new(1.0, accent),
                                );
                                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                            }
                        }
                    }

                    // Add line button
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.add_space(LINE_HEADER_HEIGHT);
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("+").strong())
                                    .min_size(egui::vec2(28.0, 28.0)),
                            )
                            .clicked()
                        {
                            let li = scene.lines.len();
                            bridge.send(ClientMessage::AddLine(
                                li,
                                Line::new(vec![1.0]),
                                ActionTiming::Immediate,
                            ));
                        }
                    });
                });
            });

        // Handle editing notifications
        if new_editing != self.currently_editing {
            if let Some((old_li, old_fi)) = self.currently_editing {
                bridge.send(ClientMessage::StoppedEditingFrame(old_li, old_fi));
            }
            if let Some((new_li, new_fi)) = new_editing {
                bridge.send(ClientMessage::StartedEditingFrame(new_li, new_fi));
            }
            self.currently_editing = new_editing;
        }

        // Void context menu handled by individual cell/header context menus above

        // Keyboard shortcuts (only when no text field has focus)
        if !ui.ctx().memory(|m| m.focused().is_some()) {
            self.handle_clipboard(ui, bridge);
            self.handle_keyboard(ui, bridge);
        }

        if has_positions {
            ui.ctx().request_repaint();
        }
    }

    fn show_line_header(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        line: &Line,
        accent: egui::Color32,
        opacity: &SceneOpacity,
        bridge: &ClientBridge,
    ) {
        let header_bg = opacity.fill(ui.visuals().faint_bg_color, 0.9);
        let header_frame = egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(4, 2))
            .fill(header_bg);

        let resp = header_frame
            .show(ui, |ui| {
                ui.set_height(LINE_HEADER_HEIGHT - 4.0);
                opacity.override_widget_visuals(ui);
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    // Left side: label + toggles
                    ui.label(
                        egui::RichText::new(format!("L{}", li))
                            .small()
                            .strong(),
                    );

                    let loop_color = if line.looping { accent } else { COLOR_MUTED };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(crate::icons::LOOPING).color(loop_color),
                            )
                            .fill(egui::Color32::TRANSPARENT),
                        )
                        .on_hover_text(t!("scene.toggle_looping"))
                        .clicked()
                    {
                        self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
                    }

                    let trail_color = if line.trailing { accent } else { COLOR_MUTED };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(crate::icons::TRAILING).color(trail_color),
                            )
                            .fill(egui::Color32::TRANSPARENT),
                        )
                        .on_hover_text(t!("scene.toggle_trailing"))
                        .clicked()
                    {
                        self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
                    }

                    // Right side: speed, frame range, peer dots
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;

                        // Peer editing dots (rightmost)
                        let peer_count: usize = (0..line.frames.len())
                            .filter(|&fi| {
                                bridge
                                    .peer_editing()
                                    .get(&(li, fi))
                                    .is_some_and(|v| !v.is_empty())
                            })
                            .count();
                        if peer_count > 0 {
                            ui.label(
                                egui::RichText::new(crate::icons::CIRCLE_FILLED)
                                    .small()
                                    .color(COLOR_OK),
                            );
                        }

                        // Frame range
                        if line.start_frame.is_some() || line.end_frame.is_some() {
                            let s = line.start_frame.map(|f| f.to_string()).unwrap_or_default();
                            let e = line.end_frame.map(|f| f.to_string()).unwrap_or_default();
                            ui.label(
                                egui::RichText::new(format!("[{}..{}]", s, e))
                                    .small()
                                    .color(COLOR_MUTED),
                            );
                        }

                        // Speed
                        let mut speed = line.speed_factor;
                        let speed_resp = ui.add(
                            egui::DragValue::new(&mut speed)
                                .range(0.01..=f64::MAX)
                                .speed(0.05)
                                .prefix("×"),
                        );
                        if speed_resp.changed() && speed > 0.0 {
                            self.toggle_line_field(li, bridge, |l| l.speed_factor = speed);
                        }
                    });
                });
            })
            .response;

        // Right-click on header
        resp.context_menu(|ui| {
            self.show_context_menu(ui, Some(ContextTarget::Header(li)), bridge, &mut PanelVisibility {
                sidebar: false, devices: false, scope: false, spectrum: false,
                vu_meter: false, scope_bar: false, logs: false, debug: false,
            });
        });
    }

    fn show_frame_cell(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        fi: usize,
        frame: &Frame,
        is_playing: bool,
        progress: f32,
        is_selected: bool,
        is_cursor: bool,
        accent: egui::Color32,
        opacity: &SceneOpacity,
        editor_settings: &EditorSettings,
        theme: &SyntaxTheme,
        bridge: &ClientBridge,
    ) -> egui::Response {
        // Background color — scaled by opacity
        let bg = if !frame.enabled {
            opacity.fill(egui::Color32::from_gray(25), 1.0)
        } else if is_cursor {
            opacity.fill(ui.visuals().extreme_bg_color, 1.0)
        } else if is_selected {
            opacity.fill(accent, 0.3)
        } else {
            opacity.fill(ui.visuals().faint_bg_color, 1.0)
        };

        // Stroke for cursor/selection
        let stroke = if is_cursor {
            egui::Stroke::new(2.0, accent)
        } else if is_selected {
            egui::Stroke::new(1.0, accent.linear_multiply(0.5))
        } else {
            egui::Stroke::NONE
        };

        // Use push_id to scope all widget IDs within this frame cell
        let resp = ui.push_id(("frame_cell", li, fi), |ui| {
            let cell_frame = egui::Frame::NONE
                .fill(bg)
                .stroke(stroke)
                .inner_margin(egui::Margin { left: 5, ..egui::Margin::ZERO });

            let frame_resp = cell_frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(HEADER_HEIGHT + CELL_HEIGHT);

                opacity.override_widget_visuals(ui);

                // Header
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.set_height(HEADER_HEIGHT);
                    if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                        state.show_header(ui, li, fi, frame, opacity, bridge);
                    }
                });

                ui.separator();

                // Body (code editor)
                let syntax = bridge.syntax_map.get(
                    self.frame_states
                        .get(&(li, fi))
                        .map(|s| s.lang.as_str())
                        .unwrap_or(""),
                );
                let syntax_pair = syntax.map(|cs| (cs, theme));

                let reference = bridge
                    .languages()
                    .iter()
                    .find(|l| {
                        self.frame_states
                            .get(&(li, fi))
                            .is_some_and(|s| s.lang == l.name)
                    })
                    .filter(|l| !l.documentation.reference.is_empty())
                    .map(|l| &l.documentation.reference);

                let peer_cursors: Vec<PeerCursor> = bridge
                    .text_cursors_for_frame(li, fi)
                    .into_iter()
                    .map(|(name, line, col)| PeerCursor {
                        name: name.to_owned(),
                        line,
                        col,
                        color: username_color(name),
                    })
                    .collect();

                if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                    state.show_body(
                        ui,
                        li,
                        fi,
                        editor_settings,
                        syntax_pair,
                        reference,
                        &peer_cursors,
                        bridge,
                    );
                }
            });

            let cell_rect = frame_resp.response.rect;

            // Playing indicator: 4px left strip filling top to bottom
            if is_playing && frame.enabled {
                let strip_w = 4.0;
                let fill_h = cell_rect.height() * progress;
                let strip_rect = egui::Rect::from_min_size(
                    cell_rect.min,
                    egui::vec2(strip_w, fill_h),
                );
                ui.painter().rect_filled(strip_rect, 0.0, accent);
            }

            // Overlay effects (compilation/mutation flashes)

            if let Some(&(success, instant)) = bridge.compilation_flashes().get(&(li, fi)) {
                let elapsed = instant.elapsed().as_secs_f32();
                if elapsed < 1.0 {
                    let alpha = ((1.0 - elapsed) * 60.0) as u8;
                    let flash = if success {
                        egui::Color32::from_rgba_unmultiplied(80, 200, 80, alpha)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(200, 80, 80, alpha)
                    };
                    ui.painter().rect_filled(cell_rect, 0.0, flash);
                    ui.ctx().request_repaint();
                }
            }

            if let Some(instant) = bridge.mutation_flashes().get(&(li, fi)) {
                let elapsed = instant.elapsed().as_secs_f32();
                if elapsed < 1.2 {
                    let alpha = ((1.0 - elapsed / 1.2) * 40.0) as u8;
                    let flash = egui::Color32::from_rgba_unmultiplied(200, 200, 220, alpha);
                    ui.painter().rect_filled(cell_rect, 0.0, flash);
                    ui.ctx().request_repaint();
                }
            }

            // Peer editing indicators in corner
            if let Some(editors) = bridge.peer_editing().get(&(li, fi))
                && !editors.is_empty()
            {
                let dot_x = cell_rect.right() - 6.0;
                let dot_y = cell_rect.top() + 6.0;
                for (i, name) in editors.iter().take(3).enumerate() {
                    let color = username_color(name);
                    ui.painter().circle_filled(
                        egui::pos2(dot_x - i as f32 * 8.0, dot_y),
                        3.0,
                        color,
                    );
                }
            }

            frame_resp.response
        });

        resp.inner
    }

    fn sync_frame_states(&mut self, scene: &sova_core::scene::Scene, _bridge: &ClientBridge) {
        let current_counts: Vec<usize> =
            scene.lines.iter().map(|l| l.frames.len()).collect();

        // If line count or frame counts changed, invalidate stale states
        if current_counts != self.last_frame_counts || scene.lines.len() != self.last_line_count {
            // Remove states for lines/frames that no longer exist
            self.frame_states.retain(|&(li, fi), _| {
                scene.lines.get(li).is_some_and(|l| fi < l.frames.len())
            });

            // If frame count changed for a line, clear all states for that line
            // (indices may have shifted)
            for (li, &count) in current_counts.iter().enumerate() {
                let old_count = self.last_frame_counts.get(li).copied().unwrap_or(0);
                if count != old_count {
                    let keys_to_remove: Vec<_> = self
                        .frame_states
                        .keys()
                        .filter(|&&(l, _)| l == li)
                        .copied()
                        .collect();
                    for key in keys_to_remove {
                        self.frame_states.remove(&key);
                    }
                }
            }

            self.last_line_count = scene.lines.len();
            self.last_frame_counts = current_counts;
        }

        // Sync remote changes for non-dirty states
        for (li, line) in scene.lines.iter().enumerate() {
            for (fi, frame) in line.frames.iter().enumerate() {
                if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                    state.sync_if_remote_changed(frame);
                }
            }
        }
    }

    fn update_cursor(&mut self, new_cursor: (usize, usize), bridge: &ClientBridge) {
        let old = self.cursor;
        self.cursor = Some(new_cursor);
        if old != self.cursor && bridge.is_connected() {
            bridge.send(ClientMessage::CursorPosition(new_cursor.0, new_cursor.1, None));
        }
    }

    fn show_context_menu(
        &mut self,
        ui: &mut egui::Ui,
        target: Option<ContextTarget>,
        bridge: &ClientBridge,
        panels: &mut PanelVisibility,
    ) {
        let m = if cfg!(target_os = "macos") {
            "Cmd"
        } else {
            "Ctrl"
        };

        match target {
            Some(ContextTarget::Cell(li, fi)) => {
                let multi = self.selection.len() > 1;
                let scene = bridge.scene();
                let line_len = scene
                    .and_then(|s| s.lines.get(li))
                    .map(|l| l.frames.len())
                    .unwrap_or(0);
                let (min_fi, max_fi) = if !self.selection.is_empty() {
                    let fis: Vec<usize> = self.selection.iter().map(|&(_, f)| f).collect();
                    (
                        *fis.iter().min().unwrap(),
                        *fis.iter().max().unwrap(),
                    )
                } else {
                    (fi, fi)
                };

                if ui
                    .add(egui::Button::new(t!("scene.cut")).shortcut_text(format!("{m}+X")))
                    .clicked()
                {
                    self.cut_selection(bridge);
                    ui.close();
                }
                if ui
                    .add(egui::Button::new(t!("scene.copy")).shortcut_text(format!("{m}+C")))
                    .clicked()
                {
                    self.copy_selection(bridge);
                    ui.close();
                }
                if ui
                    .add_enabled(
                        !self.clipboard.is_empty(),
                        egui::Button::new(t!("scene.paste_after")).shortcut_text(format!("{m}+V")),
                    )
                    .clicked()
                {
                    self.paste_after(li, fi, bridge);
                    ui.close();
                }

                ui.separator();

                if !multi {
                    if ui.button(t!("scene.insert_frame_before")).clicked() {
                        bridge.send(ClientMessage::AddFrame(
                            li, fi, Frame::default(), ActionTiming::Immediate,
                        ));
                        ui.close();
                    }
                    if ui.button(t!("scene.insert_frame_after")).clicked() {
                        bridge.send(ClientMessage::AddFrame(
                            li, fi + 1, Frame::default(), ActionTiming::Immediate,
                        ));
                        ui.close();
                    }
                }

                if ui
                    .add(egui::Button::new(t!("scene.duplicate_frame")).shortcut_text(format!("{m}+D")))
                    .clicked()
                {
                    if let Some(scene_ref) = scene {
                        let selected: Vec<(usize, usize)> =
                            self.selection.iter().copied().collect();
                        let sel_li = selected.first().map(|&(l, _)| l).unwrap_or(li);
                        let last_fi = selected.last().map(|&(_, f)| f).unwrap_or(fi);
                        let frames: Vec<Frame> = selected
                            .iter()
                            .filter_map(|&(l, f)| {
                                scene_ref.lines.get(l).and_then(|line| line.frames.get(f).cloned())
                            })
                            .collect();
                        for (offset, frame) in frames.iter().enumerate() {
                            bridge.send(ClientMessage::AddFrame(
                                sel_li, last_fi + 1 + offset, frame.clone(), ActionTiming::Immediate,
                            ));
                        }
                    }
                    ui.close();
                }

                ui.separator();

                if ui
                    .add_enabled(
                        min_fi > 0,
                        egui::Button::new(t!("scene.move_up")).shortcut_text("Alt+Up"),
                    )
                    .clicked()
                {
                    self.move_frames_vertical(-1, bridge);
                    ui.close();
                }
                if ui
                    .add_enabled(
                        max_fi + 1 < line_len,
                        egui::Button::new(t!("scene.move_down")).shortcut_text("Alt+Down"),
                    )
                    .clicked()
                {
                    self.move_frames_vertical(1, bridge);
                    ui.close();
                }

                ui.separator();

                if ui.button(t!("scene.toggle_enabled")).clicked() {
                    let selected: Vec<(usize, usize)> =
                        self.selection.iter().copied().collect();
                    for (sl, sf) in selected {
                        self.toggle_enabled(sl, sf, bridge);
                    }
                    ui.close();
                }

                ui.separator();

                if ui
                    .add(egui::Button::new(t!("scene.remove_frame")).shortcut_text("Delete"))
                    .clicked()
                {
                    let mut to_remove: Vec<(usize, usize)> =
                        self.selection.iter().copied().collect();
                    to_remove.sort_by(|a, b| b.1.cmp(&a.1));
                    for (rli, rfi) in to_remove {
                        bridge.send(ClientMessage::RemoveFrame(rli, rfi, ActionTiming::Immediate));
                    }
                    self.selection.clear();
                    self.cursor = None;
                    ui.close();
                }
            }
            Some(ContextTarget::Header(li)) => {
                let num_lines = bridge.scene().map(|s| s.lines.len()).unwrap_or(0);

                if ui.button(t!("scene.insert_line_before")).clicked() {
                    bridge.send(ClientMessage::AddLine(
                        li, Line::new(vec![1.0]), ActionTiming::Immediate,
                    ));
                    ui.close();
                }
                if ui.button(t!("scene.insert_line_after")).clicked() {
                    bridge.send(ClientMessage::AddLine(
                        li + 1, Line::new(vec![1.0]), ActionTiming::Immediate,
                    ));
                    ui.close();
                }
                if ui
                    .add(egui::Button::new(t!("scene.duplicate_line")).shortcut_text(format!("{m}+Shift+D")))
                    .clicked()
                {
                    if let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) {
                        bridge.send(ClientMessage::AddLine(
                            li + 1, line.clone(), ActionTiming::Immediate,
                        ));
                    }
                    ui.close();
                }

                ui.separator();

                if ui
                    .add_enabled(li > 0, egui::Button::new(t!("scene.move_left")).shortcut_text("Alt+Left"))
                    .clicked()
                {
                    self.move_line_horizontal(li, -1, bridge);
                    ui.close();
                }
                if ui
                    .add_enabled(
                        li + 1 < num_lines,
                        egui::Button::new(t!("scene.move_right")).shortcut_text("Alt+Right"),
                    )
                    .clicked()
                {
                    self.move_line_horizontal(li, 1, bridge);
                    ui.close();
                }

                ui.separator();

                if ui
                    .add(egui::Button::new(t!("scene.toggle_looping")).shortcut_text("L"))
                    .clicked()
                {
                    self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
                    ui.close();
                }
                if ui
                    .add(egui::Button::new(t!("scene.toggle_trailing")).shortcut_text("T"))
                    .clicked()
                {
                    self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
                    ui.close();
                }

                ui.separator();

                if ui.button(t!("scene.clear_frame_range")).clicked() {
                    self.toggle_line_field(li, bridge, |l| {
                        l.start_frame = None;
                        l.end_frame = None;
                    });
                    ui.close();
                }

                ui.separator();

                if ui
                    .add(egui::Button::new(t!("scene.remove_line")).shortcut_text(format!("{m}+Del")))
                    .clicked()
                {
                    bridge.send(ClientMessage::RemoveLine(li, ActionTiming::Immediate));
                    ui.close();
                }
            }
            Some(ContextTarget::Void) => {
                ui.checkbox(&mut panels.sidebar, t!("options.title"));
                ui.separator();
                ui.checkbox(&mut panels.devices, t!("devices.title"));
                ui.checkbox(&mut panels.scope, t!("scope.title"));
                ui.checkbox(&mut panels.spectrum, t!("spectrum.title"));
                ui.checkbox(&mut panels.vu_meter, t!("cmd.vu_meter"));
                ui.checkbox(&mut panels.scope_bar, t!("cmd.scope_bar"));
                ui.separator();
                ui.checkbox(&mut panels.logs, t!("cmd.logs"));
                ui.separator();
                ui.checkbox(&mut panels.debug, t!("debug.title"));
            }
            None => {}
        }
    }

    fn handle_clipboard(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let Some((li, fi)) = self.cursor else {
            return;
        };

        let (copy, paste, cut) = ui.input(|i| {
            (
                i.events.iter().any(|e| matches!(e, egui::Event::Copy)),
                i.events.iter().any(|e| matches!(e, egui::Event::Paste(_))),
                i.events.iter().any(|e| matches!(e, egui::Event::Cut)),
            )
        });

        if cut && !self.selection.is_empty() {
            self.cut_selection(bridge);
            ui.ctx()
                .copy_text(format!("{} frame(s)", self.clipboard.len()));
        } else if copy && !self.selection.is_empty() {
            self.copy_selection(bridge);
            ui.ctx()
                .copy_text(format!("{} frame(s)", self.clipboard.len()));
        }

        if paste && !self.clipboard.is_empty() {
            self.paste_after(li, fi, bridge);
        }
    }

    fn handle_keyboard(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let Some(scene) = bridge.scene() else {
            return;
        };
        let Some((cur_li, cur_fi)) = self.cursor else {
            return;
        };

        let num_lines = scene.lines.len();
        if num_lines == 0 {
            return;
        }
        let line_lens: Vec<usize> = scene.lines.iter().map(|l| l.frames.len()).collect();

        let (up, down, left, right, shift, alt, key_escape, key_delete, ctrl_a, ctrl_d, ctrl_shift_d, ctrl_del, key_l, key_t) =
            ui.input(|i| {
                let no_mod = !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt;
                (
                    i.key_pressed(egui::Key::ArrowUp),
                    i.key_pressed(egui::Key::ArrowDown),
                    i.key_pressed(egui::Key::ArrowLeft),
                    i.key_pressed(egui::Key::ArrowRight),
                    i.modifiers.shift,
                    i.modifiers.alt,
                    i.key_pressed(egui::Key::Escape),
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                    i.modifiers.command && i.key_pressed(egui::Key::A),
                    i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::D),
                    i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::D),
                    i.modifiers.command
                        && (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)),
                    no_mod && i.key_pressed(egui::Key::L),
                    no_mod && i.key_pressed(egui::Key::T),
                )
            });

        let li = cur_li;
        let fi = cur_fi;

        // Alt+arrow: move frame/line
        if alt && up && !self.selection.is_empty() {
            self.move_frames_vertical(-1, bridge);
        } else if alt && down && !self.selection.is_empty() {
            self.move_frames_vertical(1, bridge);
        } else if alt && left {
            self.move_line_horizontal(li, -1, bridge);
        } else if alt && right {
            self.move_line_horizontal(li, 1, bridge);
        } else {
            let mut nav_li = li;
            let mut nav_fi = fi;
            let mut moved = false;
            let mut vertical = false;

            if up && fi > 0 {
                nav_fi -= 1;
                moved = true;
                vertical = true;
            } else if down && fi + 1 < line_lens[li] {
                nav_fi += 1;
                moved = true;
                vertical = true;
            } else if left && li > 0 {
                nav_li -= 1;
                nav_fi = fi.min(line_lens[nav_li].saturating_sub(1));
                moved = true;
            } else if right && li + 1 < num_lines {
                nav_li += 1;
                nav_fi = fi.min(line_lens[nav_li].saturating_sub(1));
                moved = true;
            }

            if moved {
                self.update_cursor((nav_li, nav_fi), bridge);
                if shift && vertical {
                    self.extend_selection((nav_li, nav_fi));
                } else {
                    self.selection.clear();
                    self.selection.insert((nav_li, nav_fi));
                    self.anchor = Some((nav_li, nav_fi));
                }
            }
        }

        if ctrl_d {
            let selected: Vec<(usize, usize)> = self.selection.iter().copied().collect();
            let sel_li = selected.first().map(|&(l, _)| l).unwrap_or(li);
            let last_fi = selected.last().map(|&(_, f)| f).unwrap_or(fi);
            let frames: Vec<Frame> = selected
                .iter()
                .filter_map(|&(l, f)| {
                    scene.lines.get(l).and_then(|line| line.frames.get(f).cloned())
                })
                .collect();
            for (offset, frame) in frames.iter().enumerate() {
                bridge.send(ClientMessage::AddFrame(
                    sel_li, last_fi + 1 + offset, frame.clone(), ActionTiming::Immediate,
                ));
            }
            self.selection.clear();
            for (offset, _) in frames.iter().enumerate() {
                self.selection.insert((sel_li, last_fi + 1 + offset));
            }
            self.cursor = Some((sel_li, last_fi + frames.len()));
            self.anchor = Some((sel_li, last_fi + 1));
        }

        if ctrl_shift_d
            && let Some(line) = scene.lines.get(li)
        {
            bridge.send(ClientMessage::AddLine(
                li + 1, line.clone(), ActionTiming::Immediate,
            ));
        }

        if key_delete && !ctrl_del {
            let mut to_remove: Vec<(usize, usize)> = self.selection.iter().copied().collect();
            to_remove.sort_by(|a, b| b.1.cmp(&a.1));
            for (rli, rfi) in to_remove {
                bridge.send(ClientMessage::RemoveFrame(rli, rfi, ActionTiming::Immediate));
            }
            self.selection.clear();
            self.cursor = None;
        }

        if ctrl_del {
            bridge.send(ClientMessage::RemoveLine(li, ActionTiming::Immediate));
            self.selection.clear();
            self.cursor = None;
        }

        if key_l {
            self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
        } else if key_t {
            self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
        }

        if ctrl_a {
            self.selection.clear();
            for f in 0..line_lens[li] {
                self.selection.insert((li, f));
            }
        }

        if key_escape {
            self.selection.clear();
            self.cursor = None;
        }
    }

    fn toggle_enabled(&self, li: usize, fi: usize, bridge: &ClientBridge) {
        if let Some(frame) = bridge
            .scene()
            .and_then(|s| s.lines.get(li))
            .and_then(|l| l.frames.get(fi))
        {
            let mut f = frame.clone();
            f.enabled = !f.enabled;
            bridge.send(ClientMessage::SetFrames(
                vec![(li, fi, f)],
                ActionTiming::Immediate,
            ));
        }
    }

    fn toggle_line_field(&self, li: usize, bridge: &ClientBridge, modify: impl FnOnce(&mut Line)) {
        if let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) {
            let mut l = line.clone();
            modify(&mut l);
            bridge.send(ClientMessage::ConfigureLines(
                vec![(li, l)],
                ActionTiming::Immediate,
            ));
        }
    }

    fn copy_selection(&mut self, bridge: &ClientBridge) {
        let Some(scene) = bridge.scene() else { return };
        self.clipboard = self
            .selection
            .iter()
            .filter_map(|&(l, f)| {
                scene.lines.get(l).and_then(|line| line.frames.get(f).cloned())
            })
            .collect();
    }

    fn cut_selection(&mut self, bridge: &ClientBridge) {
        self.copy_selection(bridge);
        let mut to_remove: Vec<(usize, usize)> = self.selection.iter().copied().collect();
        to_remove.sort_by(|a, b| b.1.cmp(&a.1));
        for (rli, rfi) in to_remove {
            bridge.send(ClientMessage::RemoveFrame(rli, rfi, ActionTiming::Immediate));
        }
        self.selection.clear();
        self.cursor = None;
    }

    fn paste_after(&mut self, li: usize, fi: usize, bridge: &ClientBridge) {
        if self.clipboard.is_empty() {
            return;
        }
        for (offset, frame) in self.clipboard.iter().enumerate() {
            bridge.send(ClientMessage::AddFrame(
                li, fi + 1 + offset, frame.clone(), ActionTiming::Immediate,
            ));
        }
        let count = self.clipboard.len();
        self.selection.clear();
        for offset in 0..count {
            self.selection.insert((li, fi + 1 + offset));
        }
        self.anchor = Some((li, fi + 1));
        self.cursor = Some((li, fi + count));
    }

    fn move_frames_vertical(&mut self, direction: i32, bridge: &ClientBridge) {
        let Some(scene) = bridge.scene() else { return };
        if self.selection.is_empty() {
            return;
        }

        let selected: Vec<(usize, usize)> = self.selection.iter().copied().collect();
        let sel_li = selected[0].0;
        if !selected.iter().all(|&(l, _)| l == sel_li) {
            return;
        }

        let min_fi = selected.iter().map(|&(_, f)| f).min().unwrap();
        let max_fi = selected.iter().map(|&(_, f)| f).max().unwrap();
        let line_len = scene.lines.get(sel_li).map(|l| l.frames.len()).unwrap_or(0);

        if direction < 0 {
            if min_fi == 0 {
                return;
            }
            if let Some(frame) = scene
                .lines
                .get(sel_li)
                .and_then(|l| l.frames.get(min_fi - 1).cloned())
            {
                bridge.send(ClientMessage::RemoveFrame(sel_li, min_fi - 1, ActionTiming::Immediate));
                bridge.send(ClientMessage::AddFrame(sel_li, max_fi, frame, ActionTiming::Immediate));
            }
            self.selection.clear();
            for fi in (min_fi - 1)..=max_fi.saturating_sub(1) {
                self.selection.insert((sel_li, fi));
            }
            self.cursor = self.cursor.map(|(l, f)| (l, f.saturating_sub(1)));
            self.anchor = self.anchor.map(|(l, f)| (l, f.saturating_sub(1)));
        } else {
            if max_fi + 1 >= line_len {
                return;
            }
            if let Some(frame) = scene
                .lines
                .get(sel_li)
                .and_then(|l| l.frames.get(max_fi + 1).cloned())
            {
                bridge.send(ClientMessage::RemoveFrame(sel_li, max_fi + 1, ActionTiming::Immediate));
                bridge.send(ClientMessage::AddFrame(sel_li, min_fi, frame, ActionTiming::Immediate));
            }
            self.selection.clear();
            for fi in (min_fi + 1)..=(max_fi + 1) {
                self.selection.insert((sel_li, fi));
            }
            self.cursor = self.cursor.map(|(l, f)| (l, f + 1));
            self.anchor = self.anchor.map(|(l, f)| (l, f + 1));
        }
    }

    fn move_line_horizontal(&mut self, li: usize, direction: i32, bridge: &ClientBridge) {
        let Some(scene) = bridge.scene() else { return };
        let num_lines = scene.lines.len();

        let new_li = if direction < 0 {
            if li == 0 {
                return;
            }
            li - 1
        } else {
            if li + 1 >= num_lines {
                return;
            }
            li + 1
        };

        if let Some(line) = scene.lines.get(li) {
            let line = line.clone();
            bridge.send(ClientMessage::RemoveLine(li, ActionTiming::Immediate));
            bridge.send(ClientMessage::AddLine(new_li, line, ActionTiming::Immediate));
        }

        if let Some((cur_li, cur_fi)) = self.cursor
            && cur_li == li
        {
            self.cursor = Some((new_li, cur_fi));
            bridge.send(ClientMessage::CursorPosition(new_li, cur_fi, None));
        }
        self.anchor = self
            .anchor
            .map(|(al, af)| if al == li { (new_li, af) } else { (al, af) });
        let old_sel: Vec<(usize, usize)> = self.selection.iter().copied().collect();
        self.selection.clear();
        for (sl, sf) in old_sel {
            if sl == li {
                self.selection.insert((new_li, sf));
            } else {
                self.selection.insert((sl, sf));
            }
        }
    }

    fn extend_selection(&mut self, target: (usize, usize)) {
        let Some((anchor_li, anchor_fi)) = self.anchor else {
            return;
        };
        let (li, fi) = target;
        if li != anchor_li {
            return;
        }
        self.selection.clear();
        let start = anchor_fi.min(fi);
        let end = anchor_fi.max(fi);
        for f in start..=end {
            self.selection.insert((li, f));
        }
        self.cursor = Some(target);
    }
}
