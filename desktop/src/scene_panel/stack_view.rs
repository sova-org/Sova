use eframe::egui;
use sova_core::scene::{Frame, Line, Scene};
use sova_core::schedule::ActionTiming;
use sova_core::schedule::SchedulerMessage;

use super::prelude::{
    HeadPlayback, frame_playback, horizontal_resize_handle, paint_progress_fill,
    playing_frame_indices, vertical_resize_handle,
};
use super::{
    CELL_HEIGHT, ContextTarget, DRAG_HANDLE_HEIGHT, GAP, HEADER_HEIGHT, LINE_HEADER_HEIGHT,
    MAX_FRAME_HEIGHT, MIN_FRAME_HEIGHT, new_frame,
};

pub(super) const MIN_COL_WIDTH: f32 = 120.0;
pub(super) const MAX_COL_WIDTH: f32 = 800.0;
const DRAG_HANDLE_WIDTH: f32 = 6.0;
use super::SceneRenderCtx;
use crate::theme::{COLOR_MUTED, COLOR_OK, STROKE_EMPHASIS, username_color};
use crate::widgets::inline_scene_view::InlineFrameState;
use crate::widgets::{EditorContext, PeerCursor};

pub(crate) struct FrameCellCtx<'a> {
    pub pos: (usize, usize),
    pub n_frames: usize,
    pub frame: &'a Frame,
    pub is_playing: bool,
    pub progress: f32,
    pub is_selected: bool,
    pub is_cursor: bool,
    pub playing_fis: &'a [usize],
    pub accent: egui::Color32,
}

impl super::ScenePanel {
    pub(super) fn show_stack(
        &mut self,
        ui: &mut egui::Ui,
        scene: &Scene,
        head_progress: &[Vec<HeadPlayback>],
        ctx: &SceneRenderCtx<'_>,
        available_height: f32,
    ) {
        // Column widths are stack-view-only state. Sync once on entry so the
        // ScenePanel dispatcher in mod.rs doesn't need to know about them.
        while self.column_widths.len() < scene.lines.len() {
            self.column_widths.push(super::DEFAULT_COL_WIDTH);
        }
        self.column_widths.truncate(scene.lines.len());

        if let Some((fli, ffi)) = self.state.focused_frame() {
            self.show_focused_frame(ui, fli, ffi, scene, head_progress, ctx);
        } else {
            egui::ScrollArea::horizontal()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                        // Prelude column
                        self.show_prelude_column(ui, available_height, ctx);

                        for li in 0..scene.lines.len() {
                            ui.push_id(("line_col", li), |ui| {
                                let col_width = self.column_widths[li];
                                let line = &scene.lines[li];

                                let col_resp =
                                    ui.allocate_ui(egui::vec2(col_width, available_height), |ui| {
                                        ui.vertical(|ui| {
                                            // Line header
                                            self.show_line_header(ui, li, line, ctx);
                                            ui.add_space(4.0);

                                            // Independent vertical scroll for frames
                                            egui::ScrollArea::vertical()
                                                .id_salt(("line_scroll", li))
                                                .auto_shrink(false)
                                                .show(ui, |ui| {
                                                    let line_heads = head_progress.get(li);
                                                    let playing_fis =
                                                        playing_frame_indices(line_heads);

                                                    for fi in 0..line.frames.len() {
                                                        let frame = &line.frames[fi];
                                                        let (is_playing, line_progress) =
                                                            frame_playback(line_heads, fi);
                                                        let is_selected =
                                                            self.selection.contains(&(li, fi));
                                                        let is_cursor =
                                                            self.state.cursor() == Some((li, fi));

                                                        // Ensure frame state exists
                                                        let state_key = (li, fi);
                                                        self.frame_states
                                                            .entry(state_key)
                                                            .or_insert_with(|| {
                                                                InlineFrameState::new(frame)
                                                            });
                                                        if self.state.cursor() == Some(state_key)
                                                            && std::mem::take(
                                                                &mut self.open_picker_on_cursor,
                                                            )
                                                        {
                                                            self.frame_states
                                                                .get_mut(&state_key)
                                                                .expect("just inserted via entry().or_insert_with()")
                                                                .lang_picker_open = true;
                                                        }

                                                        let frame_ctx = FrameCellCtx {
                                                            pos: (li, fi),
                                                            n_frames: line.frames.len(),
                                                            frame,
                                                            is_playing,
                                                            progress: line_progress,
                                                            is_selected,
                                                            is_cursor,
                                                            playing_fis: &playing_fis,
                                                            accent: crate::theme::cycled_accent(
                                                                ctx.accent,
                                                                li,
                                                            ),
                                                        };
                                                        let cell_resp =
                                                            self.show_frame_cell(ui, &frame_ctx, ctx);

                                                        // Scroll cursor into view
                                                        if is_cursor && self.scroll_to_cursor {
                                                            cell_resp.scroll_to_me(Some(
                                                                egui::Align::Center,
                                                            ));
                                                        }

                                                        // Handle focus toggle from header button
                                                        if self
                                                            .frame_states
                                                            .get(&(li, fi))
                                                            .is_some_and(|s| s.focus_toggled)
                                                        {
                                                            if let Some(state) =
                                                                self.frame_states.get_mut(&(li, fi))
                                                            {
                                                                state.focus_toggled = false;
                                                            }
                                                            self.enter_focus_mode((li, fi));
                                                        }

                                                        // Click: update cursor and selection
                                                        if cell_resp.clicked() {
                                                            let shift =
                                                                ui.input(|i| i.modifiers.shift);
                                                            if shift
                                                                && self
                                                                    .anchor
                                                                    .is_some_and(|(al, _)| al == li)
                                                            {
                                                                self.extend_selection((li, fi));
                                                            } else {
                                                                self.update_cursor(
                                                                    (li, fi),
                                                                    ctx.bridge,
                                                                );
                                                                self.selection.clear();
                                                                self.selection.insert((li, fi));
                                                                self.anchor = Some((li, fi));
                                                            }
                                                        }

                                                        // Secondary click (context menu)
                                                        if cell_resp.secondary_clicked() {
                                                            self.update_cursor((li, fi), ctx.bridge);
                                                            if !self.selection.contains(&(li, fi)) {
                                                                self.selection.clear();
                                                                self.selection.insert((li, fi));
                                                                self.anchor = Some((li, fi));
                                                            }
                                                            let pos = ui
                                                                .input(|i| i.pointer.interact_pos())
                                                                .unwrap_or(cell_resp.rect.center());
                                                            self.open_context_menu(
                                                                ContextTarget::Cell(li, fi),
                                                                pos,
                                                            );
                                                        }

                                                        // Drag handle for frame height
                                                        if !self
                                                            .frame_states
                                                            .get(&(li, fi))
                                                            .is_some_and(|s| s.collapsed)
                                                        {
                                                            let handle_rect =
                                                                egui::Rect::from_min_size(
                                                                    egui::pos2(
                                                                        cell_resp.rect.left(),
                                                                        cell_resp.rect.bottom(),
                                                                    ),
                                                                    egui::vec2(
                                                                        cell_resp.rect.width(),
                                                                        DRAG_HANDLE_HEIGHT,
                                                                    ),
                                                                );
                                                            let handle_resp =
                                                                vertical_resize_handle(
                                                                    ui,
                                                                    handle_rect,
                                                                    ctx.accent,
                                                                );
                                                            if handle_resp.dragged()
                                                                && let Some(state) = self
                                                                    .frame_states
                                                                    .get_mut(&(li, fi))
                                                            {
                                                                state.height = (state.height
                                                                    + handle_resp.drag_delta().y)
                                                                    .clamp(
                                                                        MIN_FRAME_HEIGHT,
                                                                        MAX_FRAME_HEIGHT,
                                                                    );
                                                            }
                                                        }

                                                        ui.add_space(GAP);
                                                    }

                                                    // Add frame button at bottom
                                                    ui.add_space(4.0);
                                                    let add_fill = ctx.opacity.fill(
                                                        ui.visuals().widgets.inactive.bg_fill,
                                                        0.5,
                                                    );
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                egui::RichText::new("+").strong(),
                                                            )
                                                            .fill(add_fill)
                                                            .min_size(egui::vec2(
                                                                ui.available_width(),
                                                                22.0,
                                                            )),
                                                        )
                                                        .clicked()
                                                    {
                                                        let new_fi = line.frames.len();
                                                        ctx.bridge.send(SchedulerMessage::AddFrame(
                                                            li,
                                                            new_fi,
                                                            new_frame(ctx.default_lang),
                                                            ActionTiming::Immediate,
                                                        ));
                                                        self.navigate_to_frame(
                                                            (li, new_fi),
                                                            ctx.bridge,
                                                        );
                                                    }
                                                });
                                        });
                                    });

                                // Scroll column into view horizontally
                                if self.scroll_to_cursor
                                    && self.state.cursor().is_some_and(|(cur_li, _)| cur_li == li)
                                {
                                    ui.scroll_to_rect(
                                        col_resp.response.rect,
                                        Some(egui::Align::Center),
                                    );
                                }

                                // Column resize drag handle
                                let drag_resp = horizontal_resize_handle(
                                    ui,
                                    DRAG_HANDLE_WIDTH,
                                    available_height,
                                    ctx.accent,
                                );
                                if drag_resp.dragged() {
                                    self.column_widths[li] = (self.column_widths[li]
                                        + drag_resp.drag_delta().x)
                                        .clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
                                }

                                // Line header right-click is handled inside show_line_header
                            });
                        }

                        // Add line button
                        ui.add_space(4.0);
                        let add_fill = ctx.opacity.fill(ui.visuals().widgets.inactive.bg_fill, 0.5);
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("+").strong())
                                    .fill(add_fill)
                                    .min_size(egui::vec2(40.0, available_height.min(80.0))),
                            )
                            .clicked()
                        {
                            let new_li = scene.lines.len();
                            ctx.bridge.send(SchedulerMessage::AddLine(
                                new_li,
                                Line::new(vec![1.0]),
                                ActionTiming::Immediate,
                            ));
                            self.navigate_to_frame((new_li, 0), ctx.bridge);
                        }
                    });
                });
        }
    }

    pub(super) fn show_line_header(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        line: &Line,
        ctx: &SceneRenderCtx<'_>,
    ) {
        let header_bg = ctx.opacity.fill(ui.visuals().faint_bg_color, 0.9);
        let header_frame = egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(4, 2))
            .fill(header_bg);

        // Pre-register click widget so inner buttons win hit-test ties
        let hdr_bg_id = ui.id().with(("line_hdr_bg", li));
        let pre_rect = ui.available_rect_before_wrap();
        ui.interact(pre_rect, hdr_bg_id, egui::Sense::click());

        let resp = header_frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(LINE_HEADER_HEIGHT - 4.0);
            ctx.opacity.override_widget_visuals(ui);
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;

                // Left side: toggles
                self.line_flag_button(
                    ui,
                    li,
                    line.looping,
                    ctx.accent,
                    ctx.bridge,
                    t!("scene.toggle_looping"),
                    |ui, c| {
                        crate::icons::text_colored(
                            ui,
                            egui::TextStyle::Small,
                            crate::icons::LOOPING,
                            "Loop",
                            c,
                        )
                    },
                    |l| l.looping = !l.looping,
                );
                self.line_flag_button(
                    ui,
                    li,
                    line.trailing,
                    ctx.accent,
                    ctx.bridge,
                    t!("scene.toggle_trailing"),
                    |ui, c| {
                        crate::icons::text_colored(
                            ui,
                            egui::TextStyle::Small,
                            crate::icons::TRAILING,
                            "Trail",
                            c,
                        )
                    },
                    |l| l.trailing = !l.trailing,
                );
                self.line_flag_button(
                    ui,
                    li,
                    line.manual,
                    ctx.accent,
                    ctx.bridge,
                    t!("scene.toggle_manual"),
                    |ui, c| {
                        crate::icons::text_colored(
                            ui,
                            egui::TextStyle::Small,
                            crate::icons::MANUAL,
                            "Manual",
                            c,
                        )
                    },
                    |l| l.manual = !l.manual,
                );

                // Peer editing this line indicator
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let any_peer_editing = (0..ctx
                        .bridge
                        .scene()
                        .map(|s| s.lines.get(li).map(|l| l.frames.len()).unwrap_or(0))
                        .unwrap_or(0))
                        .any(|fi| !ctx.bridge.editing_peers_for_frame(li, fi).is_empty());
                    if any_peer_editing {
                        ui.add(
                            egui::Label::new(
                                crate::icons::rich(crate::icons::CIRCLE_LARGE_FILLED)
                                    .color(COLOR_OK),
                            )
                            .selectable(false),
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
                    if speed_resp.changed() {
                        self.toggle_line_field(li, ctx.bridge, |l| l.speed_factor = speed);
                    }

                    // End frame (to the left of speed in right_to_left layout)
                    let n = line.frames.len();
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

                    // Start frame (to the left of end)
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
                });
            });
        });

        // Re-register with actual rect and handle right-click
        let hdr_resp = ui.interact(resp.response.rect, hdr_bg_id, egui::Sense::click());
        if hdr_resp.secondary_clicked() {
            let pos = ui
                .input(|i| i.pointer.interact_pos())
                .unwrap_or(resp.response.rect.left_top());
            self.open_context_menu(ContextTarget::Header(li), pos);
        }
    }

    pub(super) fn show_frame_cell(
        &mut self,
        ui: &mut egui::Ui,
        frame_ctx: &FrameCellCtx<'_>,
        ctx: &SceneRenderCtx<'_>,
    ) -> egui::Response {
        let (li, fi) = frame_ctx.pos;
        let frame = frame_ctx.frame;
        // Background color
        let picker_open = self
            .frame_states
            .get(&(li, fi))
            .is_some_and(|s| s.lang_picker_open);
        let bg = if picker_open {
            ctx.opacity.fill(ui.visuals().extreme_bg_color, 1.0)
        } else if !frame.enabled {
            ctx.opacity.fill(egui::Color32::from_gray(25), 1.0)
        } else if frame_ctx.is_cursor {
            ctx.opacity.fill(ui.visuals().extreme_bg_color, 1.0)
        } else if frame_ctx.is_selected {
            ctx.opacity.fill(frame_ctx.accent, 0.3)
        } else {
            ctx.opacity.fill(ui.visuals().faint_bg_color, 1.0)
        };

        let resp = ui.push_id(("frame_cell", li, fi), |ui| {
            let bg_id = ui.id().with("cell_bg");
            let pre_rect = ui.available_rect_before_wrap();
            ui.interact(pre_rect, bg_id, egui::Sense::click());

            let cell_frame = egui::Frame::NONE.fill(bg).inner_margin(egui::Margin {
                left: 5,
                right: 5,
                ..egui::Margin::ZERO
            });

            let is_collapsed = self
                .frame_states
                .get(&(li, fi))
                .is_some_and(|s| s.collapsed);

            let frame_resp = cell_frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                if is_collapsed {
                    ui.set_height(HEADER_HEIGHT);
                } else {
                    let frame_height = self
                        .frame_states
                        .get(&(li, fi))
                        .map_or(CELL_HEIGHT, |s| s.height);
                    ui.set_height(HEADER_HEIGHT + frame_height);
                }

                ctx.opacity.override_widget_visuals(ui);

                // Progress fill behind header widgets
                if frame_ctx.is_playing && frame.enabled {
                    paint_progress_fill(ui, frame_ctx.accent, bg, frame_ctx.progress);
                }

                // Header
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.set_height(HEADER_HEIGHT);
                    let is_focused = self.state.focused_frame() == Some((li, fi));
                    if let Some(state) = self.frame_states.get_mut(&(li, fi)) {
                        state.show_header(
                            ui,
                            li,
                            fi,
                            frame_ctx.n_frames,
                            frame_ctx.playing_fis,
                            frame_ctx.accent,
                            frame,
                            ctx.bridge,
                            is_focused,
                            false,
                        );
                    }
                });

                // Frame menu popup
                if self
                    .frame_states
                    .get(&(li, fi))
                    .is_some_and(|s| s.menu_open)
                {
                    let popup_id = ui.id().with("frame_menu");
                    let popup_resp = egui::Area::new(popup_id)
                        .order(egui::Order::Foreground)
                        .fixed_pos(ui.cursor().min)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                ui.set_min_width(150.0);
                                let picker_target =
                                    self.frame_states.get_mut(&(li, fi)).and_then(|state| {
                                        state.show_frame_menu(
                                            ui,
                                            li,
                                            fi,
                                            ctx.bridge,
                                            ctx.default_lang,
                                        )
                                    });
                                if let Some(target) = picker_target {
                                    self.navigate_to_frame(target, ctx.bridge);
                                    self.open_picker_on_cursor =
                                        self.should_auto_open_picker_after_insert();
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
                    let picker_is_open = self
                        .frame_states
                        .get(&(li, fi))
                        .is_some_and(|s| s.lang_picker_open);

                    if picker_is_open {
                        if let Some(state) = self.frame_states.get_mut(&(li, fi))
                            && state.show_inline_lang_picker(ui, frame_ctx.accent, ctx.bridge)
                        {
                            state.focus_request =
                                crate::widgets::inline_scene_view::FocusRequest::Editor;
                        }
                    } else {
                        // Body (code editor)
                        let syntax = ctx.bridge.syntax_map.get(
                            self.frame_states
                                .get(&(li, fi))
                                .map(|s| s.lang.as_str())
                                .unwrap_or(""),
                        );
                        let syntax_pair = syntax.map(|cs| (cs, ctx.theme));

                        let reference = ctx
                            .bridge
                            .languages()
                            .iter()
                            .find(|l| {
                                self.frame_states
                                    .get(&(li, fi))
                                    .is_some_and(|s| s.lang == l.name)
                            })
                            .filter(|l| !l.documentation.reference.is_empty())
                            .map(|l| &l.documentation.reference);

                        let mut cursors: Vec<PeerCursor> = ctx
                            .bridge
                            .text_cursors_for_frame(li, fi)
                            .into_iter()
                            .map(|(name, line, col)| {
                                let color = username_color(&name);
                                PeerCursor {
                                    name,
                                    line,
                                    col,
                                    color,
                                }
                            })
                            .collect();

                        // Include the local user's text cursor
                        if let Some(my_name) = ctx.bridge.confirmed_username()
                            && let Some(state) = self.frame_states.get(&(li, fi))
                            && let Some((line, col)) = state.last_cursor
                        {
                            cursors.push(PeerCursor {
                                name: my_name.to_owned(),
                                line,
                                col,
                                color: username_color(my_name),
                            });
                        }

                        let editor_ctx = EditorContext {
                            settings: ctx.editor_settings,
                            syntax: syntax_pair,
                            reference,
                            peer_cursors: &cursors,
                            annotations: if self
                                .frame_states
                                .get(&(li, fi))
                                .is_some_and(|s| s.dirty)
                            {
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
                }
            });

            let cell_rect = frame_resp.response.rect;

            // Playing indicator
            if frame_ctx.is_playing && frame.enabled {
                let p = ui.painter();
                let fill_h = cell_rect.height() * frame_ctx.progress;
                let bg =
                    egui::Rect::from_min_size(cell_rect.min, egui::vec2(cell_rect.width(), fill_h));
                let bc = egui::Color32::from_rgba_unmultiplied(
                    frame_ctx.accent.r(),
                    frame_ctx.accent.g(),
                    frame_ctx.accent.b(),
                    15,
                );
                p.rect_filled(bg, 0.0, bc);
                ui.ctx().request_repaint();
            }

            super::prelude::draw_feedback_flashes(ui, li, fi, cell_rect, ctx.bridge, 60.0, 40.0);

            // Peer presence (drawn from the discrete editing signal)
            for name in ctx.bridge.editing_peers_for_frame(li, fi) {
                let color = username_color(name);
                let s = egui::Stroke::new(STROKE_EMPHASIS, color);
                ui.painter().vline(cell_rect.left(), cell_rect.y_range(), s);
            }

            // Local user cursor
            if frame_ctx.is_cursor
                && let Some(my_name) = ctx.bridge.confirmed_username()
            {
                let color = username_color(my_name);
                let s = egui::Stroke::new(STROKE_EMPHASIS, color);
                ui.painter().vline(cell_rect.left(), cell_rect.y_range(), s);
            }

            ui.interact(frame_resp.response.rect, bg_id, egui::Sense::click())
        });

        resp.inner
    }
}
