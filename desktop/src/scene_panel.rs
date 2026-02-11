use std::collections::BTreeSet;

use eframe::egui;
use sova_core::scene::{Frame, Line};
use sova_core::schedule::ActionTiming;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use crate::widgets::{SceneGrid, SceneGridResponse};

#[derive(Clone, Copy)]
enum ContextTarget {
    Cell(usize, usize),
    Header(usize),
}

#[derive(Clone, Copy, PartialEq)]
enum EditField {
    Duration,
    Repetitions,
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
    editing: Option<EditState>,
    context_target: Option<ContextTarget>,
}

impl ScenePanel {
    pub fn new() -> Self {
        Self {
            cursor: None,
            anchor: None,
            selection: BTreeSet::new(),
            editing: None,
            context_target: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if !bridge.is_connected() {
            ui.colored_label(egui::Color32::GRAY, "Not connected");
            self.cursor = None;
            self.anchor = None;
            self.selection.clear();
            self.editing = None;
            self.context_target = None;
            return;
        }

        let Some(scene) = bridge.scene() else {
            ui.colored_label(egui::Color32::GRAY, "No scene");
            return;
        };

        let has_positions = bridge.positions().iter().any(|p| !p.is_empty());
        let accent = ui.visuals().selection.bg_fill;

        let (grid_response, grid_data) = egui::ScrollArea::both()
            .show(ui, |ui| {
                SceneGrid::new(scene, bridge.positions(), self.cursor, &self.selection, accent)
                    .show(ui)
            })
            .inner;

        self.process_grid_clicks(ui, &grid_data, bridge);

        let target = self.context_target;
        grid_response.context_menu(|ui| {
            self.show_context_menu(ui, target, bridge);
        });

        self.handle_keyboard(ui, bridge);
        self.show_editing_popup(ui.ctx(), bridge);

        if has_positions {
            ui.ctx().request_repaint();
        }
    }

    fn send(bridge: &ClientBridge, msg: ClientMessage) {
        bridge.send(msg);
    }

    fn process_grid_clicks(
        &mut self,
        ui: &egui::Ui,
        grid: &SceneGridResponse,
        bridge: &ClientBridge,
    ) {
        if let Some(cell) = grid.clicked {
            let shift = ui.input(|i| i.modifiers.shift);
            if shift {
                self.extend_selection(cell);
            } else {
                self.cursor = Some(cell);
                self.anchor = Some(cell);
                self.selection.clear();
                self.selection.insert(cell);
            }
        }

        if let Some(cell) = grid.double_clicked {
            self.cursor = Some(cell);
            self.selection.clear();
            self.selection.insert(cell);
            self.start_editing(cell.0, cell.1, EditField::Duration, bridge);
        }

        if let Some(cell) = grid.secondary_clicked_cell {
            self.context_target = Some(ContextTarget::Cell(cell.0, cell.1));
            self.cursor = Some(cell);
            self.selection.clear();
            self.selection.insert(cell);
        }

        if let Some(header) = grid.secondary_clicked_header {
            self.context_target = Some(ContextTarget::Header(header));
        }

        if let Some((li, fi)) = grid.enable_toggled {
            self.toggle_enabled(li, fi, bridge);
        }

        if let Some(li) = grid.add_frame_clicked {
            let fi = bridge
                .scene()
                .map(|s| s.lines[li].frames.len())
                .unwrap_or(0);
            Self::send(
                bridge,
                ClientMessage::AddFrame(li, fi, Frame::default(), ActionTiming::Immediate),
            );
        }

        if grid.add_line_clicked {
            let li = bridge
                .scene()
                .map(|s| s.lines.len())
                .unwrap_or(0);
            Self::send(
                bridge,
                ClientMessage::AddLine(li, Line::new(vec![1.0]), ActionTiming::Immediate),
            );
        }
    }

    fn show_context_menu(
        &mut self,
        ui: &mut egui::Ui,
        target: Option<ContextTarget>,
        bridge: &ClientBridge,
    ) {
        match target {
            Some(ContextTarget::Cell(li, fi)) => {
                if ui.button("Insert Frame After").clicked() {
                    Self::send(
                        bridge,
                        ClientMessage::AddFrame(
                            li,
                            fi + 1,
                            Frame::default(),
                            ActionTiming::Immediate,
                        ),
                    );
                    ui.close();
                }
                if ui.button("Duplicate Frame").clicked() {
                    if let Some(frame) = bridge
                        .scene()
                        .and_then(|s| s.lines.get(li))
                        .and_then(|l| l.frames.get(fi))
                    {
                        let cloned = frame.clone();
                        Self::send(
                            bridge,
                            ClientMessage::AddFrame(li, fi + 1, cloned, ActionTiming::Immediate),
                        );
                    }
                    ui.close();
                }
                if ui.button("Remove Frame").clicked() {
                    Self::send(
                        bridge,
                        ClientMessage::RemoveFrame(li, fi, ActionTiming::Immediate),
                    );
                    ui.close();
                }
                ui.separator();
                if ui.button("Toggle Enabled").clicked() {
                    self.toggle_enabled(li, fi, bridge);
                    ui.close();
                }
                if ui.button("Edit Duration").clicked() {
                    self.start_editing(li, fi, EditField::Duration, bridge);
                    ui.close();
                }
                if ui.button("Edit Repetitions").clicked() {
                    self.start_editing(li, fi, EditField::Repetitions, bridge);
                    ui.close();
                }
            }
            Some(ContextTarget::Header(li)) => {
                if ui.button("Insert Line After").clicked() {
                    Self::send(
                        bridge,
                        ClientMessage::AddLine(
                            li + 1,
                            Line::new(vec![1.0]),
                            ActionTiming::Immediate,
                        ),
                    );
                    ui.close();
                }
                if ui.button("Duplicate Line").clicked() {
                    if let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) {
                        Self::send(
                            bridge,
                            ClientMessage::AddLine(li + 1, line.clone(), ActionTiming::Immediate),
                        );
                    }
                    ui.close();
                }
                if ui.button("Remove Line").clicked() {
                    Self::send(
                        bridge,
                        ClientMessage::RemoveLine(li, ActionTiming::Immediate),
                    );
                    ui.close();
                }
                ui.separator();
                if ui.button("Toggle Looping").clicked() {
                    self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
                    ui.close();
                }
                if ui.button("Toggle Trailing").clicked() {
                    self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
                    ui.close();
                }
            }
            None => {}
        }
    }

    fn handle_keyboard(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if self.editing.is_some() {
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

        let (up, down, left, right, shift, ctrl, key_i, key_l, key_d, key_m, key_x, key_enter, key_escape, key_delete, ctrl_a, ctrl_d, ctrl_del) =
            ui.input(|i| {
                (
                    i.key_pressed(egui::Key::ArrowUp),
                    i.key_pressed(egui::Key::ArrowDown),
                    i.key_pressed(egui::Key::ArrowLeft),
                    i.key_pressed(egui::Key::ArrowRight),
                    i.modifiers.shift,
                    i.modifiers.command,
                    i.key_pressed(egui::Key::I),
                    i.key_pressed(egui::Key::L),
                    i.key_pressed(egui::Key::D),
                    i.key_pressed(egui::Key::M),
                    i.key_pressed(egui::Key::X),
                    i.key_pressed(egui::Key::Enter),
                    i.key_pressed(egui::Key::Escape),
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                    i.modifiers.command && i.key_pressed(egui::Key::A),
                    i.modifiers.command && i.key_pressed(egui::Key::D),
                    i.modifiers.command
                        && (i.key_pressed(egui::Key::Delete)
                            || i.key_pressed(egui::Key::Backspace)),
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
            self.cursor = Some((li, fi));
            if shift && vertical {
                self.extend_selection((li, fi));
            } else {
                self.selection.clear();
                self.selection.insert((li, fi));
                self.anchor = Some((li, fi));
            }
        }

        if key_i && !ctrl {
            Self::send(
                bridge,
                ClientMessage::AddFrame(li, fi + 1, Frame::default(), ActionTiming::Immediate),
            );
        }

        if key_l && !ctrl {
            Self::send(
                bridge,
                ClientMessage::AddLine(li + 1, Line::new(vec![1.0]), ActionTiming::Immediate),
            );
        }

        if key_d && !ctrl
            && let Some(frame) = scene.lines.get(li).and_then(|l| l.frames.get(fi))
        {
            Self::send(
                bridge,
                ClientMessage::AddFrame(li, fi + 1, frame.clone(), ActionTiming::Immediate),
            );
        }

        if ctrl_d
            && let Some(line) = scene.lines.get(li)
        {
            Self::send(
                bridge,
                ClientMessage::AddLine(li + 1, line.clone(), ActionTiming::Immediate),
            );
        }

        if key_delete && !ctrl {
            let mut to_remove: Vec<(usize, usize)> = self.selection.iter().copied().collect();
            to_remove.sort_by(|a, b| b.1.cmp(&a.1));
            for (rli, rfi) in to_remove {
                Self::send(
                    bridge,
                    ClientMessage::RemoveFrame(rli, rfi, ActionTiming::Immediate),
                );
            }
            self.selection.clear();
            self.cursor = None;
        }

        if ctrl_del {
            Self::send(
                bridge,
                ClientMessage::RemoveLine(li, ActionTiming::Immediate),
            );
            self.selection.clear();
            self.cursor = None;
        }

        if key_m {
            let sel: Vec<(usize, usize)> = self.selection.iter().copied().collect();
            for (sli, sfi) in sel {
                self.toggle_enabled(sli, sfi, bridge);
            }
        }

        if key_enter {
            self.start_editing(li, fi, EditField::Duration, bridge);
        }

        if key_x && !ctrl {
            self.start_editing(li, fi, EditField::Repetitions, bridge);
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

    fn show_editing_popup(&mut self, ctx: &egui::Context, bridge: &ClientBridge) {
        let mut edit = match self.editing.take() {
            Some(e) => e,
            None => return,
        };

        let mut close = false;
        let mut commit = false;

        let title = match edit.field {
            EditField::Duration => "Duration",
            EditField::Repetitions => "Repetitions",
        };

        egui::Window::new(title)
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_size([100.0, 24.0])
            .show(ctx, |ui| {
                let resp = ui.text_edit_singleline(&mut edit.buf);
                if edit.first_frame {
                    resp.request_focus();
                    edit.first_frame = false;
                }
                if resp.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        commit = true;
                    } else {
                        close = true;
                    }
                }
            });

        if commit {
            self.commit_edit(&edit, bridge);
        }
        if !commit && !close {
            self.editing = Some(edit);
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
        };

        if valid {
            Self::send(
                bridge,
                ClientMessage::SetFrames(
                    vec![(edit.line, edit.frame, f)],
                    ActionTiming::Immediate,
                ),
            );
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
            Self::send(
                bridge,
                ClientMessage::SetFrames(vec![(li, fi, f)], ActionTiming::Immediate),
            );
        }
    }

    fn toggle_line_field(
        &self,
        li: usize,
        bridge: &ClientBridge,
        modify: impl FnOnce(&mut Line),
    ) {
        if let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) {
            let mut l = line.clone();
            modify(&mut l);
            Self::send(
                bridge,
                ClientMessage::ConfigureLines(vec![(li, l)], ActionTiming::Immediate),
            );
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
