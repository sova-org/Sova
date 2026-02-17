use std::collections::BTreeSet;

use eframe::egui;
use sova_core::scene::{Frame, Line};
use sova_core::schedule::ActionTiming;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use crate::widgets::{
    InlineEdit, InlineEditAction, InlineEditRegion, SceneGrid, SceneGridResponse,
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

pub struct ScenePanel {
    cursor: Option<(usize, usize)>,
    anchor: Option<(usize, usize)>,
    selection: BTreeSet<(usize, usize)>,
    clipboard: Vec<Frame>,
    editing: Option<EditState>,
    context_target: Option<ContextTarget>,
}

impl ScenePanel {
    pub fn new() -> Self {
        Self {
            cursor: None,
            anchor: None,
            selection: BTreeSet::new(),
            clipboard: Vec::new(),
            editing: None,
            context_target: None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        panels: &mut PanelVisibility,
    ) -> Option<(usize, usize)> {
        let Some(scene) = bridge.scene() else {
            ui.colored_label(egui::Color32::GRAY, t!("scene.no_scene"));
            return None;
        };

        let has_positions = bridge.positions().iter().any(|p| !p.is_empty());
        let accent = ui.visuals().selection.bg_fill;

        let mut edit_state = self.editing.take();
        let was_editing = edit_state.is_some();

        let (grid_response, grid_data) = egui::ScrollArea::both()
            .show(ui, |ui| {
                let mut ie = edit_state.as_mut().map(|es| InlineEdit {
                    line: es.line,
                    frame: es.frame,
                    region: match es.field {
                        EditField::Duration | EditField::Repetitions => InlineEditRegion::Detail,
                        EditField::Name => InlineEditRegion::Label,
                    },
                    buf: &mut es.buf,
                    request_focus: es.first_frame,
                });
                SceneGrid::new(
                    scene,
                    bridge.positions(),
                    self.cursor,
                    &self.selection,
                    bridge.peer_editing(),
                    bridge.peer_cursors(),
                    accent,
                )
                .show(ui, ie.as_mut())
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
                    if es.field == EditField::Duration {
                        self.start_editing(es.line, es.frame, EditField::Repetitions, bridge);
                    }
                }
            }
            Some(InlineEditAction::BackTabbed) => {
                if let Some(ref es) = edit_state {
                    self.commit_edit(es, bridge);
                    if es.field == EditField::Repetitions {
                        self.start_editing(es.line, es.frame, EditField::Duration, bridge);
                    }
                }
            }
            None => {
                self.editing = edit_state;
            }
        }

        let open_editor = self.process_grid_clicks(ui, &grid_data, &grid_response, bridge);

        let target = self.context_target;
        grid_response.context_menu(|ui| {
            self.show_context_menu(ui, target, bridge, panels);
        });

        if !was_editing {
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
            self.update_cursor(cell, bridge);
            self.selection.clear();
            self.selection.insert(cell);
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

        open_editor
    }

    fn show_context_menu(
        &mut self,
        ui: &mut egui::Ui,
        target: Option<ContextTarget>,
        bridge: &ClientBridge,
        panels: &mut PanelVisibility,
    ) {
        match target {
            Some(ContextTarget::Cell(li, fi)) => {
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
                if ui.button(t!("scene.duplicate_frame")).clicked() {
                    if let Some(frame) = bridge
                        .scene()
                        .and_then(|s| s.lines.get(li))
                        .and_then(|l| l.frames.get(fi))
                    {
                        let cloned = frame.clone();
                        bridge.send(ClientMessage::AddFrame(
                            li,
                            fi + 1,
                            cloned,
                            ActionTiming::Immediate,
                        ));
                    }
                    ui.close();
                }
                ui.separator();
                if ui.button(t!("scene.copy")).clicked() {
                    if let Some(frame) = bridge
                        .scene()
                        .and_then(|s| s.lines.get(li))
                        .and_then(|l| l.frames.get(fi))
                    {
                        self.clipboard = vec![frame.clone()];
                    }
                    ui.close();
                }
                if ui
                    .add_enabled(
                        !self.clipboard.is_empty(),
                        egui::Button::new(t!("scene.paste_after")),
                    )
                    .clicked()
                {
                    for (offset, frame) in self.clipboard.iter().enumerate() {
                        bridge.send(ClientMessage::AddFrame(
                            li,
                            fi + 1 + offset,
                            frame.clone(),
                            ActionTiming::Immediate,
                        ));
                    }
                    ui.close();
                }
                ui.separator();
                if ui.button(t!("scene.remove_frame")).clicked() {
                    bridge.send(ClientMessage::RemoveFrame(li, fi, ActionTiming::Immediate));
                    ui.close();
                }
                ui.separator();
                if ui.button(t!("scene.toggle_enabled")).clicked() {
                    self.toggle_enabled(li, fi, bridge);
                    ui.close();
                }
                if ui.button(t!("scene.edit_duration")).clicked() {
                    self.start_editing(li, fi, EditField::Duration, bridge);
                    ui.close();
                }
                if ui.button(t!("scene.edit_repetitions")).clicked() {
                    self.start_editing(li, fi, EditField::Repetitions, bridge);
                    ui.close();
                }
                if ui.button(t!("scene.rename")).clicked() {
                    self.start_editing(li, fi, EditField::Name, bridge);
                    ui.close();
                }
            }
            Some(ContextTarget::Header(li)) => {
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
                if ui.button(t!("scene.duplicate_line")).clicked() {
                    if let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) {
                        bridge.send(ClientMessage::AddLine(
                            li + 1,
                            line.clone(),
                            ActionTiming::Immediate,
                        ));
                    }
                    ui.close();
                }
                if ui.button(t!("scene.remove_line")).clicked() {
                    bridge.send(ClientMessage::RemoveLine(li, ActionTiming::Immediate));
                    ui.close();
                }
                ui.separator();
                if ui.button(t!("scene.toggle_looping")).clicked() {
                    self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
                    ui.close();
                }
                if ui.button(t!("scene.toggle_trailing")).clicked() {
                    self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
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
                ui.separator();
                ui.checkbox(&mut panels.logs, t!("cmd.logs"));
                ui.separator();
                ui.checkbox(&mut panels.options, t!("options.title"));
                ui.checkbox(&mut panels.debug, t!("debug.title"));
            }
            None => {}
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
            key_enter,
            key_escape,
            key_delete,
            ctrl_a,
            ctrl_c,
            ctrl_v,
            ctrl_d,
            ctrl_del,
            key_d,
            key_r,
            key_n,
            key_l,
            key_t,
        ) = ui.input(|i| {
            let no_mod = !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt;
            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::ArrowRight),
                i.modifiers.shift,
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                i.modifiers.command && i.key_pressed(egui::Key::A),
                i.modifiers.command && i.key_pressed(egui::Key::C),
                i.modifiers.command && i.key_pressed(egui::Key::V),
                i.modifiers.command && i.key_pressed(egui::Key::D),
                i.modifiers.command
                    && (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)),
                no_mod && i.key_pressed(egui::Key::D),
                no_mod && i.key_pressed(egui::Key::R),
                no_mod && i.key_pressed(egui::Key::N),
                no_mod && i.key_pressed(egui::Key::L),
                no_mod && i.key_pressed(egui::Key::T),
            )
        });

        let mut li = cur_li;
        let mut fi = cur_fi;
        let mut moved = false;
        let mut vertical = false;

        if up && fi > 0 {
            fi -= 1;
            moved = true;
            vertical = true;
        } else if down && fi + 1 < line_lens[li] {
            fi += 1;
            moved = true;
            vertical = true;
        } else if left && li > 0 {
            li -= 1;
            fi = fi.min(line_lens[li].saturating_sub(1));
            moved = true;
        } else if right && li + 1 < num_lines {
            li += 1;
            fi = fi.min(line_lens[li].saturating_sub(1));
            moved = true;
        }

        if moved {
            self.update_cursor((li, fi), bridge);
            if shift && vertical {
                self.extend_selection((li, fi));
            } else {
                self.selection.clear();
                self.selection.insert((li, fi));
                self.anchor = Some((li, fi));
            }
        }

        if ctrl_d {
            if self.selection.len() > 1 {
                let selected: Vec<(usize, usize)> = self.selection.iter().copied().collect();
                let sel_li = selected[0].0;
                let last_fi = selected.last().unwrap().1;
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
                let new_last = last_fi + frames.len();
                self.cursor = Some((sel_li, new_last));
                self.anchor = Some((sel_li, last_fi + 1));
            } else if let Some(line) = scene.lines.get(li) {
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

        if ctrl_c && !self.selection.is_empty() {
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

        if ctrl_v && !self.clipboard.is_empty() {
            let insert_after = fi;
            for (offset, frame) in self.clipboard.iter().enumerate() {
                bridge.send(ClientMessage::AddFrame(
                    li,
                    insert_after + 1 + offset,
                    frame.clone(),
                    ActionTiming::Immediate,
                ));
            }
            let count = self.clipboard.len();
            self.selection.clear();
            for offset in 0..count {
                self.selection.insert((li, insert_after + 1 + offset));
            }
            self.anchor = Some((li, insert_after + 1));
            self.cursor = Some((li, insert_after + count));
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
