use std::sync::Arc;
use std::time::Instant;

use eframe::{egui, glow};
use hydra_rust::renderer::{self, RenderUniforms, ShaderRenderer};

pub struct VisualsEngine {
    renderer: Option<ShaderRenderer>,
    start_time: Instant,
    code: String,
}

impl VisualsEngine {
    pub fn new(gl: Option<Arc<glow::Context>>) -> Self {
        Self {
            renderer: gl.map(ShaderRenderer::new),
            start_time: Instant::now(),
            code: String::new(),
        }
    }

    pub fn apply_scheduled_code(&mut self, code: String) {
        if self.code == code {
            return;
        }
        self.code = code;
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        if self.code.is_empty() {
            renderer.compile_buffers(
                &[
                    Some(hydra_rust::shader::DEFAULT_SHADER.to_owned()),
                    None,
                    None,
                    None,
                ],
                Default::default(),
            );
            return;
        }
        match hydra_rust::eval(&self.code) {
            Ok(result) => {
                if let Some(ref td) = result.text_data {
                    renderer.upload_text(td);
                }
                renderer.compile_buffers(&result.shaders, result.render_mode);
            }
            Err(e) => sova_core::log_eprintln!("[hydra] compile error: {e}"),
        }
    }

    pub fn paint_background_central(
        &mut self,
        ctx: &egui::Context,
        enabled: bool,
        beat: f32,
        tempo: f32,
        phase: f32,
    ) {
        if !enabled {
            return;
        }
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        ctx.request_repaint_after(std::time::Duration::from_millis(16));
        let rect = ctx.content_rect();
        let time = self.start_time.elapsed().as_secs_f32();
        let ppp = ctx.pixels_per_point();
        let res_w = (rect.width() * ppp) as u32;
        let res_h = (rect.height() * ppp) as u32;

        let mouse = ctx.input(|i| {
            i.pointer.hover_pos().map_or([0.0, 0.0], |pos| {
                [
                    ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0),
                    (1.0 - (pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0),
                ]
            })
        });

        renderer.ensure_resolution(res_w, res_h);

        let snap = renderer.snapshot();
        let ping = renderer.ping().clone();
        let uniforms = RenderUniforms {
            time,
            resolution: [res_w as f32, res_h as f32],
            mouse,
            beat,
            tempo,
            phase,
        };

        let cb = eframe::egui_glow::CallbackFn::new(move |_info, painter| {
            renderer::render_multipass(painter.gl(), &snap, &ping, uniforms);
        });

        let painter = ctx.layer_painter(egui::LayerId::background());
        painter.add(egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        });
    }
}
