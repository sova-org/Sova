use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use glow::{HasContext, PixelUnpackData};

use super::hydra::RenderMode;
use super::shader;

#[derive(Clone, Copy)]
pub struct RenderUniforms {
    pub time: f32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 2],
    pub beat: f32,
    pub tempo: f32,
    pub phase: f32,
}

const NUM_BUFFERS: usize = 4;

#[derive(Clone, Copy)]
struct ProgramState {
    program: glow::Program,
    loc_time: Option<glow::UniformLocation>,
    loc_resolution: Option<glow::UniformLocation>,
    loc_mouse: Option<glow::UniformLocation>,
    loc_beat: Option<glow::UniformLocation>,
    loc_tempo: Option<glow::UniformLocation>,
    loc_phase: Option<glow::UniformLocation>,
    loc_buffers: [Option<glow::UniformLocation>; NUM_BUFFERS],
    loc_text0: Option<glow::UniformLocation>,
}

#[derive(Clone, Copy)]
struct BufferTarget {
    fbo: [glow::Framebuffer; 2],
    texture: [glow::Texture; 2],
}

#[derive(Clone, Copy)]
pub struct RenderSnapshot {
    targets: [BufferTarget; NUM_BUFFERS],
    programs: [Option<ProgramState>; NUM_BUFFERS],
    display: ProgramState,
    vao: glow::VertexArray,
    render_mode: RenderMode,
    text_texture: Option<glow::Texture>,
}

pub struct ShaderRenderer {
    snapshot: RenderSnapshot,
    vbo: glow::Buffer,
    gl: Arc<glow::Context>,
    resolution: [u32; 2],
    ping: Arc<AtomicBool>,
}

impl Drop for ShaderRenderer {
    fn drop(&mut self) {
        unsafe {
            for target in &self.snapshot.targets {
                for fbo in &target.fbo {
                    self.gl.delete_framebuffer(*fbo);
                }
                for tex in &target.texture {
                    self.gl.delete_texture(*tex);
                }
            }
            for prog in self.snapshot.programs.iter().flatten() {
                self.gl.delete_program(prog.program);
            }
            self.gl.delete_program(self.snapshot.display.program);
            if let Some(tex) = self.snapshot.text_texture {
                self.gl.delete_texture(tex);
            }
            self.gl.delete_vertex_array(self.snapshot.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
}

impl ShaderRenderer {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        let (vao, vbo) = create_fullscreen_quad(&gl);
        let resolution = [512, 512];
        let targets = std::array::from_fn(|_| create_buffer_target(&gl, resolution));
        let display_program = compile_program(&gl, &shader::display_fragment_source())
            .expect("display shader must compile");
        let display = resolve_program_state(&gl, display_program);

        let default_program =
            compile_program(&gl, &shader::fragment_source(shader::DEFAULT_SHADER))
                .expect("default shader must compile");
        let slot0 = Some(resolve_program_state(&gl, default_program));

        Self {
            snapshot: RenderSnapshot {
                targets,
                programs: [slot0, None, None, None],
                display,
                vao,
                render_mode: RenderMode::Single0,
                text_texture: None,
            },
            vbo,
            gl,
            resolution,
            ping: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn ping(&self) -> &Arc<AtomicBool> {
        &self.ping
    }

    pub fn snapshot(&self) -> RenderSnapshot {
        self.snapshot
    }

    pub fn ensure_resolution(&mut self, width: u32, height: u32) {
        if self.resolution == [width, height] || width == 0 || height == 0 {
            return;
        }
        self.resolution = [width, height];
        unsafe {
            for target in &self.snapshot.targets {
                for i in 0..2 {
                    self.gl
                        .bind_texture(glow::TEXTURE_2D, Some(target.texture[i]));
                    self.gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        glow::RGBA8 as i32,
                        width as i32,
                        height as i32,
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        PixelUnpackData::Slice(None),
                    );
                }
            }
            self.gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    pub fn upload_text(&mut self, data: &super::text::TextData) {
        unsafe {
            let tex = self.snapshot.text_texture.unwrap_or_else(|| {
                let tex = self.gl.create_texture().expect("create text texture");
                self.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                self.gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::LINEAR as i32,
                );
                self.gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::LINEAR as i32,
                );
                self.gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_S,
                    glow::CLAMP_TO_EDGE as i32,
                );
                self.gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_T,
                    glow::CLAMP_TO_EDGE as i32,
                );
                tex
            });

            self.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                data.width as i32,
                data.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(Some(&data.pixels)),
            );
            self.gl.bind_texture(glow::TEXTURE_2D, None);
            self.snapshot.text_texture = Some(tex);
        }
    }

    pub fn compile_buffers(
        &mut self,
        shaders: &[Option<String>; NUM_BUFFERS],
        render_mode: RenderMode,
    ) {
        for (i, shader_src) in shaders.iter().enumerate() {
            if let Some(old) = self.snapshot.programs[i].take() {
                unsafe { self.gl.delete_program(old.program) };
            }
            if let Some(code) = shader_src {
                let full_src = shader::fragment_source(code);
                if let Ok(program) = compile_program(&self.gl, &full_src) {
                    self.snapshot.programs[i] =
                        Some(resolve_program_state(&self.gl, program));
                }
            }
        }
        self.snapshot.render_mode = render_mode;
    }
}

fn resolve_program_state(gl: &glow::Context, program: glow::Program) -> ProgramState {
    unsafe {
        ProgramState {
            program,
            loc_time: gl.get_uniform_location(program, "iTime"),
            loc_resolution: gl.get_uniform_location(program, "iResolution"),
            loc_mouse: gl.get_uniform_location(program, "iMouse"),
            loc_beat: gl.get_uniform_location(program, "iBeat"),
            loc_tempo: gl.get_uniform_location(program, "iTempo"),
            loc_phase: gl.get_uniform_location(program, "iPhase"),
            loc_buffers: [
                gl.get_uniform_location(program, "iBuffer0"),
                gl.get_uniform_location(program, "iBuffer1"),
                gl.get_uniform_location(program, "iBuffer2"),
                gl.get_uniform_location(program, "iBuffer3"),
            ],
            loc_text0: gl.get_uniform_location(program, "iText0"),
        }
    }
}

fn create_buffer_target(gl: &glow::Context, resolution: [u32; 2]) -> BufferTarget {
    unsafe {
        let mut textures: [Option<glow::Texture>; 2] = [None, None];
        let mut fbos: [Option<glow::Framebuffer>; 2] = [None, None];

        for i in 0..2 {
            let tex = gl.create_texture().expect("create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                resolution[0] as i32,
                resolution[1] as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );

            let fbo = gl.create_framebuffer().expect("create FBO");
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(tex),
                0,
            );

            textures[i] = Some(tex);
            fbos[i] = Some(fbo);
        }

        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.bind_texture(glow::TEXTURE_2D, None);

        BufferTarget {
            fbo: [fbos[0].unwrap(), fbos[1].unwrap()],
            texture: [textures[0].unwrap(), textures[1].unwrap()],
        }
    }
}

pub fn render_multipass(
    gl: &glow::Context,
    snap: &RenderSnapshot,
    ping: &AtomicBool,
    u: RenderUniforms,
) {
    let write = ping.load(Ordering::Relaxed) as usize;
    let read = 1 - write;

    unsafe {
        let saved_fbo = gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING);
        let mut saved_vp = [0_i32; 4];
        gl.get_parameter_i32_slice(glow::VIEWPORT, &mut saved_vp);
        let blend_was_on = gl.is_enabled(glow::BLEND);

        // Compute passes write directly to FBOs — blending must be off.
        // Hydra handles blending in the shader (layer, blend, etc.), not via GL state.
        gl.disable(glow::BLEND);

        let res_w = u.resolution[0] as i32;
        let res_h = u.resolution[1] as i32;

        for (i, prog) in snap.programs.iter().enumerate() {
            let Some(p) = prog else { continue };

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(snap.targets[i].fbo[write]));
            gl.viewport(0, 0, res_w, res_h);
            gl.use_program(Some(p.program));

            if let Some(ref loc) = p.loc_time {
                gl.uniform_1_f32(Some(loc), u.time);
            }
            if let Some(ref loc) = p.loc_beat {
                gl.uniform_1_f32(Some(loc), u.beat);
            }
            if let Some(ref loc) = p.loc_tempo {
                gl.uniform_1_f32(Some(loc), u.tempo);
            }
            if let Some(ref loc) = p.loc_phase {
                gl.uniform_1_f32(Some(loc), u.phase);
            }
            if let Some(ref loc) = p.loc_resolution {
                gl.uniform_2_f32(Some(loc), u.resolution[0], u.resolution[1]);
            }
            if let Some(ref loc) = p.loc_mouse {
                gl.uniform_2_f32(Some(loc), u.mouse[0], u.mouse[1]);
            }

            for (j, loc) in p.loc_buffers.iter().enumerate() {
                if let Some(loc) = loc {
                    gl.active_texture(glow::TEXTURE0 + j as u32);
                    gl.bind_texture(
                        glow::TEXTURE_2D,
                        Some(snap.targets[j].texture[read]),
                    );
                    gl.uniform_1_i32(Some(loc), j as i32);
                }
            }

            if let (Some(tex), Some(loc)) = (snap.text_texture, &p.loc_text0) {
                gl.active_texture(glow::TEXTURE0 + NUM_BUFFERS as u32);
                gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                gl.uniform_1_i32(Some(loc), NUM_BUFFERS as i32);
            }

            gl.bind_vertex_array(Some(snap.vao));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }

        // Flip ping-pong
        ping.store(write == 0, Ordering::Relaxed);

        // Restore original FBO and blending for the display pass
        let restored_fbo = std::num::NonZeroU32::new(saved_fbo as u32)
            .map(glow::NativeFramebuffer);
        gl.bind_framebuffer(glow::FRAMEBUFFER, restored_fbo);
        gl.viewport(saved_vp[0], saved_vp[1], saved_vp[2], saved_vp[3]);
        if blend_was_on {
            gl.enable(glow::BLEND);
        }

        // Display pass
        let d = &snap.display;
        gl.use_program(Some(d.program));

        if let Some(ref loc) = d.loc_resolution {
            gl.uniform_2_f32(Some(loc), u.resolution[0], u.resolution[1]);
        }

        match snap.render_mode {
            RenderMode::Single0 => {
                draw_display_buffer(gl, snap, d, write, 0);
            }
            RenderMode::Single(n) => {
                draw_display_buffer(gl, snap, d, write, n.min(NUM_BUFFERS - 1));
            }
            RenderMode::All => {
                let hw = saved_vp[2] / 2;
                let hh = saved_vp[3] / 2;
                for (idx, (vx, vy)) in
                    [(0, hh), (hw, hh), (0, 0), (hw, 0)].iter().enumerate()
                {
                    gl.viewport(saved_vp[0] + vx, saved_vp[1] + vy, hw, hh);
                    draw_display_buffer(gl, snap, d, write, idx);
                }
                gl.viewport(saved_vp[0], saved_vp[1], saved_vp[2], saved_vp[3]);
            }
        }

        gl.bind_vertex_array(None);
        gl.use_program(None);
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, None);
    }
}

fn draw_display_buffer(
    gl: &glow::Context,
    snap: &RenderSnapshot,
    d: &ProgramState,
    write: usize,
    buf_idx: usize,
) {
    unsafe {
        if let Some(ref loc) = d.loc_buffers[0] {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(
                glow::TEXTURE_2D,
                Some(snap.targets[buf_idx].texture[write]),
            );
            gl.uniform_1_i32(Some(loc), 0);
        }
        gl.bind_vertex_array(Some(snap.vao));
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
    }
}

fn create_fullscreen_quad(gl: &glow::Context) -> (glow::VertexArray, glow::Buffer) {
    #[rustfmt::skip]
    let vertices: [f32; 8] = [
        -1.0, -1.0,
         1.0, -1.0,
        -1.0,  1.0,
         1.0,  1.0,
    ];

    unsafe {
        let vao = gl.create_vertex_array().expect("create VAO");
        let vbo = gl.create_buffer().expect("create VBO");

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            as_u8_slice(&vertices),
            glow::STATIC_DRAW,
        );
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
        gl.bind_vertex_array(None);

        (vao, vbo)
    }
}

fn as_u8_slice(data: &[f32]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
    }
}

fn compile_program(gl: &glow::Context, frag_src: &str) -> Result<glow::Program, String> {
    unsafe {
        let vert = gl
            .create_shader(glow::VERTEX_SHADER)
            .map_err(|e| e.to_string())?;
        gl.shader_source(vert, shader::vertex_source());
        gl.compile_shader(vert);
        if !gl.get_shader_compile_status(vert) {
            let log = gl.get_shader_info_log(vert);
            gl.delete_shader(vert);
            return Err(format!("vertex shader: {log}"));
        }

        let frag = gl
            .create_shader(glow::FRAGMENT_SHADER)
            .map_err(|e| e.to_string())?;
        gl.shader_source(frag, frag_src);
        gl.compile_shader(frag);
        if !gl.get_shader_compile_status(frag) {
            let log = gl.get_shader_info_log(frag);
            gl.delete_shader(vert);
            gl.delete_shader(frag);
            return Err(format!("fragment shader: {log}"));
        }

        let program = gl.create_program().map_err(|e| e.to_string())?;
        gl.attach_shader(program, vert);
        gl.attach_shader(program, frag);
        gl.link_program(program);
        gl.delete_shader(vert);
        gl.delete_shader(frag);

        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            gl.delete_program(program);
            return Err(format!("link: {log}"));
        }

        Ok(program)
    }
}
