use std::sync::{Arc, Mutex};

use rhai::{CustomType, Dynamic, Engine, Scope, TypeBuilder};

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
}

#[derive(Debug, Clone, CustomType)]
pub struct Node {
    ops: Vec<Op>,
}

#[derive(Debug, Clone)]
enum Op {
    Source {
        func: &'static str,
        args: Vec<f64>,
    },
    Geo {
        func: &'static str,
        args: Vec<f64>,
    },
    Color {
        func: &'static str,
        args: Vec<f64>,
    },
    Blend {
        func: &'static str,
        other: Node,
        args: Vec<f64>,
    },
    Modulate {
        func: &'static str,
        other: Node,
        args: Vec<f64>,
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
    fn source(func: &'static str, args: Vec<f64>) -> Self {
        Self { ops: vec![Op::Source { func, args }] }
    }

    fn push_geo(mut self, func: &'static str, args: Vec<f64>) -> Self {
        self.ops.push(Op::Geo { func, args });
        self
    }

    fn push_color(mut self, func: &'static str, args: Vec<f64>) -> Self {
        self.ops.push(Op::Color { func, args });
        self
    }

    fn push_blend(mut self, func: &'static str, other: Node, args: Vec<f64>) -> Self {
        self.ops.push(Op::Blend { func, other, args });
        self
    }

    fn push_modulate(mut self, func: &'static str, other: Node, args: Vec<f64>) -> Self {
        self.ops.push(Op::Modulate { func, other, args });
        self
    }
}

fn fill_args(provided: &[f64], defaults: &'static [f64]) -> Vec<f64> {
    let mut args = provided.to_vec();
    for d in defaults.iter().skip(args.len()) {
        args.push(*d);
    }
    args
}

fn as_f64(d: Dynamic) -> f64 {
    if let Ok(v) = d.as_float() { v }
    else if let Ok(v) = d.as_int() { v as f64 }
    else { 0.0 }
}

fn fmt_f(v: f64) -> String {
    if v.fract() == 0.0 { format!("{v:.1}") } else { format!("{v}") }
}

fn fmt_args(args: &[f64]) -> String {
    args.iter().map(|a| fmt_f(*a)).collect::<Vec<_>>().join(", ")
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
            let buf_idx = args.first().map(|a| *a as usize).unwrap_or(0).min(3);
            self.lines.push(format!(
                "  vec4 {current_var} = texture(iBuffer{buf_idx}, {current_st});"
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

pub const DEFAULT_SCRIPT: &str = "\
osc(60.0, 0.1).rotate(0.0, 0.1)
    .add(voronoi(8.0, 0.3, 0.3), 0.5)
    .colorama(0.05)
    .out()";

pub fn eval(code: &str) -> Result<EvalResult, String> {
    let state = Arc::new(Mutex::new(PatchState {
        buffers: [None, None, None, None],
        render_mode: RenderMode::default(),
    }));

    let mut engine = Engine::new();
    engine.build_type::<Node>();
    register_functions(&mut engine);

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
        Node::source("src", vec![idx as f64])
    });
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
    })
}

fn register_source(engine: &mut Engine, meta: &FnMeta) {
    let name = meta.name;
    let defaults = meta.defaults;
    let n = defaults.len();

    engine.register_fn(name, move || Node::source(name, fill_args(&[], defaults)));
    if n >= 1 {
        engine.register_fn(name, move |a: Dynamic| {
            Node::source(name, fill_args(&[as_f64(a)], defaults))
        });
    }
    if n >= 2 {
        engine.register_fn(name, move |a: Dynamic, b: Dynamic| {
            Node::source(name, fill_args(&[as_f64(a), as_f64(b)], defaults))
        });
    }
    if n >= 3 {
        engine.register_fn(name, move |a: Dynamic, b: Dynamic, c: Dynamic| {
            Node::source(name, fill_args(&[as_f64(a), as_f64(b), as_f64(c)], defaults))
        });
    }
    if n >= 4 {
        engine.register_fn(name, move |a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| {
            Node::source(name, fill_args(&[as_f64(a), as_f64(b), as_f64(c), as_f64(d)], defaults))
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
            node.push_geo(name, fill_args(&[as_f64(a)], defaults))
        });
    }
    if n >= 2 {
        engine.register_fn(name, move |node: Node, a: Dynamic, b: Dynamic| {
            node.push_geo(name, fill_args(&[as_f64(a), as_f64(b)], defaults))
        });
    }
    if n >= 3 {
        engine.register_fn(name, move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic| {
            node.push_geo(name, fill_args(&[as_f64(a), as_f64(b), as_f64(c)], defaults))
        });
    }
    if n >= 4 {
        engine.register_fn(
            name,
            move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| {
                node.push_geo(name, fill_args(&[as_f64(a), as_f64(b), as_f64(c), as_f64(d)], defaults))
            },
        );
    }
    if n >= 5 {
        engine.register_fn(
            name,
            move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic, e: Dynamic| {
                node.push_geo(
                    name,
                    fill_args(&[as_f64(a), as_f64(b), as_f64(c), as_f64(d), as_f64(e)], defaults),
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
            node.push_color(name, fill_args(&[as_f64(a)], defaults))
        });
    }
    if n >= 2 {
        engine.register_fn(name, move |node: Node, a: Dynamic, b: Dynamic| {
            node.push_color(name, fill_args(&[as_f64(a), as_f64(b)], defaults))
        });
    }
    if n >= 3 {
        engine.register_fn(name, move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic| {
            node.push_color(name, fill_args(&[as_f64(a), as_f64(b), as_f64(c)], defaults))
        });
    }
    if n >= 4 {
        engine.register_fn(
            name,
            move |node: Node, a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| {
                node.push_color(
                    name,
                    fill_args(&[as_f64(a), as_f64(b), as_f64(c), as_f64(d)], defaults),
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
            node.push_blend(name, other, fill_args(&[as_f64(a)], defaults))
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
            node.push_modulate(name, other, fill_args(&[as_f64(a)], defaults))
        });
    }
    if n >= 2 {
        engine.register_fn(name, move |node: Node, other: Node, a: Dynamic, b: Dynamic| {
            node.push_modulate(name, other, fill_args(&[as_f64(a), as_f64(b)], defaults))
        });
    }
    if n >= 3 {
        engine.register_fn(
            name,
            move |node: Node, other: Node, a: Dynamic, b: Dynamic, c: Dynamic| {
                node.push_modulate(
                    name,
                    other,
                    fill_args(&[as_f64(a), as_f64(b), as_f64(c)], defaults),
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
                    fill_args(&[as_f64(a), as_f64(b), as_f64(c), as_f64(d)], defaults),
                )
            },
        );
    }
}
