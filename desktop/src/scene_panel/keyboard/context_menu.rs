use eframe::egui;
use sova_core::scene::{Frame, Line};
use sova_core::schedule::{ActionTiming, SchedulerMessage};

use crate::client_bridge::ClientBridge;
use crate::scene_panel::{ContextTarget, Overlay, ViewMode, new_frame};
use crate::widgets::shortcut::{self, Enabled, Key, Shortcut};

fn menu_item(ui: &mut egui::Ui, label: &str, sc: Option<&Shortcut>) -> bool {
    menu_item_enabled(ui, true, label, sc)
}

fn menu_item_enabled(ui: &mut egui::Ui, enabled: bool, label: &str, sc: Option<&Shortcut>) -> bool {
    let text = match sc {
        None => egui::WidgetText::from(label),
        Some(sc) => shortcut::labeled(
            ui,
            label,
            sc,
            if enabled { Enabled::Yes } else { Enabled::No },
        ),
    };
    ui.add_enabled(
        enabled,
        egui::Button::new(text).fill(egui::Color32::TRANSPARENT),
    )
    .clicked()
}

impl crate::scene_panel::ScenePanel {
    pub(crate) fn update_cursor(&mut self, new_cursor: (usize, usize), bridge: &ClientBridge) {
        self.move_cursor(new_cursor, bridge);
    }

    pub(crate) fn show_context_menu(
        &mut self,
        ui: &mut egui::Ui,
        target: Option<ContextTarget>,
        bridge: &ClientBridge,
        default_lang: &str,
    ) {
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
                    let min = *fis.iter().min().expect("non-empty: checked above");
                    let max = *fis.iter().max().expect("non-empty: checked above");
                    (min, max)
                } else {
                    (fi, fi)
                };

                let is_seq = self.view_mode == ViewMode::Sequencer;

                if menu_item(ui, &t!("scene.cut"), Some(&Shortcut::cmd(Key::Char('X')))) {
                    self.cut_selection(bridge);
                    self.overlay = Overlay::None;
                }
                if menu_item(ui, &t!("scene.copy"), Some(&Shortcut::cmd(Key::Char('C')))) {
                    self.copy_selection(bridge);
                    self.overlay = Overlay::None;
                }
                let paste_label = if is_seq {
                    t!("kb.paste")
                } else {
                    t!("scene.paste_after")
                };
                if menu_item_enabled(
                    ui,
                    !self.clipboard.is_empty(),
                    &paste_label,
                    Some(&Shortcut::cmd(Key::Char('V'))),
                ) {
                    if is_seq {
                        self.paste_replace(li, fi, bridge);
                    } else {
                        self.paste_after(li, fi, bridge);
                    }
                    self.overlay = Overlay::None;
                }

                ui.separator();

                if !multi {
                    if menu_item(
                        ui,
                        &t!("scene.insert_frame_before"),
                        Some(&Shortcut::cmd_shift(Key::Char('I'))),
                    ) {
                        bridge.send(SchedulerMessage::AddFrame(
                            li,
                            fi,
                            new_frame(default_lang),
                            ActionTiming::Immediate,
                        ));
                        self.navigate_to_frame((li, fi), bridge);
                        self.open_picker_on_cursor = self.should_auto_open_picker_after_insert();
                        self.overlay = Overlay::None;
                    }
                    if menu_item(
                        ui,
                        &t!("scene.insert_frame_after"),
                        Some(&Shortcut::shift(Key::Char('I'))),
                    ) {
                        bridge.send(SchedulerMessage::AddFrame(
                            li,
                            fi + 1,
                            new_frame(default_lang),
                            ActionTiming::Immediate,
                        ));
                        self.navigate_to_frame((li, fi + 1), bridge);
                        self.open_picker_on_cursor = self.should_auto_open_picker_after_insert();
                        self.overlay = Overlay::None;
                    }
                }

                if menu_item(
                    ui,
                    &t!("scene.duplicate_frame"),
                    Some(&Shortcut::cmd(Key::Char('D'))),
                ) {
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
                            bridge.send(SchedulerMessage::AddFrame(
                                sel_li,
                                last_fi + 1 + offset,
                                frame.clone(),
                                ActionTiming::Immediate,
                            ));
                        }
                    }
                    self.overlay = Overlay::None;
                }

                ui.separator();

                let up_sc = if is_seq {
                    Shortcut::shift(Key::Char('K'))
                } else {
                    Shortcut::alt(Key::Up)
                };
                if menu_item_enabled(ui, min_fi > 0, &t!("scene.move_up"), Some(&up_sc)) {
                    self.move_frames_vertical(-1, bridge);
                    self.overlay = Overlay::None;
                }
                let down_sc = if is_seq {
                    Shortcut::shift(Key::Char('J'))
                } else {
                    Shortcut::alt(Key::Down)
                };
                if menu_item_enabled(
                    ui,
                    max_fi + 1 < line_len,
                    &t!("scene.move_down"),
                    Some(&down_sc),
                ) {
                    self.move_frames_vertical(1, bridge);
                    self.overlay = Overlay::None;
                }

                ui.separator();

                if menu_item(
                    ui,
                    &t!("scene.toggle_enabled"),
                    Some(&Shortcut::plain(Key::Char('E'))),
                ) {
                    let selected: Vec<(usize, usize)> = self.selection.iter().copied().collect();
                    for (sl, sf) in selected {
                        self.toggle_enabled(sl, sf, bridge);
                    }
                    self.overlay = Overlay::None;
                }

                ui.separator();

                if menu_item(ui, &t!("scene.set_start_frame"), None) {
                    self.toggle_line_field(li, bridge, |l| l.start_frame = Some(fi));
                    self.overlay = Overlay::None;
                }
                if menu_item(ui, &t!("scene.set_end_frame"), None) {
                    self.toggle_line_field(li, bridge, |l| l.end_frame = Some(fi));
                    self.overlay = Overlay::None;
                }

                ui.separator();

                if menu_item(
                    ui,
                    &t!("scene.remove_frame"),
                    Some(&Shortcut::plain(Key::Delete)),
                ) {
                    let frames: Vec<(usize, usize)> = self.selection.iter().copied().collect();
                    if !frames.is_empty() {
                        self.request_remove_frames(frames);
                    } else {
                        self.overlay = Overlay::None;
                    }
                }
            }
            Some(ContextTarget::Header(li)) => {
                let num_lines = bridge.scene().map(|s| s.lines.len()).unwrap_or(0);

                if menu_item(ui, &t!("scene.insert_line_before"), None) {
                    bridge.send(SchedulerMessage::AddLine(
                        li,
                        Line::new(vec![1.0]),
                        ActionTiming::Immediate,
                    ));
                    self.navigate_to_frame((li, 0), bridge);
                    self.open_picker_on_cursor = self.should_auto_open_picker_after_insert();
                    self.overlay = Overlay::None;
                }
                if menu_item(ui, &t!("scene.insert_line_after"), None) {
                    bridge.send(SchedulerMessage::AddLine(
                        li + 1,
                        Line::new(vec![1.0]),
                        ActionTiming::Immediate,
                    ));
                    self.navigate_to_frame((li + 1, 0), bridge);
                    self.open_picker_on_cursor = self.should_auto_open_picker_after_insert();
                    self.overlay = Overlay::None;
                }
                if menu_item(
                    ui,
                    &t!("scene.duplicate_line"),
                    Some(&Shortcut::cmd_shift(Key::Char('D'))),
                ) {
                    if let Some(line) = bridge.scene().and_then(|s| s.lines.get(li)) {
                        bridge.send(SchedulerMessage::AddLine(
                            li + 1,
                            line.clone(),
                            ActionTiming::Immediate,
                        ));
                    }
                    self.overlay = Overlay::None;
                }

                ui.separator();

                if menu_item_enabled(
                    ui,
                    li > 0,
                    &t!("scene.move_left"),
                    Some(&Shortcut::alt(Key::Char('H'))),
                ) {
                    self.move_line_horizontal(li, -1, bridge);
                    self.overlay = Overlay::None;
                }
                if menu_item_enabled(
                    ui,
                    li + 1 < num_lines,
                    &t!("scene.move_right"),
                    Some(&Shortcut::alt(Key::Char('L'))),
                ) {
                    self.move_line_horizontal(li, 1, bridge);
                    self.overlay = Overlay::None;
                }

                ui.separator();

                if menu_item(
                    ui,
                    &t!("scene.toggle_looping"),
                    Some(&Shortcut::literal("r")),
                ) {
                    self.toggle_line_field(li, bridge, |l| l.looping = !l.looping);
                    self.overlay = Overlay::None;
                }
                if menu_item(
                    ui,
                    &t!("scene.toggle_trailing"),
                    Some(&Shortcut::plain(Key::Char(','))),
                ) {
                    self.toggle_line_field(li, bridge, |l| l.trailing = !l.trailing);
                    self.overlay = Overlay::None;
                }

                ui.separator();

                if menu_item(ui, &t!("scene.clear_frame_range"), None) {
                    self.toggle_line_field(li, bridge, |l| {
                        l.start_frame = None;
                        l.end_frame = None;
                    });
                    self.overlay = Overlay::None;
                }

                ui.separator();

                if menu_item(
                    ui,
                    &t!("scene.remove_line"),
                    Some(&Shortcut::cmd(Key::Delete)),
                ) {
                    self.request_remove_line(li);
                }
            }
            None => {}
        }
    }
}
