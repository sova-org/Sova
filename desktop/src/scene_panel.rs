use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

use eframe::egui;
use sova_core::scene::script::Script;
use sova_core::scene::{Frame, Line};
use sova_core::schedule::ActionTiming;
use sova_core::vm::language::LanguageDefinition;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use crate::widgets::syntax_highlight::SyntaxTheme;
use crate::widgets::{
    EditorContext, EditorSettings, PeerCursor, username_color, COLOR_MUTED, COLOR_OK,
};
use crate::widgets::inline_scene_view::{InlineFrameState, InlineScriptState};
use sova_core::schedule::SchedulerMessage;

pub fn resolve_default_language(preferred: &str, available: &[LanguageDefinition]) -> String {
    if available.is_empty() || available.iter().any(|l| l.name == preferred) {
        preferred.to_string()
    } else {
        available[0].name.clone()
    }
}

pub fn new_frame(lang: &str) -> Frame {
    Frame::from(Script::new(String::new(), lang.to_string()))
}

const MIN_COL_WIDTH: f32 = 120.0;
const MAX_COL_WIDTH: f32 = 800.0;
const DEFAULT_COL_WIDTH: f32 = 450.0;
pub const CELL_HEIGHT: f32 = 180.0;
const MIN_FRAME_HEIGHT: f32 = 60.0;
const MAX_FRAME_HEIGHT: f32 = 600.0;
const DRAG_HANDLE_HEIGHT: f32 = 6.0;
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
enum ContextTarget {
    Cell(usize, usize),
    Header(usize),
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

pub struct ScenePanel {
    cursor: Option<(usize, usize)>,
    anchor: Option<(usize, usize)>,
    selection: BTreeSet<(usize, usize)>,
    clipboard: Vec<Frame>,
    context_target: Option<ContextTarget>,
    frame_states: HashMap<(usize, usize), InlineFrameState>,
    column_widths: Vec<f32>,
    currently_editing: Option<(usize, usize)>,
    edit_mode: bool,
    scroll_to_cursor: bool,
    last_line_count: usize,
    last_frame_counts: Vec<usize>,
    pub prelude_states: Vec<InlineScriptState>,
    pub prelude_collapsed: bool,
    pub prelude_col_width: f32,
}

impl Default for ScenePanel {
    fn default() -> Self {
        Self {
            cursor: None,
            anchor: None,
            selection: BTreeSet::new(),
            clipboard: Vec::new(),
            context_target: None,
            frame_states: HashMap::new(),
            column_widths: Vec::new(),
            currently_editing: None,
            edit_mode: false,
            scroll_to_cursor: false,
            last_line_count: 0,
            last_frame_counts: Vec::new(),
            prelude_states: Vec::new(),
            prelude_collapsed: true,
            prelude_col_width: 300.0,
        }
    }
}

impl ScenePanel {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        visuals_enabled: bool,
        scene_opacity: f32,
        editor_settings: &EditorSettings,
        pending_edits: Vec<(usize, usize, Vec<sova_server::TextOp>)>,
    ) {
        let Some(scene) = bridge.scene() else {
            ui.colored_label(egui::Color32::GRAY, t!("scene.no_scene"));
            return;
        };

        let default_lang = resolve_default_language(
            &editor_settings.default_language,
            bridge.languages(),
        );

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
                    ((elapsed % dur) / dur) as f32
                })
                .collect()
        };

        // Sync frame state lifecycle
        self.sync_frame_states(scene, bridge);

        // Sync prelude states
        self.sync_prelude_states(&scene.prelude);

        // Integrate pending script edits from peers
        for (li, fi, ops) in pending_edits {
            if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                for op in &ops {
                    state.integrate_remote_op(op);
                }
            }
        }

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

                    // Prelude column
                    self.show_prelude_column(ui, available_height, accent, &opacity, &theme, editor_settings, bridge, &default_lang);

                    for li in 0..scene.lines.len() {
                        let col_width = self.column_widths[li];
                        let line = &scene.lines[li];

                        let col_resp = ui.allocate_ui(egui::vec2(col_width, available_height), |ui| {
                            ui.vertical(|ui| {
                                // Line header
                                self.show_line_header(ui, li, line, accent, &opacity, bridge, &default_lang);

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
                                                &default_lang,
                                            );

                                            // Scroll cursor into view (only on cursor change)
                                            if is_cursor && self.scroll_to_cursor {
                                                cell_resp.scroll_to_me(Some(egui::Align::Center));
                                            }

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
                                                    &default_lang,
                                                );
                                            });

                                            ui.add_space(GAP);

                                            // Drag handle below every frame for vertical resizing (hidden when collapsed)
                                            let frame_collapsed = self.frame_states.get(&(li, fi))
                                                .is_some_and(|s| s.collapsed);
                                            if frame_collapsed {
                                                ui.add_space(DRAG_HANDLE_HEIGHT);
                                            } else {
                                                let handle_width = ui.available_width();
                                                let (handle_rect, handle_resp) = ui.allocate_exact_size(
                                                    egui::vec2(handle_width, DRAG_HANDLE_HEIGHT),
                                                    egui::Sense::drag(),
                                                );
                                                if handle_resp.dragged() {
                                                    let delta = handle_resp.drag_delta().y;
                                                    if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                                                        state.height = (state.height + delta).clamp(MIN_FRAME_HEIGHT, MAX_FRAME_HEIGHT);
                                                    }
                                                }
                                                if handle_resp.hovered() || handle_resp.dragged() {
                                                    let center_y = handle_rect.center().y;
                                                    ui.painter().hline(
                                                        handle_rect.x_range(),
                                                        center_y,
                                                        egui::Stroke::new(1.0, accent),
                                                    );
                                                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                                                }
                                            }
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
                                                new_frame(&default_lang),
                                                ActionTiming::Immediate,
                                            ));
                                        }
                                    });
                            });
                        });

                        // Scroll column into view horizontally (only on cursor change)
                        if self.scroll_to_cursor && self.cursor.is_some_and(|(cur_li, _)| cur_li == li) {
                            ui.scroll_to_rect(col_resp.response.rect, Some(egui::Align::Center));
                        }

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
                        let add_line_fill = ui.visuals().widgets.inactive.bg_fill.linear_multiply(0.5);
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new(crate::icons::ADD))
                                    .fill(add_line_fill)
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

        // Modal: detect click-into-editor entering/switching edit mode
        if let Some((li, fi)) = new_editing
            && (self.cursor != Some((li, fi)) || !self.edit_mode)
        {
            self.edit_mode = true;
            self.update_cursor((li, fi), bridge);
            self.selection.clear();
            self.selection.insert((li, fi));
        }

        // Modal: detect Escape from editor exiting edit mode
        if self.edit_mode {
            let escaped = self.frame_states.values().any(|s| s.escape_pressed);
            if escaped {
                self.edit_mode = false;
                for state in self.frame_states.values_mut() {
                    state.escape_pressed = false;
                }
            }
            if new_editing.is_none() && !escaped {
                self.edit_mode = false;
            }
        }

        // Navigation mode: process keyboard shortcuts (only when no text widget has focus)
        if !self.edit_mode && !ui.ctx().memory(|m| m.focused().is_some()) {
            self.handle_clipboard(ui, bridge);
            self.handle_keyboard(ui, bridge, &default_lang);
        }

        self.scroll_to_cursor = false;

        if has_positions {
            ui.ctx().request_repaint();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn show_line_header(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        line: &Line,
        accent: egui::Color32,
        opacity: &SceneOpacity,
        bridge: &ClientBridge,
        default_lang: &str,
    ) {
        let header_bg = opacity.fill(ui.visuals().faint_bg_color, 0.9);
        let header_frame = egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(4, 2))
            .fill(header_bg);

        // Pre-register click widget so inner buttons win hit-test ties
        let hdr_bg_id = ui.id().with(("line_hdr_bg", li));
        let pre_rect = ui.available_rect_before_wrap();
        ui.interact(pre_rect, hdr_bg_id, egui::Sense::click());

        let resp = header_frame
            .show(ui, |ui| {
                ui.set_height(LINE_HEADER_HEIGHT - 4.0);
                opacity.override_widget_visuals(ui);
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    // Left side: toggles
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
                                .prefix("speed: "),
                        );
                        if speed_resp.changed() && speed > 0.0 {
                            self.toggle_line_field(li, bridge, |l| l.speed_factor = speed);
                        }
                    });
                });
            })
            .response;

        // Re-register with actual rect for correct context_menu positioning
        let resp = ui.interact(resp.rect, hdr_bg_id, egui::Sense::click());

        // Right-click on header
        resp.context_menu(|ui| {
            self.show_context_menu(ui, Some(ContextTarget::Header(li)), bridge, default_lang);
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
        default_lang: &str,
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

        // Use push_id to scope all widget IDs within this frame cell
        let resp = ui.push_id(("frame_cell", li, fi), |ui| {
            // Pre-register a click-sensing widget BEFORE drawing content.
            // Inner buttons drawn later will win hit-test ties over this earlier widget.
            let bg_id = ui.id().with("cell_bg");
            let pre_rect = ui.available_rect_before_wrap();
            ui.interact(pre_rect, bg_id, egui::Sense::click());

            let cell_frame = egui::Frame::NONE
                .fill(bg)
                .inner_margin(egui::Margin { left: 5, right: 5, ..egui::Margin::ZERO });

            let is_collapsed = self.frame_states.get(&(li, fi))
                .is_some_and(|s| s.collapsed);

            let frame_resp = cell_frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                if is_collapsed {
                    ui.set_height(HEADER_HEIGHT);
                } else {
                    let frame_height = self.frame_states.get(&(li, fi))
                        .map_or(CELL_HEIGHT, |s| s.height);
                    ui.set_height(HEADER_HEIGHT + frame_height);
                }

                opacity.override_widget_visuals(ui);

                // Progress fill behind header widgets
                if is_playing && frame.enabled {
                    let header_rect = egui::Rect::from_min_size(
                        ui.min_rect().min,
                        egui::vec2(ui.available_width() * progress, HEADER_HEIGHT),
                    );
                    let blend = |a: u8, b: u8| -> u8 {
                        ((a as u16 * 2 + b as u16) / 3) as u8
                    };
                    let fill = egui::Color32::from_rgb(
                        blend(accent.r(), bg.r()),
                        blend(accent.g(), bg.g()),
                        blend(accent.b(), bg.b()),
                    );
                    ui.painter().rect_filled(header_rect, 0.0, fill);
                    ui.ctx().request_repaint();
                }

                // Header
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.set_height(HEADER_HEIGHT);
                    if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                        state.show_header(ui, li, fi, frame, opacity, bridge);
                    }
                });

                // Frame menu popup
                if self.frame_states.get(&(li, fi)).is_some_and(|s| s.menu_open) {
                    let popup_id = ui.id().with("frame_menu");
                    let popup_resp = egui::Area::new(popup_id)
                        .order(egui::Order::Foreground)
                        .fixed_pos(ui.cursor().min)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                ui.set_min_width(150.0);
                                if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                                    state.show_frame_menu(ui, li, fi, bridge, default_lang);
                                }
                            });
                        });

                    // Close on click outside popup or Escape
                    let clicked_outside = ui.input(|i| i.pointer.any_pressed())
                        && ui.input(|i| {
                            i.pointer
                                .interact_pos()
                                .is_some_and(|pos| !popup_resp.response.rect.contains(pos))
                        });
                    if (clicked_outside || ui.input(|i| i.key_pressed(egui::Key::Escape)))
                        && let Some(state) = self.frame_states.get_mut(&(li, fi))
                    {
                        state.menu_open = false;
                    }
                }

                if !is_collapsed {
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

                    let mut cursors: Vec<PeerCursor> = bridge
                        .text_cursors_for_frame(li, fi)
                        .into_iter()
                        .map(|(name, line, col)| PeerCursor {
                            name: name.to_owned(),
                            line,
                            col,
                            color: username_color(name),
                        })
                        .collect();

                    // Include the local user's text cursor
                    if let Some(my_name) = bridge.confirmed_username()
                        && let Some(state) = self.frame_states.get(&(li, fi))
                        && let (Some(line), Some(col)) =
                            (state.last_cursor_line, state.last_cursor_col)
                    {
                        cursors.push(PeerCursor {
                            name: my_name.to_owned(),
                            line,
                            col,
                            color: username_color(my_name),
                        });
                    }

                    let editor_ctx = EditorContext {
                        settings: editor_settings,
                        syntax: syntax_pair,
                        reference,
                        peer_cursors: &cursors,
                        opacity: Some(opacity),
                    };
                    if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                        state.show_body(ui, li, fi, &editor_ctx, bridge);
                    }
                }
            });

            let cell_rect = frame_resp.response.rect;

            // Playing indicator
            if is_playing && frame.enabled {
                let p = ui.painter();
                // Static accent strip on the left edge
                let accent_strip = egui::Rect::from_min_size(
                    cell_rect.min,
                    egui::vec2(3.0, cell_rect.height()),
                );
                p.rect_filled(accent_strip, 0.0, accent);

                // Background fill (top→bottom)
                let fill_h = cell_rect.height() * progress;
                let bg = egui::Rect::from_min_size(
                    cell_rect.min,
                    egui::vec2(cell_rect.width(), fill_h),
                );
                let bc = egui::Color32::from_rgba_unmultiplied(
                    accent.r(), accent.g(), accent.b(), 15,
                );
                p.rect_filled(bg, 0.0, bc);
                ui.ctx().request_repaint();
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

            // Peer presence: colored left bar for peers on this cell
            for (name, &(pli, pfi, _)) in bridge.peer_cursors() {
                if pli == li && pfi == fi {
                    let color = username_color(name);
                    let s = egui::Stroke::new(2.0, color);
                    ui.painter().vline(cell_rect.left(), cell_rect.y_range(), s);
                }
            }

            // Local user: colored left bar on cursor frame
            if is_cursor
                && let Some(my_name) = bridge.confirmed_username()
            {
                let color = username_color(my_name);
                let s = egui::Stroke::new(2.0, color);
                ui.painter().vline(cell_rect.left(), cell_rect.y_range(), s);
            }

            // Re-register with actual rect (updates in-place, keeping early list position)
            ui.interact(frame_resp.response.rect, bg_id, egui::Sense::click())
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
            // (indices may have shifted), but preserve UI-only fields (collapsed, height)
            for (li, &count) in current_counts.iter().enumerate() {
                let old_count = self.last_frame_counts.get(li).copied().unwrap_or(0);
                if count != old_count {
                    let saved_ui: Vec<_> = self
                        .frame_states
                        .iter()
                        .filter(|&(&(l, _), _)| l == li)
                        .map(|(&(_, fi), s)| (fi, s.collapsed, s.height))
                        .collect();
                    let keys_to_remove: Vec<_> = self
                        .frame_states
                        .keys()
                        .filter(|&&(l, _)| l == li)
                        .copied()
                        .collect();
                    for key in keys_to_remove {
                        self.frame_states.remove(&key);
                    }
                    // Restore collapsed/height for indices that still exist
                    for (fi, collapsed, height) in saved_ui {
                        if fi < count {
                            if let Some(frame) = scene.lines[li].frames.get(fi) {
                                let mut state = InlineFrameState::new(frame);
                                state.collapsed = collapsed;
                                state.height = height;
                                self.frame_states.insert((li, fi), state);
                            }
                        }
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
        if old != self.cursor {
            self.scroll_to_cursor = true;
            if bridge.is_connected() {
                bridge.send(ClientMessage::CursorPosition(new_cursor.0, new_cursor.1, None));
            }
        }
    }

    fn show_context_menu(
        &mut self,
        ui: &mut egui::Ui,
        target: Option<ContextTarget>,
        bridge: &ClientBridge,
        default_lang: &str,
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
                            li, fi, new_frame(default_lang), ActionTiming::Immediate,
                        ));
                        ui.close();
                    }
                    if ui.button(t!("scene.insert_frame_after")).clicked() {
                        bridge.send(ClientMessage::AddFrame(
                            li, fi + 1, new_frame(default_lang), ActionTiming::Immediate,
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

    fn handle_keyboard(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge, default_lang: &str) {
        let Some(scene) = bridge.scene() else {
            return;
        };
        let Some((li, fi)) = self.cursor else {
            // No cursor: only allow initial cursor placement
            let any_nav = ui.input(|i| {
                i.key_pressed(egui::Key::ArrowDown)
                    || i.key_pressed(egui::Key::J)
                    || i.key_pressed(egui::Key::ArrowRight)
                    || i.key_pressed(egui::Key::L)
            });
            if any_nav && !scene.lines.is_empty() {
                self.update_cursor((0, 0), bridge);
                self.selection.insert((0, 0));
                self.anchor = Some((0, 0));
            }
            return;
        };

        let num_lines = scene.lines.len();
        if num_lines == 0 {
            self.cursor = None;
            return;
        }
        let line_lens: Vec<usize> = scene.lines.iter().map(|l| l.frames.len()).collect();

        // Clamp cursor if scene shrank
        let li = li.min(num_lines - 1);
        let fi = fi.min(line_lens[li].saturating_sub(1));
        if (li, fi) != self.cursor.unwrap() {
            self.cursor = Some((li, fi));
        }

        // Read all keys
        let (
            up, down, left, right, shift,
            key_delete, cmd_d, cmd_shift_d, shift_i, cmd_shift_i,
            key_shift_j, key_shift_k,
            key_e, key_dot, key_comma,
            key_enter, key_i, key_escape,
            ctrl_a, ctrl_del, alt_h, alt_l,
        ) = ui.input(|i| {
            let no_mod = !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && !i.modifiers.shift;
            let shift_only = i.modifiers.shift && !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt;
            (
                // Movement (arrows + vim)
                i.key_pressed(egui::Key::ArrowUp) || (no_mod && i.key_pressed(egui::Key::K)),
                i.key_pressed(egui::Key::ArrowDown) || (no_mod && i.key_pressed(egui::Key::J)),
                i.key_pressed(egui::Key::ArrowLeft) || (no_mod && i.key_pressed(egui::Key::H)),
                i.key_pressed(egui::Key::ArrowRight) || (no_mod && i.key_pressed(egui::Key::L)),
                i.modifiers.shift,
                // Frame ops
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::D),
                i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::D),
                shift_only && i.key_pressed(egui::Key::I),
                i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::I),
                shift_only && i.key_pressed(egui::Key::J),
                shift_only && i.key_pressed(egui::Key::K),
                // Toggles
                no_mod && i.key_pressed(egui::Key::E),
                no_mod && i.key_pressed(egui::Key::Period),
                no_mod && i.key_pressed(egui::Key::Comma),
                // Mode switch
                i.key_pressed(egui::Key::Enter) && !i.modifiers.command && !i.modifiers.ctrl,
                no_mod && i.key_pressed(egui::Key::I),
                i.key_pressed(egui::Key::Escape),
                // Ctrl combos
                i.modifiers.command && i.key_pressed(egui::Key::A),
                i.modifiers.command && (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)),
                i.modifiers.alt && i.key_pressed(egui::Key::H),
                i.modifiers.alt && i.key_pressed(egui::Key::L),
            )
        });

        // Enter edit mode
        if (key_enter || key_i) && line_lens[li] > 0 {
            self.edit_mode = true;
            if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                state.request_focus = true;
            }
            return;
        }

        // Escape: clear selection
        if key_escape {
            self.selection.clear();
            self.cursor = None;
            return;
        }

        // Move frames: Shift+J/K
        if key_shift_j && !self.selection.is_empty() {
            self.move_frames_vertical(1, bridge);
        } else if key_shift_k && !self.selection.is_empty() {
            self.move_frames_vertical(-1, bridge);
        }

        // Move line: Alt+H/L
        if alt_h {
            self.move_line_horizontal(li, -1, bridge);
        } else if alt_l {
            self.move_line_horizontal(li, 1, bridge);
        }

        // Navigation
        if !key_shift_j && !key_shift_k {
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

        // Frame ops
        if key_delete && !ctrl_del {
            let mut to_remove: Vec<(usize, usize)> = self.selection.iter().copied().collect();
            to_remove.sort_by(|a, b| b.1.cmp(&a.1));
            for (rli, rfi) in to_remove {
                bridge.send(ClientMessage::RemoveFrame(rli, rfi, ActionTiming::Immediate));
            }
            self.selection.clear();
            self.cursor = None;
        }

        if cmd_d {
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

        if cmd_shift_d
            && let Some(frame_data) = scene.lines.get(li).and_then(|l| l.frames.get(fi))
        {
            bridge.send(ClientMessage::AddFrame(
                li, fi, frame_data.clone(), ActionTiming::Immediate,
            ));
        }

        if shift_i {
            bridge.send(ClientMessage::AddFrame(
                li, fi + 1, new_frame(default_lang), ActionTiming::Immediate,
            ));
        }

        if cmd_shift_i {
            bridge.send(ClientMessage::AddFrame(
                li, fi, new_frame(default_lang), ActionTiming::Immediate,
            ));
        }

        // Toggles
        if key_e {
            self.toggle_enabled(li, fi, bridge);
        }
        if key_dot {
            self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
        }
        if key_comma {
            self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
        }

        // Ctrl combos
        if ctrl_a {
            self.selection.clear();
            for f in 0..line_lens[li] {
                self.selection.insert((li, f));
            }
        }

        if ctrl_del {
            bridge.send(ClientMessage::RemoveLine(li, ActionTiming::Immediate));
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

    fn sync_prelude_states(&mut self, prelude: &[Script]) {
        // Grow or shrink to match
        while self.prelude_states.len() < prelude.len() {
            self.prelude_states
                .push(InlineScriptState::new(&prelude[self.prelude_states.len()]));
        }
        self.prelude_states.truncate(prelude.len());
        // Sync content from remote
        for (state, script) in self.prelude_states.iter_mut().zip(prelude.iter()) {
            state.sync_from_script(script);
        }
    }

    fn show_prelude_column(
        &mut self,
        ui: &mut egui::Ui,
        available_height: f32,
        accent: egui::Color32,
        opacity: &SceneOpacity,
        theme: &SyntaxTheme,
        editor_settings: &EditorSettings,
        bridge: &ClientBridge,
        default_lang: &str,
    ) {
        // Collapsed strip
        if self.prelude_collapsed {
            let strip_width = 24.0;
            ui.allocate_ui(egui::vec2(strip_width, available_height), |ui| {
                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    rect,
                    0.0,
                    opacity.fill(ui.visuals().faint_bg_color, 1.0),
                );

                // Click to expand
                let resp = ui.allocate_rect(rect, egui::Sense::click());
                if resp.clicked() {
                    self.prelude_collapsed = false;
                }
            });
            ui.add_space(DRAG_HANDLE_WIDTH);
            return;
        }

        let col_width = self.prelude_col_width;
        ui.allocate_ui(egui::vec2(col_width, available_height), |ui| {
            ui.vertical(|ui| {
                // Header — matches line header pattern
                let header_bg = opacity.fill(ui.visuals().faint_bg_color, 0.9);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .fill(header_bg)
                    .show(ui, |ui| {
                        opacity.override_widget_visuals(ui);
                        ui.set_height(LINE_HEADER_HEIGHT - 4.0);
                        ui.horizontal_centered(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;

                            // Collapse button
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(crate::icons::CHEVRON_DOWN)
                                            .small(),
                                    )
                                    .fill(egui::Color32::TRANSPARENT),
                                )
                                .clicked()
                            {
                                self.prelude_collapsed = true;
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(crate::icons::ADD)
                                                    .small(),
                                            )
                                            .fill(egui::Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        let mut scripts: Vec<Script> = bridge
                                            .scene()
                                            .map(|s| s.prelude.clone())
                                            .unwrap_or_default();
                                        scripts.push(Script::new(
                                            String::new(),
                                            default_lang.to_string(),
                                        ));
                                        bridge.send(ClientMessage::SchedulerControl(
                                            SchedulerMessage::SetScenePrelude(scripts),
                                        ));
                                    }
                                },
                            );
                        });
                    });

                // Script cells
                egui::ScrollArea::vertical()
                    .id_salt("prelude_scroll")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        let prelude_len = self.prelude_states.len();
                        for idx in 0..prelude_len {
                            ui.push_id(("prelude_cell", idx), |ui| {
                                let bg = opacity.fill(ui.visuals().faint_bg_color, 1.0);
                                let cell_frame = egui::Frame::NONE
                                    .fill(bg)
                                    .inner_margin(egui::Margin {
                                        left: 5,
                                        right: 5,
                                        ..egui::Margin::ZERO
                                    });

                                let frame_resp = cell_frame.show(ui, |ui| {
                                    let frame_height = self.prelude_states[idx].height;
                                    ui.set_width(ui.available_width());
                                    ui.set_height(HEADER_HEIGHT + frame_height);

                                    opacity.override_widget_visuals(ui);

                                    // Header
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        ui.set_height(HEADER_HEIGHT);
                                        self.prelude_states[idx].show_header(
                                            ui,
                                            idx,
                                            prelude_len,
                                            opacity,
                                            bridge,
                                        );
                                    });

                                    ui.separator();

                                    // Body (code editor)
                                    let syntax = bridge.syntax_map.get(
                                        self.prelude_states[idx].lang.as_str(),
                                    );
                                    let syntax_pair = syntax.map(|cs| (cs, theme));
                                    let reference = bridge
                                        .languages()
                                        .iter()
                                        .find(|l| l.name == self.prelude_states[idx].lang)
                                        .filter(|l| !l.documentation.reference.is_empty())
                                        .map(|l| &l.documentation.reference);
                                    let ctx = EditorContext {
                                        settings: editor_settings,
                                        syntax: syntax_pair,
                                        reference,
                                        peer_cursors: &[],
                                        opacity: Some(opacity),
                                    };
                                    self.prelude_states[idx].show_body(ui, idx, &ctx, bridge);
                                });

                                let cell_rect = frame_resp.response.rect;

                                // Height resize handle
                                let handle_rect = egui::Rect::from_min_size(
                                    egui::pos2(cell_rect.left(), cell_rect.bottom()),
                                    egui::vec2(cell_rect.width(), DRAG_HANDLE_HEIGHT),
                                );
                                let handle_resp =
                                    ui.allocate_rect(handle_rect, egui::Sense::drag());
                                if handle_resp.dragged() {
                                    self.prelude_states[idx].height =
                                        (self.prelude_states[idx].height
                                            + handle_resp.drag_delta().y)
                                            .clamp(MIN_FRAME_HEIGHT, MAX_FRAME_HEIGHT);
                                }
                                if handle_resp.hovered() || handle_resp.dragged() {
                                    let center_y = handle_rect.center().y;
                                    ui.painter().hline(
                                        handle_rect.x_range(),
                                        center_y,
                                        egui::Stroke::new(1.0, accent),
                                    );
                                    ui.ctx()
                                        .set_cursor_icon(egui::CursorIcon::ResizeVertical);
                                }

                                ui.add_space(GAP);
                            });
                        }

                        // Add script button (matches add frame button pattern)
                        ui.add_space(4.0);
                        let add_fill =
                            opacity.fill(ui.visuals().widgets.inactive.bg_fill, 0.5);
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("+").strong(),
                                )
                                .fill(add_fill)
                                .min_size(egui::vec2(ui.available_width(), 22.0)),
                            )
                            .clicked()
                        {
                            let mut scripts: Vec<Script> = bridge
                                .scene()
                                .map(|s| s.prelude.clone())
                                .unwrap_or_default();
                            scripts.push(Script::new(
                                String::new(),
                                default_lang.to_string(),
                            ));
                            bridge.send(ClientMessage::SchedulerControl(
                                SchedulerMessage::SetScenePrelude(scripts),
                            ));
                        }
                    });
            });
        });

        // Column resize drag handle
        let (drag_rect, drag_resp) = ui.allocate_exact_size(
            egui::vec2(DRAG_HANDLE_WIDTH, available_height),
            egui::Sense::drag(),
        );
        if drag_resp.dragged() {
            self.prelude_col_width = (self.prelude_col_width + drag_resp.drag_delta().x)
                .clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
        }
        if drag_resp.hovered() || drag_resp.dragged() {
            let center_x = drag_rect.center().x;
            ui.painter().vline(
                center_x,
                drag_rect.y_range(),
                egui::Stroke::new(1.0, accent),
            );
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
    }
}
