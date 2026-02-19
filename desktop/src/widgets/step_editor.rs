use std::collections::HashMap;
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
    last_eval: Option<Instant>,
    last_cursor_line: Option<usize>,
    last_cursor_col: Option<usize>,
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
            last_eval: None,
            last_cursor_line: None,
            last_cursor_col: None,
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

                let body = egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show_inside(ui, |ui| {
                        self.show_body(ui, settings, syntax);
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
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let languages = bridge.languages();
                    egui::ComboBox::from_id_salt(("step_lang", self.line_idx, self.frame_idx))
                        .selected_text(&self.lang)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            for lang in languages {
                                if ui.selectable_label(self.lang == *lang, lang).clicked()
                                    && self.lang != *lang
                                {
                                    self.lang = lang.clone();
                                    self.dirty = true;
                                }
                            }
                        });

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

                    self.show_compilation_dot(ui, bridge);

                    if self.dirty {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let discard_fill = COLOR_ERROR.linear_multiply(0.3);
                                let discard_text =
                                    egui::RichText::new(t!("step.discard").to_string()).small();
                                if ui
                                    .add(egui::Button::new(discard_text).fill(discard_fill))
                                    .clicked()
                                {
                                    self.sync_from_bridge(bridge);
                                }
                                ui.label(
                                    egui::RichText::new(crate::icons::MODIFIED)
                                        .color(COLOR_ERROR),
                                );
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
    ) {
        let editor_id = egui::Id::new(("step_editor_body", self.line_idx, self.frame_idx));
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let output =
                    self.editor
                        .show(ui, editor_id, &mut self.content, settings, syntax);
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
            }

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

pub struct StepEditorManager {
    editors: Vec<StepEditor>,
    syntax_map: HashMap<String, CompiledSyntax>,
}

impl StepEditorManager {
    pub fn new() -> Self {
        let center = langs::create_language_center();
        let mut syntax_map = HashMap::new();
        for (name, (_doc, syn)) in center.all_languages_definitions() {
            if let Some(syn) = syn
                && let Some(compiled) = CompiledSyntax::new(&syn)
            {
                syntax_map.insert(name, compiled);
            }
        }

        Self {
            editors: Vec::new(),
            syntax_map,
        }
    }

    pub fn open(&mut self, li: usize, fi: usize, frame: &Frame) {
        if let Some(editor) = self.editors.iter_mut().find(|e| e.id() == (li, fi)) {
            editor.open = true;
            return;
        }
        self.editors.push(StepEditor::new(li, fi, frame));
    }

    pub fn show(&mut self, ctx: &egui::Context, bridge: &ClientBridge, settings: &EditorSettings) {
        let syntax_map = &self.syntax_map;
        let theme = SyntaxTheme::from_pref(settings.syntax_theme);
        for editor in &mut self.editors {
            if !editor.exists_in_scene(bridge) {
                editor.open = false;
            }
            if editor.open {
                let syntax = syntax_map.get(&editor.lang).map(|cs| (cs, &theme));
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
