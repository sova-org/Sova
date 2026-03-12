use std::time::Instant;

use eframe::egui;
use sova_core::compiler::CompilationState;
use sova_core::scene::Frame;
use sova_core::scene::script::Script;
use sova_core::schedule::ActionTiming;
use sova_server::ClientMessage;

use super::syntax_highlight::{CompiledSyntax, SyntaxTheme};
use super::{COLOR_ERROR, COLOR_MUTED, COLOR_OK, CodeEditor, EditorSettings, username_color};
use crate::client_bridge::ClientBridge;

struct StepEditor {
    line_idx: usize,
    frame_idx: usize,
    editor: CodeEditor,
    content: String,
    lang: String,
    dirty: bool,
    open: bool,
    lang_popup_open: bool,
    lang_filter: String,
    lang_popup_selection: usize,
    last_eval: Option<Instant>,
    last_cursor_line: Option<usize>,
    last_cursor_col: Option<usize>,
    header_name_buf: String,
}

impl StepEditor {
    fn new(li: usize, fi: usize, frame: &Frame) -> Self {
        Self {
            line_idx: li,
            frame_idx: fi,
            editor: CodeEditor::new(),
            content: frame.script().content().to_owned(),
            lang: frame.script().lang().to_owned(),
            dirty: false,
            open: true,
            lang_popup_open: false,
            lang_filter: String::new(),
            lang_popup_selection: 0,
            last_eval: None,
            last_cursor_line: None,
            last_cursor_col: None,
            header_name_buf: frame.name.clone().unwrap_or_default(),
        }
    }

    fn id(&self) -> (usize, usize) {
        (self.line_idx, self.frame_idx)
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        settings: &EditorSettings,
        syntax: Option<(&CompiledSyntax, &SyntaxTheme)>,
    ) {
        let id = egui::Id::new(("step_editor", self.line_idx, self.frame_idx));
        let frame_name = bridge
            .scene()
            .and_then(|s| s.lines.get(self.line_idx))
            .and_then(|l| l.frames.get(self.frame_idx))
            .and_then(|f| f.name.as_deref());
        let title: String = match frame_name {
            Some(name) => t!(
                "step.title",
                lang = &self.lang,
                li = self.line_idx,
                fi = self.frame_idx,
                name = name
            )
            .into(),
            None => t!(
                "step.title_no_name",
                lang = &self.lang,
                li = self.line_idx,
                fi = self.frame_idx
            )
            .into(),
        };

        let mut open = self.open;
        egui::Window::new(title)
            .id(id)
            .open(&mut open)
            .default_size([560.0, 420.0])
            .min_size([300.0, 200.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top(id.with("header")).show_inside(ui, |ui| {
                    self.show_header(ui, bridge);
                    if let Some(editors) =
                        bridge.peer_editing().get(&(self.line_idx, self.frame_idx))
                        && !editors.is_empty()
                    {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(t!("step.also_editing"))
                                    .small()
                                    .color(COLOR_MUTED),
                            );
                            for name in editors {
                                ui.label(
                                    egui::RichText::new(name)
                                        .small()
                                        .strong()
                                        .color(username_color(name)),
                                );
                            }
                        });
                    }
                });

                egui::TopBottomPanel::bottom(id.with("status"))
                    .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(4, 1)))
                    .show_inside(ui, |ui| {
                        self.show_status(ui, bridge);
                    });

                let reference = bridge
                    .languages()
                    .iter()
                    .find(|l| l.name == self.lang)
                    .filter(|l| !l.documentation.reference.is_empty())
                    .map(|l| &l.documentation.reference);

                let body = egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show_inside(ui, |ui| {
                        self.show_body(ui, settings, syntax, reference);
                        self.handle_eval_shortcut(ui, bridge);
                    });

                // Eval flash
                if let Some(eval_time) = self.last_eval {
                    let elapsed = eval_time.elapsed().as_secs_f32();
                    if elapsed < 0.3 {
                        let t = elapsed / 0.3;
                        let alpha = ((1.0 - t) * 30.0) as u8;
                        let is_error = matches!(
                            bridge.compilation_state(self.line_idx, self.frame_idx),
                            Some(CompilationState::Error(_))
                        );
                        let flash = if is_error {
                            egui::Color32::from_rgba_unmultiplied(255, 60, 60, alpha)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha)
                        };
                        ui.painter()
                            .rect_filled(body.response.rect, 0.0, flash);
                        ui.ctx().request_repaint();
                    } else {
                        self.last_eval = None;
                    }
                }
            });
        self.open = open;
    }

    fn show_header(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        // Ctrl+L / Cmd+L shortcut to open language popup (only if this editor is focused)
        if self.has_editor_focus(ui) {
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

        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Language selector button
                    let subtle_fill = ui.visuals().widgets.inactive.bg_fill;
                    let lang_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(format!("{} {}", self.lang, crate::icons::CHEVRON_DOWN)),
                        )
                        .fill(subtle_fill),
                    );
                    if lang_btn.clicked() {
                        self.lang_popup_open = !self.lang_popup_open;
                        self.lang_filter.clear();
                        self.lang_popup_selection = 0;
                    }

                    // Language popup
                    let popup_id =
                        egui::Id::new(("lang_popup", self.line_idx, self.frame_idx));
                    if self.lang_popup_open {
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
                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                    ui.set_min_width(160.0);

                                    // Filter input
                                    let filter_id = popup_id.with("filter");
                                    let filter_resp = ui.add(
                                        egui::TextEdit::singleline(&mut self.lang_filter)
                                            .id(filter_id)
                                            .desired_width(150.0)
                                            .hint_text("Filter..."),
                                    );
                                    filter_resp.request_focus();

                                    // Keyboard navigation
                                    let key_up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
                                    let key_down =
                                        ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
                                    let key_enter =
                                        ui.input(|i| i.key_pressed(egui::Key::Enter));
                                    let key_escape =
                                        ui.input(|i| i.key_pressed(egui::Key::Escape));

                                    if key_escape {
                                        close = true;
                                    }

                                    if !filtered.is_empty() {
                                        if key_up {
                                            self.lang_popup_selection =
                                                self.lang_popup_selection.saturating_sub(1);
                                        }
                                        if key_down {
                                            self.lang_popup_selection = (self
                                                .lang_popup_selection
                                                + 1)
                                            .min(filtered.len() - 1);
                                        }
                                        self.lang_popup_selection =
                                            self.lang_popup_selection.min(
                                                filtered.len().saturating_sub(1),
                                            );

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

                                    // Language list
                                    egui::ScrollArea::vertical()
                                        .max_height(200.0)
                                        .show(ui, |ui| {
                                            for (i, lang) in filtered.iter().enumerate() {
                                                let selected =
                                                    i == self.lang_popup_selection;
                                                let resp = ui.selectable_label(
                                                    selected,
                                                    &lang.name,
                                                );
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

                        // Close on click outside
                        if close
                            || (ui.input(|i| i.pointer.any_pressed())
                                && !area_resp
                                    .response
                                    .rect
                                    .contains(
                                        ui.input(|i| {
                                            i.pointer.interact_pos().unwrap_or_default()
                                        }),
                                    )
                                && !lang_btn.rect.contains(
                                    ui.input(|i| {
                                        i.pointer.interact_pos().unwrap_or_default()
                                    }),
                                ))
                        {
                            self.lang_popup_open = false;
                        }
                    }

                    let accent = ui.visuals().selection.bg_fill;
                    let eval_text = egui::RichText::new(format!(
                        "{} {}",
                        crate::icons::PLAY,
                        t!("step.eval")
                    ))
                    .strong();
                    if ui
                        .add(egui::Button::new(eval_text).fill(accent))
                        .clicked()
                    {
                        self.evaluate(bridge);
                    }

                    ui.add_space(4.0);
                    self.show_compilation_dot(ui, bridge);

                    // Frame properties
                    if let Some(frame) = bridge
                        .scene()
                        .and_then(|s| s.lines.get(self.line_idx))
                        .and_then(|l| l.frames.get(self.frame_idx))
                    {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Name
                        let name_id = egui::Id::new(("step_hdr_name", self.line_idx, self.frame_idx));
                        let name_focused = ui.memory(|m| m.has_focus(name_id));
                        if !name_focused {
                            self.header_name_buf = frame.name.clone().unwrap_or_default();
                        }
                        let name_resp = ui.add(
                            egui::TextEdit::singleline(&mut self.header_name_buf)
                                .id(name_id)
                                .desired_width(100.0)
                                .hint_text("name"),
                        );
                        if name_resp.lost_focus() {
                            let trimmed = self.header_name_buf.trim();
                            let new_name = if trimmed.is_empty() { None } else { Some(trimmed.to_owned()) };
                            if new_name != frame.name {
                                let mut f = frame.clone();
                                f.name = new_name;
                                bridge.send(ClientMessage::SetFrames(
                                    vec![(self.line_idx, self.frame_idx, f)],
                                    ActionTiming::Immediate,
                                ));
                            }
                        }

                        // Duration
                        let mut dur = frame.duration;
                        let dur_resp = ui.add(
                            egui::DragValue::new(&mut dur)
                                .range(0.001..=f64::MAX)
                                .speed(0.1)
                                .suffix("b"),
                        );
                        if dur_resp.changed() && dur > 0.0 {
                            let mut f = frame.clone();
                            f.duration = dur;
                            bridge.send(ClientMessage::SetFrames(
                                vec![(self.line_idx, self.frame_idx, f)],
                                ActionTiming::Immediate,
                            ));
                        }

                        // Repetitions
                        let mut rep = frame.repetitions;
                        let rep_resp = ui.add(
                            egui::DragValue::new(&mut rep)
                                .range(1..=usize::MAX)
                                .prefix("×"),
                        );
                        if rep_resp.changed() && rep > 0 {
                            let mut f = frame.clone();
                            f.repetitions = rep;
                            bridge.send(ClientMessage::SetFrames(
                                vec![(self.line_idx, self.frame_idx, f)],
                                ActionTiming::Immediate,
                            ));
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
                            .add(egui::Button::new(
                                egui::RichText::new(toggle_icon).color(toggle_color),
                            ).fill(egui::Color32::TRANSPARENT))
                            .clicked()
                        {
                            let mut f = frame.clone();
                            f.enabled = !enabled;
                            bridge.send(ClientMessage::SetFrames(
                                vec![(self.line_idx, self.frame_idx, f)],
                                ActionTiming::Immediate,
                            ));
                        }
                    }

                    if self.dirty {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let discard_fill = COLOR_ERROR.linear_multiply(0.3);
                                let discard_text = egui::RichText::new(format!(
                                    "{} {}",
                                    crate::icons::MODIFIED,
                                    t!("step.discard")
                                ))
                                .color(COLOR_ERROR);
                                if ui
                                    .add(egui::Button::new(discard_text).fill(discard_fill))
                                    .clicked()
                                {
                                    self.sync_from_bridge(bridge);
                                }
                            },
                        );
                    }
                });
            });
    }

    fn show_compilation_dot(&self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let state = bridge.compilation_state(self.line_idx, self.frame_idx);
        let (color, tip) = match state {
            Some(CompilationState::Compiled(_)) | Some(CompilationState::Parsed(_)) => {
                (COLOR_OK, t!("step.compiled"))
            }
            Some(CompilationState::Error(_)) => (COLOR_ERROR, t!("step.error")),
            Some(CompilationState::Compiling) => (COLOR_MUTED, t!("step.compiling")),
            _ => (COLOR_MUTED, t!("step.not_compiled")),
        };
        let dot = egui::RichText::new(crate::icons::CIRCLE_LARGE_FILLED).color(color);
        ui.label(dot).on_hover_text(tip);
    }

    fn show_body(
        &mut self,
        ui: &mut egui::Ui,
        settings: &EditorSettings,
        syntax: Option<(&CompiledSyntax, &SyntaxTheme)>,
        reference: Option<&std::collections::BTreeMap<sova_core::vm::language::LanguageElement, sova_core::vm::language::ReferenceEntry>>,
    ) {
        let editor_id = egui::Id::new(("step_editor_body", self.line_idx, self.frame_idx));
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let output =
                    self.editor
                        .show(ui, editor_id, &mut self.content, settings, syntax, reference);
                if output.response.changed() {
                    self.dirty = true;
                }
                self.last_cursor_line = output.cursor_line;
                self.last_cursor_col = output.cursor_col;
            });
    }

    fn show_status(&self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        ui.horizontal(|ui| {
            let state = bridge.compilation_state(self.line_idx, self.frame_idx);
            if let Some(CompilationState::Error(e)) = state {
                ui.colored_label(COLOR_ERROR, &e.info);
            } else if let Some(e) = bridge.errors.get(&(self.line_idx, self.frame_idx)) {
                ui.colored_label(COLOR_ERROR, e.to_string());
            }
        });

        ui.horizontal(|ui| {
            if let (Some(line), Some(col)) = (self.last_cursor_line, self.last_cursor_col) {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("Ln {}, Col {}", line + 1, col + 1))
                            .small()
                            .color(COLOR_MUTED),
                    );
                });
            }
        });
    }

    fn handle_eval_shortcut(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        if !self.has_editor_focus(ui) {
            return;
        }
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
            self.evaluate(bridge);
        }
    }

    fn has_editor_focus(&self, ui: &egui::Ui) -> bool {
        let editor_id =
            egui::Id::new(("step_editor_body", self.line_idx, self.frame_idx)).with("editor");
        ui.memory(|m| m.has_focus(editor_id))
    }

    fn evaluate(&mut self, bridge: &ClientBridge) {
        let Some(frame) = bridge
            .scene()
            .and_then(|s| s.lines.get(self.line_idx))
            .and_then(|l| l.frames.get(self.frame_idx))
        else {
            return;
        };
        let mut f = frame.clone();
        f.set_script(Script::new(self.content.clone(), self.lang.clone()));
        bridge.send(ClientMessage::SetFrames(
            vec![(self.line_idx, self.frame_idx, f)],
            ActionTiming::Immediate,
        ));
        self.dirty = false;
        self.last_eval = Some(Instant::now());
    }

    fn sync_from_bridge(&mut self, bridge: &ClientBridge) {
        if let Some(frame) = bridge
            .scene()
            .and_then(|s| s.lines.get(self.line_idx))
            .and_then(|l| l.frames.get(self.frame_idx))
        {
            self.content = frame.script().content().to_owned();
            self.lang = frame.script().lang().to_owned();
            self.dirty = false;
        }
    }

    fn exists_in_scene(&self, bridge: &ClientBridge) -> bool {
        bridge
            .scene()
            .and_then(|s| s.lines.get(self.line_idx))
            .and_then(|l| l.frames.get(self.frame_idx))
            .is_some()
    }
}

#[derive(Default)]
pub struct StepEditorManager {
    editors: Vec<StepEditor>,
}

impl StepEditorManager {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn open(&mut self, li: usize, fi: usize, frame: &Frame) {
        if let Some(editor) = self.editors.iter_mut().find(|e| e.id() == (li, fi)) {
            editor.open = true;
            return;
        }
        self.editors.push(StepEditor::new(li, fi, frame));
    }

    pub fn show(&mut self, ctx: &egui::Context, bridge: &ClientBridge, settings: &EditorSettings) {
        let theme = SyntaxTheme::from_pref(settings.syntax_theme);
        for editor in &mut self.editors {
            if !editor.exists_in_scene(bridge) {
                editor.open = false;
            }
            if editor.open {
                let syntax = bridge.syntax_map.get(&editor.lang).map(|cs| (cs, &theme));
                editor.show(ctx, bridge, settings, syntax);
            }
        }

        let closed: Vec<_> = self
            .editors
            .iter()
            .filter(|e| !e.open)
            .map(|e| e.id())
            .collect();
        for id in &closed {
            bridge.send(ClientMessage::StoppedEditingFrame(id.0, id.1));
        }
        self.editors.retain(|e| e.open);
    }

    pub fn has_open(&self) -> bool {
        !self.editors.is_empty()
    }

    pub fn close_all(&mut self) {
        self.editors.clear();
    }
}
