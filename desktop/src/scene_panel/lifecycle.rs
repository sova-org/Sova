use std::collections::BTreeSet;

use eframe::egui;
use sova_core::scene::Scene;

use crate::client_bridge::ClientBridge;
use crate::widgets::inline_scene_view::InlineFrameState;

use super::{KeyboardContext, Overlay, PendingDestructive, ScenePanel, SceneState};

impl ScenePanel {
    fn first_remaining_frame_in_line(
        scene: &Scene,
        li: usize,
        removed_frames: &BTreeSet<(usize, usize)>,
    ) -> Option<(usize, usize)> {
        let line = scene.lines.get(li)?;
        line.frames
            .iter()
            .enumerate()
            .find(|(fi, _)| !removed_frames.contains(&(li, *fi)))
            .map(|(fi, _)| (li, fi))
    }

    pub(super) fn next_frame_after_removal(
        &self,
        scene: &Scene,
        removed: &[(usize, usize)],
    ) -> Option<(usize, usize)> {
        let removed_frames: BTreeSet<(usize, usize)> = removed.iter().copied().collect();
        let translate = |li: usize, fi: usize| {
            let removed_before = removed
                .iter()
                .filter(|&&(rli, rfi)| rli == li && rfi < fi)
                .count();
            (li, fi.saturating_sub(removed_before))
        };
        let preferred = self
            .state
            .cursor()
            .or_else(|| removed.first().copied())
            .unwrap_or((0, 0));

        if let Some(line) = scene.lines.get(preferred.0) {
            for fi in preferred.1..line.frames.len() {
                if !removed_frames.contains(&(preferred.0, fi)) {
                    return Some(translate(preferred.0, fi));
                }
            }
            for fi in (0..preferred.1).rev() {
                if !removed_frames.contains(&(preferred.0, fi)) {
                    return Some(translate(preferred.0, fi));
                }
            }
        }

        for li in (preferred.0 + 1)..scene.lines.len() {
            if let Some(pos) = Self::first_remaining_frame_in_line(scene, li, &removed_frames) {
                return Some(translate(pos.0, pos.1));
            }
        }
        for li in (0..preferred.0).rev() {
            if let Some(pos) = Self::first_remaining_frame_in_line(scene, li, &removed_frames) {
                return Some(translate(pos.0, pos.1));
            }
        }

        None
    }

    pub(super) fn next_frame_after_line_removal(
        &self,
        scene: &Scene,
        removed_li: usize,
    ) -> Option<(usize, usize)> {
        for li in removed_li + 1..scene.lines.len() {
            if let Some(frame) = scene
                .lines
                .get(li)
                .and_then(|line| (!line.frames.is_empty()).then_some((li - 1, 0)))
            {
                return Some(frame);
            }
        }
        for li in (0..removed_li).rev() {
            if let Some(frame) = scene
                .lines
                .get(li)
                .and_then(|line| (!line.frames.is_empty()).then_some((li, 0)))
            {
                return Some(frame);
            }
        }
        None
    }

    pub(super) fn restore_scene_state_after_frame_removal(
        &mut self,
        next: Option<(usize, usize)>,
        bridge: &ClientBridge,
    ) {
        match next {
            Some(pos) => match self.state {
                SceneState::EditingFrame { .. } => self.enter_frame_edit(pos),
                SceneState::FocusedFrame { .. } => self.enter_focus_mode(pos),
                _ => self.navigate_to_frame(pos, bridge),
            },
            None => self.deselect_all(),
        }
    }

    fn any_lang_picker_open(&self) -> bool {
        self.frame_states.values().any(|s| s.lang_picker_open)
            || self.prelude_states.iter().any(|s| s.lang_picker_open)
    }

    pub(crate) fn active_keyboard_context(&self, ctx: &egui::Context) -> KeyboardContext {
        if matches!(
            self.overlay,
            Overlay::ConfirmDialog { .. } | Overlay::ContextMenu { .. }
        ) {
            return KeyboardContext::Overlay;
        }
        if self.any_lang_picker_open() {
            return KeyboardContext::LangPicker;
        }
        if self.sequencer_inline_edit.is_some() || self.sequencer_line_speed_focus.is_some() {
            return KeyboardContext::WidgetFocused;
        }
        if ctx.memory(|m| m.focused().is_some()) {
            return KeyboardContext::WidgetFocused;
        }
        if self.state.is_editing() {
            return KeyboardContext::Editing;
        }
        KeyboardContext::Navigating
    }

    pub fn clear_frame_states(&mut self) {
        self.frame_states.clear();
        self.clear_sequencer_inline_edit();
        self.clear_sequencer_line_speed_focus();
    }

    pub(crate) fn request_remove_line(&mut self, li: usize) {
        self.overlay = Overlay::ConfirmDialog {
            action: PendingDestructive::RemoveLine(li),
        };
        self.confirm_dialog.open(
            t!("scene.remove_line"),
            t!("scene.confirm_remove_line", line = li + 1),
        );
    }

    pub(crate) fn request_remove_frames(&mut self, frames: Vec<(usize, usize)>) {
        let count = frames.len();
        self.overlay = Overlay::ConfirmDialog {
            action: PendingDestructive::RemoveFrames(frames),
        };
        self.confirm_dialog.open(
            t!("scene.remove_frame"),
            t!("scene.confirm_remove_frames", count = count),
        );
    }

    pub(super) fn sync_frame_states(&mut self, scene: &Scene, _bridge: &ClientBridge) {
        self.frame_states
            .retain(|&(li, fi), _| scene.lines.get(li).is_some_and(|l| fi < l.frames.len()));

        for (li, line) in scene.lines.iter().enumerate() {
            for (fi, frame) in line.frames.iter().enumerate() {
                let state = self
                    .frame_states
                    .entry((li, fi))
                    .or_insert_with(|| InlineFrameState::new(frame));
                state.sync_if_remote_changed(frame);
            }
        }
    }
}
