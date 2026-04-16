use eframe::egui;
use sova_core::scene::script::Script;
use sova_core::schedule::SchedulerMessage;
use sova_server::ClientMessage;

use super::classic_view::{MAX_COL_WIDTH, MIN_COL_WIDTH};
use super::{
    DRAG_HANDLE_HEIGHT, GAP, HEADER_HEIGHT, LINE_HEADER_HEIGHT, MAX_FRAME_HEIGHT, MIN_FRAME_HEIGHT,
    SceneRenderCtx,
};

const DRAG_HANDLE_WIDTH: f32 = 6.0;
use crate::client_bridge::ClientBridge;
use crate::theme::STROKE_HAIRLINE;
use crate::widgets::inline_scene_view::{InlineScriptState, show_lang_picker};
use crate::widgets::EditorContext;

pub(super) type HeadPlayback = (usize, usize, f32);

/// Returns `(is_playing, progress)` for frame `fi` in a line.
/// `heads` comes from the per-line slot in `head_progress`. Walks heads once
/// instead of building a `playing_fis: Vec<usize>` and re-filtering for
/// progress like the call sites used to.
pub(super) fn frame_playback(heads: Option<&Vec<HeadPlayback>>, fi: usize) -> (bool, f32) {
    let Some(heads) = heads else {
        return (false, 0.0);
    };
    let mut playing = false;
    let mut progress = 0.0_f32;
    for &(hfi, _, p) in heads {
        if hfi == fi {
            playing = true;
            progress = progress.max(p);
        }
    }
    (playing, progress)
}

/// Flattens a per-line head list down to the frame indices being played.
/// Downstream renderers (e.g. `InlineFrameState::show_header`) need a slice to
/// drive playing-indicator badges across multiple heads.
pub(super) fn playing_frame_indices(heads: Option<&Vec<HeadPlayback>>) -> Vec<usize> {
    heads
        .map(|h| h.iter().map(|&(fi, _, _)| fi).collect())
        .unwrap_or_default()
}

pub(super) fn current_repetition(heads: Option<&Vec<HeadPlayback>>, fi: usize) -> Option<usize> {
    heads.and_then(|heads| {
        heads
            .iter()
            .filter(|&&(hfi, _, _)| hfi == fi)
            .map(|&(_, rep, _)| rep)
            .max()
    })
}

/// Resize bar at a fixed `handle_rect`. Drag changes vertical layout (cell
/// height). Caller reads `dragged()` and `drag_delta().y` from the response.
pub(super) fn vertical_resize_handle(
    ui: &mut egui::Ui,
    handle_rect: egui::Rect,
    accent: egui::Color32,
) -> egui::Response {
    let resp = ui.allocate_rect(handle_rect, egui::Sense::drag());
    if resp.hovered() || resp.dragged() {
        let center_y = handle_rect.center().y;
        ui.painter().hline(
            handle_rect.x_range(),
            center_y,
            egui::Stroke::new(STROKE_HAIRLINE, accent),
        );
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }
    resp
}

/// Inline resize bar that advances layout by `width × height`. Drag changes
/// horizontal layout (column width). Caller reads `dragged()` and
/// `drag_delta().x` from the response.
pub(super) fn horizontal_resize_handle(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    accent: egui::Color32,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::drag());
    if resp.hovered() || resp.dragged() {
        let center_x = rect.center().x;
        ui.painter().vline(
            center_x,
            rect.y_range(),
            egui::Stroke::new(STROKE_HAIRLINE, accent),
        );
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    resp
}

/// Draw the multiplayer compilation + mutation flash overlays for a cell.
/// Peak alphas differ per view (classic vs sequencer), so they are passed in.
pub(super) fn draw_feedback_flashes(
    ui: &egui::Ui,
    li: usize,
    fi: usize,
    rect: egui::Rect,
    bridge: &ClientBridge,
    compile_peak: f32,
    mutation_peak: f32,
) {
    if let Some(&(success, instant)) = bridge.compilation_flashes().get(&(li, fi)) {
        let elapsed = instant.elapsed().as_secs_f32();
        if elapsed < 1.0 {
            let alpha = ((1.0 - elapsed) * compile_peak) as u8;
            let flash = if success {
                egui::Color32::from_rgba_unmultiplied(80, 200, 80, alpha)
            } else {
                egui::Color32::from_rgba_unmultiplied(200, 80, 80, alpha)
            };
            ui.painter().rect_filled(rect, 0.0, flash);
            ui.ctx().request_repaint();
        }
    }

    if let Some(instant) = bridge.mutation_flashes().get(&(li, fi)) {
        let elapsed = instant.elapsed().as_secs_f32();
        if elapsed < 1.2 {
            let alpha = ((1.0 - elapsed / 1.2) * mutation_peak) as u8;
            let flash = egui::Color32::from_rgba_unmultiplied(200, 200, 220, alpha);
            ui.painter().rect_filled(rect, 0.0, flash);
            ui.ctx().request_repaint();
        }
    }
}

impl super::ScenePanel {
    pub(super) fn sync_prelude_states(&mut self, prelude: &[Script]) {
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

    pub(super) fn show_prelude_column(
        &mut self,
        ui: &mut egui::Ui,
        available_height: f32,
        ctx: &SceneRenderCtx<'_>,
    ) {
        // Collapsed strip
        if self.prelude_collapsed {
            let strip_width = 24.0;
            ui.allocate_ui(egui::vec2(strip_width, available_height), |ui| {
                let rect = ui.available_rect_before_wrap();
                let strip_fill =
                    egui::Color32::from_rgba_unmultiplied(ctx.accent.r(), ctx.accent.g(), ctx.accent.b(), 96);
                ui.painter().rect_filled(rect, 0.0, strip_fill);

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
                // Header
                let header_bg = ctx.opacity.fill(ui.visuals().faint_bg_color, 0.9);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .fill(header_bg)
                    .show(ui, |ui| {
                        ctx.opacity.override_widget_visuals(ui);
                        ui.set_height(LINE_HEADER_HEIGHT - 4.0);
                        ui.horizontal_centered(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;

                            // Collapse button
                            if ui
                                .add(
                                    egui::Button::new(crate::icons::small(
                                        crate::icons::CHEVRON_DOWN,
                                    ))
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
                                            egui::Button::new(crate::icons::small(
                                                crate::icons::ADD,
                                            ))
                                            .fill(egui::Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        let mut scripts: Vec<Script> = ctx.bridge
                                            .scene()
                                            .map(|s| s.prelude.clone())
                                            .unwrap_or_default();
                                        scripts.push(Script::new(
                                            String::new(),
                                            ctx.default_lang.to_string(),
                                        ));
                                        ctx.bridge.send(ClientMessage::SchedulerControl(
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
                                let bg = ctx.opacity.fill(ui.visuals().faint_bg_color, 1.0);
                                let cell_frame =
                                    egui::Frame::NONE.fill(bg).inner_margin(egui::Margin {
                                        left: 5,
                                        right: 5,
                                        ..egui::Margin::ZERO
                                    });

                                let frame_resp = cell_frame.show(ui, |ui| {
                                    let frame_height = self.prelude_states[idx].height;
                                    ui.set_width(ui.available_width());
                                    ui.set_height(HEADER_HEIGHT + frame_height);

                                    ctx.opacity.override_widget_visuals(ui);

                                    // Header
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        ui.set_height(HEADER_HEIGHT);
                                        self.prelude_states[idx].show_header(
                                            ui,
                                            idx,
                                            prelude_len,
                                            ctx.bridge,
                                        );
                                    });

                                    ui.separator();

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
                                        // Body (code editor)
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
                                        self.prelude_states[idx]
                                            .show_body(ui, idx, &editor_ctx, ctx.bridge);
                                    }
                                });

                                let cell_rect = frame_resp.response.rect;

                                // Height resize handle
                                let handle_rect = egui::Rect::from_min_size(
                                    egui::pos2(cell_rect.left(), cell_rect.bottom()),
                                    egui::vec2(cell_rect.width(), DRAG_HANDLE_HEIGHT),
                                );
                                let handle_resp =
                                    vertical_resize_handle(ui, handle_rect, ctx.accent);
                                if handle_resp.dragged() {
                                    self.prelude_states[idx].height = (self.prelude_states[idx]
                                        .height
                                        + handle_resp.drag_delta().y)
                                        .clamp(MIN_FRAME_HEIGHT, MAX_FRAME_HEIGHT);
                                }

                                ui.add_space(GAP);
                            });
                        }

                        // Add script button
                        ui.add_space(4.0);
                        let add_fill =
                            ctx.opacity.fill(ui.visuals().widgets.inactive.bg_fill, 0.5);
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("+").strong())
                                    .fill(add_fill)
                                    .min_size(egui::vec2(ui.available_width(), 22.0)),
                            )
                            .clicked()
                        {
                            let mut scripts: Vec<Script> = ctx.bridge
                                .scene()
                                .map(|s| s.prelude.clone())
                                .unwrap_or_default();
                            scripts.push(Script::new(String::new(), ctx.default_lang.to_string()));
                            ctx.bridge.send(ClientMessage::SchedulerControl(
                                SchedulerMessage::SetScenePrelude(scripts),
                            ));
                        }
                    });
            });
        });

        // Column resize drag handle
        let drag_resp =
            horizontal_resize_handle(ui, DRAG_HANDLE_WIDTH, available_height, ctx.accent);
        if drag_resp.dragged() {
            self.prelude_col_width = (self.prelude_col_width + drag_resp.drag_delta().x)
                .clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
        }
    }
}
