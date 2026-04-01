use std::time::Instant;

use eframe::egui;
use sova_core::compiler::CompilationState;
use sova_core::scene::Frame;
use sova_core::scene::script::Script;
use sova_core::schedule::SchedulerMessage;
use crate::scene_panel::new_frame;
use sova_core::schedule::ActionTiming;
use sova_server::{ClientMessage, TextOp};

use super::{COLOR_ERROR, COLOR_MUTED, COLOR_OK, CodeEditor, EditorContext, cycled_accent};
use crate::client_bridge::ClientBridge;

/// Full-body language picker grid. Replaces the code editor area with a grid
/// of clickable language tiles. Returns `Some(lang_name)` when a language is
/// selected, `None` while browsing or on cancel.
pub fn show_lang_picker(
    ui: &mut egui::Ui,
    picker_open: &mut bool,
    picker_filter: &mut String,
    picker_selection: &mut usize,
    current_lang: &str,
    accent: egui::Color32,
    bridge: &ClientBridge,
) -> Option<String> {
    let languages = bridge.languages();
    let filter_lower = picker_filter.to_lowercase();
    let filtered: Vec<_> = languages
        .iter()
        .enumerate()
        .filter(|(_, l)| l.name.to_lowercase().contains(&filter_lower))
        .collect();

    // Clamp selection
    *picker_selection = (*picker_selection).min(filtered.len().saturating_sub(1));

    // Consume keyboard events
    let escape = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    let enter = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
    let arrow_left = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft));
    let arrow_right = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight));
    let arrow_up = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
    let arrow_down = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
    let backspace = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace));

    // Number keys 1-9 for instant selection
    let mut digit_pick: Option<usize> = None;
    for d in 1..=9u8 {
        let key = match d {
            1 => egui::Key::Num1,
            2 => egui::Key::Num2,
            3 => egui::Key::Num3,
            4 => egui::Key::Num4,
            5 => egui::Key::Num5,
            6 => egui::Key::Num6,
            7 => egui::Key::Num7,
            8 => egui::Key::Num8,
            9 => egui::Key::Num9,
            _ => unreachable!(),
        };
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
            digit_pick = Some((d - 1) as usize);
        }
    }

    // Text input for filtering (consume typed characters)
    let typed: Vec<char> = ui.input_mut(|i| {
        let chars: Vec<char> = i
            .events
            .iter()
            .filter_map(|e| {
                if let egui::Event::Text(s) = e {
                    s.chars().next()
                } else {
                    None
                }
            })
            .collect();
        i.events.retain(|e| !matches!(e, egui::Event::Text(_)));
        chars
    });

    if backspace {
        picker_filter.pop();
    }
    for ch in typed {
        picker_filter.push(ch);
    }

    // Escape cancels
    if escape {
        *picker_open = false;
        picker_filter.clear();
        return None;
    }

    // Instant digit selection
    if let Some(idx) = digit_pick
        && idx < filtered.len()
    {
        *picker_open = false;
        picker_filter.clear();
        return Some(filtered[idx].1.name.clone());
    }

    // Arrow navigation
    let available = ui.available_size();
    let cols = ((available.x / 140.0) as usize).max(1).min(filtered.len().max(1));

    if !filtered.is_empty() {
        if arrow_left {
            *picker_selection = picker_selection.saturating_sub(1);
        }
        if arrow_right {
            *picker_selection = (*picker_selection + 1).min(filtered.len() - 1);
        }
        if arrow_up {
            *picker_selection = picker_selection.saturating_sub(cols);
        }
        if arrow_down {
            *picker_selection = (*picker_selection + cols).min(filtered.len() - 1);
        }
    }

    // Enter confirms
    if enter && !filtered.is_empty() {
        *picker_open = false;
        picker_filter.clear();
        return Some(filtered[*picker_selection].1.name.clone());
    }

    // Fill the entire body with a dark background so the frame's bg doesn't bleed through
    ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, ui.visuals().extreme_bg_color);

    // Render filter hint
    if !picker_filter.is_empty() {
        ui.label(
            egui::RichText::new(format!("filter: {}", picker_filter))
                .monospace()
                .color(COLOR_MUTED),
        );
    }

    if filtered.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("No matching languages").color(COLOR_MUTED));
        });
        return None;
    }

    // Compute tile sizes
    let rows = filtered.len().div_ceil(cols);
    let spacing = 2.0;
    let tile_w = (available.x - spacing * (cols as f32 - 1.0)) / cols as f32;
    let tile_h = ((available.y - spacing * (rows as f32 - 1.0)) / rows as f32)
        .clamp(32.0, 64.0);

    let mut result = None;

    egui::Grid::new("lang_picker_grid")
        .num_columns(cols)
        .spacing(egui::vec2(spacing, spacing))
        .show(ui, |ui| {
            for (i, &(orig_idx, lang)) in filtered.iter().enumerate() {
                let is_selected = i == *picker_selection;
                let is_current = lang.name == current_lang;

                let shortcut = if i < 9 {
                    format!("[{}] ", i + 1)
                } else {
                    String::new()
                };

                let lang_color = cycled_accent(accent, orig_idx);
                let (fill, text_color) = if is_selected {
                    (lang_color, egui::Color32::WHITE)
                } else if is_current {
                    (lang_color.linear_multiply(0.4), ui.visuals().text_color())
                } else {
                    (lang_color.linear_multiply(0.2), ui.visuals().text_color())
                };

                let label = format!("{}{}", shortcut, lang.name);
                let btn = egui::Button::new(
                    egui::RichText::new(label).color(text_color),
                )
                .fill(fill)
                .corner_radius(0.0)
                .min_size(egui::vec2(tile_w, tile_h));

                if ui.add(btn).clicked() {
                    result = Some(lang.name.clone());
                }

                if (i + 1) % cols == 0 {
                    ui.end_row();
                }
            }
        });

    if result.is_some() {
        *picker_open = false;
        picker_filter.clear();
    }

    result
}

pub struct InlineFrameState {
    pub editor: CodeEditor,
    pub content: String,
    pub lang: String,
    pub dirty: bool,
    pub lang_picker_open: bool,
    pub lang_picker_filter: String,
    pub lang_picker_selection: usize,
    pub last_eval: Option<Instant>,
    pub last_cursor_line: Option<usize>,
    pub last_cursor_col: Option<usize>,
    pub sent_cursor: Option<(usize, usize)>,
    pub last_cursor_send: Instant,

    pub editor_has_focus: bool,
    pub request_focus: bool,
    pub escape_pressed: bool,
    pub menu_open: bool,
    pub prev_content: String,
    pub pending_ops: Vec<TextOp>,
    pub last_op_send: Instant,
    pub has_remote_edits: bool,
    pub height: f32,
    pub collapsed: bool,
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
            lang_picker_open: false,
            lang_picker_filter: String::new(),
            lang_picker_selection: 0,
            last_eval: None,
            last_cursor_line: None,
            last_cursor_col: None,
            sent_cursor: None,
            last_cursor_send: Instant::now(),

            editor_has_focus: false,
            request_focus: false,
            escape_pressed: false,
            menu_open: false,
            pending_ops: Vec::new(),
            last_op_send: Instant::now(),
            has_remote_edits: false,
            height: crate::scene_panel::CELL_HEIGHT,
            collapsed: false,
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
        bridge.send(SchedulerMessage::SetFrames(
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
        n_frames: usize,
        current_playing_fi: Option<usize>,
        accent: egui::Color32,
        frame: &Frame,
        bridge: &ClientBridge,
    ) {
        // Subdued style: transparent backgrounds so the header doesn't compete with the code
        let wv = &mut ui.style_mut().visuals.widgets;
        wv.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        wv.inactive.bg_stroke = egui::Stroke::NONE;

        // Cmd/Ctrl+L shortcut — only for the frame whose editor has focus
        if self.editor_has_focus {
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
                self.lang_picker_open = !self.lang_picker_open;
                self.lang_picker_filter.clear();
                self.lang_picker_selection = 0;
            }
        }

        // Collapse toggle (chevron)
        let collapse_icon = if self.collapsed {
            crate::icons::CHEVRON_RIGHT
        } else {
            crate::icons::CHEVRON_DOWN
        };
        if ui
            .add(
                egui::Button::new(egui::RichText::new(collapse_icon).color(COLOR_MUTED))
                    .fill(egui::Color32::TRANSPARENT),
            )
            .clicked()
        {
            self.collapsed = !self.collapsed;
        }

        // Enabled toggle
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
            bridge.send(SchedulerMessage::SetFrames(
                vec![(li, fi, f)],
                ActionTiming::Immediate,
            ));
        }

        // Language selector
        let lang_btn = ui.add(
            egui::Button::new(
                egui::RichText::new(&self.lang)
                    .small()
                    .color(ui.visuals().text_color()),
            )
            .fill(egui::Color32::TRANSPARENT),
        );
        if lang_btn.clicked() {
            self.lang_picker_open = !self.lang_picker_open;
            self.lang_picker_filter.clear();
            self.lang_picker_selection = 0;
        }

        ui.separator();

        // Duration
        let mut dur = frame.duration;
        let dur_resp = ui.add(
            egui::DragValue::new(&mut dur)
                .range(0.001..=f64::MAX)
                .speed(0.1)
                .prefix("dur: ")
                .custom_formatter(|v, _| format!("{v:.1}")),
        );
        if dur_resp.changed() && dur > 0.0 {
            let mut f = frame.clone();
            f.duration = dur;
            bridge.send(SchedulerMessage::SetFrames(
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
            bridge.send(SchedulerMessage::SetFrames(
                vec![(li, fi, f)],
                ActionTiming::Immediate,
            ));
        }

        // Right-aligned: menu button + dirty indicator + step badge
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

            // Step badge
            if n_frames > 1 {
                let label = format!("{}/{}", fi + 1, n_frames);
                let (badge_bg, badge_fg) = match current_playing_fi {
                    Some(pfi) if fi == pfi => (accent, egui::Color32::WHITE),
                    Some(pfi) if fi < pfi => (
                        egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 50),
                        COLOR_MUTED,
                    ),
                    _ => (ui.visuals().extreme_bg_color, COLOR_MUTED),
                };
                let font = egui::FontId::monospace(10.0);
                let galley = ui.painter().layout_no_wrap(label, font, badge_fg);
                let pad = egui::vec2(5.0, 2.0);
                let desired = galley.size() + pad * 2.0;
                let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
                ui.painter().rect_filled(rect, 0.0, badge_bg);
                let center = rect.center();
                ui.painter().galley(
                    egui::pos2(center.x - galley.size().x / 2.0, center.y - galley.size().y / 2.0),
                    galley,
                    badge_fg,
                );
            }
        });
    }

    /// Returns `Some((li, fi))` if a new empty frame was inserted (for picker auto-open).
    pub fn show_frame_menu(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        fi: usize,
        bridge: &ClientBridge,
        default_lang: &str,
    ) -> Option<(usize, usize)> {
        let mut picker_target = None;
        if ui.button(t!("scene.insert_frame_before")).clicked() {
            bridge.send(SchedulerMessage::AddFrame(
                li, fi, new_frame(default_lang), ActionTiming::Immediate,
            ));
            picker_target = Some((li, fi));
            self.menu_open = false;
            ui.close();
        }
        if ui.button(t!("scene.insert_frame_after")).clicked() {
            bridge.send(SchedulerMessage::AddFrame(
                li, fi + 1, new_frame(default_lang), ActionTiming::Immediate,
            ));
            picker_target = Some((li, fi + 1));
            self.menu_open = false;
            ui.close();
        }
        if ui.button(t!("scene.duplicate_frame")).clicked() {
            if let Some(frame) = bridge
                .scene()
                .and_then(|s| s.lines.get(li))
                .and_then(|l| l.frames.get(fi))
            {
                bridge.send(SchedulerMessage::AddFrame(
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
            bridge.send(SchedulerMessage::RemoveFrame(li, fi, ActionTiming::Immediate));
            self.menu_open = false;
            ui.close();
        }
        picker_target
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

pub struct InlineScriptState {
    pub editor: CodeEditor,
    pub content: String,
    pub lang: String,
    pub dirty: bool,
    pub lang_picker_open: bool,
    pub lang_picker_filter: String,
    pub lang_picker_selection: usize,
    pub height: f32,
    pub editor_has_focus: bool,
    pub request_focus: bool,
    pub last_eval: Option<Instant>,
}

impl InlineScriptState {
    pub fn new(script: &Script) -> Self {
        Self {
            editor: CodeEditor::new(),
            content: script.content().to_owned(),
            lang: script.lang().to_owned(),
            dirty: false,
            lang_picker_open: false,
            lang_picker_filter: String::new(),
            lang_picker_selection: 0,
            height: crate::scene_panel::CELL_HEIGHT,
            editor_has_focus: false,
            request_focus: false,
            last_eval: None,
        }
    }

    pub fn sync_from_script(&mut self, script: &Script) {
        if self.dirty {
            return;
        }
        let remote_content = script.content();
        let remote_lang = script.lang();
        if remote_content != self.content || remote_lang != self.lang {
            self.content = remote_content.to_owned();
            self.lang = remote_lang.to_owned();
        }
    }

    pub fn to_script(&self) -> Script {
        Script::new(self.content.clone(), self.lang.clone())
    }

    pub fn show_header(
        &mut self,
        ui: &mut egui::Ui,
        idx: usize,
        prelude_len: usize,
        bridge: &ClientBridge,
    ) {
        // Subdued style
        let wv = &mut ui.style_mut().visuals.widgets;
        wv.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        wv.inactive.bg_stroke = egui::Stroke::NONE;

        // Cmd/Ctrl+L shortcut — only for the script whose editor has focus
        if self.editor_has_focus {
            let is_mac = ui.ctx().os().is_mac();
            let shortcut_pressed = ui.input(|i| {
                i.key_pressed(egui::Key::L)
                    && if is_mac { i.modifiers.mac_cmd } else { i.modifiers.ctrl }
            });
            if shortcut_pressed {
                self.lang_picker_open = !self.lang_picker_open;
                self.lang_picker_filter.clear();
                self.lang_picker_selection = 0;
            }
        }

        // Language selector
        let lang_btn = ui.add(
            egui::Button::new(
                egui::RichText::new(&self.lang)
                    .small()
                    .color(ui.visuals().text_color()),
            )
            .fill(egui::Color32::TRANSPARENT),
        );
        if lang_btn.clicked() {
            self.lang_picker_open = !self.lang_picker_open;
            self.lang_picker_filter.clear();
            self.lang_picker_selection = 0;
        }

        // Right-aligned: delete button + dirty indicator
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Delete button (only if more than one script, or always allow)
            if prelude_len > 0 {
                let del_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new(crate::icons::CLOSE).small().color(COLOR_MUTED),
                    )
                    .fill(egui::Color32::TRANSPARENT),
                );
                if del_btn.clicked() {
                    let mut scripts: Vec<Script> = bridge
                        .scene()
                        .map(|s| s.prelude.clone())
                        .unwrap_or_default();
                    if idx < scripts.len() {
                        scripts.remove(idx);
                        bridge.send(ClientMessage::SchedulerControl(
                            SchedulerMessage::SetScenePrelude(scripts),
                        ));
                    }
                }
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
                    if let Some(script) = bridge
                        .scene()
                        .and_then(|s| s.prelude.get(idx))
                    {
                        self.content = script.content().to_owned();
                        self.lang = script.lang().to_owned();
                        self.dirty = false;
                    }
                }
            }
        });
    }



    pub fn show_body(
        &mut self,
        ui: &mut egui::Ui,
        idx: usize,
        ctx: &EditorContext,
        bridge: &ClientBridge,
    ) {
        let editor_id = ui.id().with("prelude_editor_body");
        let editor_id_focus = editor_id.with("editor");

        if self.request_focus {
            self.request_focus = false;
            ui.memory_mut(|m| m.request_focus(editor_id_focus));
        }

        egui::ScrollArea::vertical()
            .id_salt(("prelude_editor_scroll", idx))
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
                }
            });

        self.editor_has_focus = ui.memory(|m| m.has_focus(editor_id_focus));
        if self.editor_has_focus {
            let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
            if escape && !self.editor.is_completion_open() {
                ui.memory_mut(|m| m.surrender_focus(editor_id_focus));
                self.editor_has_focus = false;
            }
        }
        if self.editor_has_focus {
            let is_mac = ui.ctx().os().is_mac();
            let eval = ui.input(|i| {
                i.key_pressed(egui::Key::Enter)
                    && if is_mac { i.modifiers.mac_cmd } else { i.modifiers.ctrl }
            });
            if eval {
                self.evaluate(idx, bridge);
            }
        }

        // Eval flash
        if let Some(eval_time) = self.last_eval {
            let elapsed = eval_time.elapsed().as_secs_f32();
            if elapsed < 0.3 {
                let t = elapsed / 0.3;
                let alpha = ((1.0 - t) * 30.0) as u8;
                let is_error = bridge
                    .scene()
                    .and_then(|s| s.prelude.get(idx))
                    .is_some_and(|s| matches!(s.compilation_state(), &CompilationState::Error(_)));
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

    fn evaluate(&mut self, idx: usize, bridge: &ClientBridge) {
        let mut scripts: Vec<Script> = bridge
            .scene()
            .map(|s| s.prelude.clone())
            .unwrap_or_default();
        if idx < scripts.len() {
            scripts[idx] = self.to_script();
        } else {
            scripts.push(self.to_script());
        }
        bridge.send(ClientMessage::SchedulerControl(
            SchedulerMessage::SetScenePrelude(scripts),
        ));
        self.dirty = false;
        self.last_eval = Some(Instant::now());
    }
}
