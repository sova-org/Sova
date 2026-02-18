use super::glsl;

const VERTEX_SHADER: &str = "\
#version 330 core
layout(location = 0) in vec2 a_position;
void main() {
  gl_Position = vec4(a_position, 0.0, 1.0);
}";

const FRAGMENT_PREAMBLE: &str = "\
#version 330 core
precision highp float;
uniform float iTime;
uniform vec2 iResolution;
uniform vec2 iMouse;
out vec4 fragColor;
";

const FRAGMENT_MAIN_WRAP: &str = "
void main() {
  vec2 uv = gl_FragCoord.xy / iResolution.xy;
  vec2 st = vec2(uv.x, uv.y);
  mainImage(fragColor, st);
}";

pub const DEFAULT_SHADER: &str = "\
void mainImage(out vec4 c, in vec2 st) {
  vec4 o = osc(st, 60.0, 0.1, 0.0);
  st = rotate(st, 0.0, 0.1);
  vec4 v = voronoi(st, 8.0, 0.3, 0.3);
  c = add(o, v, 0.5);
  c = colorama(c, iTime * 0.05);
}";

pub fn vertex_source() -> &'static str {
    VERTEX_SHADER
}

pub fn fragment_source(user_code: &str) -> String {
    format!(
        "{preamble}\n{library}\n{user_code}\n{main}",
        preamble = FRAGMENT_PREAMBLE,
        library = glsl::LIBRARY,
        main = FRAGMENT_MAIN_WRAP,
    )
}
