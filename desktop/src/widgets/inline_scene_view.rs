use std::time::Instant;

use crate::scene_panel::new_frame;
use eframe::egui;
use sova_core::compiler::CompilationState;
use sova_core::scene::Frame;
use sova_core::scene::script::Script;
use sova_core::schedule::ActionTiming;
use sova_core::schedule::SchedulerMessage;
use sova_server::{ClientMessage, FrameTextId, FrameTextStore};

use super::{CodeEditor, EditorContext};
use crate::client_bridge::ClientBridge;
use crate::theme::{COLOR_ERROR, COLOR_MUTED, COLOR_OK, cycled_accent};

/// Toggle the language picker via Cmd/Ctrl+L when the editor has focus.
/// Resets the picker filter and selection on each toggle.
fn handle_lang_picker_shortcut(
    ui: &egui::Ui,
    editor_has_focus: bool,
    open: &mut bool,
    filter: &mut String,
    selection: &mut usize,
) {
    if !editor_has_focus {
        return;
    }
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
        *open = !*open;
        filter.clear();
        *selection = 0;
    }
}

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
    let cols = ((available.x / 140.0) as usize)
        .max(1)
        .min(filtered.len().max(1));

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
    ui.painter().rect_filled(
        ui.available_rect_before_wrap(),
        0.0,
        ui.visuals().extreme_bg_color,
    );

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
    let tile_h = ((available.y - spacing * (rows as f32 - 1.0)) / rows as f32).clamp(32.0, 64.0);

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
                let btn = egui::Button::new(egui::RichText::new(label).color(text_color))
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FocusRequest {
    None,
    Editor,
    Duration,
    Repetitions,
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
    pub last_cursor: Option<(usize, usize)>,

    pub editor_has_focus: bool,
    pub focus_request: FocusRequest,
    pub escape_pressed: bool,
    pub menu_open: bool,
    pub frame_text_id: Option<FrameTextId>,
    pub last_seen_doc_string: String,
    /// Loro `Cursor` for the local user's caret — persists across frames so it
    /// can be re-resolved into egui's `TextEditState` whenever a remote
    /// `ScriptEdit` shifts the doc under the caret.
    pub local_caret_cursor: Option<loro::cursor::Cursor>,
    pub last_cursor_publish: Instant,
    pub height: f32,
    pub collapsed: bool,
    pub focus_toggled: bool,
}

impl InlineFrameState {
    pub fn new(frame: &Frame) -> Self {
        let content = frame.script().content().to_owned();
        Self {
            editor: CodeEditor::new(),
            last_seen_doc_string: content.clone(),
            content,
            lang: frame.script().lang().to_owned(),
            dirty: false,
            lang_picker_open: false,
            lang_picker_filter: String::new(),
            lang_picker_selection: 0,
            last_eval: None,
            last_cursor: None,

            editor_has_focus: false,
            focus_request: FocusRequest::None,
            escape_pressed: false,
            menu_open: false,
            frame_text_id: None,
            local_caret_cursor: None,
            last_cursor_publish: Instant::now(),
            height: crate::scene_panel::CELL_HEIGHT,
            collapsed: false,
            focus_toggled: false,
        }
    }

    pub fn sync_if_remote_changed(&mut self, frame: &Frame) {
        // Lang is server-authoritative and changes only via SetFrames; sync it
        // from the canonical Frame whenever the user is not editing.
        if frame.script().lang() != self.lang {
            self.lang = frame.script().lang().to_owned();
        }
    }

    pub fn sync_from_frame(&mut self, frame: &Frame) {
        // Discard local edits and reset the projection to the canonical content.
        self.content = frame.script().content().to_owned();
        self.lang = frame.script().lang().to_owned();
        self.dirty = false;
        self.last_seen_doc_string = self.content.clone();
    }

    pub fn evaluate(&mut self, li: usize, fi: usize, frame: &Frame, bridge: &ClientBridge) {
        // Always evaluate the live Loro doc text (in case other peers added
        // characters that haven't been mirrored to self.content yet this frame).
        let live = self
            .frame_text_id
            .and_then(|id| bridge.frame_doc_text(id))
            .unwrap_or_else(|| self.content.clone());
        let mut f = frame.clone();
        f.set_script(Script::new(live, self.lang.clone()));
        bridge.send(SchedulerMessage::SetFrames(
            vec![(li, fi, f)],
            ActionTiming::Immediate,
        ));
        self.dirty = false;
        self.last_eval = Some(Instant::now());
    }

    /// Render the inline language picker. Returns `true` when the picker just
    /// closed (via pick, escape, or click-out) so the caller can re-focus the
    /// editor.
    pub fn show_inline_lang_picker(
        &mut self,
        ui: &mut egui::Ui,
        accent: egui::Color32,
        bridge: &ClientBridge,
    ) -> bool {
        let was_open = self.lang_picker_open;
        if let Some(lang) = show_lang_picker(
            ui,
            &mut self.lang_picker_open,
            &mut self.lang_picker_filter,
            &mut self.lang_picker_selection,
            &self.lang,
            accent,
            bridge,
        ) {
            self.lang = lang;
            self.dirty = true;
            return true;
        }
        was_open && !self.lang_picker_open
    }

    #[allow(clippy::too_many_arguments)]
    pub fn show_header(
        &mut self,
        ui: &mut egui::Ui,
        li: usize,
        fi: usize,
        n_frames: usize,
        playing_fis: &[usize],
        accent: egui::Color32,
        frame: &Frame,
        bridge: &ClientBridge,
        is_focused: bool,
        is_sequencer: bool,
    ) {
        // Subdued style: transparent backgrounds so the header doesn't compete with the code
        let wv = &mut ui.style_mut().visuals.widgets;
        wv.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        wv.inactive.bg_stroke = egui::Stroke::NONE;

        // Cmd/Ctrl+L shortcut — only for the frame whose editor has focus
        handle_lang_picker_shortcut(
            ui,
            self.editor_has_focus,
            &mut self.lang_picker_open,
            &mut self.lang_picker_filter,
            &mut self.lang_picker_selection,
        );

        // Collapse toggle (chevron) — not in sequencer editor panel
        if !is_sequencer {
            let collapse_icon = if self.collapsed {
                crate::icons::CHEVRON_RIGHT
            } else {
                crate::icons::CHEVRON_DOWN
            };
            if ui
                .add(
                    egui::Button::new(crate::icons::rich(collapse_icon).color(COLOR_MUTED))
                        .fill(egui::Color32::TRANSPARENT),
                )
                .clicked()
            {
                self.collapsed = !self.collapsed;
            }
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
                egui::Button::new(crate::icons::rich(toggle_icon).color(toggle_color))
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

        // Duration — if a focus request is pending, grab the id the widget is
        // about to use and request focus on it, so DragValue enters kb edit mode.
        let mut dur = frame.duration;
        if self.focus_request == FocusRequest::Duration {
            self.focus_request = FocusRequest::None;
            let id = ui.next_auto_id();
            ui.memory_mut(|m| m.request_focus(id));
        }
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
        if self.focus_request == FocusRequest::Repetitions {
            self.focus_request = FocusRequest::None;
            let id = ui.next_auto_id();
            ui.memory_mut(|m| m.request_focus(id));
        }
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
                egui::Button::new(crate::icons::small(crate::icons::CHEVRON_DOWN))
                    .fill(egui::Color32::TRANSPARENT),
            );
            if menu_btn.clicked() {
                self.menu_open = !self.menu_open;
            }

            // Focus toggle (not in sequencer mode)
            if !is_sequencer {
                let focus_icon = if is_focused {
                    crate::icons::UNFOCUS
                } else {
                    crate::icons::FOCUS
                };
                if ui
                    .add(
                        egui::Button::new(crate::icons::small(focus_icon).color(COLOR_MUTED))
                            .fill(egui::Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    self.focus_toggled = true;
                }
            }

            // Dirty indicator
            if self.dirty {
                let discard_fill = COLOR_ERROR.linear_multiply(0.3);
                let discard_text = crate::icons::rich(crate::icons::MODIFIED)
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
                let (badge_bg, badge_fg) = if playing_fis.contains(&fi) {
                    (accent, egui::Color32::WHITE)
                } else if playing_fis.iter().any(|&pfi| fi < pfi) {
                    (
                        egui::Color32::from_rgba_unmultiplied(
                            accent.r(),
                            accent.g(),
                            accent.b(),
                            50,
                        ),
                        COLOR_MUTED,
                    )
                } else {
                    (ui.visuals().extreme_bg_color, COLOR_MUTED)
                };
                crate::widgets::shortcut::badge_text(
                    ui,
                    format!("{}/{}", fi + 1, n_frames),
                    badge_bg,
                    badge_fg,
                    10.0,
                    egui::vec2(5.0, 2.0),
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
        let item = |ui: &mut egui::Ui, label: &str| -> bool {
            ui.add(egui::Button::new(label).fill(egui::Color32::TRANSPARENT))
                .clicked()
        };

        if item(ui, &t!("scene.insert_frame_before")) {
            bridge.send(SchedulerMessage::AddFrame(
                li,
                fi,
                new_frame(default_lang),
                ActionTiming::Immediate,
            ));
            picker_target = Some((li, fi));
            self.menu_open = false;
        }
        if item(ui, &t!("scene.insert_frame_after")) {
            bridge.send(SchedulerMessage::AddFrame(
                li,
                fi + 1,
                new_frame(default_lang),
                ActionTiming::Immediate,
            ));
            picker_target = Some((li, fi + 1));
            self.menu_open = false;
        }
        if item(ui, &t!("scene.duplicate_frame")) {
            if let Some(frame) = bridge
                .scene()
                .and_then(|s| s.lines.get(li))
                .and_then(|l| l.frames.get(fi))
            {
                bridge.send(SchedulerMessage::AddFrame(
                    li,
                    fi + 1,
                    frame.clone(),
                    ActionTiming::Immediate,
                ));
            }
            self.menu_open = false;
        }

        ui.separator();

        if ui
            .add(
                egui::Button::new(egui::RichText::new(t!("scene.remove_frame")).color(COLOR_ERROR))
                    .fill(egui::Color32::TRANSPARENT),
            )
            .clicked()
        {
            bridge.send(SchedulerMessage::RemoveFrame(
                li,
                fi,
                ActionTiming::Immediate,
            ));
            self.menu_open = false;
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
        if self.focus_request == FocusRequest::Editor {
            self.focus_request = FocusRequest::None;
            ui.memory_mut(|m| m.request_focus(editor_id_focus));
        }

        // Resolve our FrameTextId from the current layout, refresh the projection
        // from the Loro doc if remote ops have moved it ahead of last_seen_doc_string,
        // and re-anchor egui's caret using the saved Loro Cursor so the local user's
        // caret follows the character it was on through remote inserts/deletes.
        self.frame_text_id = bridge.frame_text_id_at(li, fi);
        if let Some(id) = self.frame_text_id
            && let Some(doc) = bridge.frame_doc(id)
        {
            let live = doc.get_text(FrameTextStore::CONTENT_CONTAINER).to_string();
            if live != self.last_seen_doc_string {
                if let Some(cur) = &self.local_caret_cursor
                    && let Ok(pq) = doc.get_cursor_pos(cur)
                    && let Some(mut state) =
                        egui::widgets::text_edit::TextEditState::load(ui.ctx(), editor_id_focus)
                {
                    let new_cp = pq.current.pos;
                    let mut range = state.cursor.char_range().unwrap_or_else(|| {
                        egui::text::CCursorRange::two(
                            egui::text::CCursor::new(new_cp),
                            egui::text::CCursor::new(new_cp),
                        )
                    });
                    range.primary = egui::text::CCursor::new(new_cp);
                    range.secondary = egui::text::CCursor::new(new_cp);
                    state.cursor.set_char_range(Some(range));
                    state.store(ui.ctx(), editor_id_focus);
                }
                self.content = live.clone();
                self.last_seen_doc_string = live;
            }
        }

        egui::ScrollArea::vertical()
            .id_salt(("editor_scroll", li, fi))
            .auto_shrink(false)
            .show(ui, |ui| {
                let output = self.editor.show(ui, editor_id, &mut self.content, ctx);
                if output.response.changed() {
                    self.dirty = true;
                    if let Some(id) = self.frame_text_id
                        && let Some(doc) = bridge.frame_doc(id)
                    {
                        let text = doc.get_text(FrameTextStore::CONTENT_CONTAINER);
                        apply_local_diff_to_loro(&text, &self.last_seen_doc_string, &self.content);
                        doc.commit();
                        self.last_seen_doc_string = self.content.clone();
                    }
                }
                self.last_cursor = output.cursor_line.zip(output.cursor_col);
            });

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

        // Capture the local caret as a Loro Cursor every frame the editor has
        // focus. The same cursor anchors egui's caret across remote ops AND is
        // what we publish for peer presence (throttled to ~10 Hz).
        if self.editor_has_focus
            && let Some(id) = self.frame_text_id
            && let Some(doc) = bridge.frame_doc(id)
            && let Some(state) =
                egui::widgets::text_edit::TextEditState::load(ui.ctx(), editor_id_focus)
            && let Some(range) = state.cursor.char_range()
        {
            let text = doc.get_text(FrameTextStore::CONTENT_CONTAINER);
            self.local_caret_cursor =
                text.get_cursor(range.primary.index, loro::cursor::Side::Left);

            if self.last_cursor_publish.elapsed().as_millis() >= 100
                && let Some(cursor) = &self.local_caret_cursor
                && let Some(name) = bridge.confirmed_username()
                && !name.is_empty()
            {
                let frame_key = format!("peer/{}/cursor_frame", name);
                let pos_key = format!("peer/{}/cursor_pos", name);
                bridge
                    .presence
                    .set(&frame_key, loro::LoroValue::I64(id.0 as i64));
                bridge.presence.set(
                    &pos_key,
                    loro::LoroValue::Binary(cursor.encode().into()),
                );
                self.last_cursor_publish = Instant::now();
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
    pub escape_pressed: bool,
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
            escape_pressed: false,
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

    /// Render the inline language picker. Returns `true` when the picker just
    /// closed so the caller can re-focus the editor.
    pub fn show_inline_lang_picker(
        &mut self,
        ui: &mut egui::Ui,
        accent: egui::Color32,
        bridge: &ClientBridge,
    ) -> bool {
        let was_open = self.lang_picker_open;
        if let Some(lang) = show_lang_picker(
            ui,
            &mut self.lang_picker_open,
            &mut self.lang_picker_filter,
            &mut self.lang_picker_selection,
            &self.lang,
            accent,
            bridge,
        ) {
            self.lang = lang;
            self.dirty = true;
            return true;
        }
        was_open && !self.lang_picker_open
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
        handle_lang_picker_shortcut(
            ui,
            self.editor_has_focus,
            &mut self.lang_picker_open,
            &mut self.lang_picker_filter,
            &mut self.lang_picker_selection,
        );

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
                        crate::icons::rich(crate::icons::CLOSE)
                            .small()
                            .color(COLOR_MUTED),
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
                let discard_text = crate::icons::rich(crate::icons::MODIFIED)
                    .small()
                    .color(COLOR_ERROR);
                if ui
                    .add(egui::Button::new(discard_text).fill(discard_fill))
                    .on_hover_text(t!("step.discard"))
                    .clicked()
                    && let Some(script) = bridge.scene().and_then(|s| s.prelude.get(idx))
                {
                    self.content = script.content().to_owned();
                    self.lang = script.lang().to_owned();
                    self.dirty = false;
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
                let output = self.editor.show(ui, editor_id, &mut self.content, ctx);
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


fn apply_local_diff_to_loro(text: &loro::LoroText, prev: &str, new: &str) {
    if prev == new {
        return;
    }
    let old_chars: Vec<char> = prev.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();
    let prefix = old_chars
        .iter()
        .zip(new_chars.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let old_rem = old_chars.len() - prefix;
    let new_rem = new_chars.len() - prefix;
    let suffix = old_chars[prefix..]
        .iter()
        .rev()
        .zip(new_chars[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(old_rem)
        .min(new_rem);
    let del_codepoints = old_chars.len() - prefix - suffix;
    let ins_text: String = new_chars[prefix..new_chars.len() - suffix].iter().collect();
    if del_codepoints > 0 {
        let _ = text.delete(prefix, del_codepoints);
    }
    if !ins_text.is_empty() {
        let _ = text.insert(prefix, &ins_text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loro::LoroDoc;

    #[test]
    fn local_diff_round_trip_through_loro() {
        let doc = LoroDoc::new();
        let text = doc.get_text("content");
        text.insert(0, "abc").unwrap();
        doc.commit();
        apply_local_diff_to_loro(&text, "abc", "aXbc");
        doc.commit();
        assert_eq!(text.to_string(), "aXbc");
    }

    #[test]
    fn local_diff_handles_multibyte() {
        let doc = LoroDoc::new();
        let text = doc.get_text("content");
        text.insert(0, "héllo").unwrap();
        doc.commit();
        apply_local_diff_to_loro(&text, "héllo", "hZéllo");
        doc.commit();
        assert_eq!(text.to_string(), "hZéllo");
    }

    #[test]
    fn local_diff_pure_delete() {
        let doc = LoroDoc::new();
        let text = doc.get_text("content");
        text.insert(0, "hello world").unwrap();
        doc.commit();
        apply_local_diff_to_loro(&text, "hello world", "hello");
        doc.commit();
        assert_eq!(text.to_string(), "hello");
    }

    #[test]
    fn two_docs_converge_via_export_import() {
        let a = LoroDoc::new();
        a.set_peer_id(1).unwrap();
        let b = LoroDoc::new();
        b.set_peer_id(2).unwrap();
        a.get_text("content").insert(0, "Hello").unwrap();
        a.commit();
        b.get_text("content").insert(0, "World").unwrap();
        b.commit();

        let a_to_b = a.export(loro::ExportMode::all_updates()).unwrap();
        let b_to_a = b.export(loro::ExportMode::all_updates()).unwrap();
        a.import(&b_to_a).unwrap();
        b.import(&a_to_b).unwrap();

        assert_eq!(
            a.get_text("content").to_string(),
            b.get_text("content").to_string()
        );
    }
}
