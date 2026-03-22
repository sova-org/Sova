use std::time::Instant;

use eframe::egui;
use sova_core::compiler::CompilationState;
use sova_core::scene::Frame;
use sova_core::scene::script::Script;
use crate::scene_panel::new_frame;
use sova_core::schedule::ActionTiming;
use sova_server::{ClientMessage, TextOp};

use super::{COLOR_ERROR, COLOR_MUTED, COLOR_OK, CodeEditor, EditorContext};
use crate::client_bridge::ClientBridge;

pub struct InlineFrameState {
    pub editor: CodeEditor,
    pub content: String,
    pub lang: String,
    pub dirty: bool,
    pub lang_popup_open: bool,
    pub lang_filter: String,
    pub lang_popup_selection: usize,
    pub last_eval: Option<Instant>,
    pub last_cursor_line: Option<usize>,
    pub last_cursor_col: Option<usize>,
    pub sent_cursor: Option<(usize, usize)>,
    pub last_cursor_send: Instant,
    pub header_name_buf: String,
    pub editor_has_focus: bool,
    pub request_focus: bool,
    pub escape_pressed: bool,
    pub menu_open: bool,
    pub prev_content: String,
    pub pending_ops: Vec<TextOp>,
    pub last_op_send: Instant,
    pub has_remote_edits: bool,
    pub height: f32,
}

impl InlineFrameState {
    pub fn new(frame: &Frame) -> Self {
        let content = frame.script().content().to_owned();
        Self {
            prev_content: content.clone(),
            editor: CodeEditor::new(),
            content,
            lang: frame.script().lang().to_owned(),
            dirty: false,
            lang_popup_open: false,
            lang_filter: String::new(),
            lang_popup_selection: 0,
            last_eval: None,
            last_cursor_line: None,
            last_cursor_col: None,
            sent_cursor: None,
            last_cursor_send: Instant::now(),
            header_name_buf: frame.name.clone().unwrap_or_default(),
            editor_has_focus: false,
            request_focus: false,
            escape_pressed: false,
            menu_open: false,
            pending_ops: Vec::new(),
            last_op_send: Instant::now(),
            has_remote_edits: false,
            height: crate::scene_panel::CELL_HEIGHT,
        }
    }

    pub fn compute_diff_ops(&mut self) {
        if self.content == self.prev_content {
            return;
        }
        let old: Vec<char> = self.prev_content.chars().collect();
        let new: Vec<char> = self.content.chars().collect();

        let prefix = old.iter().zip(new.iter()).take_while(|(a, b)| a == b).count();
        let old_rem = old.len() - prefix;
        let new_rem = new.len() - prefix;
        let suffix = old[prefix..]
            .iter()
            .rev()
            .zip(new[prefix..].iter().rev())
            .take_while(|(a, b)| a == b)
            .count()
            .min(old_rem)
            .min(new_rem);

        let del_len = old.len() - prefix - suffix;
        let ins_len = new.len() - prefix - suffix;

        let byte_prefix: usize = old.iter().take(prefix).map(|c| c.len_utf8()).sum();

        if del_len > 0 {
            let byte_del: usize = old[prefix..prefix + del_len].iter().map(|c| c.len_utf8()).sum();
            self.pending_ops.push(TextOp::Delete { pos: byte_prefix, len: byte_del });
        }
        if ins_len > 0 {
            let ins_text: String = new[prefix..prefix + ins_len].iter().collect();
            self.pending_ops.push(TextOp::Insert { pos: byte_prefix, text: ins_text });
        }

        self.prev_content = self.content.clone();
    }

    pub fn integrate_remote_op(&mut self, op: &TextOp) {
        self.has_remote_edits = true;
        match op {
            TextOp::Insert { pos, text } => {
                let pos = (*pos).min(self.content.len());
                self.content.insert_str(pos, text);
                self.prev_content = self.content.clone();
            }
            TextOp::Delete { pos, len } => {
                let pos = (*pos).min(self.content.len());
                let end = (pos + len).min(self.content.len());
                if pos < end {
                    self.content.drain(pos..end);
                    self.prev_content = self.content.clone();
                }
            }
        }
    }

    pub fn flush_pending_ops(&mut self, li: usize, fi: usize, bridge: &ClientBridge) {
        if self.pending_ops.is_empty() {
            return;
        }
        if self.last_op_send.elapsed().as_millis() < 30 {
            return;
        }
        let ops = std::mem::take(&mut self.pending_ops);
        bridge.send(ClientMessage::ScriptEdit { li, fi, ops });
        self.last_op_send = Instant::now();
    }

    pub fn sync_if_remote_changed(&mut self, frame: &Frame) {
        if self.dirty {
            return;
        }
        let remote_content = frame.script().content();
        let remote_lang = frame.script().lang();
        if self.has_remote_edits {
            if remote_content == self.content {
                self.has_remote_edits = false;
                self.lang = remote_lang.to_owned();
                self.prev_content = self.content.clone();
                self.header_name_buf = frame.name.clone().unwrap_or_default();
            }
            return;
        }
        if remote_content != self.content || remote_lang != self.lang {
            self.content = remote_content.to_owned();
            self.lang = remote_lang.to_owned();
            self.prev_content = self.content.clone();
            self.pending_ops.clear();
        }
    }

    pub fn sync_from_frame(&mut self, frame: &Frame) {
        self.content = frame.script().content().to_owned();
        self.lang = frame.script().lang().to_owned();
        self.dirty = false;
        self.prev_content = self.content.clone();
        self.pending_ops.clear();
        self.has_remote_edits = false;
    }

    pub fn evaluate(&mut self, li: usize, fi: usize, frame: &Frame, bridge: &ClientBridge) {
        let mut f = frame.clone();
        f.set_script(Script::new(self.content.clone(), self.lang.clone()));
        bridge.send(ClientMessage::SetFrames(
            vec![(li, fi, f)],
            ActionTiming::Immediate,
        ));
        self.dirty = false;
        self.last_eval = Some(Instant::now());
    }

    pub fn show_header(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        fi: usize,
        frame: &Frame,
        _opacity: &crate::scene_panel::SceneOpacity,
        bridge: &ClientBridge,
    ) {
        // Cmd/Ctrl+L shortcut — check focus on any widget in this cell
        let any_focus = ui.memory(|m| m.focused().is_some());
        if any_focus {
            let is_mac = ui.ctx().os().is_mac();
            let shortcut_pressed = ui.input(|i| {
                i.key_pressed(egui::Key::L)
                    && if is_mac {
                        i.modifiers.mac_cmd
                    } else {
                        i.modifiers.ctrl
                    }
            });
            if shortcut_pressed {
                self.lang_popup_open = !self.lang_popup_open;
                self.lang_filter.clear();
                self.lang_popup_selection = 0;
            }
        }

        // Enabled toggle (first)
        let enabled = frame.enabled;
        let toggle_icon = if enabled {
            crate::icons::CIRCLE_LARGE_FILLED
        } else {
            crate::icons::CIRCLE_LARGE_OUTLINE
        };
        let toggle_color = if enabled { COLOR_OK } else { COLOR_MUTED };
        if ui
            .add(
                egui::Button::new(egui::RichText::new(toggle_icon).color(toggle_color))
                    .fill(egui::Color32::TRANSPARENT),
            )
            .clicked()
        {
            let mut f = frame.clone();
            f.enabled = !enabled;
            bridge.send(ClientMessage::SetFrames(
                vec![(li, fi, f)],
                ActionTiming::Immediate,
            ));
        }

        // Language selector
        let btn_fill = ui.visuals().widgets.inactive.bg_fill;
        let lang_btn = ui.add(
            egui::Button::new(
                egui::RichText::new(format!("{} {}", self.lang, crate::icons::CHEVRON_DOWN))
                    .small(),
            )
            .fill(btn_fill),
        );
        if lang_btn.clicked() {
            self.lang_popup_open = !self.lang_popup_open;
            self.lang_filter.clear();
            self.lang_popup_selection = 0;
        }

        self.show_lang_popup(ui, &lang_btn, _opacity, bridge);

        ui.separator();

        // Duration
        let mut dur = frame.duration;
        let dur_resp = ui.add(
            egui::DragValue::new(&mut dur)
                .range(0.001..=f64::MAX)
                .speed(0.1)
                .prefix("dur: "),
        );
        if dur_resp.changed() && dur > 0.0 {
            let mut f = frame.clone();
            f.duration = dur;
            bridge.send(ClientMessage::SetFrames(
                vec![(li, fi, f)],
                ActionTiming::Immediate,
            ));
        }

        // Repetitions
        let mut rep = frame.repetitions;
        let rep_resp = ui.add(
            egui::DragValue::new(&mut rep)
                .range(1..=usize::MAX)
                .prefix("rep: "),
        );
        if rep_resp.changed() && rep > 0 {
            let mut f = frame.clone();
            f.repetitions = rep;
            bridge.send(ClientMessage::SetFrames(
                vec![(li, fi, f)],
                ActionTiming::Immediate,
            ));
        }

        // Name (last)
        let name_id = ui.id().with("hdr_name");
        let name_focused = ui.memory(|m| m.has_focus(name_id));
        if !name_focused {
            self.header_name_buf = frame.name.clone().unwrap_or_default();
        }
        let name_resp = ui.add(
            egui::TextEdit::singleline(&mut self.header_name_buf)
                .id(name_id)
                .desired_width(60.0)
                .hint_text("name")
                .font(egui::TextStyle::Small),
        );
        if name_resp.lost_focus() {
            let trimmed = self.header_name_buf.trim();
            let new_name = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            };
            if new_name != frame.name {
                let mut f = frame.clone();
                f.name = new_name;
                bridge.send(ClientMessage::SetFrames(
                    vec![(li, fi, f)],
                    ActionTiming::Immediate,
                ));
            }
        }

        // Right-aligned: menu button + dirty indicator
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Menu button
            let menu_btn = ui.add(
                egui::Button::new(
                    egui::RichText::new(crate::icons::CHEVRON_DOWN).small(),
                )
                .fill(egui::Color32::TRANSPARENT),
            );
            if menu_btn.clicked() {
                self.menu_open = !self.menu_open;
            }

            // Dirty indicator
            if self.dirty {
                let discard_fill = COLOR_ERROR.linear_multiply(0.3);
                let discard_text = egui::RichText::new(crate::icons::MODIFIED)
                    .small()
                    .color(COLOR_ERROR);
                if ui
                    .add(egui::Button::new(discard_text).fill(discard_fill))
                    .on_hover_text(t!("step.discard"))
                    .clicked()
                {
                    self.sync_from_frame(frame);
                }
            }
        });
    }

    pub fn show_frame_menu(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        fi: usize,
        bridge: &ClientBridge,
        default_lang: &str,
    ) {
        if ui.button(t!("scene.insert_frame_before")).clicked() {
            bridge.send(ClientMessage::AddFrame(
                li, fi, new_frame(default_lang), ActionTiming::Immediate,
            ));
            self.menu_open = false;
            ui.close();
        }
        if ui.button(t!("scene.insert_frame_after")).clicked() {
            bridge.send(ClientMessage::AddFrame(
                li, fi + 1, new_frame(default_lang), ActionTiming::Immediate,
            ));
            self.menu_open = false;
            ui.close();
        }
        if ui.button(t!("scene.duplicate_frame")).clicked() {
            if let Some(frame) = bridge
                .scene()
                .and_then(|s| s.lines.get(li))
                .and_then(|l| l.frames.get(fi))
            {
                bridge.send(ClientMessage::AddFrame(
                    li, fi + 1, frame.clone(), ActionTiming::Immediate,
                ));
            }
            self.menu_open = false;
            ui.close();
        }

        ui.separator();

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(t!("scene.remove_frame")).color(COLOR_ERROR),
                ),
            )
            .clicked()
        {
            bridge.send(ClientMessage::RemoveFrame(li, fi, ActionTiming::Immediate));
            self.menu_open = false;
            ui.close();
        }
    }

    fn show_lang_popup(
        &mut self,
        ui: &mut egui::Ui,
        lang_btn: &egui::Response,
        opacity: &crate::scene_panel::SceneOpacity,
        bridge: &ClientBridge,
    ) {
        if !self.lang_popup_open {
            return;
        }

        let popup_id = ui.id().with("lang_popup");
        let languages = bridge.languages();
        let filter_lower = self.lang_filter.to_lowercase();
        let filtered: Vec<_> = languages
            .iter()
            .filter(|l| l.name.to_lowercase().contains(&filter_lower))
            .collect();

        let mut close = false;
        let area_resp = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(lang_btn.rect.left_bottom())
            .show(ui.ctx(), |ui| {
                let popup_frame = {
                    let mut f = egui::Frame::popup(ui.style());
                    f.fill = opacity.fill(f.fill, 1.0);
                    f
                };
                popup_frame.show(ui, |ui| {
                    opacity.override_widget_visuals(ui);
                    ui.set_min_width(160.0);
                    let filter_id = popup_id.with("filter");
                    let filter_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.lang_filter)
                            .id(filter_id)
                            .desired_width(150.0)
                            .hint_text("Filter..."),
                    );
                    filter_resp.request_focus();

                    let key_up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
                    let key_down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
                    let key_enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let key_escape = ui.input(|i| i.key_pressed(egui::Key::Escape));

                    if key_escape {
                        close = true;
                    }

                    if !filtered.is_empty() {
                        if key_up {
                            self.lang_popup_selection =
                                self.lang_popup_selection.saturating_sub(1);
                        }
                        if key_down {
                            self.lang_popup_selection =
                                (self.lang_popup_selection + 1).min(filtered.len() - 1);
                        }
                        self.lang_popup_selection =
                            self.lang_popup_selection.min(filtered.len().saturating_sub(1));

                        if key_enter {
                            let selected = &filtered[self.lang_popup_selection];
                            if self.lang != selected.name {
                                self.lang = selected.name.clone();
                                self.dirty = true;
                            }
                            close = true;
                        }
                    }

                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (i, lang) in filtered.iter().enumerate() {
                                let selected = i == self.lang_popup_selection;
                                let resp = ui.selectable_label(selected, &lang.name);
                                if resp.clicked() {
                                    if self.lang != lang.name {
                                        self.lang = lang.name.clone();
                                        self.dirty = true;
                                    }
                                    close = true;
                                }
                            }
                        });
                });
            });

        if close
            || (ui.input(|i| i.pointer.any_pressed())
                && !area_resp
                    .response
                    .rect
                    .contains(ui.input(|i| i.pointer.interact_pos().unwrap_or_default()))
                && !lang_btn
                    .rect
                    .contains(ui.input(|i| i.pointer.interact_pos().unwrap_or_default())))
        {
            self.lang_popup_open = false;
        }
    }

    pub fn show_body(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        fi: usize,
        ctx: &EditorContext,
        bridge: &ClientBridge,
    ) {
        let editor_id = ui.id().with("editor_body");
        let editor_id_focus = editor_id.with("editor");

        // Handle focus request from Nav → Edit mode transition
        if self.request_focus {
            self.request_focus = false;
            ui.memory_mut(|m| m.request_focus(editor_id_focus));
        }

        egui::ScrollArea::vertical()
            .id_salt(("editor_scroll", li, fi))
            .auto_shrink(false)
            .show(ui, |ui| {
                let output = self.editor.show(
                    ui,
                    editor_id,
                    &mut self.content,
                    ctx,
                );
                if output.response.changed() {
                    self.dirty = true;
                    self.compute_diff_ops();
                }
                self.last_cursor_line = output.cursor_line;
                self.last_cursor_col = output.cursor_col;
            });

        // Flush pending CRDT operations (throttled)
        self.flush_pending_ops(li, fi, bridge);

        // Track focus and handle edit mode shortcuts
        self.editor_has_focus = ui.memory(|m| m.has_focus(editor_id_focus));
        if self.editor_has_focus {
            // Escape exits edit mode (but not if completion popup is open)
            let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
            if escape && !self.editor.is_completion_open() {
                ui.memory_mut(|m| m.surrender_focus(editor_id_focus));
                self.editor_has_focus = false;
                self.escape_pressed = true;
            }
        }
        if self.editor_has_focus {
            let is_mac = ui.ctx().os().is_mac();
            let eval = ui.input(|i| {
                i.key_pressed(egui::Key::Enter)
                    && if is_mac {
                        i.modifiers.mac_cmd
                    } else {
                        i.modifiers.ctrl
                    }
            });
            if eval
                && let Some(frame) = bridge
                    .scene()
                    .and_then(|s| s.lines.get(li))
                    .and_then(|l| l.frames.get(fi))
            {
                self.evaluate(li, fi, frame, bridge);
            }
        }

        // Send text cursor position to peers (throttled)
        if let (Some(line), Some(col)) = (self.last_cursor_line, self.last_cursor_col) {
            let pos = (line, col);
            if self.sent_cursor != Some(pos)
                && self.last_cursor_send.elapsed().as_millis() >= 50
            {
                self.sent_cursor = Some(pos);
                self.last_cursor_send = Instant::now();
                bridge.send(ClientMessage::CursorPosition(li, fi, Some(pos)));
            }
        }

        // Eval flash
        if let Some(eval_time) = self.last_eval {
            let elapsed = eval_time.elapsed().as_secs_f32();
            if elapsed < 0.3 {
                let t = elapsed / 0.3;
                let alpha = ((1.0 - t) * 30.0) as u8;
                let is_error = matches!(
                    bridge.compilation_state(li, fi),
                    Some(CompilationState::Error(_))
                );
                let flash = if is_error {
                    egui::Color32::from_rgba_unmultiplied(255, 60, 60, alpha)
                } else {
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha)
                };
                ui.painter().rect_filled(ui.max_rect(), 0.0, flash);
                ui.ctx().request_repaint();
            } else {
                self.last_eval = None;
            }
        }
    }
}
