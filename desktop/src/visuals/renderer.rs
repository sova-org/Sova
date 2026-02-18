use std::sync::Arc;

use glow::HasContext;

use super::shader;

#[derive(Clone, Copy)]
pub struct RenderHandles {
    pub program: glow::Program,
    pub vao: glow::VertexArray,
    pub loc_time: Option<glow::UniformLocation>,
    pub loc_resolution: Option<glow::UniformLocation>,
    pub loc_mouse: Option<glow::UniformLocation>,
}

pub struct ShaderRenderer {
    handles: RenderHandles,
    vbo: glow::Buffer,
    gl: Arc<glow::Context>,
}

impl Drop for ShaderRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.handles.program);
            self.gl.delete_vertex_array(self.handles.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
}

impl ShaderRenderer {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        let (vao, vbo) = create_fullscreen_quad(&gl);
        let program =
            compile_program(&gl, shader::DEFAULT_SHADER).expect("default shader must compile");
        let loc_time = unsafe { gl.get_uniform_location(program, "iTime") };
        let loc_resolution = unsafe { gl.get_uniform_location(program, "iResolution") };
        let loc_mouse = unsafe { gl.get_uniform_location(program, "iMouse") };
        Self {
            handles: RenderHandles {
                program,
                vao,
                loc_time,
                loc_resolution,
                loc_mouse,
            },
            vbo,
            gl,
        }
    }

    pub fn handles(&self) -> RenderHandles {
        self.handles
    }

    pub fn compile(&mut self, user_code: &str) -> Result<(), String> {
        let new_program = compile_program(&self.gl, user_code)?;
        unsafe { self.gl.delete_program(self.handles.program) };
        self.handles.program = new_program;
        self.handles.loc_time =
            unsafe { self.gl.get_uniform_location(self.handles.program, "iTime") };
        self.handles.loc_resolution =
            unsafe { self.gl.get_uniform_location(self.handles.program, "iResolution") };
        self.handles.loc_mouse =
            unsafe { self.gl.get_uniform_location(self.handles.program, "iMouse") };
        Ok(())
    }

}

pub fn render_with_handles(gl: &glow::Context, h: &RenderHandles, time: f32, res: [f32; 2]) {
    unsafe {
        gl.use_program(Some(h.program));
        if let Some(ref loc) = h.loc_time {
            gl.uniform_1_f32(Some(loc), time);
        }
        if let Some(ref loc) = h.loc_resolution {
            gl.uniform_2_f32(Some(loc), res[0], res[1]);
        }
        if let Some(ref loc) = h.loc_mouse {
            gl.uniform_2_f32(Some(loc), 0.0, 0.0);
        }
        gl.bind_vertex_array(Some(h.vao));
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        gl.bind_vertex_array(None);
        gl.use_program(None);
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
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data)) }
}

fn compile_program(gl: &glow::Context, user_code: &str) -> Result<glow::Program, String> {
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

        let frag_src = shader::fragment_source(user_code);
        let frag = gl
            .create_shader(glow::FRAGMENT_SHADER)
            .map_err(|e| e.to_string())?;
        gl.shader_source(frag, &frag_src);
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
