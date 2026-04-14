use eframe::egui;
use sova_core::scene::script::Script;
use sova_core::scene::{Frame, Line};
use sova_core::schedule::{ActionTiming, SchedulerMessage};

use crate::client_bridge::ClientBridge;
use crate::scene_panel::{SceneState, ViewMode};

fn line_enable_updates(line: &Line) -> Vec<(usize, Frame)> {
    if line.frames.is_empty() {
        return Vec::new();
    }

    let target_enabled = line.frames.iter().all(|frame| !frame.enabled);
    line.frames
        .iter()
        .enumerate()
        .filter(|&(_, frame)| frame.enabled != target_enabled)
        .map(|(fi, frame)| {
            let mut updated = frame.clone();
            updated.enabled = target_enabled;
            (fi, updated)
        })
        .collect()
}

impl crate::scene_panel::ScenePanel {
    pub(crate) fn handle_clipboard(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let Some((li, fi)) = self.state.cursor() else {
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
            ui.ctx()
                .copy_text(format!("{} frame(s)", self.clipboard.len()));
        } else if copy && !self.selection.is_empty() {
            self.copy_selection(bridge);
            ui.ctx()
                .copy_text(format!("{} frame(s)", self.clipboard.len()));
        }

        if paste && !self.clipboard.is_empty() {
            if self.view_mode == ViewMode::Sequencer {
                self.paste_replace(li, fi, bridge);
            } else {
                self.paste_after(li, fi, bridge);
            }
        }
    }

    pub(crate) fn duplicate_selected_frames(
        &mut self,
        li: usize,
        fi: usize,
        scene: &sova_core::scene::Scene,
        bridge: &ClientBridge,
    ) {
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
            bridge.send(SchedulerMessage::AddFrame(
                sel_li,
                last_fi + 1 + offset,
                frame.clone(),
                ActionTiming::Immediate,
            ));
        }
        let new_cursor = (sel_li, last_fi + frames.len());
        self.move_cursor(new_cursor, bridge);
        self.selection.clear();
        for (offset, _) in frames.iter().enumerate() {
            self.selection.insert((sel_li, last_fi + 1 + offset));
        }
        self.anchor = Some((sel_li, last_fi + 1));
    }

    pub(crate) fn clear_selected_frames(
        &self,
        scene: &sova_core::scene::Scene,
        bridge: &ClientBridge,
    ) {
        let selected: Vec<(usize, usize)> = self.selection.iter().copied().collect();
        for (sl, sf) in selected {
            if let Some(frame) = scene.lines.get(sl).and_then(|l| l.frames.get(sf)) {
                let mut f = frame.clone();
                f.set_script(Script::new(
                    String::new(),
                    frame.script().lang().to_string(),
                ));
                bridge.send(SchedulerMessage::SetFrames(
                    vec![(sl, sf, f)],
                    ActionTiming::Immediate,
                ));
            }
        }
    }

    pub(crate) fn toggle_enabled(&self, li: usize, fi: usize, bridge: &ClientBridge) {
        if let Some(frame) = bridge
            .scene()
            .and_then(|s| s.lines.get(li))
            .and_then(|l| l.frames.get(fi))
        {
            let mut f = frame.clone();
            f.enabled = !f.enabled;
            bridge.send(SchedulerMessage::SetFrames(
                vec![(li, fi, f)],
                ActionTiming::Immediate,
            ));
        }
    }

    pub(crate) fn toggle_line_enabled(&self, li: usize, bridge: &ClientBridge) {
        if let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) {
            let updates: Vec<(usize, usize, Frame)> = line_enable_updates(line)
                .into_iter()
                .map(|(fi, frame)| (li, fi, frame))
                .collect();
            if !updates.is_empty() {
                bridge.send(SchedulerMessage::SetFrames(
                    updates,
                    ActionTiming::Immediate,
                ));
            }
        }
    }

    pub(crate) fn toggle_line_field(
        &self,
        li: usize,
        bridge: &ClientBridge,
        modify: impl FnOnce(&mut Line),
    ) {
        if let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) {
            let mut l = line.clone();
            modify(&mut l);
            bridge.send(SchedulerMessage::ConfigureLines(
                vec![(li, l)],
                ActionTiming::Immediate,
            ));
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn line_flag_button(
        &self,
        ui: &mut egui::Ui,
        li: usize,
        active: bool,
        accent: egui::Color32,
        bridge: &ClientBridge,
        tooltip: impl Into<egui::WidgetText>,
        build_icon: impl FnOnce(&egui::Ui, egui::Color32) -> egui::WidgetText,
        set_field: impl FnOnce(&mut Line),
    ) {
        let color = if active {
            accent
        } else {
            crate::theme::COLOR_MUTED
        };
        let icon = build_icon(ui, color);
        if ui
            .add(egui::Button::new(icon).fill(egui::Color32::TRANSPARENT))
            .on_hover_text(tooltip)
            .clicked()
        {
            self.toggle_line_field(li, bridge, set_field);
        }
    }

    pub(crate) fn copy_selection(&mut self, bridge: &ClientBridge) {
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

    pub(crate) fn cut_selection(&mut self, bridge: &ClientBridge) {
        self.copy_selection(bridge);
        let mut to_remove: Vec<(usize, usize)> = self.selection.iter().copied().collect();
        to_remove.sort_by(|a, b| b.1.cmp(&a.1));
        for (rli, rfi) in to_remove {
            bridge.send(SchedulerMessage::RemoveFrame(
                rli,
                rfi,
                ActionTiming::Immediate,
            ));
        }
        self.deselect_all();
    }

    pub(crate) fn paste_after(&mut self, li: usize, fi: usize, bridge: &ClientBridge) {
        if self.clipboard.is_empty() {
            return;
        }
        for (offset, frame) in self.clipboard.iter().enumerate() {
            bridge.send(SchedulerMessage::AddFrame(
                li,
                fi + 1 + offset,
                frame.clone(),
                ActionTiming::Immediate,
            ));
        }
        let count = self.clipboard.len();
        self.move_cursor((li, fi + count), bridge);
        self.selection.clear();
        for offset in 0..count {
            self.selection.insert((li, fi + 1 + offset));
        }
        self.anchor = Some((li, fi + 1));
    }

    pub(crate) fn paste_replace(&mut self, li: usize, fi: usize, bridge: &ClientBridge) {
        if self.clipboard.is_empty() {
            return;
        }
        let Some(scene) = bridge.scene() else { return };
        let line_len = scene.lines.get(li).map(|l| l.frames.len()).unwrap_or(0);
        let mut updates = Vec::new();
        for (offset, src) in self.clipboard.iter().enumerate() {
            let target_fi = fi + offset;
            if target_fi >= line_len {
                break;
            }
            updates.push((li, target_fi, src.clone()));
        }
        if !updates.is_empty() {
            let last_fi = updates.last().expect("non-empty: guarded by is_empty check above").1;
            bridge.send(SchedulerMessage::SetFrames(
                updates,
                ActionTiming::Immediate,
            ));
            self.move_cursor((li, last_fi), bridge);
            self.selection.clear();
            for offset in 0..self.clipboard.len().min(line_len - fi) {
                self.selection.insert((li, fi + offset));
            }
            self.anchor = Some((li, fi));
        }
    }

    pub(crate) fn move_frames_vertical(&mut self, direction: i32, bridge: &ClientBridge) {
        let Some(scene) = bridge.scene() else { return };
        if self.selection.is_empty() {
            return;
        }

        let selected: Vec<(usize, usize)> = self.selection.iter().copied().collect();
        let sel_li = selected[0].0;
        if !selected.iter().all(|&(l, _)| l == sel_li) {
            return;
        }

        let min_fi = selected
            .iter()
            .map(|&(_, f)| f)
            .min()
            .expect("non-empty: selection emptiness guarded above");
        let max_fi = selected
            .iter()
            .map(|&(_, f)| f)
            .max()
            .expect("non-empty: selection emptiness guarded above");
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
                bridge.send(SchedulerMessage::RemoveFrame(
                    sel_li,
                    min_fi - 1,
                    ActionTiming::Immediate,
                ));
                bridge.send(SchedulerMessage::AddFrame(
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
            if let Some((l, f)) = self.state.cursor() {
                self.move_cursor((l, f.saturating_sub(1)), bridge);
            }
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
                bridge.send(SchedulerMessage::RemoveFrame(
                    sel_li,
                    max_fi + 1,
                    ActionTiming::Immediate,
                ));
                bridge.send(SchedulerMessage::AddFrame(
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
            if let Some((l, f)) = self.state.cursor() {
                self.move_cursor((l, f + 1), bridge);
            }
            self.anchor = self.anchor.map(|(l, f)| (l, f + 1));
        }
    }

    pub(crate) fn move_line_horizontal(
        &mut self,
        li: usize,
        direction: i32,
        bridge: &ClientBridge,
    ) {
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
            bridge.send(SchedulerMessage::RemoveLine(li, ActionTiming::Immediate));
            bridge.send(SchedulerMessage::AddLine(
                new_li,
                line,
                ActionTiming::Immediate,
            ));
        }

        if let Some((cur_li, cur_fi)) = self.state.cursor()
            && cur_li == li
        {
            self.move_cursor((new_li, cur_fi), bridge);
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

    pub(crate) fn extend_selection(&mut self, target: (usize, usize)) {
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
        if let SceneState::NavigatingFrame { ref mut cursor, .. } = self.state {
            *cursor = target;
        }
    }
}
