use eframe::egui;
use sova_core::scene::Frame;
use sova_core::schedule::{ActionTiming, SchedulerMessage};

use crate::client_bridge::ClientBridge;
use crate::scene_panel::{SequencerInlineField, parse_inline_duration, parse_inline_repetitions};

impl crate::scene_panel::ScenePanel {
    pub(super) fn show_inline_tile_editor(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        li: usize,
        fi: usize,
        frame: &Frame,
        bridge: &ClientBridge,
    ) -> bool {
        if self
            .sequencer_inline_edit
            .as_ref()
            .is_none_or(|edit| edit.target != (li, fi))
        {
            return false;
        }

        if let Some(pressed_pos) = ui.input(|i| {
            i.pointer
                .any_pressed()
                .then(|| i.pointer.interact_pos())
                .flatten()
        }) && !rect.contains(pressed_pos)
        {
            self.clear_sequencer_inline_edit();
            return false;
        }

        let editor_rect = egui::Rect::from_center_size(
            rect.center(),
            egui::vec2((rect.width() - 16.0).max(48.0), 24.0),
        );
        let mut overlay = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(editor_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        let (lost_focus, submit) = {
            let edit = self
                .sequencer_inline_edit
                .as_mut()
                .expect("checked inline edit target above");
            let resp = overlay.add_sized(
                editor_rect.size(),
                egui::TextEdit::singleline(&mut edit.buffer)
                    .font(egui::TextStyle::Monospace)
                    .horizontal_align(egui::Align::Center),
            );
            if edit.request_focus {
                edit.request_focus = false;
                resp.request_focus();
            }
            let lost_focus = resp.lost_focus();
            let mut submit = false;
            if lost_focus {
                submit = crate::widgets::consume_key_on_lost_focus(ui, &resp, egui::Key::Enter);
                if !submit {
                    let _ = crate::widgets::consume_key_on_lost_focus(ui, &resp, egui::Key::Escape);
                }
            }
            (lost_focus, submit)
        };

        if !lost_focus {
            return true;
        }

        if submit {
            if self.commit_inline_tile_edit(li, fi, frame, bridge) {
                return false;
            }
            if let Some(edit) = self.sequencer_inline_edit.as_mut() {
                edit.request_focus = true;
            }
            return true;
        }

        self.clear_sequencer_inline_edit();
        false
    }

    fn commit_inline_tile_edit(
        &mut self,
        li: usize,
        fi: usize,
        frame: &Frame,
        bridge: &ClientBridge,
    ) -> bool {
        let Some((field, buffer)) = self
            .sequencer_inline_edit
            .as_ref()
            .filter(|edit| edit.target == (li, fi))
            .map(|edit| (edit.field, edit.buffer.clone()))
        else {
            return false;
        };

        let mut updated = frame.clone();
        match field {
            SequencerInlineField::Duration => {
                let Some(duration) = parse_inline_duration(&buffer) else {
                    return false;
                };
                updated.duration = duration;
            }
            SequencerInlineField::Repetitions => {
                let Some(repetitions) = parse_inline_repetitions(&buffer) else {
                    return false;
                };
                updated.repetitions = repetitions;
            }
        }

        bridge.send(SchedulerMessage::SetFrames(
            vec![(li, fi, updated)],
            ActionTiming::Immediate,
        ));
        self.clear_sequencer_inline_edit();
        true
    }
}
