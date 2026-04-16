use eframe::egui;
use sova_core::scene::Line;
use sova_core::schedule::{ActionTiming, SchedulerMessage};

use crate::client_bridge::ClientBridge;
use crate::scene_panel::{ScenePanel, SequencerInlineField, ViewMode, new_frame};

impl ScenePanel {
    pub(crate) fn handle_keyboard(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        default_lang: &str,
    ) {
        let Some(scene) = bridge.scene() else {
            return;
        };

        if let Some(idx) = self.state.selected_prelude() {
            self.handle_prelude_keyboard(ui, idx, scene, bridge);
            return;
        }

        let Some((orig_li, orig_fi)) = self.state.cursor() else {
            let any_nav = ui.input(|i| {
                i.key_pressed(egui::Key::ArrowDown)
                    || i.key_pressed(egui::Key::J)
                    || i.key_pressed(egui::Key::ArrowRight)
                    || i.key_pressed(egui::Key::L)
            });
            if any_nav && !scene.lines.is_empty() {
                self.navigate_to_frame((0, 0), bridge);
            }
            return;
        };

        let is_sequencer = self.view_mode == ViewMode::Sequencer;
        let num_lines = scene.lines.len();
        if num_lines == 0 {
            self.deselect_all();
            return;
        }
        let line_lens: Vec<usize> = scene.lines.iter().map(|l| l.frames.len()).collect();

        let li = orig_li.min(num_lines - 1);
        let fi = orig_fi.min(line_lens[li].saturating_sub(1));
        if (li, fi) != (orig_li, orig_fi) {
            self.move_cursor((li, fi), bridge);
        }

        let (
            up,
            down,
            left,
            right,
            shift,
            key_delete,
            cmd_d,
            cmd_shift_d,
            shift_i,
            cmd_shift_i,
            key_shift_j,
            key_shift_k,
            key_shift_e,
            key_e,
            key_r,
            key_comma,
            key_p,
            key_enter,
            key_i,
            key_escape,
            ctrl_a,
            ctrl_del,
            alt_h,
            alt_l,
            key_f,
            key_o,
            key_shift_o,
            key_shift_s,
            key_m,
            key_x,
            key_b,
            key_c,
            key_d,
            key_a,
            key_s,
            key_t,
            key_y,
            key_z,
        ) = ui.input(|i| {
            let no_mod =
                !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && !i.modifiers.shift;
            let shift_only =
                i.modifiers.shift && !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt;
            (
                i.key_pressed(egui::Key::ArrowUp) || (no_mod && i.key_pressed(egui::Key::K)),
                i.key_pressed(egui::Key::ArrowDown) || (no_mod && i.key_pressed(egui::Key::J)),
                i.key_pressed(egui::Key::ArrowLeft) || (no_mod && i.key_pressed(egui::Key::H)),
                i.key_pressed(egui::Key::ArrowRight) || (no_mod && i.key_pressed(egui::Key::L)),
                i.modifiers.shift,
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::D),
                i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::D),
                shift_only && i.key_pressed(egui::Key::I),
                i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::I),
                shift_only && i.key_pressed(egui::Key::J),
                shift_only && i.key_pressed(egui::Key::K),
                shift_only && i.key_pressed(egui::Key::E),
                no_mod && i.key_pressed(egui::Key::E),
                no_mod && i.key_pressed(egui::Key::R),
                no_mod && i.key_pressed(egui::Key::Comma),
                no_mod && i.key_pressed(egui::Key::P),
                i.key_pressed(egui::Key::Enter) && !i.modifiers.command && !i.modifiers.ctrl,
                no_mod && i.key_pressed(egui::Key::I),
                i.key_pressed(egui::Key::Escape),
                i.modifiers.command && i.key_pressed(egui::Key::A),
                i.modifiers.command
                    && (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)),
                i.modifiers.alt && i.key_pressed(egui::Key::H),
                i.modifiers.alt && i.key_pressed(egui::Key::L),
                no_mod && i.key_pressed(egui::Key::F),
                no_mod && i.key_pressed(egui::Key::O),
                shift_only && i.key_pressed(egui::Key::O),
                shift_only && i.key_pressed(egui::Key::S),
                no_mod && i.key_pressed(egui::Key::M),
                no_mod && i.key_pressed(egui::Key::X),
                no_mod && i.key_pressed(egui::Key::B),
                no_mod && i.key_pressed(egui::Key::C),
                no_mod && i.key_pressed(egui::Key::D),
                no_mod && i.key_pressed(egui::Key::A),
                no_mod && i.key_pressed(egui::Key::S),
                no_mod && i.key_pressed(egui::Key::T),
                no_mod && i.key_pressed(egui::Key::Y),
                no_mod && i.key_pressed(egui::Key::Z),
            )
        });

        if (key_enter || (!is_sequencer && key_i)) && line_lens[li] > 0 {
            self.enter_frame_edit((li, fi));
            return;
        }

        if key_escape {
            if self.state.focused_frame().is_some() {
                self.exit_focus_mode(bridge);
            } else if is_sequencer && line_lens[li] > 0 {
                self.enter_frame_edit((li, fi));
            } else {
                self.deselect_all();
            }
            return;
        }

        if key_f && !is_sequencer && line_lens[li] > 0 {
            if self.state.focused_frame() == Some((li, fi)) {
                self.exit_focus_mode(bridge);
            } else {
                self.enter_focus_mode((li, fi));
            }
            return;
        }

        if is_sequencer && (key_t || key_y) {
            if let Some(frame) = scene.lines.get(li).and_then(|l| l.frames.get(fi)) {
                let field = if key_t {
                    SequencerInlineField::Duration
                } else {
                    SequencerInlineField::Repetitions
                };
                self.begin_sequencer_inline_edit((li, fi), field, frame);
            }
            return;
        }

        if is_sequencer && key_shift_s {
            self.begin_sequencer_line_speed_focus(li);
            return;
        }

        if key_shift_j && !self.selection.is_empty() {
            self.move_frames_vertical(1, bridge);
            for &(sl, sf) in &self.selection {
                self.pending_mutation_flashes.push((sl, sf));
            }
            return;
        }
        if key_shift_k && !self.selection.is_empty() {
            self.move_frames_vertical(-1, bridge);
            for &(sl, sf) in &self.selection {
                self.pending_mutation_flashes.push((sl, sf));
            }
            return;
        }

        if alt_h {
            self.move_line_horizontal(li, -1, bridge);
            return;
        }
        if alt_l {
            self.move_line_horizontal(li, 1, bridge);
            return;
        }

        let (frame_prev, frame_next, line_prev, line_next) = if is_sequencer {
            (left, right, up, down)
        } else {
            (up, down, left, right)
        };
        let any_nav = frame_prev || frame_next || line_prev || line_next;
        if any_nav {
            let mut nav_li = li;
            let mut nav_fi = fi;
            let mut moved = false;
            let mut within_line = false;

            if frame_prev && fi > 0 {
                nav_fi -= 1;
                moved = true;
                within_line = true;
            } else if frame_next && fi + 1 < line_lens[li] {
                nav_fi += 1;
                moved = true;
                within_line = true;
            } else if line_prev && li > 0 {
                nav_li -= 1;
                nav_fi = fi.min(line_lens[nav_li].saturating_sub(1));
                moved = true;
            } else if line_prev && li == 0 && is_sequencer && !self.prelude_states.is_empty() {
                self.navigate_to_prelude(0);
                return;
            } else if line_next && li + 1 < num_lines {
                nav_li += 1;
                nav_fi = fi.min(line_lens[nav_li].saturating_sub(1));
                moved = true;
            }

            if moved {
                if shift && within_line {
                    self.move_cursor((nav_li, nav_fi), bridge);
                    self.extend_selection((nav_li, nav_fi));
                } else {
                    self.navigate_to_frame((nav_li, nav_fi), bridge);
                }
                return;
            }
        }

        if key_delete && !ctrl_del {
            let frames: Vec<(usize, usize)> = self.selection.iter().copied().collect();
            if !frames.is_empty() {
                self.request_remove_frames(frames);
            }
            return;
        }

        if cmd_d || (is_sequencer && key_d) {
            self.duplicate_selected_frames(li, fi, scene, bridge);
            return;
        }

        if cmd_shift_d {
            if let Some(line) = scene.lines.get(li) {
                bridge.send(SchedulerMessage::AddLine(
                    li + 1,
                    line.clone(),
                    ActionTiming::Immediate,
                ));
            }
            return;
        }

        if shift_i || (is_sequencer && key_i) {
            bridge.send(SchedulerMessage::AddFrame(
                li,
                fi + 1,
                new_frame(default_lang),
                ActionTiming::Immediate,
            ));
            self.navigate_to_frame((li, fi + 1), bridge);
            self.open_picker_on_cursor = self.should_auto_open_picker_after_insert();
            return;
        }

        if cmd_shift_i {
            bridge.send(SchedulerMessage::AddFrame(
                li,
                fi,
                new_frame(default_lang),
                ActionTiming::Immediate,
            ));
            self.navigate_to_frame((li, fi), bridge);
            self.open_picker_on_cursor = self.should_auto_open_picker_after_insert();
            return;
        }
        if is_sequencer && key_b {
            bridge.send(SchedulerMessage::AddFrame(
                li,
                fi,
                new_frame(default_lang),
                ActionTiming::Immediate,
            ));
            self.navigate_to_frame((li, fi), bridge);
            self.open_picker_on_cursor = self.should_auto_open_picker_after_insert();
            return;
        }

        if key_shift_e {
            self.toggle_line_enabled(li, bridge);
            return;
        }
        if key_e {
            self.toggle_enabled(li, fi, bridge);
            return;
        }
        if key_r {
            self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
            return;
        }
        if key_comma {
            self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
            return;
        }
        if key_m {
            self.toggle_line_field(li, bridge, |l| l.manual = !l.manual);
            return;
        }

        if key_o {
            bridge.send(SchedulerMessage::AddLine(
                li + 1,
                Line::new(vec![1.0]),
                ActionTiming::Immediate,
            ));
            self.navigate_to_frame((li + 1, 0), bridge);
            return;
        }
        if key_shift_o {
            bridge.send(SchedulerMessage::AddLine(
                li,
                Line::new(vec![1.0]),
                ActionTiming::Immediate,
            ));
            self.navigate_to_frame((li, 0), bridge);
            return;
        }
        if is_sequencer && key_a {
            bridge.send(SchedulerMessage::AddLine(
                li,
                Line::new(vec![1.0]),
                ActionTiming::Immediate,
            ));
            self.navigate_to_frame((li, 0), bridge);
            return;
        }
        if is_sequencer && key_s {
            if let Some(line) = scene.lines.get(li) {
                bridge.send(SchedulerMessage::AddLine(
                    li + 1,
                    line.clone(),
                    ActionTiming::Immediate,
                ));
            }
            return;
        }

        if key_x || (is_sequencer && key_c) {
            self.clear_selected_frames(scene, bridge);
            return;
        }

        if key_p {
            if let Some(frame) = scene.lines.get(li).and_then(|l| l.frames.get(fi)) {
                bridge.send(SchedulerMessage::RunSnippet(
                    frame.script().clone(),
                    frame.duration,
                ));
                self.pending_compilation_flashes.push(((li, fi), true));
            }
            return;
        }

        if ctrl_a {
            self.selection.clear();
            for f in 0..line_lens[li] {
                self.selection.insert((li, f));
            }
            return;
        }

        if ctrl_del || (is_sequencer && key_z) {
            self.request_remove_line(li);
        }
    }
}
