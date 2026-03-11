use std::sync::{Arc, Mutex};

use rhai::{Array, CustomType, Dynamic, Engine, ImmutableString, Scope, TypeBuilder};

use super::text::{self, TextData};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum RenderMode {
    #[default]
    Single0,
    Single(usize),
    All,
}

pub struct EvalResult {
    pub shaders: [Option<String>; 4],
    pub render_mode: RenderMode,
    pub text_data: Option<TextData>,
}

#[derive(Debug, Clone)]
enum Arg {
    Lit(f64),
    Expr(String),
}

#[derive(Debug, Clone)]
struct GlslExpr(String);

#[derive(Debug, Clone)]
struct Pattern {
    values: Vec<f64>,
    speed: f64,
    offset: f64,
    smooth: bool,
}

impl Pattern {
    fn from_array(arr: Array) -> Self {
        let values = arr
            .iter()
            .map(|d| {
                d.as_float()
                    .unwrap_or_else(|_| d.as_int().map(|i| i as f64).unwrap_or(0.0))
            })
            .collect();
        Self { values, speed: 1.0, offset: 0.0, smooth: false }
    }

    fn to_glsl(&self) -> String {
        let n = self.values.len();
        if n == 0 {
            return "0.0".into();
        }
        if n == 1 {
            return fmt_f(self.values[0]);
        }

        let offset_part = if self.offset != 0.0 {
            format!(" + {}", fmt_f(self.offset))
        } else {
            String::new()
        };
        let base = format!("mod(iTime * {}{}, {}.0)", fmt_f(self.speed), offset_part, n);

        let terms: Vec<String> = if self.smooth {
            self.values
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let next = self.values[(i + 1) % n];
                    format!(
                        "mix({}, {}, fract({base})) * step(abs(floor({base}) - {}.0), 0.5)",
                        fmt_f(*v),
                        fmt_f(next),
                        i
                    )
                })
                .collect()
        } else {
            self.values
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    format!(
                        "{} * step(abs(floor({base}) - {}.0), 0.5)",
                        fmt_f(*v),
                        i
                    )
                })
                .collect()
        };

        format!("({})", terms.join(" + "))
    }
}

#[derive(Debug, Clone, CustomType)]
pub struct Node {
    ops: Vec<Op>,
}

#[derive(Debug, Clone)]
enum Op {
    Source {
        func: &'static str,
        args: Vec<Arg>,
    },
    Geo {
        func: &'static str,
        args: Vec<Arg>,
    },
    Color {
        func: &'static str,
        args: Vec<Arg>,
    },
    Blend {
        func: &'static str,
        other: Node,
        args: Vec<Arg>,
    },
    Modulate {
        func: &'static str,
        other: Node,
        args: Vec<Arg>,
    },
}

#[derive(Clone, Copy)]
enum OpKind {
    Source,
    Geo,
    Color,
    Blend,
    Modulate,
}

struct FnMeta {
    name: &'static str,
    kind: OpKind,
    defaults: &'static [f64],
}

const FUNCTIONS: &[FnMeta] = &[
    FnMeta { name: "osc", kind: OpKind::Source, defaults: &[60.0, 0.1, 0.0] },
    FnMeta { name: "noise", kind: OpKind::Source, defaults: &[10.0, 0.1] },
    FnMeta { name: "voronoi", kind: OpKind::Source, defaults: &[5.0, 0.3, 0.3] },
    FnMeta { name: "shape", kind: OpKind::Source, defaults: &[3.0, 0.3, 0.01] },
    FnMeta { name: "gradient", kind: OpKind::Source, defaults: &[0.0] },
    FnMeta { name: "solid", kind: OpKind::Source, defaults: &[0.0, 0.0, 0.0, 1.0] },
    FnMeta { name: "rotate", kind: OpKind::Geo, defaults: &[10.0, 0.0] },
    FnMeta { name: "scale", kind: OpKind::Geo, defaults: &[1.5, 1.0, 1.0, 0.5, 0.5] },
    FnMeta { name: "scroll", kind: OpKind::Geo, defaults: &[0.5, 0.5, 0.0, 0.0] },
    FnMeta { name: "kaleid", kind: OpKind::Geo, defaults: &[4.0] },
    FnMeta { name: "pixelate", kind: OpKind::Geo, defaults: &[20.0, 20.0] },
    FnMeta { name: "repeat", kind: OpKind::Geo, defaults: &[3.0, 3.0, 0.0, 0.0] },
    FnMeta { name: "scrollX", kind: OpKind::Geo, defaults: &[0.5, 0.0] },
    FnMeta { name: "scrollY", kind: OpKind::Geo, defaults: &[0.5, 0.0] },
    FnMeta { name: "repeatX", kind: OpKind::Geo, defaults: &[3.0, 0.0] },
    FnMeta { name: "repeatY", kind: OpKind::Geo, defaults: &[3.0, 0.0] },
    FnMeta { name: "color", kind: OpKind::Color, defaults: &[1.0, 1.0, 1.0, 1.0] },
    FnMeta { name: "invert", kind: OpKind::Color, defaults: &[1.0] },
    FnMeta { name: "contrast", kind: OpKind::Color, defaults: &[1.6] },
    FnMeta { name: "brightness", kind: OpKind::Color, defaults: &[0.4] },
    FnMeta { name: "saturate", kind: OpKind::Color, defaults: &[2.0] },
    FnMeta { name: "hue", kind: OpKind::Color, defaults: &[0.4] },
    FnMeta { name: "posterize", kind: OpKind::Color, defaults: &[3.0, 0.6] },
    FnMeta { name: "luma", kind: OpKind::Color, defaults: &[0.5, 0.1] },
    FnMeta { name: "colorama", kind: OpKind::Color, defaults: &[0.005] },
    FnMeta { name: "shift", kind: OpKind::Color, defaults: &[0.5, 0.0, 0.0, 0.0] },
    FnMeta { name: "thresh", kind: OpKind::Color, defaults: &[0.5, 0.04] },
    FnMeta { name: "r", kind: OpKind::Color, defaults: &[1.0, 0.0] },
    FnMeta { name: "g", kind: OpKind::Color, defaults: &[1.0, 0.0] },
    FnMeta { name: "b", kind: OpKind::Color, defaults: &[1.0, 0.0] },
    FnMeta { name: "add", kind: OpKind::Blend, defaults: &[1.0] },
    FnMeta { name: "mult", kind: OpKind::Blend, defaults: &[1.0] },
    FnMeta { name: "blend", kind: OpKind::Blend, defaults: &[0.5] },
    FnMeta { name: "diff", kind: OpKind::Blend, defaults: &[] },
    FnMeta { name: "layer", kind: OpKind::Blend, defaults: &[] },
    FnMeta { name: "mask", kind: OpKind::Blend, defaults: &[] },
    FnMeta { name: "sub", kind: OpKind::Blend, defaults: &[1.0] },
    FnMeta { name: "modulate", kind: OpKind::Modulate, defaults: &[0.1] },
    FnMeta { name: "modulateScale", kind: OpKind::Modulate, defaults: &[1.0, 1.0] },
    FnMeta { name: "modulateRotate", kind: OpKind::Modulate, defaults: &[1.0, 0.0] },
    FnMeta { name: "modulateRepeat", kind: OpKind::Modulate, defaults: &[3.0, 3.0, 0.5, 0.5] },
    FnMeta { name: "modulateRepeatX", kind: OpKind::Modulate, defaults: &[3.0, 0.5] },
    FnMeta { name: "modulateRepeatY", kind: OpKind::Modulate, defaults: &[3.0, 0.5] },
    FnMeta { name: "modulateKaleid", kind: OpKind::Modulate, defaults: &[4.0] },
    FnMeta { name: "modulateScrollX", kind: OpKind::Modulate, defaults: &[0.5, 0.0] },
    FnMeta { name: "modulateScrollY", kind: OpKind::Modulate, defaults: &[0.5, 0.0] },
    FnMeta { name: "modulatePixelate", kind: OpKind::Modulate, defaults: &[10.0, 13.0] },
    FnMeta { name: "modulateHue", kind: OpKind::Modulate, defaults: &[1.0] },
];

impl Node {
    fn source(func: &'static str, args: Vec<Arg>) -> Self {
        Self { ops: vec![Op::Source { func, args }] }
    }

    fn push_geo(mut self, func: &'static str, args: Vec<Arg>) -> Self {
        self.ops.push(Op::Geo { func, args });
        self
    }

    fn push_color(mut self, func: &'static str, args: Vec<Arg>) -> Self {
        self.ops.push(Op::Color { func, args });
        self
    }

    fn push_blend(mut self, func: &'static str, other: Node, args: Vec<Arg>) -> Self {
        self.ops.push(Op::Blend { func, other, args });
        self
    }

    fn push_modulate(mut self, func: &'static str, other: Node, args: Vec<Arg>) -> Self {
        self.ops.push(Op::Modulate { func, other, args });
        self
    }
}

fn fill_args(provided: &[Arg], defaults: &'static [f64]) -> Vec<Arg> {
    let mut args = provided.to_vec();
    for d in defaults.iter().skip(args.len()) {
        args.push(Arg::Lit(*d));
    }
    args
}

fn as_arg(d: Dynamic) -> Arg {
    if let Ok(v) = d.as_float() {
        Arg::Lit(v)
    } else if let Ok(v) = d.as_int() {
        Arg::Lit(v as f64)
    } else if d.is::<GlslExpr>() {
        Arg::Expr(d.cast::<GlslExpr>().0)
    } else if d.is::<Pattern>() {
        Arg::Expr(d.cast::<Pattern>().to_glsl())
    } else if d.is_array() {
        Arg::Expr(Pattern::from_array(d.into_array().unwrap()).to_glsl())
    } else {
        Arg::Lit(0.0)
    }
}

fn fmt_f(v: f64) -> String {
    if v.fract() == 0.0 { format!("{v:.1}") } else { format!("{v}") }
}

fn fmt_arg(a: &Arg) -> String {
    match a {
        Arg::Lit(v) => fmt_f(*v),
        Arg::Expr(s) => s.clone(),
    }
}

fn fmt_args(args: &[Arg]) -> String {
    args.iter().map(fmt_arg).collect::<Vec<_>>().join(", ")
}

struct Emitter {
    lines: Vec<String>,
    st_counter: usize,
    var_counter: usize,
    depth: usize,
}

const MAX_DEPTH: usize = 16;

impl Emitter {
    fn new() -> Self {
        Self { lines: Vec::new(), st_counter: 0, var_counter: 0, depth: 0 }
    }

    fn next_st(&mut self) -> String {
        let name = format!("_st{}", self.st_counter);
        self.st_counter += 1;
        name
    }

    fn next_var(&mut self) -> String {
        let name = format!("_n{}", self.var_counter);
        self.var_counter += 1;
        name
    }

    fn compile_node(&mut self, node: &Node) -> Result<String, String> {
        if self.depth >= MAX_DEPTH {
            return Err("nesting too deep (max 16)".into());
        }
        self.depth += 1;

        let mut source: Option<&Op> = None;
        let mut geo_mods: Vec<&Op> = Vec::new();
        let mut colors: Vec<&Op> = Vec::new();

        for op in &node.ops {
            match op {
                Op::Source { .. } => {
                    if source.is_some() {
                        return Err("chain must have exactly one source".into());
                    }
                    source = Some(op);
                }
                Op::Geo { .. } | Op::Modulate { .. } => geo_mods.push(op),
                Op::Color { .. } | Op::Blend { .. } => colors.push(op),
            }
        }

        let source = source.ok_or("chain must start with a source (osc, noise, src, etc.)")?;

        let mut current_st = "st".to_string();
        for op in geo_mods.iter().rev() {
            match op {
                Op::Geo { func, args } => {
                    let new_st = self.next_st();
                    let a = fmt_args(args);
                    let sep = if a.is_empty() { "" } else { ", " };
                    self.lines.push(format!(
                        "  vec2 {new_st} = {func}({current_st}{sep}{a});"
                    ));
                    current_st = new_st;
                }
                Op::Modulate { func, other, args } => {
                    let sub_var = self.compile_node(other)?;
                    let new_st = self.next_st();
                    let a = fmt_args(args);
                    let sep = if a.is_empty() { "" } else { ", " };
                    self.lines.push(format!(
                        "  vec2 {new_st} = {func}({current_st}, {sub_var}{sep}{a});"
                    ));
                    current_st = new_st;
                }
                _ => unreachable!(),
            }
        }

        let Op::Source { func, args } = source else { unreachable!() };
        let current_var = self.next_var();

        if *func == "src" {
            let buf_idx = match args.first() {
                Some(Arg::Lit(v)) => (*v as usize).min(3),
                _ => 0,
            };
            self.lines.push(format!(
                "  vec4 {current_var} = texture(iBuffer{buf_idx}, {current_st});"
            ));
        } else if *func == "text_src" {
            self.lines.push(format!(
                "  vec4 {current_var} = texture(iText0, vec2({current_st}.x, 1.0 - {current_st}.y));"
            ));
        } else {
            let a = fmt_args(args);
            let sep = if a.is_empty() { "" } else { ", " };
            self.lines.push(format!(
                "  vec4 {current_var} = {func}({current_st}{sep}{a});"
            ));
        }

        let mut prev_var = current_var;
        for op in &colors {
            match op {
                Op::Color { func, args } => {
                    let new_var = self.next_var();
                    let a = fmt_args(args);
                    let sep = if a.is_empty() { "" } else { ", " };
                    self.lines.push(format!(
                        "  vec4 {new_var} = {func}({prev_var}{sep}{a});"
                    ));
                    prev_var = new_var;
                }
                Op::Blend { func, other, args } => {
                    let sub_var = self.compile_node(other)?;
                    let new_var = self.next_var();
                    let a = fmt_args(args);
                    let sep = if a.is_empty() { "" } else { ", " };
                    self.lines.push(format!(
                        "  vec4 {new_var} = {func}({prev_var}, {sub_var}{sep}{a});"
                    ));
                    prev_var = new_var;
                }
                _ => unreachable!(),
            }
        }

        self.depth -= 1;
        Ok(prev_var)
    }
}

fn compile_node(node: &Node) -> Result<String, String> {
    let mut emitter = Emitter::new();
    let final_var = emitter.compile_node(node)?;
    let body = emitter.lines.join("\n");
    Ok(format!(
        "void mainImage(out vec4 c, in vec2 st) {{\n{body}\n  c = {final_var};\n}}"
    ))
}

struct PatchState {
    buffers: [Option<Node>; 4],
    render_mode: RenderMode,
    text_data: Option<TextData>,
}

fn register_functions(engine: &mut Engine) {
    for meta in FUNCTIONS {
        match meta.kind {
            OpKind::Source => register_source(engine, meta),
            OpKind::Geo => register_geo(engine, meta),
            OpKind::Color => register_color(engine, meta),
            OpKind::Blend => register_blend(engine, meta),
            OpKind::Modulate => register_modulate(engine, meta),
        }
    }
}

fn register_glsl_ops(engine: &mut Engine) {
    macro_rules! binop {
        ($op:literal) => {
            engine.register_fn($op, |a: GlslExpr, b: GlslExpr| -> GlslExpr {
                GlslExpr(format!(concat!("({} ", $op, " {})"), a.0, b.0))
            });
            engine.register_fn($op, |a: GlslExpr, b: f64| -> GlslExpr {
                GlslExpr(format!(concat!("({} ", $op, " {})"), a.0, fmt_f(b)))
            });
            engine.register_fn($op, |a: f64, b: GlslExpr| -> GlslExpr {
                GlslExpr(format!(concat!("({} ", $op, " {})"), fmt_f(a), b.0))
            });
            engine.register_fn($op, |a: GlslExpr, b: i64| -> GlslExpr {
                GlslExpr(format!(concat!("({} ", $op, " {})"), a.0, fmt_f(b as f64)))
            });
            engine.register_fn($op, |a: i64, b: GlslExpr| -> GlslExpr {
                GlslExpr(format!(concat!("({} ", $op, " {})"), fmt_f(a as f64), b.0))
            });
        };
    }

    binop!("+");
    binop!("-");
    binop!("*");
    binop!("/");

    engine.register_fn("-", |a: GlslExpr| -> GlslExpr {
        GlslExpr(format!("(-{})", a.0))
    });

    macro_rules! glsl_fn {
        ($name:literal) => {
            engine.register_fn($name, |a: GlslExpr| -> GlslExpr {
                GlslExpr(format!(concat!($name, "({})"), a.0))
            });
        };
    }

    glsl_fn!("sin");
    glsl_fn!("cos");
    glsl_fn!("abs");
    glsl_fn!("fract");

    engine.register_fn("fract", |x: f64| -> f64 { x.fract() });
}

fn register_patterns(engine: &mut Engine) {
    engine.register_fn("fast", |arr: Array, speed: f64| -> Pattern {
        let mut p = Pattern::from_array(arr);
        p.speed = speed;
        p
    });
    engine.register_fn("fast", |arr: Array, speed: i64| -> Pattern {
        let mut p = Pattern::from_array(arr);
        p.speed = speed as f64;
        p
    });
    engine.register_fn("smooth", |arr: Array| -> Pattern {
        let mut p = Pattern::from_array(arr);
        p.smooth = true;
        p
    });
    engine.register_fn("fast", |mut p: Pattern, speed: f64| -> Pattern {
        p.speed = speed;
        p
    });
    engine.register_fn("fast", |mut p: Pattern, speed: i64| -> Pattern {
        p.speed = speed as f64;
        p
    });
    engine.register_fn("smooth", |mut p: Pattern| -> Pattern {
        p.smooth = true;
        p
    });
    engine.register_fn("offset", |mut p: Pattern, o: f64| -> Pattern {
        p.offset = o;
        p
    });
    engine.register_fn("offset", |mut p: Pattern, o: i64| -> Pattern {
        p.offset = o as f64;
        p
    });
}

pub const DEFAULT_SCRIPT: &str = "\
osc(60.0, 0.1).rotate(0.0, 0.1)
    .add(voronoi(8.0, 0.3, 0.3), 0.5)
    .colorama(0.05)
    .out()";

pub fn eval(code: &str) -> Result<EvalResult, String> {
    let state = Arc::new(Mutex::new(PatchState {
        buffers: [None, None, None, None],
        render_mode: RenderMode::default(),
        text_data: None,
    }));

    let mut engine = Engine::new();
    engine.build_type::<Node>();
    register_functions(&mut engine);
    register_glsl_ops(&mut engine);
    register_patterns(&mut engine);

    // .out() → buffer 0
    {
        let s = state.clone();
        engine.register_fn("out", move |node: Node| {
            s.lock().unwrap().buffers[0] = Some(node);
        });
    }
    // .out(oN) → buffer N
    {
        let s = state.clone();
        engine.register_fn("out", move |node: Node, idx: i64| {
            let i = idx as usize;
            if i < 4 {
                s.lock().unwrap().buffers[i] = Some(node);
            }
        });
    }
    // src(oN) → texture read from buffer N
    engine.register_fn("src", |idx: i64| -> Node {
        Node::source("src", vec![Arg::Lit(idx as f64)])
    });
    // text("string") → rasterize text to iText0 texture
    {
        let s = state.clone();
        engine.register_fn("text", move |txt: ImmutableString| -> Node {
            let data = text::rasterize(&txt);
            s.lock().unwrap().text_data = Some(data);
            Node::source("text_src", vec![])
        });
    }
    // render() → 2x2 grid
    {
        let s = state.clone();
        engine.register_fn("render", move || {
            s.lock().unwrap().render_mode = RenderMode::All;
        });
    }
    // render(oN) → display buffer N
    {
        let s = state.clone();
        engine.register_fn("render", move |idx: i64| {
            s.lock().unwrap().render_mode = RenderMode::Single(idx as usize);
        });
    }

    let mut scope = Scope::new();
    scope.push_constant("o0", 0_i64);
    scope.push_constant("o1", 1_i64);
    scope.push_constant("o2", 2_i64);
    scope.push_constant("o3", 3_i64);
    scope.push_constant("time", GlslExpr("iTime".to_string()));
    scope.push_constant("beat", GlslExpr("iBeat".to_string()));
    scope.push_constant("tempo", GlslExpr("iTempo".to_string()));
    scope.push_constant("phase", GlslExpr("iPhase".to_string()));
    scope.push_constant("mouseX", GlslExpr("iMouse.x".to_string()));
    scope.push_constant("mouseY", GlslExpr("iMouse.y".to_string()));

    let result = engine
        .eval_with_scope::<Dynamic>(&mut scope, code)
        .map_err(|e| e.to_string())?;

    let mut patch = state.lock().unwrap();

    // Backward compat: if no .out() was called, try result as Node → buffer 0
    if patch.buffers.iter().all(|b| b.is_none()) && result.is::<Node>() {
        patch.buffers[0] = result.try_cast::<Node>();
    }

    let mut shaders: [Option<String>; 4] = [None, None, None, None];
    for (i, buf) in patch.buffers.iter().enumerate() {
        if let Some(node) = buf {
            shaders[i] = Some(compile_node(node)?);
        }
    }

    Ok(EvalResult {
        shaders,
        render_mode: patch.render_mode,
        text_data: patch.text_data.take(),
    })
}

fn register_source(engine: &mut Engine, meta: &FnMeta) {
    let name = meta.name;
    let defaults = meta.defaults;
    let n = defaults.len();

    engine.register_fn(name, move || Node::source(name, fill_args(&[], defaults)));
    if n >= 1 {
        engine.register_fn(name, move |a: Dynamic| {
            Node::source(name, fill_args(&[as_arg(a)], defaults))
        });
    }
    if n >= 2 {
        engine.register_fn(name, move |a: Dynamic, b: Dynamic| {
            Node::source(name, fill_args(&[as_arg(a), as_arg(b)], defaults))
        });
    }
    if n >= 3 {
        engine.register_fn(name, move |a: Dynamic, b: Dynamic, c: Dynamic| {
            Node::source(name, fill_args(&[as_arg(a), as_arg(b), as_arg(c)], defaults))
        });
    }
    if n >= 4 {
        engine.register_fn(name, move |a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| {
            Node::source(name, fill_args(&[as_arg(a), as_arg(b), as_arg(c), as_arg(d)], defaults))
        });
    }
}

fn register_geo(engine: &mut Engine, meta: &FnMeta) {
    let name = meta.name;
    let defaults = meta.defaults;
    let n = defaults.len();

    engine.register_fn(name, move |node: Node| node.push_geo(name, fill_args(&[], defaults)));
    if n >= 1 {
        engine.register_fn(name, move |node: Node, a: Dynamic| {
            node.push_geo(name, fill_args(&[as_arg(a)], defaults))
        });
    }
    if n >= 2 {
        engine.register_fn(name, move |node: Node, a: Dynamic, b: Dynamic| {
            node.push_geo(name, fill_args(&[as_arg(a), as_arg(b)], defaults))
        });
    }
    if n >= 3 {
        engine.register_fn(name, move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic| {
            node.push_geo(name, fill_args(&[as_arg(a), as_arg(b), as_arg(c)], defaults))
        });
    }
    if n >= 4 {
        engine.register_fn(
            name,
            move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| {
                node.push_geo(name, fill_args(&[as_arg(a), as_arg(b), as_arg(c), as_arg(d)], defaults))
            },
        );
    }
    if n >= 5 {
        engine.register_fn(
            name,
            move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic, e: Dynamic| {
                node.push_geo(
                    name,
                    fill_args(&[as_arg(a), as_arg(b), as_arg(c), as_arg(d), as_arg(e)], defaults),
                )
            },
        );
    }
}

fn register_color(engine: &mut Engine, meta: &FnMeta) {
    let name = meta.name;
    let defaults = meta.defaults;
    let n = defaults.len();

    engine.register_fn(name, move |node: Node| node.push_color(name, fill_args(&[], defaults)));
    if n >= 1 {
        engine.register_fn(name, move |node: Node, a: Dynamic| {
            node.push_color(name, fill_args(&[as_arg(a)], defaults))
        });
    }
    if n >= 2 {
        engine.register_fn(name, move |node: Node, a: Dynamic, b: Dynamic| {
            node.push_color(name, fill_args(&[as_arg(a), as_arg(b)], defaults))
        });
    }
    if n >= 3 {
        engine.register_fn(name, move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic| {
            node.push_color(name, fill_args(&[as_arg(a), as_arg(b), as_arg(c)], defaults))
        });
    }
    if n >= 4 {
        engine.register_fn(
            name,
            move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| {
                node.push_color(
                    name,
                    fill_args(&[as_arg(a), as_arg(b), as_arg(c), as_arg(d)], defaults),
                )
            },
        );
    }
}

fn register_blend(engine: &mut Engine, meta: &FnMeta) {
    let name = meta.name;
    let defaults = meta.defaults;
    let n = defaults.len();

    engine.register_fn(name, move |node: Node, other: Node| {
        node.push_blend(name, other, fill_args(&[], defaults))
    });
    if n >= 1 {
        engine.register_fn(name, move |node: Node, other: Node, a: Dynamic| {
            node.push_blend(name, other, fill_args(&[as_arg(a)], defaults))
        });
    }
}

fn register_modulate(engine: &mut Engine, meta: &FnMeta) {
    let name = meta.name;
    let defaults = meta.defaults;
    let n = defaults.len();

    engine.register_fn(name, move |node: Node, other: Node| {
        node.push_modulate(name, other, fill_args(&[], defaults))
    });
    if n >= 1 {
        engine.register_fn(name, move |node: Node, other: Node, a: Dynamic| {
            node.push_modulate(name, other, fill_args(&[as_arg(a)], defaults))
        });
    }
    if n >= 2 {
        engine.register_fn(name, move |node: Node, other: Node, a: Dynamic, b: Dynamic| {
            node.push_modulate(name, other, fill_args(&[as_arg(a), as_arg(b)], defaults))
        });
    }
    if n >= 3 {
        engine.register_fn(
            name,
            move |node: Node, other: Node, a: Dynamic, b: Dynamic, c: Dynamic| {
                node.push_modulate(
                    name,
                    other,
                    fill_args(&[as_arg(a), as_arg(b), as_arg(c)], defaults),
                )
            },
        );
    }
    if n >= 4 {
        engine.register_fn(
            name,
            move |node: Node, other: Node, a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| {
                node.push_modulate(
                    name,
                    other,
                    fill_args(&[as_arg(a), as_arg(b), as_arg(c), as_arg(d)], defaults),
                )
            },
        );
    }
}
