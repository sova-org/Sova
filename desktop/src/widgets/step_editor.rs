use eframe::egui;
use sova_core::compiler::CompilationState;
use sova_core::scene::script::Script;
use sova_core::scene::Frame;
use sova_core::schedule::ActionTiming;
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use super::{CodeEditor, EditorSettings, COLOR_OK, COLOR_ERROR, COLOR_MUTED};

struct StepEditor {
    line_idx: usize,
    frame_idx: usize,
    editor: CodeEditor,
    content: String,
    lang: String,
    dirty: bool,
    open: bool,
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
    ) {
        let id = egui::Id::new(("step_editor", self.line_idx, self.frame_idx));
        let frame_name = bridge
            .scene()
            .and_then(|s| s.lines.get(self.line_idx))
            .and_then(|l| l.frames.get(self.frame_idx))
            .and_then(|f| f.name.as_deref());
        let title = match frame_name {
            Some(name) => format!("Step [{}:{}] {}", self.line_idx, self.frame_idx, name),
            None => format!("Step [{}:{}]", self.line_idx, self.frame_idx),
        };

        let mut open = self.open;
        egui::Window::new(title)
            .id(id)
            .open(&mut open)
            .default_size([500.0, 400.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top(id.with("header"))
                    .show_inside(ui, |ui| {
                        self.show_header(ui, bridge);
                    });

                egui::TopBottomPanel::bottom(id.with("status"))
                    .show_inside(ui, |ui| {
                        self.show_status(ui, bridge);
                    });

                egui::CentralPanel::default()
                    .show_inside(ui, |ui| {
                        self.show_body(ui, settings);
                        self.handle_eval_shortcut(ui, bridge);
                    });
            });
        self.open = open;
    }

    fn show_header(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
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

            if ui.button("Eval").clicked() {
                self.evaluate(bridge);
            }

            self.show_compilation_dot(ui, bridge);

            if self.dirty {
                ui.label(egui::RichText::new(crate::icons::MODIFIED).color(COLOR_ERROR));
                if ui.small_button("Discard").clicked() {
                    self.sync_from_bridge(bridge);
                }
            }
        });
    }

    fn show_compilation_dot(&self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let state = bridge.compilation_state(self.line_idx, self.frame_idx);
        let (color, tip) = match state {
            Some(CompilationState::Compiled(_)) | Some(CompilationState::Parsed(_)) => {
                (COLOR_OK, "Compiled")
            }
            Some(CompilationState::Error(_)) => (COLOR_ERROR, "Error"),
            Some(CompilationState::Compiling) => (COLOR_MUTED, "Compiling..."),
            _ => (COLOR_MUTED, "Not compiled"),
        };
        let dot = egui::RichText::new(crate::icons::CIRCLE_LARGE_FILLED).color(color);
        ui.label(dot).on_hover_text(tip);
    }

    fn show_body(&mut self, ui: &mut egui::Ui, settings: &EditorSettings) {
        let editor_id = egui::Id::new(("step_editor_body", self.line_idx, self.frame_idx));
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let output =
                    self.editor
                        .show(ui, editor_id, &mut self.content, settings);
                if output.response.changed() {
                    self.dirty = true;
                }
            });
    }

    fn show_status(&self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let state = bridge.compilation_state(self.line_idx, self.frame_idx);
        match state {
            Some(CompilationState::Error(e)) => {
                ui.colored_label(COLOR_ERROR, &e.info);
            }
            Some(s) => {
                ui.colored_label(COLOR_MUTED, format!("{s}"));
            }
            None => {}
        }
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
}

impl StepEditorManager {
    pub fn new() -> Self {
        Self {
            editors: Vec::new(),
        }
    }

    pub fn open(&mut self, li: usize, fi: usize, frame: &Frame) {
        if let Some(editor) = self.editors.iter_mut().find(|e| e.id() == (li, fi)) {
            editor.open = true;
            return;
        }
        self.editors.push(StepEditor::new(li, fi, frame));
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        settings: &EditorSettings,
    ) {
        for editor in &mut self.editors {
            if !editor.exists_in_scene(bridge) {
                editor.open = false;
            }
            if editor.open {
                editor.show(ctx, bridge, settings);
            }
        }

        let closed: Vec<_> = self.editors.iter()
            .filter(|e| !e.open)
            .map(|e| e.id())
            .collect();
        for id in &closed {
            bridge.send(ClientMessage::StoppedEditingFrame(id.0, id.1));
        }
        self.editors.retain(|e| e.open);
    }

    pub fn close_all(&mut self) {
        self.editors.clear();
    }
}
