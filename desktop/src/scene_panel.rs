use std::collections::BTreeSet;

use eframe::egui;
use egui::text::{LayoutJob, LayoutSection, TextFormat};
use sova_core::scene::{Frame, Line};
use sova_core::schedule::ActionTiming;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use crate::widgets::syntax_highlight::SyntaxTheme;
use crate::widgets::{
    EditorSettings, HeaderEditField, HeaderInlineEdit, InlineEdit, InlineEditAction,
    InlineEditRegion, SceneGrid, SceneGridResponse,
};

#[derive(Clone, Copy)]
enum ContextTarget {
    Cell(usize, usize),
    Header(usize),
    Void,
}

pub struct PanelVisibility {
    pub server: bool,
    pub audio: bool,
    pub devices: bool,
    pub scope: bool,
    pub spectrum: bool,
    pub vu_meter: bool,
    pub scope_bar: bool,
    pub logs: bool,
    pub options: bool,
    pub debug: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum EditField {
    Duration,
    Repetitions,
    Name,
}

struct EditState {
    line: usize,
    frame: usize,
    field: EditField,
    buf: String,
    first_frame: bool,
}

struct HeaderEditState {
    line: usize,
    field: HeaderEditField,
    buf: String,
    first_frame: bool,
}

#[derive(Default)]
pub struct ScenePanel {
    cursor: Option<(usize, usize)>,
    anchor: Option<(usize, usize)>,
    selection: BTreeSet<(usize, usize)>,
    clipboard: Vec<Frame>,
    editing: Option<EditState>,
    header_editing: Option<HeaderEditState>,
    context_target: Option<ContextTarget>,
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
        editor_settings: &EditorSettings,
    ) -> Option<(usize, usize)> {
        let Some(scene) = bridge.scene() else {
            ui.colored_label(egui::Color32::GRAY, t!("scene.no_scene"));
            return None;
        };

        let has_positions = bridge.positions().iter().any(|p| !p.is_empty());
        let accent = ui.visuals().selection.bg_fill;

        let progress: Vec<f32> = {
            let now = std::time::Instant::now();
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

        let code_preview = self.cursor.and_then(|(li, fi)| {
            let frame = scene.lines.get(li)?.frames.get(fi)?;
            let content = frame.script().content();
            if content.is_empty() {
                return None;
            }
            Some(self.build_preview_job(content, frame.script().lang(), editor_settings, bridge))
        });

        let mut edit_state = self.editing.take();
        let mut header_edit_state = self.header_editing.take();
        let was_editing = edit_state.is_some() || header_edit_state.is_some();

        let avail = ui.available_size();
        let (grid_response, grid_data) = egui::ScrollArea::both()
            .show(ui, |ui| {
                let mut ie = edit_state.as_mut().map(|es| InlineEdit {
                    line: es.line,
                    frame: es.frame,
                    region: match es.field {
                        EditField::Name => InlineEditRegion::Name,
                        EditField::Duration => InlineEditRegion::Duration,
                        EditField::Repetitions => InlineEditRegion::Repetitions,
                    },
                    buf: &mut es.buf,
                    request_focus: es.first_frame,
                });
                let mut hie = header_edit_state.as_mut().map(|hes| HeaderInlineEdit {
                    line: hes.line,
                    field: hes.field,
                    buf: &mut hes.buf,
                    request_focus: hes.first_frame,
                });
                let focused_line = self.cursor.map(|(li, _)| li);
                SceneGrid::new(
                    scene,
                    bridge.positions(),
                    &progress,
                    self.cursor,
                    &self.selection,
                    bridge.peer_editing(),
                    bridge.peer_cursors(),
                    accent,
                    focused_line,
                    avail,
                    visuals_enabled,
                )
                .show(ui, ie.as_mut(), hie.as_mut(), code_preview)
            })
            .inner;

        match grid_data.edit_action {
            Some(InlineEditAction::Active) => {
                if let Some(ref mut es) = edit_state {
                    es.first_frame = false;
                }
                self.editing = edit_state;
            }
            Some(InlineEditAction::Committed) => {
                if let Some(ref es) = edit_state {
                    self.commit_edit(es, bridge);
                }
            }
            Some(InlineEditAction::Cancelled) => {}
            Some(InlineEditAction::Tabbed) => {
                if let Some(ref es) = edit_state {
                    self.commit_edit(es, bridge);
                    match es.field {
                        EditField::Name => {
                            self.start_editing(es.line, es.frame, EditField::Duration, bridge);
                        }
                        EditField::Duration => {
                            self.start_editing(es.line, es.frame, EditField::Repetitions, bridge);
                        }
                        EditField::Repetitions => {}
                    }
                }
            }
            Some(InlineEditAction::BackTabbed) => {
                if let Some(ref es) = edit_state {
                    self.commit_edit(es, bridge);
                    match es.field {
                        EditField::Repetitions => {
                            self.start_editing(es.line, es.frame, EditField::Duration, bridge);
                        }
                        EditField::Duration => {
                            self.start_editing(es.line, es.frame, EditField::Name, bridge);
                        }
                        EditField::Name => {}
                    }
                }
            }
            None => {
                self.editing = edit_state;
            }
        }

        match grid_data.header_edit_action {
            Some(InlineEditAction::Active) => {
                if let Some(ref mut hes) = header_edit_state {
                    hes.first_frame = false;
                }
                self.header_editing = header_edit_state;
            }
            Some(InlineEditAction::Committed) => {
                if let Some(ref hes) = header_edit_state {
                    self.commit_header_edit(hes, bridge);
                }
            }
            Some(InlineEditAction::Tabbed) => {
                if let Some(ref hes) = header_edit_state {
                    self.commit_header_edit(hes, bridge);
                    if hes.field == HeaderEditField::StartFrame {
                        self.start_header_editing(hes.line, HeaderEditField::EndFrame, bridge);
                    }
                }
            }
            Some(InlineEditAction::BackTabbed) => {
                if let Some(ref hes) = header_edit_state {
                    self.commit_header_edit(hes, bridge);
                    if hes.field == HeaderEditField::EndFrame {
                        self.start_header_editing(hes.line, HeaderEditField::StartFrame, bridge);
                    }
                }
            }
            Some(InlineEditAction::Cancelled) => {}
            None => {
                self.header_editing = header_edit_state;
            }
        }

        let open_editor = self.process_grid_clicks(ui, &grid_data, &grid_response, bridge);

        let target = self.context_target;
        grid_response.context_menu(|ui| {
            self.show_context_menu(ui, target, bridge, panels);
        });

        if !was_editing {
            self.handle_clipboard(ui, bridge);
            self.handle_keyboard(ui, bridge);
        }

        if has_positions {
            ui.ctx().request_repaint();
        }

        open_editor
    }

    fn update_cursor(&mut self, new_cursor: (usize, usize), bridge: &ClientBridge) {
        let old = self.cursor;
        self.cursor = Some(new_cursor);
        if old != self.cursor && bridge.is_connected() {
            bridge.send(ClientMessage::CursorPosition(new_cursor.0, new_cursor.1));
        }
    }

    fn process_grid_clicks(
        &mut self,
        ui: &egui::Ui,
        grid: &SceneGridResponse,
        grid_response: &egui::Response,
        bridge: &ClientBridge,
    ) -> Option<(usize, usize)> {
        let mut open_editor = None;

        if let Some(cell) = grid.clicked {
            let shift = ui.input(|i| i.modifiers.shift);
            if shift {
                self.extend_selection(cell);
            } else {
                self.update_cursor(cell, bridge);
                self.anchor = Some(cell);
                self.selection.clear();
                self.selection.insert(cell);
            }
        }

        if let Some(cell) = grid.double_clicked {
            self.update_cursor(cell, bridge);
            self.selection.clear();
            self.selection.insert(cell);
            open_editor = Some(cell);
        }

        if let Some(cell) = grid.secondary_clicked_cell {
            self.context_target = Some(ContextTarget::Cell(cell.0, cell.1));
            if !self.selection.contains(&cell) {
                self.update_cursor(cell, bridge);
                self.selection.clear();
                self.selection.insert(cell);
                self.anchor = Some(cell);
            }
        }

        if let Some(header) = grid.secondary_clicked_header {
            self.context_target = Some(ContextTarget::Header(header));
        } else if grid.secondary_clicked_cell.is_none() && grid_response.secondary_clicked() {
            self.context_target = Some(ContextTarget::Void);
        }

        if let Some((li, fi)) = grid.enable_toggled {
            self.toggle_enabled(li, fi, bridge);
        }

        if let Some(li) = grid.add_frame_clicked {
            let fi = bridge
                .scene()
                .map(|s| s.lines[li].frames.len())
                .unwrap_or(0);
            bridge.send(ClientMessage::AddFrame(
                li,
                fi,
                Frame::default(),
                ActionTiming::Immediate,
            ));
        }

        if let Some(li) = grid.looping_toggled {
            self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
        }

        if let Some(li) = grid.trailing_toggled {
            self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
        }

        if grid.add_line_clicked {
            let li = bridge.scene().map(|s| s.lines.len()).unwrap_or(0);
            bridge.send(ClientMessage::AddLine(
                li,
                Line::new(vec![1.0]),
                ActionTiming::Immediate,
            ));
        }

        if let Some(((li, fi), region)) = grid.subcol_clicked {
            let field = match region {
                InlineEditRegion::Name => EditField::Name,
                InlineEditRegion::Duration => EditField::Duration,
                InlineEditRegion::Repetitions => EditField::Repetitions,
            };
            self.start_editing(li, fi, field, bridge);
        }

        if let Some(li) = grid.speed_clicked {
            self.start_header_editing(li, HeaderEditField::Speed, bridge);
        }
        if let Some(li) = grid.start_frame_clicked {
            self.start_header_editing(li, HeaderEditField::StartFrame, bridge);
        }
        if let Some(li) = grid.end_frame_clicked {
            self.start_header_editing(li, HeaderEditField::EndFrame, bridge);
        }

        open_editor
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

                // Cut / Copy / Paste
                if ui
                    .add(
                        egui::Button::new(t!("scene.cut"))
                            .shortcut_text(format!("{m}+X")),
                    )
                    .clicked()
                {
                    self.cut_selection(bridge);
                    ui.close();
                }
                if ui
                    .add(
                        egui::Button::new(t!("scene.copy"))
                            .shortcut_text(format!("{m}+C")),
                    )
                    .clicked()
                {
                    self.copy_selection(bridge);
                    ui.close();
                }
                if ui
                    .add_enabled(
                        !self.clipboard.is_empty(),
                        egui::Button::new(t!("scene.paste_after"))
                            .shortcut_text(format!("{m}+V")),
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
                            li,
                            fi,
                            Frame::default(),
                            ActionTiming::Immediate,
                        ));
                        ui.close();
                    }
                    if ui.button(t!("scene.insert_frame_after")).clicked() {
                        bridge.send(ClientMessage::AddFrame(
                            li,
                            fi + 1,
                            Frame::default(),
                            ActionTiming::Immediate,
                        ));
                        ui.close();
                    }
                }

                if ui
                    .add(
                        egui::Button::new(t!("scene.duplicate_frame"))
                            .shortcut_text(format!("{m}+D")),
                    )
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
                                scene_ref
                                    .lines
                                    .get(l)
                                    .and_then(|line| line.frames.get(f).cloned())
                            })
                            .collect();
                        for (offset, frame) in frames.iter().enumerate() {
                            bridge.send(ClientMessage::AddFrame(
                                sel_li,
                                last_fi + 1 + offset,
                                frame.clone(),
                                ActionTiming::Immediate,
                            ));
                        }
                    }
                    ui.close();
                }

                ui.separator();

                if ui
                    .add_enabled(
                        min_fi > 0,
                        egui::Button::new(t!("scene.move_up"))
                            .shortcut_text("Alt+Up"),
                    )
                    .clicked()
                {
                    self.move_frames_vertical(-1, bridge);
                    ui.close();
                }
                if ui
                    .add_enabled(
                        max_fi + 1 < line_len,
                        egui::Button::new(t!("scene.move_down"))
                            .shortcut_text("Alt+Down"),
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

                if !multi {
                    if ui
                        .add(
                            egui::Button::new(t!("scene.edit_duration"))
                                .shortcut_text("D"),
                        )
                        .clicked()
                    {
                        self.start_editing(li, fi, EditField::Duration, bridge);
                        ui.close();
                    }
                    if ui
                        .add(
                            egui::Button::new(t!("scene.edit_repetitions"))
                                .shortcut_text("R"),
                        )
                        .clicked()
                    {
                        self.start_editing(li, fi, EditField::Repetitions, bridge);
                        ui.close();
                    }
                    if ui
                        .add(
                            egui::Button::new(t!("scene.rename")).shortcut_text("N"),
                        )
                        .clicked()
                    {
                        self.start_editing(li, fi, EditField::Name, bridge);
                        ui.close();
                    }
                }

                ui.separator();

                if ui
                    .add(
                        egui::Button::new(t!("scene.remove_frame"))
                            .shortcut_text("Delete"),
                    )
                    .clicked()
                {
                    let mut to_remove: Vec<(usize, usize)> =
                        self.selection.iter().copied().collect();
                    to_remove.sort_by(|a, b| b.1.cmp(&a.1));
                    for (rli, rfi) in to_remove {
                        bridge.send(ClientMessage::RemoveFrame(
                            rli,
                            rfi,
                            ActionTiming::Immediate,
                        ));
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
                        li,
                        Line::new(vec![1.0]),
                        ActionTiming::Immediate,
                    ));
                    ui.close();
                }
                if ui.button(t!("scene.insert_line_after")).clicked() {
                    bridge.send(ClientMessage::AddLine(
                        li + 1,
                        Line::new(vec![1.0]),
                        ActionTiming::Immediate,
                    ));
                    ui.close();
                }
                if ui
                    .add(
                        egui::Button::new(t!("scene.duplicate_line"))
                            .shortcut_text(format!("{m}+Shift+D")),
                    )
                    .clicked()
                {
                    if let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) {
                        bridge.send(ClientMessage::AddLine(
                            li + 1,
                            line.clone(),
                            ActionTiming::Immediate,
                        ));
                    }
                    ui.close();
                }

                ui.separator();

                if ui
                    .add_enabled(
                        li > 0,
                        egui::Button::new(t!("scene.move_left"))
                            .shortcut_text("Alt+Left"),
                    )
                    .clicked()
                {
                    self.move_line_horizontal(li, -1, bridge);
                    ui.close();
                }
                if ui
                    .add_enabled(
                        li + 1 < num_lines,
                        egui::Button::new(t!("scene.move_right"))
                            .shortcut_text("Alt+Right"),
                    )
                    .clicked()
                {
                    self.move_line_horizontal(li, 1, bridge);
                    ui.close();
                }

                ui.separator();

                if ui
                    .add(
                        egui::Button::new(t!("scene.toggle_looping"))
                            .shortcut_text("L"),
                    )
                    .clicked()
                {
                    self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
                    ui.close();
                }
                if ui
                    .add(
                        egui::Button::new(t!("scene.toggle_trailing"))
                            .shortcut_text("T"),
                    )
                    .clicked()
                {
                    self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
                    ui.close();
                }

                ui.separator();

                if ui
                    .add(
                        egui::Button::new(t!("scene.edit_speed"))
                            .shortcut_text("S"),
                    )
                    .clicked()
                {
                    self.start_header_editing(li, HeaderEditField::Speed, bridge);
                    ui.close();
                }
                if ui.button(t!("scene.set_start_frame")).clicked() {
                    self.start_header_editing(li, HeaderEditField::StartFrame, bridge);
                    ui.close();
                }
                if ui.button(t!("scene.set_end_frame")).clicked() {
                    self.start_header_editing(li, HeaderEditField::EndFrame, bridge);
                    ui.close();
                }
                if ui.button(t!("scene.clear_frame_range")).clicked() {
                    self.toggle_line_field(li, bridge, |l| {
                        l.start_frame = None;
                        l.end_frame = None;
                    });
                    ui.close();
                }

                ui.separator();

                if ui
                    .add(
                        egui::Button::new(t!("scene.remove_line"))
                            .shortcut_text(format!("{m}+Del")),
                    )
                    .clicked()
                {
                    bridge.send(ClientMessage::RemoveLine(li, ActionTiming::Immediate));
                    ui.close();
                }
            }
            Some(ContextTarget::Void) => {
                ui.checkbox(&mut panels.server, t!("server.title"));
                ui.separator();
                ui.checkbox(&mut panels.audio, t!("audio.title"));
                ui.checkbox(&mut panels.devices, t!("devices.title"));
                ui.checkbox(&mut panels.scope, t!("scope.title"));
                ui.checkbox(&mut panels.spectrum, t!("spectrum.title"));
                ui.checkbox(&mut panels.vu_meter, t!("cmd.vu_meter"));
                ui.checkbox(&mut panels.scope_bar, t!("cmd.scope_bar"));
                ui.separator();
                ui.checkbox(&mut panels.logs, t!("cmd.logs"));
                ui.separator();
                ui.checkbox(&mut panels.options, t!("options.title"));
                ui.checkbox(&mut panels.debug, t!("debug.title"));
            }
            None => {}
        }
    }

    fn handle_clipboard(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if self.editing.is_some() {
            return;
        }
        if ui.ctx().memory(|m| m.focused().is_some()) {
            return;
        }
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
            ui.ctx().copy_text(format!("{} frame(s)", self.clipboard.len()));
        } else if copy && !self.selection.is_empty() {
            self.copy_selection(bridge);
            ui.ctx().copy_text(format!("{} frame(s)", self.clipboard.len()));
        }

        if paste && !self.clipboard.is_empty() {
            self.paste_after(li, fi, bridge);
        }
    }

    fn handle_keyboard(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if self.editing.is_some() {
            return;
        }
        if ui.ctx().memory(|m| m.focused().is_some()) {
            return;
        }
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

        let (
            up,
            down,
            left,
            right,
            shift,
            alt,
            key_enter,
            key_escape,
            key_delete,
            ctrl_a,
            ctrl_d,
            ctrl_shift_d,
            ctrl_del,
            key_d,
            key_r,
            key_n,
            key_l,
            key_t,
            key_s,
        ) = ui.input(|i| {
            let no_mod = !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt;
            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::ArrowRight),
                i.modifiers.shift,
                i.modifiers.alt,
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                i.modifiers.command && i.key_pressed(egui::Key::A),
                i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::D),
                i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::D),
                i.modifiers.command
                    && (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)),
                no_mod && i.key_pressed(egui::Key::D),
                no_mod && i.key_pressed(egui::Key::R),
                no_mod && i.key_pressed(egui::Key::N),
                no_mod && i.key_pressed(egui::Key::L),
                no_mod && i.key_pressed(egui::Key::T),
                no_mod && i.key_pressed(egui::Key::S),
            )
        });

        let li = cur_li;
        let fi = cur_fi;

        // Alt+arrow: move frame/line (checked before normal navigation)
        if alt && up && !self.selection.is_empty() {
            self.move_frames_vertical(-1, bridge);
        } else if alt && down && !self.selection.is_empty() {
            self.move_frames_vertical(1, bridge);
        } else if alt && left {
            self.move_line_horizontal(li, -1, bridge);
        } else if alt && right {
            self.move_line_horizontal(li, 1, bridge);
        } else {
            // Normal navigation
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
                    scene
                        .lines
                        .get(l)
                        .and_then(|line| line.frames.get(f).cloned())
                })
                .collect();
            for (offset, frame) in frames.iter().enumerate() {
                bridge.send(ClientMessage::AddFrame(
                    sel_li,
                    last_fi + 1 + offset,
                    frame.clone(),
                    ActionTiming::Immediate,
                ));
            }
            self.selection.clear();
            for (offset, _) in frames.iter().enumerate() {
                self.selection.insert((sel_li, last_fi + 1 + offset));
            }
            self.cursor = Some((sel_li, last_fi + frames.len()));
            self.anchor = Some((sel_li, last_fi + 1));
        }

        if ctrl_shift_d {
            if let Some(line) = scene.lines.get(li) {
                bridge.send(ClientMessage::AddLine(
                    li + 1,
                    line.clone(),
                    ActionTiming::Immediate,
                ));
            }
        }

        if key_delete && !ctrl_del {
            let mut to_remove: Vec<(usize, usize)> = self.selection.iter().copied().collect();
            to_remove.sort_by(|a, b| b.1.cmp(&a.1));
            for (rli, rfi) in to_remove {
                bridge.send(ClientMessage::RemoveFrame(
                    rli,
                    rfi,
                    ActionTiming::Immediate,
                ));
            }
            self.selection.clear();
            self.cursor = None;
        }

        if ctrl_del {
            bridge.send(ClientMessage::RemoveLine(li, ActionTiming::Immediate));
            self.selection.clear();
            self.cursor = None;
        }

        if key_enter || key_d {
            self.start_editing(li, fi, EditField::Duration, bridge);
        } else if key_r {
            self.start_editing(li, fi, EditField::Repetitions, bridge);
        } else if key_n {
            self.start_editing(li, fi, EditField::Name, bridge);
        } else if key_s {
            self.start_header_editing(li, HeaderEditField::Speed, bridge);
        } else if key_l {
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

    fn commit_edit(&self, edit: &EditState, bridge: &ClientBridge) {
        let Some(frame) = bridge
            .scene()
            .and_then(|s| s.lines.get(edit.line))
            .and_then(|l| l.frames.get(edit.frame))
        else {
            return;
        };

        let mut f = frame.clone();
        let valid = match edit.field {
            EditField::Duration => {
                if let Ok(dur) = edit.buf.parse::<f64>() {
                    if dur > 0.0 {
                        f.duration = dur;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            EditField::Repetitions => {
                if let Ok(rep) = edit.buf.parse::<usize>() {
                    if rep > 0 {
                        f.repetitions = rep;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            EditField::Name => {
                let trimmed = edit.buf.trim();
                f.name = if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                };
                true
            }
        };

        if valid {
            bridge.send(ClientMessage::SetFrames(
                vec![(edit.line, edit.frame, f)],
                ActionTiming::Immediate,
            ));
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

    fn start_editing(&mut self, li: usize, fi: usize, field: EditField, bridge: &ClientBridge) {
        let Some(frame) = bridge
            .scene()
            .and_then(|s| s.lines.get(li))
            .and_then(|l| l.frames.get(fi))
        else {
            return;
        };
        let buf = match field {
            EditField::Duration => format!("{:.2}", frame.duration),
            EditField::Repetitions => format!("{}", frame.repetitions),
            EditField::Name => frame.name.as_deref().unwrap_or("").to_string(),
        };
        self.editing = Some(EditState {
            line: li,
            frame: fi,
            field,
            buf,
            first_frame: true,
        });
        self.header_editing = None;
    }

    fn start_header_editing(
        &mut self,
        li: usize,
        field: HeaderEditField,
        bridge: &ClientBridge,
    ) {
        let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) else {
            return;
        };
        let buf = match field {
            HeaderEditField::Speed => format!("{:.1}", line.speed_factor),
            HeaderEditField::StartFrame => {
                line.start_frame.map(|f| f.to_string()).unwrap_or_default()
            }
            HeaderEditField::EndFrame => {
                line.end_frame.map(|f| f.to_string()).unwrap_or_default()
            }
        };
        self.header_editing = Some(HeaderEditState {
            line: li,
            field,
            buf,
            first_frame: true,
        });
        self.editing = None;
    }

    fn commit_header_edit(&self, edit: &HeaderEditState, bridge: &ClientBridge) {
        let Some(line) = bridge.scene().and_then(|s| s.lines.get(edit.line)) else {
            return;
        };
        let mut l = line.clone();
        let valid = match edit.field {
            HeaderEditField::Speed => {
                if let Ok(speed) = edit.buf.parse::<f64>() {
                    if speed > 0.0 {
                        l.speed_factor = speed;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HeaderEditField::StartFrame => {
                let trimmed = edit.buf.trim();
                if trimmed.is_empty() {
                    l.start_frame = None;
                    true
                } else if let Ok(f) = trimmed.parse::<usize>() {
                    l.start_frame = Some(f);
                    true
                } else {
                    false
                }
            }
            HeaderEditField::EndFrame => {
                let trimmed = edit.buf.trim();
                if trimmed.is_empty() {
                    l.end_frame = None;
                    true
                } else if let Ok(f) = trimmed.parse::<usize>() {
                    l.end_frame = Some(f);
                    true
                } else {
                    false
                }
            }
        };
        if valid {
            bridge.send(ClientMessage::ConfigureLines(
                vec![(edit.line, l)],
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
                scene
                    .lines
                    .get(l)
                    .and_then(|line| line.frames.get(f).cloned())
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
                li,
                fi + 1 + offset,
                frame.clone(),
                ActionTiming::Immediate,
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
                bridge.send(ClientMessage::RemoveFrame(
                    sel_li,
                    min_fi - 1,
                    ActionTiming::Immediate,
                ));
                bridge.send(ClientMessage::AddFrame(
                    sel_li,
                    max_fi,
                    frame,
                    ActionTiming::Immediate,
                ));
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
                bridge.send(ClientMessage::RemoveFrame(
                    sel_li,
                    max_fi + 1,
                    ActionTiming::Immediate,
                ));
                bridge.send(ClientMessage::AddFrame(
                    sel_li,
                    min_fi,
                    frame,
                    ActionTiming::Immediate,
                ));
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
            bridge.send(ClientMessage::CursorPosition(new_li, cur_fi));
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

    fn build_preview_job(
        &self,
        text: &str,
        lang: &str,
        editor_settings: &EditorSettings,
        bridge: &ClientBridge
    ) -> LayoutJob {
        let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
        let font_id = egui::FontId::monospace(11.0);
        let text_color = egui::Color32::from_gray(200);
        let default_fmt = TextFormat::simple(font_id.clone(), text_color);

        let mut job = LayoutJob {
            text: text.to_owned(),
            ..Default::default()
        };

        let syntax_spans: Vec<_> = if let Some(compiled) = bridge.syntax_map.get(lang) {
            compiled
                .tokenize(text)
                .map(|(range, cat)| (range, theme.color(cat)))
                .collect()
        } else {
            Vec::new()
        };

        if syntax_spans.is_empty() {
            job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: 0..text.len(),
                format: default_fmt,
            });
        } else {
            let mut pos = 0;
            for (range, color) in &syntax_spans {
                if range.start > pos {
                    job.sections.push(LayoutSection {
                        leading_space: 0.0,
                        byte_range: pos..range.start,
                        format: default_fmt.clone(),
                    });
                }
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: range.clone(),
                    format: TextFormat::simple(font_id.clone(), *color),
                });
                pos = range.end;
            }
            if pos < text.len() {
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: pos..text.len(),
                    format: default_fmt,
                });
            }
        }

        job
    }
}
