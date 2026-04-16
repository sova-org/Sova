use eframe::egui;
use sova_core::scene::Scene;
use sova_core::schedule::{ActionTiming, SchedulerMessage};
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use crate::widgets::EditorSettings;
use crate::InputOwner;

use super::{
    KeyboardContext, Overlay, PendingDestructive, ScenePanel, SceneRenderCtx, SceneState,
    ViewMode, resolve_default_language,
};

impl ScenePanel {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        visuals_enabled: bool,
        scene_opacity: f32,
        editor_settings: &EditorSettings,
        pending_edits: Vec<(usize, usize, Vec<sova_server::TextOp>)>,
        sample_names: &[String],
        input_owner: InputOwner,
    ) {
        let Some(scene) = bridge.scene() else {
            ui.colored_label(egui::Color32::GRAY, t!("scene.no_scene"));
            return;
        };

        let default_lang =
            resolve_default_language(&editor_settings.default_language, bridge.languages());

        let has_positions = bridge.positions().iter().any(|p| !p.is_empty());
        let accent = ui.visuals().selection.bg_fill;
        let opacity = super::SceneOpacity::new(visuals_enabled, scene_opacity);

        let head_progress: Vec<Vec<super::prelude::HeadPlayback>> = {
            let beat = bridge.clock().beat;
            let positions = bridge.positions();
            let starts = bridge.position_start_beat();
            (0..scene.lines.len())
                .map(|li| {
                    let Some(heads) = positions.get(li) else {
                        return Vec::new();
                    };
                    let line = &scene.lines[li];
                    let head_starts = starts.get(li);
                    heads
                        .iter()
                        .enumerate()
                        .filter_map(|(hi, &(fi, rep))| {
                            let frame = line.frames.get(fi)?;
                            let start_beat =
                                head_starts.and_then(|s| s.get(hi)).copied().unwrap_or(beat);
                            let elapsed = (beat - start_beat).max(0.0);
                            let dur = frame.duration / line.speed_factor;
                            if dur <= 0.0 {
                                return None;
                            }
                            Some((fi, rep, ((elapsed % dur) / dur) as f32))
                        })
                        .collect()
                })
                .collect()
        };

        self.sync_frame_states(scene, bridge);
        self.sync_sequencer_inline_edit(scene);
        self.sync_sequencer_line_speed_focus(scene);
        self.sync_prelude_states(&scene.prelude);

        for (li, fi, ops) in pending_edits {
            if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                for op in &ops {
                    state.integrate_remote_op(op);
                }
            }
        }

        let theme = crate::widgets::syntax_highlight::SyntaxTheme::from_pref(
            editor_settings.syntax_theme,
        );
        let available_height = ui.available_height();

        if let Some((fli, ffi)) = self.state.focused_frame()
            && (fli >= scene.lines.len() || ffi >= scene.lines[fli].frames.len())
        {
            self.deselect_all();
        }

        let ctx = SceneRenderCtx {
            bridge,
            accent,
            opacity: &opacity,
            theme: &theme,
            editor_settings,
            default_lang: &default_lang,
            sample_names,
        };

        match self.view_mode {
            ViewMode::Classic => self.show_classic(ui, scene, &head_progress, &ctx, available_height),
            ViewMode::Sequencer => self.show_sequencer(ui, scene, &head_progress, &ctx),
        }

        if let Overlay::ContextMenu { target, pos, .. } = self.overlay {
            let popup_id = ui.id().with("scene_context_menu");
            let popup_resp = egui::Area::new(popup_id)
                .order(egui::Order::Foreground)
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(150.0);
                        self.show_context_menu(ui, Some(target), bridge, &default_lang);
                    });
                });

            if let Overlay::ContextMenu {
                just_opened: true, ..
            } = self.overlay
            {
                if let Overlay::ContextMenu {
                    ref mut just_opened,
                    ..
                } = self.overlay
                {
                    *just_opened = false;
                }
            } else if matches!(self.overlay, Overlay::ContextMenu { .. }) {
                let clicked_outside = ui.input(|i| i.pointer.any_pressed())
                    && ui.input(|i| {
                        i.pointer
                            .interact_pos()
                            .is_some_and(|p| !popup_resp.response.rect.contains(p))
                    });
                let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
                if clicked_outside || esc {
                    self.overlay = Overlay::None;
                }
            }
        }

        self.scroll_to_cursor = false;

        let frame_escaped = self.frame_states.values().any(|s| s.escape_pressed);
        let prelude_escaped = self.state.selected_prelude().is_some_and(|idx| {
            self.prelude_states
                .get(idx)
                .is_some_and(|s| s.escape_pressed)
        });
        let mut escape_this_frame = frame_escaped || prelude_escaped;

        if escape_this_frame && self.state.is_editing() {
            self.exit_edit_mode();
            self.scroll_to_cursor = true;
        }

        if self.state.is_editing() && !escape_this_frame {
            let raw_escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
            let no_focus = !ui.ctx().memory(|m| m.focused().is_some());
            let no_modal = !matches!(
                self.overlay,
                Overlay::ConfirmDialog { .. } | Overlay::ContextMenu { .. }
            ) && !matches!(
                self.active_keyboard_context(ui.ctx()),
                KeyboardContext::LangPicker
            );
            if raw_escape && no_focus && no_modal {
                self.exit_edit_mode();
                self.scroll_to_cursor = true;
                escape_this_frame = true;
            }
        }

        for state in self.frame_states.values_mut() {
            state.escape_pressed = false;
        }
        for state in &mut self.prelude_states {
            state.escape_pressed = false;
        }

        if !escape_this_frame {
            let clicked_editor = self.frame_states.iter().find_map(|(&pos, state)| {
                if state.editor_has_focus
                    && (self.state.cursor() != Some(pos) || !self.state.is_editing())
                {
                    Some(pos)
                } else {
                    None
                }
            });
            if let Some(pos) = clicked_editor {
                self.enter_frame_edit(pos);
            }
        }

        let current_editing = match self.state {
            SceneState::EditingFrame { cursor } => Some(cursor),
            _ => None,
        };
        if current_editing != self.prev_editing {
            if let Some((li, fi)) = self.prev_editing {
                bridge.send(ClientMessage::StoppedEditingFrame(li, fi));
            }
            if let Some((li, fi)) = current_editing {
                bridge.send(ClientMessage::StartedEditingFrame(li, fi));
            }
            self.prev_editing = current_editing;
        }

        if input_owner == InputOwner::Scene
            && matches!(
                self.active_keyboard_context(ui.ctx()),
                KeyboardContext::Navigating
            )
            && !escape_this_frame
        {
            self.handle_clipboard(ui, bridge);
            self.handle_keyboard(ui, bridge, &default_lang);
        }

        if has_positions {
            ui.ctx().request_repaint();
        }

        use crate::widgets::ConfirmAction;
        match self.confirm_dialog.show(ui.ctx()) {
            ConfirmAction::Confirmed => {
                if let Overlay::ConfirmDialog { action } =
                    std::mem::replace(&mut self.overlay, Overlay::None)
                {
                    match action {
                        PendingDestructive::RemoveLine(li) => {
                            let next = self.next_frame_after_line_removal(scene, li);
                            bridge.send(SchedulerMessage::RemoveLine(li, ActionTiming::Immediate));
                            self.restore_scene_state_after_frame_removal(next, bridge);
                        }
                        PendingDestructive::RemoveFrames(frames) => {
                            let mut to_remove = frames;
                            to_remove.sort_by(|a, b| b.1.cmp(&a.1));
                            let next = self.next_frame_after_removal(scene, &to_remove);
                            for (rli, rfi) in to_remove {
                                bridge.send(SchedulerMessage::RemoveFrame(
                                    rli,
                                    rfi,
                                    ActionTiming::Immediate,
                                ));
                            }
                            self.restore_scene_state_after_frame_removal(next, bridge);
                        }
                    }
                }
            }
            ConfirmAction::Cancelled => {
                self.overlay = Overlay::None;
            }
            ConfirmAction::None => {}
        }
    }

    pub(crate) fn show_focused_frame(
        &mut self,
        ui: &mut egui::Ui,
        fli: usize,
        ffi: usize,
        scene: &Scene,
        head_progress: &[Vec<super::prelude::HeadPlayback>],
        ctx: &SceneRenderCtx<'_>,
    ) {
        let line = &scene.lines[fli];
        let frame = &line.frames[ffi];
        let line_heads = head_progress.get(fli);
        let playing_fis = super::prelude::playing_frame_indices(line_heads);
        let (is_playing, progress) = super::prelude::frame_playback(line_heads, ffi);

        ui.horizontal(|ui| {
            ui.set_height(super::LINE_HEADER_HEIGHT);
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.label(
                egui::RichText::new(format!("Line {} / Frame {}", fli + 1, ffi + 1))
                    .small()
                    .color(ui.visuals().text_color()),
            );
            if ui
                .add(
                    egui::Button::new(
                        crate::icons::small(crate::icons::UNFOCUS).color(ui.visuals().text_color()),
                    )
                    .fill(egui::Color32::TRANSPARENT),
                )
                .on_hover_text("Exit focus mode (Esc)")
                .clicked()
            {
                self.deselect_all();
            }
        });

        if self.state.focused_frame().is_none() {
            return;
        }

        let available_height = ui.available_height();
        let available_width = ui.available_width();
        let body_height = available_height - super::HEADER_HEIGHT;
        let old_height = self
            .frame_states
            .get(&(fli, ffi))
            .map_or(super::CELL_HEIGHT, |s| s.height);
        if let Some(state) = self.frame_states.get_mut(&(fli, ffi)) {
            state.height = body_height;
            state.collapsed = false;
        }

        self.frame_states
            .entry((fli, ffi))
            .or_insert_with(|| crate::widgets::inline_scene_view::InlineFrameState::new(frame));

        let frame_ctx = super::classic_view::FrameCellCtx {
            pos: (fli, ffi),
            n_frames: line.frames.len(),
            frame,
            is_playing,
            progress,
            is_selected: true,
            is_cursor: true,
            playing_fis: &playing_fis,
            accent: crate::theme::cycled_accent(ctx.accent, fli),
        };

        ui.push_id("focused_frame", |ui| {
            ui.allocate_ui(egui::vec2(available_width, available_height), |ui| {
                let cell_resp = self.show_frame_cell(ui, &frame_ctx, ctx);

                if self
                    .frame_states
                    .get(&(fli, ffi))
                    .is_some_and(|s| s.focus_toggled)
                {
                    if let Some(state) = self.frame_states.get_mut(&(fli, ffi)) {
                        state.focus_toggled = false;
                    }
                    self.deselect_all();
                }

                if cell_resp.clicked() {
                    self.navigate_to_frame((fli, ffi), ctx.bridge);
                    self.selection.clear();
                    self.selection.insert((fli, ffi));
                }
            });
        });

        if let Some(state) = self.frame_states.get_mut(&(fli, ffi)) {
            state.height = old_height;
        }
    }
}
