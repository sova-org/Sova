mod glsl;
mod hydra;
mod renderer;
mod shader;

use std::sync::Arc;
use std::time::Instant;

use eframe::egui;

use crate::settings::VisualsSettings;
use crate::widgets::{CodeEditor, EditorSettings, COLOR_ERROR, COLOR_MUTED, COLOR_OK};
use renderer::ShaderRenderer;

pub struct VisualsEngine {
    renderer: Option<ShaderRenderer>,
    hydra_engine: rhai::Engine,
    start_time: Instant,
    error: Option<String>,
    pub open: bool,
    code: String,
    editor: CodeEditor,
    dirty: bool,
    last_eval: Option<Instant>,
    last_cursor_line: Option<usize>,
    last_cursor_col: Option<usize>,
}

impl VisualsEngine {
    pub fn new(gl: Option<Arc<glow::Context>>, settings: &VisualsSettings) -> Self {
        let renderer = gl.map(ShaderRenderer::new);
        let mut engine = Self {
            renderer,
            hydra_engine: hydra::build_engine(),
            start_time: Instant::now(),
            error: None,
            open: false,
            code: settings.code.clone(),
            editor: CodeEditor::new(),
            dirty: false,
            last_eval: None,
            last_cursor_line: None,
            last_cursor_col: None,
        };
        if !engine.code.is_empty() {
            engine.compile_code();
        }
        engine
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn show_editor(&mut self, ctx: &egui::Context, settings: &EditorSettings) {
        if !self.open {
            return;
        }

        let mut open = self.open;
        egui::Window::new(t!("visuals.title"))
            .id(egui::Id::new("visuals_editor"))
            .open(&mut open)
            .default_size([560.0, 420.0])
            .min_size([300.0, 200.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("visuals_header").show_inside(ui, |ui| {
                    self.show_header(ui);
                });

                egui::TopBottomPanel::bottom("visuals_status")
                    .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(4, 1)))
                    .show_inside(ui, |ui| {
                        self.show_status(ui);
                    });

                let body = egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show_inside(ui, |ui| {
                        self.show_body(ui, settings);
                        self.handle_eval_shortcut(ui);
                    });

                if let Some(eval_time) = self.last_eval {
                    let elapsed = eval_time.elapsed().as_secs_f32();
                    if elapsed < 0.3 {
                        let t = elapsed / 0.3;
                        let alpha = ((1.0 - t) * 30.0) as u8;
                        let flash = if self.error.is_some() {
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

    fn show_header(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let accent = ui.visuals().selection.bg_fill;
                    let eval_text = egui::RichText::new(format!(
                        "{} {}",
                        crate::icons::PLAY,
                        t!("visuals.eval")
                    ))
                    .strong();
                    if ui
                        .add(egui::Button::new(eval_text).fill(accent))
                        .clicked()
                    {
                        self.evaluate();
                    }

                    self.show_compilation_dot(ui);

                    if self.dirty {
                        ui.label(
                            egui::RichText::new(crate::icons::MODIFIED).color(COLOR_ERROR),
                        );
                    }
                });
            });
    }

    fn show_compilation_dot(&self, ui: &mut egui::Ui) {
        let (color, tip) = if self.error.is_some() {
            (COLOR_ERROR, t!("visuals.error"))
        } else if self.last_eval.is_some() || (self.renderer.is_some() && !self.dirty) {
            (COLOR_OK, t!("visuals.compiled"))
        } else {
            (COLOR_MUTED, t!("visuals.title"))
        };
        let dot = egui::RichText::new(crate::icons::CIRCLE_LARGE_FILLED).color(color);
        ui.label(dot).on_hover_text(tip);
    }

    fn show_body(&mut self, ui: &mut egui::Ui, settings: &EditorSettings) {
        let editor_id = egui::Id::new("visuals_editor_body");
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let output =
                    self.editor
                        .show(ui, editor_id, &mut self.code, settings, None);
                if output.response.changed() {
                    self.dirty = true;
                }
                self.last_cursor_line = output.cursor_line;
                self.last_cursor_col = output.cursor_col;
            });
    }

    fn show_status(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(e) = &self.error {
                ui.colored_label(COLOR_ERROR, e);
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

    fn handle_eval_shortcut(&mut self, ui: &mut egui::Ui) {
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
            self.evaluate();
        }
    }

    fn evaluate(&mut self) {
        self.compile_code();
        self.dirty = false;
        self.last_eval = Some(Instant::now());
    }

    fn compile_code(&mut self) {
        let code = if self.code.is_empty() {
            hydra::DEFAULT_SCRIPT
        } else {
            &self.code
        };
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        match hydra::eval(&self.hydra_engine, code) {
            Ok(glsl) => match renderer.compile(&glsl) {
                Ok(()) => self.error = None,
                Err(e) => self.error = Some(e),
            },
            Err(e) => self.error = Some(e),
        }
    }

    pub fn paint_background_central(&self, ctx: &egui::Context, enabled: bool) {
        if !enabled {
            return;
        }
        let Some(renderer) = &self.renderer else {
            return;
        };

        ctx.request_repaint();
        let rect = ctx.available_rect();
        let time = self.start_time.elapsed().as_secs_f32();
        let ppp = ctx.pixels_per_point();
        let resolution = [rect.width() * ppp, rect.height() * ppp];
        let handles = renderer.handles();

        let cb = eframe::egui_glow::CallbackFn::new(move |_info, painter| {
            renderer::render_with_handles(painter.gl(), &handles, time, resolution);
        });

        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.add(egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        });
    }

}
