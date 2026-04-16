use eframe::egui;
use sova_core::scene::Scene;

use super::prelude::{HeadPlayback, frame_playback, playing_frame_indices};
use super::{HEADER_HEIGHT, SceneRenderCtx};
use crate::theme::{COLOR_MUTED, username_color};
use crate::widgets::inline_scene_view::{InlineFrameState, show_lang_picker};
use crate::widgets::{EditorContext, PeerCursor};

impl super::ScenePanel {
    pub(super) fn show_sequencer(
        &mut self,
        ui: &mut egui::Ui,
        scene: &Scene,
        head_progress: &[Vec<HeadPlayback>],
        ctx: &SceneRenderCtx<'_>,
    ) {
        // Auto-select first frame if no cursor is set
        if matches!(self.state, super::SceneState::Empty)
            && !scene.lines.is_empty()
            && !scene.lines[0].frames.is_empty()
        {
            self.enter_frame_edit((0, 0));
            self.selection.insert((0, 0));
            self.anchor = Some((0, 0));
        }

        // If focused, delegate to focused frame view
        if let Some((fli, ffi)) = self.state.focused_frame() {
            self.show_focused_frame(ui, fli, ffi, scene, head_progress, ctx);
            return;
        }

        let panel_h = ui.available_height();
        ui.allocate_ui(egui::vec2(ui.available_width(), panel_h), |ui| {
            if self.state.shows_sequencer_grid() {
                egui::TopBottomPanel::bottom(ui.id().with("sequencer_hud"))
                    .show_separator_line(true)
                    .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(6, 2)))
                    .show_inside(ui, |ui| {
                        self.show_hud_bar(ui);
                    });

                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show_inside(ui, |ui| {
                        self.show_grid_panel(ui, scene, head_progress, ctx);
                    });
            } else {
                self.show_editor_panel(ui, scene, head_progress, ctx);
            }
        });
    }

    fn show_grid_panel(
        &mut self,
        ui: &mut egui::Ui,
        scene: &Scene,
        head_progress: &[Vec<HeadPlayback>],
        ctx: &SceneRenderCtx<'_>,
    ) {
        let bg = ctx.opacity.fill(ui.visuals().extreme_bg_color, 1.0);
        egui::Frame::NONE
            .fill(bg)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ctx.opacity.override_widget_visuals(ui);
                self.show_tile_grid(
                    ui,
                    scene,
                    head_progress,
                    ctx.accent,
                    ctx.opacity,
                    ctx.bridge,
                    ctx.default_lang,
                );
            });
    }

    fn show_editor_panel(
        &mut self,
        ui: &mut egui::Ui,
        scene: &Scene,
        head_progress: &[Vec<HeadPlayback>],
        ctx: &SceneRenderCtx<'_>,
    ) {
        // Prelude editor takes priority when a prelude script is selected
        if let Some(idx) = self.state.selected_prelude() {
            self.show_prelude_editor(ui, idx, ctx);
            return;
        }

        let Some((li, fi)) = self.state.cursor() else {
            // No selection: show empty state
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new(t!("scene.select_frame"))
                        .color(COLOR_MUTED)
                        .italics(),
                );
            });
            return;
        };

        // Validate cursor still exists
        let Some(line) = scene.lines.get(li) else {
            return;
        };
        let Some(frame) = line.frames.get(fi) else {
            return;
        };

        let line_heads = head_progress.get(li);
        let playing_fis = playing_frame_indices(line_heads);
        let (is_playing, progress) = frame_playback(line_heads, fi);

        let line_accent = crate::theme::cycled_accent(ctx.accent, li);

        // Ensure frame state exists
        self.frame_states
            .entry((li, fi))
            .or_insert_with(|| InlineFrameState::new(frame));
        if std::mem::take(&mut self.open_picker_on_cursor) {
            self.frame_states
                .get_mut(&(li, fi))
                .expect("just inserted via entry().or_insert_with()")
                .lang_picker_open = true;
        }

        // Editor panel background
        let bg = ctx.opacity.fill(ui.visuals().extreme_bg_color, 1.0);
        egui::Frame::NONE
            .fill(bg)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ctx.opacity.override_widget_visuals(ui);

                // Progress fill behind header
                if is_playing && frame.enabled {
                    let header_rect = egui::Rect::from_min_size(
                        ui.min_rect().min,
                        egui::vec2(ui.available_width() * progress, HEADER_HEIGHT),
                    );
                    let blend = |a: u8, b: u8| -> u8 { ((a as u16 * 2 + b as u16) / 3) as u8 };
                    let fill = egui::Color32::from_rgb(
                        blend(line_accent.r(), bg.r()),
                        blend(line_accent.g(), bg.g()),
                        blend(line_accent.b(), bg.b()),
                    );
                    ui.painter().rect_filled(header_rect, 0.0, fill);
                    ui.ctx().request_repaint();
                }

                // Header: breadcrumb + frame controls
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.set_height(HEADER_HEIGHT);

                    // Breadcrumb
                    ui.label(
                        egui::RichText::new(format!("Line {} / Frame {}", li + 1, fi + 1))
                            .small()
                            .color(COLOR_MUTED),
                    );

                    // Line frame range
                    let n = line.frames.len();
                    let (start_changed, new_start) = ui
                        .scope(|ui| {
                            if line.start_frame.is_none() {
                                ui.visuals_mut().override_text_color = Some(COLOR_MUTED);
                            }
                            let mut val = line.start_frame.unwrap_or(0);
                            let changed = ui
                                .add(
                                    egui::DragValue::new(&mut val)
                                        .range(0..=n.saturating_sub(1))
                                        .speed(1.0)
                                        .prefix("s: "),
                                )
                                .changed();
                            (changed, val)
                        })
                        .inner;
                    if start_changed {
                        self.toggle_line_field(li, ctx.bridge, |l| l.start_frame = Some(new_start));
                    }
                    let (end_changed, new_end) = ui
                        .scope(|ui| {
                            if line.end_frame.is_none() {
                                ui.visuals_mut().override_text_color = Some(COLOR_MUTED);
                            }
                            let mut val = line.end_frame.unwrap_or(n.saturating_sub(1));
                            let changed = ui
                                .add(
                                    egui::DragValue::new(&mut val)
                                        .range(0..=n.saturating_sub(1))
                                        .speed(1.0)
                                        .prefix("e: "),
                                )
                                .changed();
                            (changed, val)
                        })
                        .inner;
                    if end_changed {
                        self.toggle_line_field(li, ctx.bridge, |l| l.end_frame = Some(new_end));
                    }

                    // Reuse InlineFrameState header (language, duration, reps, etc.)
                    let is_focused = self.state.focused_frame() == Some((li, fi));
                    if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                        state.show_header(
                            ui,
                            li,
                            fi,
                            line.frames.len(),
                            &playing_fis,
                            line_accent,
                            frame,
                            ctx.bridge,
                            is_focused,
                            true,
                        );
                    }
                });

                // Frame menu popup: reuse unified context menu
                let menu_open = self
                    .frame_states
                    .get_mut(&(li, fi))
                    .is_some_and(|s| std::mem::take(&mut s.menu_open));
                if menu_open {
                    // Ensure cursor/selection are set for this frame
                    if !self.selection.contains(&(li, fi)) {
                        self.selection.clear();
                        self.selection.insert((li, fi));
                        self.anchor = Some((li, fi));
                    }
                    self.open_context_menu(super::ContextTarget::Cell(li, fi), ui.cursor().min);
                }

                // Handle focus toggle from header button
                let focus_toggled = self
                    .frame_states
                    .get_mut(&(li, fi))
                    .is_some_and(|s| std::mem::take(&mut s.focus_toggled));
                if focus_toggled {
                    self.enter_focus_mode((li, fi));
                }

                // Cmd+Left/Right/T/R: cycle frames and open header fields.
                // Only active when the editor owns input (editing context).
                if matches!(
                    self.active_keyboard_context(ui.ctx()),
                    super::KeyboardContext::Editing
                ) {
                    let (cycle_prev, cycle_next, open_dur, open_reps) = ui.input(|i| {
                        (
                            i.modifiers.command && i.key_pressed(egui::Key::ArrowLeft),
                            i.modifiers.command && i.key_pressed(egui::Key::ArrowRight),
                            i.modifiers.command && i.key_pressed(egui::Key::T),
                            i.modifiers.command && i.key_pressed(egui::Key::R),
                        )
                    });
                    if cycle_prev && fi > 0 {
                        self.enter_frame_edit((li, fi - 1));
                    } else if cycle_next && fi + 1 < line.frames.len() {
                        self.enter_frame_edit((li, fi + 1));
                    }
                    if (open_dur || open_reps)
                        && let Some(state) = self.frame_states.get_mut(&(li, fi))
                    {
                        state.focus_request = if open_dur {
                            crate::widgets::inline_scene_view::FocusRequest::Duration
                        } else {
                            crate::widgets::inline_scene_view::FocusRequest::Repetitions
                        };
                    }
                }

                // Body: language picker or code editor
                let picker_is_open = self
                    .frame_states
                    .get(&(li, fi))
                    .is_some_and(|s| s.lang_picker_open);

                // Ensure editor body is visible (don't write height — classic
                // mode owns that field; sequencer fills available space via layout)
                if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                    state.collapsed = false;
                }

                if picker_is_open {
                    if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                        if let Some(lang) = show_lang_picker(
                            ui,
                            &mut state.lang_picker_open,
                            &mut state.lang_picker_filter,
                            &mut state.lang_picker_selection,
                            &state.lang,
                            line_accent,
                            ctx.bridge,
                        ) {
                            state.lang = lang;
                            state.dirty = true;
                            state.focus_request =
                                crate::widgets::inline_scene_view::FocusRequest::Editor;
                        }
                        if !state.lang_picker_open {
                            state.focus_request =
                                crate::widgets::inline_scene_view::FocusRequest::Editor;
                        }
                    }
                } else {
                    // Code editor
                    let syntax = ctx.bridge.syntax_map.get(
                        self.frame_states
                            .get(&(li, fi))
                            .map(|s| s.lang.as_str())
                            .unwrap_or(""),
                    );
                    let syntax_pair = syntax.map(|cs| (cs, ctx.theme));

                    let reference = ctx.bridge
                        .languages()
                        .iter()
                        .find(|l| {
                            self.frame_states
                                .get(&(li, fi))
                                .is_some_and(|s| s.lang == l.name)
                        })
                        .filter(|l| !l.documentation.reference.is_empty())
                        .map(|l| &l.documentation.reference);

                    let mut cursors: Vec<PeerCursor> = ctx.bridge
                        .text_cursors_for_frame(li, fi)
                        .into_iter()
                        .map(|(name, line, col)| PeerCursor {
                            name: name.to_owned(),
                            line,
                            col,
                            color: username_color(name),
                        })
                        .collect();

                    if let Some(my_name) = ctx.bridge.confirmed_username()
                        && let Some(state) = self.frame_states.get(&(li, fi))
                        && let Some((cursor_line, cursor_col)) = state.last_cursor
                    {
                        cursors.push(PeerCursor {
                            name: my_name.to_owned(),
                            line: cursor_line,
                            col: cursor_col,
                            color: username_color(my_name),
                        });
                    }

                    let editor_ctx = EditorContext {
                        settings: ctx.editor_settings,
                        syntax: syntax_pair,
                        reference,
                        peer_cursors: &cursors,
                        annotations: if self.frame_states.get(&(li, fi)).is_some_and(|s| s.dirty) {
                            &[]
                        } else {
                            ctx.bridge.frame_annotations(li, fi)
                        },
                        opacity: Some(ctx.opacity),
                        sample_names: ctx.sample_names,
                    };
                    if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                        state.show_body(ui, li, fi, &editor_ctx, ctx.bridge);
                    }
                }

                // Flush pending CRDT ops
                if let Some(state) = self.frame_states.get_mut(&(li, fi))
                    && state.dirty
                {
                    state.compute_diff_ops();
                    state.flush_pending_ops(li, fi, ctx.bridge);
                }
            });
    }

    fn show_prelude_editor(
        &mut self,
        ui: &mut egui::Ui,
        idx: usize,
        ctx: &SceneRenderCtx<'_>,
    ) {
        // Validate index
        if idx >= self.prelude_states.len() {
            self.deselect_all();
            return;
        }

        let bg = ctx.opacity.fill(ui.visuals().extreme_bg_color, 1.0);
        egui::Frame::NONE
            .fill(bg)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ctx.opacity.override_widget_visuals(ui);

                // Header
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.set_height(HEADER_HEIGHT);

                    ui.label(
                        egui::RichText::new(format!("Prelude / Script {}", idx + 1))
                            .small()
                            .color(COLOR_MUTED),
                    );

                    ui.separator();

                    let prelude_len = self.prelude_states.len();
                    self.prelude_states[idx].show_header(ui, idx, prelude_len, ctx.bridge);
                });

                ui.separator();

                // Set height to fill available space
                let body_height = ui.available_height();
                self.prelude_states[idx].height = body_height;

                // Language picker or code editor
                if self.prelude_states[idx].lang_picker_open {
                    let state = &mut self.prelude_states[idx];
                    if let Some(lang) = show_lang_picker(
                        ui,
                        &mut state.lang_picker_open,
                        &mut state.lang_picker_filter,
                        &mut state.lang_picker_selection,
                        &state.lang,
                        ctx.accent,
                        ctx.bridge,
                    ) {
                        state.lang = lang;
                        state.dirty = true;
                        state.request_focus = true;
                    }
                    if !self.prelude_states[idx].lang_picker_open {
                        self.prelude_states[idx].request_focus = true;
                    }
                } else {
                    let syntax = ctx.bridge
                        .syntax_map
                        .get(self.prelude_states[idx].lang.as_str());
                    let syntax_pair = syntax.map(|cs| (cs, ctx.theme));
                    let reference = ctx.bridge
                        .languages()
                        .iter()
                        .find(|l| l.name == self.prelude_states[idx].lang)
                        .filter(|l| !l.documentation.reference.is_empty())
                        .map(|l| &l.documentation.reference);
                    let editor_ctx = EditorContext {
                        settings: ctx.editor_settings,
                        syntax: syntax_pair,
                        reference,
                        peer_cursors: &[],
                        annotations: &[],
                        opacity: Some(ctx.opacity),
                        sample_names: ctx.sample_names,
                    };
                    self.prelude_states[idx].show_body(ui, idx, &editor_ctx, ctx.bridge);
                }
            });
    }
}
