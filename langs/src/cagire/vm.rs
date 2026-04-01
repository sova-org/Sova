use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use sova_core::clock::SyncTime;
use sova_core::protocol::osc::OSCMessage;
use sova_core::vm::event::ConcreteEvent;
use sova_core::vm::variable::{Variable, VariableValue};
use sova_core::vm::EvaluationContext;

use super::compiler::{compile_script, Dictionary};
use super::ops::Op;
use super::pattern;
use super::types::{float_to_value, CagireError, CmdRegister, ResolvedValue, Span, Stack, Value};

pub(super) struct StepContext {
    pub step: usize,
    pub beat: f64,
    pub tempo: f64,
    pub phase: f64,
    pub slot: usize,
    pub runs: usize,
    pub iter: usize,
    pub speed: f64,
    pub step_duration: f64,
    pub frame_index: usize,
    pub nudge_secs: f64,
    pub default_device: usize,
}

impl StepContext {
    pub fn from_eval_ctx(ctx: &EvaluationContext) -> Self {
        let tempo = ctx.clock.tempo();
        let beat = ctx.clock.beat();
        let quantum = ctx.clock.quantum();
        let speed = 1.0;
        let step_duration = if tempo > 0.0 && speed > 0.0 {
            ctx.frame_len * 60.0 / tempo
        } else {
            0.5
        };
        Self {
            step: ctx.frame_triggers,
            beat,
            tempo,
            phase: if quantum > 0.0 {
                (beat / quantum).fract()
            } else {
                0.0
            },
            slot: ctx.line_index,
            runs: ctx.frame_triggers,
            iter: ctx.line_iterations,
            speed,
            step_duration,
            frame_index: ctx.frame_index,
            nudge_secs: 0.0,
            default_device: 1,
        }
    }
}

pub(super) struct CagireVM {
    source: String,
    pub vars: HashMap<String, Value>,
    pub dict: Dictionary,
    pub rng: StdRng,
    global_params: Vec<(&'static str, Value)>,
    pub resolved: Vec<(Span, ResolvedValue)>,
    pub selected: Vec<Span>,
    pub event_annotations: Vec<Vec<Span>>,
    active_emit_annotations: Vec<Span>,
}

impl CagireVM {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            source: String::new(),
            vars: HashMap::new(),
            dict: Dictionary::new(),
            rng: StdRng::from_os_rng(),
            global_params: Vec::new(),
            resolved: Vec::new(),
            selected: Vec::new(),
            event_annotations: Vec::new(),
            active_emit_annotations: Vec::new(),
        }
    }

    pub fn with_dict(dict: Dictionary) -> Self {
        Self {
            source: String::new(),
            vars: HashMap::new(),
            dict,
            rng: StdRng::from_os_rng(),
            global_params: Vec::new(),
            resolved: Vec::new(),
            selected: Vec::new(),
            event_annotations: Vec::new(),
            active_emit_annotations: Vec::new(),
        }
    }

    pub fn evaluate(
        &mut self,
        script: &str,
        ctx: &mut EvaluationContext,
    ) -> Result<Vec<(ConcreteEvent, SyncTime)>, CagireError> {
        self.source.clear();
        self.source.push_str(script);
        self.resolved.clear();
        self.selected.clear();
        self.event_annotations.clear();
        self.active_emit_annotations.clear();
        if script.trim().is_empty() {
            return Err(CagireError::new("empty script", Span::default()));
        }
        let (ops, spans) = compile_script(script, &mut self.dict)?;
        let sctx = StepContext::from_eval_ctx(ctx);
        let mut stack = Stack::new();
        let mut events = Vec::with_capacity(8);
        let mut cmd = CmdRegister::new();
        cmd.set_global(self.global_params.clone());
        self.execute_ops(&ops, &spans, &sctx, ctx, &mut stack, &mut events, &mut cmd)?;
        self.global_params = cmd.take_global();
        Ok(events)
    }

    fn push_event(
        &mut self,
        events: &mut Vec<(ConcreteEvent, SyncTime)>,
        event: ConcreteEvent,
        time: SyncTime,
    ) {
        events.push((event, time));
        self.event_annotations
            .push(self.active_emit_annotations.clone());
    }

    fn pattern_hit_span(&self, origin: Span, local_start: usize, local_end: usize) -> Option<Span> {
        if origin == Span::default() || local_start >= local_end {
            return None;
        }
        let token = self.source.get(origin.start..origin.end)?;
        if !token.starts_with('"') || !token.ends_with('"') || token.len() < 2 {
            return Some(origin);
        }

        let content_start = origin.start + 1;
        let content_end = origin.end - 1;
        let start = content_start + local_start;
        let end = content_start + local_end;
        if start >= end || end > content_end {
            return Some(origin);
        }

        Some(Span { start, end })
    }

    fn execute_ops(
        &mut self,
        ops: &[Op],
        op_spans: &[Span],
        ctx: &StepContext,
        eval_ctx: &mut EvaluationContext,
        stack: &mut Stack,
        events: &mut Vec<(ConcreteEvent, SyncTime)>,
        cmd: &mut CmdRegister,
    ) -> Result<(), CagireError> {
        let mut pc = 0;
        let mut marks: Vec<usize> = Vec::new();

        macro_rules! at {
            ($expr:expr) => {
                $expr.map_err(|msg: String| {
                    CagireError::new(msg, op_spans.get(pc).copied().unwrap_or_default())
                })
            };
        }

        macro_rules! span {
            () => {
                op_spans.get(pc).copied().unwrap_or_default()
            };
        }

        while pc < ops.len() {
            match &ops[pc] {
                Op::PushInt(n) => stack.push(Value::Int(*n), span!()),
                Op::PushFloat(f) => stack.push(Value::Float(*f), span!()),
                Op::PushStr(s) => stack.push(Value::Str(s.clone()), span!()),

                Op::Dup => {
                    at!(stack.ensure(1))?;
                    let v = stack.last().unwrap().clone();
                    let o = stack.origin(stack.len() - 1);
                    stack.push(v, o);
                }
                Op::Dupn => {
                    let n = at!(stack.pop_int())?;
                    let o = stack.origin(stack.len() - 1);
                    let v = at!(stack.pop())?;
                    for _ in 0..n {
                        stack.push(v.clone(), o);
                    }
                }
                Op::Drop => {
                    at!(stack.pop())?;
                }
                Op::Swap => {
                    at!(stack.ensure(2))?;
                    let len = stack.len();
                    stack.swap(len - 1, len - 2);
                }
                Op::Over => {
                    at!(stack.ensure(2))?;
                    let v = stack.values[stack.len() - 2].clone();
                    let o = stack.origin(stack.len() - 2);
                    stack.push(v, o);
                }
                Op::Rot => {
                    at!(stack.ensure(3))?;
                    let idx = stack.len() - 3;
                    let o = stack.origin(idx);
                    let v = stack.remove(idx);
                    stack.push(v, o);
                }
                Op::Nip => {
                    at!(stack.ensure(2))?;
                    stack.remove(stack.len() - 2);
                }
                Op::Tuck => {
                    at!(stack.ensure(2))?;
                    let len = stack.len();
                    let v = stack.values[len - 1].clone();
                    let o = stack.origin(len - 1);
                    stack.insert(len - 2, v, o);
                }
                Op::Dup2 => {
                    at!(stack.ensure(2))?;
                    let len = stack.len();
                    let a = stack.values[len - 2].clone();
                    let oa = stack.origin(len - 2);
                    let b = stack.values[len - 1].clone();
                    let ob = stack.origin(len - 1);
                    stack.push(a, oa);
                    stack.push(b, ob);
                }
                Op::Drop2 => {
                    at!(stack.ensure(2))?;
                    stack.pop().ok();
                    stack.pop().ok();
                }
                Op::Swap2 => {
                    at!(stack.ensure(4))?;
                    let len = stack.len();
                    stack.swap(len - 4, len - 2);
                    stack.swap(len - 3, len - 1);
                }
                Op::Over2 => {
                    at!(stack.ensure(4))?;
                    let len = stack.len();
                    let a = stack.values[len - 4].clone();
                    let oa = stack.origin(len - 4);
                    let b = stack.values[len - 3].clone();
                    let ob = stack.origin(len - 3);
                    stack.push(a, oa);
                    stack.push(b, ob);
                }
                Op::Rev => {
                    let count = at!(stack.pop_int())? as usize;
                    at!(stack.ensure(count))?;
                    let start = stack.len() - count;
                    stack.values[start..].reverse();
                    stack.origins[start..].reverse();
                }
                Op::Shuffle => {
                    let count = at!(stack.pop_int())? as usize;
                    at!(stack.ensure(count))?;
                    let start = stack.len() - count;
                    let n = stack.len() - start;
                    for i in (1..n).rev() {
                        let j = self.rng.random_range(0..=i);
                        stack.values[start..].swap(i, j);
                        stack.origins[start..].swap(i, j);
                    }
                }
                Op::Sort => {
                    let count = at!(stack.pop_int())? as usize;
                    at!(stack.ensure(count))?;
                    let start = stack.len() - count;
                    sort_paired(
                        &mut stack.values[start..],
                        &mut stack.origins[start..],
                        false,
                    );
                }
                Op::RSort => {
                    let count = at!(stack.pop_int())? as usize;
                    at!(stack.ensure(count))?;
                    let start = stack.len() - count;
                    sort_paired(
                        &mut stack.values[start..],
                        &mut stack.origins[start..],
                        true,
                    );
                }
                Op::Sum => {
                    let count = at!(stack.pop_int())? as usize;
                    at!(stack.ensure(count))?;
                    let start = stack.len() - count;
                    let total: f64 = stack
                        .values
                        .drain(start..)
                        .map(|v| v.as_float().unwrap_or(0.0))
                        .sum();
                    stack.origins.truncate(start);
                    stack.push(float_to_value(total), span!());
                }
                Op::Prod => {
                    let count = at!(stack.pop_int())? as usize;
                    at!(stack.ensure(count))?;
                    let start = stack.len() - count;
                    let product: f64 = stack
                        .values
                        .drain(start..)
                        .map(|v| v.as_float().unwrap_or(1.0))
                        .product();
                    stack.origins.truncate(start);
                    stack.push(float_to_value(product), span!());
                }

                Op::Add => at!(stack.binary_op(|a, b| a + b))?,
                Op::Sub => at!(stack.binary_op(|a, b| a - b))?,
                Op::Mul => at!(stack.binary_op(|a, b| a * b))?,
                Op::Div => {
                    let b = at!(stack.pop())?;
                    let a = at!(stack.pop())?;
                    if b.as_float().map_or(true, |v| v == 0.0) {
                        return Err(CagireError::new("division by zero", span!()));
                    }
                    stack.push(at!(lift_binary(a, b, |x, y| x / y))?, span!());
                }
                Op::Mod => {
                    let b = at!(stack.pop())?;
                    let a = at!(stack.pop())?;
                    if b.as_float().map_or(true, |v| v == 0.0) {
                        return Err(CagireError::new("modulo by zero", span!()));
                    }
                    stack.push(
                        at!(lift_binary(a, b, |x, y| (x as i64 % y as i64) as f64))?,
                        span!(),
                    );
                }
                Op::Neg => {
                    let v = at!(stack.pop())?;
                    stack.push(at!(lift_unary(v, |x| -x))?, span!());
                }
                Op::Abs => {
                    let v = at!(stack.pop())?;
                    stack.push(at!(lift_unary(v, |x| x.abs()))?, span!());
                }
                Op::Floor => {
                    let v = at!(stack.pop())?;
                    stack.push(at!(lift_unary(v, |x| x.floor()))?, span!());
                }
                Op::Ceil => {
                    let v = at!(stack.pop())?;
                    stack.push(at!(lift_unary(v, |x| x.ceil()))?, span!());
                }
                Op::Round => {
                    let v = at!(stack.pop())?;
                    stack.push(at!(lift_unary(v, |x| x.round()))?, span!());
                }
                Op::Min => at!(stack.binary_op(|a, b| a.min(b)))?,
                Op::Max => at!(stack.binary_op(|a, b| a.max(b)))?,
                Op::Pow => at!(stack.binary_op(|a, b| a.powf(b)))?,
                Op::Sqrt => {
                    let v = at!(stack.pop())?;
                    stack.push(at!(lift_unary(v, |x| x.sqrt()))?, span!());
                }
                Op::Sin => {
                    let v = at!(stack.pop())?;
                    stack.push(at!(lift_unary(v, |x| x.sin()))?, span!());
                }
                Op::Cos => {
                    let v = at!(stack.pop())?;
                    stack.push(at!(lift_unary(v, |x| x.cos()))?, span!());
                }
                Op::Log => {
                    let v = at!(stack.pop())?;
                    stack.push(at!(lift_unary(v, |x| x.ln()))?, span!());
                }

                Op::Eq => at!(stack.cmp_op(|a, b| (a - b).abs() < f64::EPSILON))?,
                Op::Ne => at!(stack.cmp_op(|a, b| (a - b).abs() >= f64::EPSILON))?,
                Op::Lt => at!(stack.cmp_op(|a, b| a < b))?,
                Op::Gt => at!(stack.cmp_op(|a, b| a > b))?,
                Op::Le => at!(stack.cmp_op(|a, b| a <= b))?,
                Op::Ge => at!(stack.cmp_op(|a, b| a >= b))?,

                Op::And => {
                    let b = at!(stack.pop_bool())?;
                    let a = at!(stack.pop_bool())?;
                    stack.push(Value::Int(if a && b { 1 } else { 0 }), span!());
                }
                Op::Or => {
                    let b = at!(stack.pop_bool())?;
                    let a = at!(stack.pop_bool())?;
                    stack.push(Value::Int(if a || b { 1 } else { 0 }), span!());
                }
                Op::Not => {
                    let v = at!(stack.pop_bool())?;
                    stack.push(Value::Int(if v { 0 } else { 1 }), span!());
                }
                Op::Xor => {
                    let b = at!(stack.pop_bool())?;
                    let a = at!(stack.pop_bool())?;
                    stack.push(Value::Int(if a ^ b { 1 } else { 0 }), span!());
                }
                Op::Nand => {
                    let b = at!(stack.pop_bool())?;
                    let a = at!(stack.pop_bool())?;
                    stack.push(Value::Int(if !(a && b) { 1 } else { 0 }), span!());
                }
                Op::Nor => {
                    let b = at!(stack.pop_bool())?;
                    let a = at!(stack.pop_bool())?;
                    stack.push(Value::Int(if !(a || b) { 1 } else { 0 }), span!());
                }

                Op::BranchIfZero(offset) => {
                    let v = at!(stack.pop())?;
                    if !v.is_truthy() {
                        pc += offset;
                    }
                }
                Op::Branch(offset) => {
                    pc += offset;
                }

                Op::NewCmd => {
                    at!(stack.ensure(1))?;
                    let values = drain_skip_quotations(stack);
                    if values.is_empty() {
                        return Err(CagireError::new("expected sound name", span!()));
                    }
                    let val = if values.len() == 1 {
                        values.into_iter().next().unwrap()
                    } else {
                        Value::CycleList(Arc::from(values))
                    };
                    cmd.set_sound(val);
                }
                Op::SetParam(param) => {
                    at!(stack.ensure(1))?;
                    let values = drain_skip_quotations(stack);
                    if values.is_empty() {
                        return Err(CagireError::new("expected parameter value", span!()));
                    }
                    let val = if values.len() == 1 {
                        values.into_iter().next().unwrap()
                    } else {
                        Value::CycleList(Arc::from(values))
                    };
                    cmd.set_param(param, val);
                }

                Op::Emit => {
                    at!(self.emit_events(cmd, ctx, events))?;
                }

                Op::Get => {
                    let name = at!(stack.pop())?;
                    let name = at!(name.as_str())?;
                    let val = self.get_var(name, eval_ctx);
                    stack.push(val, span!());
                }
                Op::Set => {
                    let name = at!(stack.pop())?;
                    let name = at!(name.as_str())?.to_string();
                    let val = at!(stack.pop())?;
                    self.set_var(&name, val, eval_ctx);
                }
                Op::SetKeep => {
                    let name = at!(stack.pop())?;
                    let name = at!(name.as_str())?.to_string();
                    let val =
                        at!(stack.last().ok_or_else(|| "stack underflow".to_string()))?.clone();
                    self.set_var(&name, val, eval_ctx);
                }

                Op::GetContext(name) => {
                    let val = match *name {
                        "step" => Value::Int(ctx.step as i64),
                        "beat" => Value::Float(ctx.beat),
                        "pattern" => Value::Int(ctx.frame_index as i64),
                        "tempo" => Value::Float(ctx.tempo),
                        "phase" => Value::Float(ctx.phase),
                        "slot" => Value::Int(ctx.slot as i64),
                        "runs" => Value::Int(ctx.runs as i64),
                        "iter" => Value::Int(ctx.iter as i64),
                        "speed" => Value::Float(ctx.speed),
                        "stepdur" => Value::Float(ctx.step_duration),
                        "fill" => Value::Int(0), // deferred
                        _ => Value::Int(0),
                    };
                    stack.push(val, span!());
                }

                Op::Rand(word_span) => {
                    let b = at!(stack.pop())?;
                    let a = at!(stack.pop())?;
                    match (&a, &b) {
                        (Value::Int(a_i), Value::Int(b_i)) => {
                            let (lo, hi) = if a_i <= b_i {
                                (*a_i, *b_i)
                            } else {
                                (*b_i, *a_i)
                            };
                            let val = self.rng.random_range(lo..=hi);
                            if let Some(s) = word_span {
                                self.resolved.push((*s, ResolvedValue::Int(val)));
                            }
                            stack.push(Value::Int(val), span!());
                        }
                        _ => {
                            let a_f = at!(a.as_float())?;
                            let b_f = at!(b.as_float())?;
                            let (lo, hi) = if a_f <= b_f { (a_f, b_f) } else { (b_f, a_f) };
                            let val = if (hi - lo).abs() < f64::EPSILON {
                                lo
                            } else {
                                self.rng.random_range(lo..hi)
                            };
                            if let Some(s) = word_span {
                                self.resolved.push((*s, ResolvedValue::Float(val)));
                            }
                            stack.push(Value::Float(val), span!());
                        }
                    }
                }
                Op::ExpRand(word_span) => {
                    let hi = at!(stack.pop_float())?;
                    let lo = at!(stack.pop_float())?;
                    if lo <= 0.0 || hi <= 0.0 {
                        return Err(CagireError::new(
                            "exprand requires positive values",
                            span!(),
                        ));
                    }
                    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
                    let u: f64 = self.rng.random();
                    let val = lo * (hi / lo).powf(u);
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Float(val)));
                    }
                    stack.push(Value::Float(val), span!());
                }
                Op::LogRand(word_span) => {
                    let hi = at!(stack.pop_float())?;
                    let lo = at!(stack.pop_float())?;
                    if lo <= 0.0 || hi <= 0.0 {
                        return Err(CagireError::new(
                            "logrand requires positive values",
                            span!(),
                        ));
                    }
                    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
                    let u: f64 = self.rng.random();
                    let val = hi * (lo / hi).powf(u);
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Float(val)));
                    }
                    stack.push(Value::Float(val), span!());
                }
                Op::Seed => {
                    let s = at!(stack.pop_int())?;
                    self.rng = StdRng::seed_from_u64(s as u64);
                }

                Op::Cycle(word_span) | Op::PCycle(word_span) => {
                    let count = at!(stack.pop_int())? as usize;
                    if count == 0 {
                        return Err(CagireError::new("cycle count must be > 0", span!()));
                    }
                    let idx = match &ops[pc] {
                        Op::Cycle(_) => ctx.runs,
                        _ => ctx.iter,
                    } % count;
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Int(idx as i64)));
                    }
                    drain_select_run(
                        count, idx, stack, events, cmd, self, op_spans, pc, ctx, eval_ctx,
                    )?;
                }

                Op::Choose(word_span) => {
                    let count = at!(stack.pop_int())? as usize;
                    if count == 0 {
                        return Err(CagireError::new("choose count must be > 0", span!()));
                    }
                    let idx = self.rng.random_range(0..count);
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Int(idx as i64)));
                    }
                    drain_select_run(
                        count, idx, stack, events, cmd, self, op_spans, pc, ctx, eval_ctx,
                    )?;
                }

                Op::Bounce(word_span) | Op::PBounce(word_span) => {
                    let count = at!(stack.pop_int())? as usize;
                    if count == 0 {
                        return Err(CagireError::new("bounce count must be > 0", span!()));
                    }
                    let counter = match &ops[pc] {
                        Op::Bounce(_) => ctx.runs,
                        _ => ctx.iter,
                    };
                    let idx = if count == 1 {
                        0
                    } else {
                        let period = 2 * (count - 1);
                        let raw = counter % period;
                        if raw < count {
                            raw
                        } else {
                            period - raw
                        }
                    };
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Int(idx as i64)));
                    }
                    drain_select_run(
                        count, idx, stack, events, cmd, self, op_spans, pc, ctx, eval_ctx,
                    )?;
                }

                Op::Index(word_span) => {
                    let idx = at!(stack.pop_int())?;
                    let count = at!(stack.pop_int())? as usize;
                    if count == 0 {
                        return Err(CagireError::new("index count must be > 0", span!()));
                    }
                    let resolved_idx =
                        ((idx % count as i64 + count as i64) % count as i64) as usize;
                    if let Some(s) = word_span {
                        self.resolved
                            .push((*s, ResolvedValue::Int(resolved_idx as i64)));
                    }
                    drain_select_run(
                        count,
                        resolved_idx,
                        stack,
                        events,
                        cmd,
                        self,
                        op_spans,
                        pc,
                        ctx,
                        eval_ctx,
                    )?;
                }

                Op::WChoose(word_span) => {
                    let count = at!(stack.pop_int())? as usize;
                    if count == 0 {
                        return Err(CagireError::new("wchoose count must be > 0", span!()));
                    }
                    let pairs_needed = count * 2;
                    at!(stack.ensure(pairs_needed))?;
                    let start = stack.len() - pairs_needed;
                    let mut values = Vec::with_capacity(count);
                    let mut weights = Vec::with_capacity(count);
                    for i in 0..count {
                        let val = stack.values[start + i * 2].clone();
                        let w = at!(stack.values[start + i * 2 + 1].as_float())?;
                        if w < 0.0 {
                            return Err(CagireError::new("wchoose: negative weight", span!()));
                        }
                        values.push(val);
                        weights.push(w);
                    }
                    stack.truncate(start);
                    let total: f64 = weights.iter().sum();
                    if total <= 0.0 {
                        return Err(CagireError::new(
                            "wchoose: total weight must be > 0",
                            span!(),
                        ));
                    }
                    let threshold: f64 = self.rng.random::<f64>() * total;
                    let mut cumulative = 0.0;
                    let mut selected_idx = count - 1;
                    for (i, &w) in weights.iter().enumerate() {
                        cumulative += w;
                        if threshold < cumulative {
                            selected_idx = i;
                            break;
                        }
                    }
                    if let Some(s) = word_span {
                        self.resolved
                            .push((*s, ResolvedValue::Int(selected_idx as i64)));
                    }
                    let selected = values.swap_remove(selected_idx);
                    select_and_run(selected, stack, events, cmd, self, ctx, eval_ctx)?;
                }

                Op::ChanceExec(word_span) | Op::ProbExec(word_span) => {
                    let threshold = at!(stack.pop_float())?;
                    let (quot, quot_origin) = at!(stack.pop_with_origin())?;
                    let val: f64 = self.rng.random();
                    let limit = match &ops[pc] {
                        Op::ChanceExec(_) => threshold,
                        _ => threshold / 100.0,
                    };
                    let hit = val < limit;
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Bool(hit)));
                    }
                    if hit {
                        if quot_origin != Span::default() {
                            self.selected.push(quot_origin);
                        }
                        run_quotation(quot, stack, events, cmd, self, ctx, eval_ctx)?;
                    }
                }

                Op::Coin(word_span) => {
                    let val: f64 = self.rng.random();
                    let result = val < 0.5;
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Bool(result)));
                    }
                    stack.push(Value::Int(if result { 1 } else { 0 }), span!());
                }

                Op::Every(word_span) => {
                    let n = at!(stack.pop_int())?;
                    let (quot, quot_origin) = at!(stack.pop_with_origin())?;
                    if n <= 0 {
                        return Err(CagireError::new("every count must be > 0", span!()));
                    }
                    let hit = ctx.iter as i64 % n == 0;
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Bool(hit)));
                    }
                    if hit {
                        if quot_origin != Span::default() {
                            self.selected.push(quot_origin);
                        }
                        run_quotation(quot, stack, events, cmd, self, ctx, eval_ctx)?;
                    }
                }

                Op::Except(word_span) => {
                    let n = at!(stack.pop_int())?;
                    let (quot, quot_origin) = at!(stack.pop_with_origin())?;
                    if n <= 0 {
                        return Err(CagireError::new("except count must be > 0", span!()));
                    }
                    let hit = ctx.iter as i64 % n != 0;
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Bool(hit)));
                    }
                    if hit {
                        if quot_origin != Span::default() {
                            self.selected.push(quot_origin);
                        }
                        run_quotation(quot, stack, events, cmd, self, ctx, eval_ctx)?;
                    }
                }

                Op::EveryOffset(word_span) => {
                    let offset = at!(stack.pop_int())?;
                    let n = at!(stack.pop_int())?;
                    let (quot, quot_origin) = at!(stack.pop_with_origin())?;
                    if n <= 0 {
                        return Err(CagireError::new("every+ count must be > 0", span!()));
                    }
                    let hit = ctx.iter as i64 % n == offset.rem_euclid(n);
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Bool(hit)));
                    }
                    if hit {
                        if quot_origin != Span::default() {
                            self.selected.push(quot_origin);
                        }
                        run_quotation(quot, stack, events, cmd, self, ctx, eval_ctx)?;
                    }
                }

                Op::ExceptOffset(word_span) => {
                    let offset = at!(stack.pop_int())?;
                    let n = at!(stack.pop_int())?;
                    let (quot, quot_origin) = at!(stack.pop_with_origin())?;
                    if n <= 0 {
                        return Err(CagireError::new("except+ count must be > 0", span!()));
                    }
                    let hit = ctx.iter as i64 % n != offset.rem_euclid(n);
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Bool(hit)));
                    }
                    if hit {
                        if quot_origin != Span::default() {
                            self.selected.push(quot_origin);
                        }
                        run_quotation(quot, stack, events, cmd, self, ctx, eval_ctx)?;
                    }
                }

                Op::Bjork(word_span) | Op::PBjork(word_span) => {
                    let n = at!(stack.pop_int())?;
                    let k = at!(stack.pop_int())?;
                    let (quot, quot_origin) = at!(stack.pop_with_origin())?;
                    if n <= 0 || k < 0 {
                        return Err(CagireError::new(
                            "bjork: n must be > 0, k must be >= 0",
                            span!(),
                        ));
                    }
                    let counter = match &ops[pc] {
                        Op::Bjork(_) => ctx.runs,
                        _ => ctx.iter,
                    };
                    let pos = counter % n as usize;
                    let hit = k >= n || euclidean_hit(k as usize, n as usize, pos);
                    if let Some(s) = word_span {
                        self.resolved.push((*s, ResolvedValue::Bool(hit)));
                    }
                    if hit {
                        if quot_origin != Span::default() {
                            self.selected.push(quot_origin);
                        }
                        run_quotation(quot, stack, events, cmd, self, ctx, eval_ctx)?;
                    }
                }

                Op::Quotation(quote_ops, quote_spans) => {
                    stack.push(
                        Value::Quotation(quote_ops.clone(), quote_spans.clone()),
                        span!(),
                    );
                }

                Op::When | Op::Unless => {
                    let cond = at!(stack.pop())?;
                    let (quot, quot_origin) = at!(stack.pop_with_origin())?;
                    let should_run = match &ops[pc] {
                        Op::When => cond.is_truthy(),
                        _ => !cond.is_truthy(),
                    };
                    if should_run {
                        if quot_origin != Span::default() {
                            self.selected.push(quot_origin);
                        }
                        run_quotation(quot, stack, events, cmd, self, ctx, eval_ctx)?;
                    }
                }

                Op::IfElse => {
                    let cond = at!(stack.pop())?;
                    let false_quot = at!(stack.pop())?;
                    let true_quot = at!(stack.pop())?;
                    let quot = if cond.is_truthy() {
                        true_quot
                    } else {
                        false_quot
                    };
                    run_quotation(quot, stack, events, cmd, self, ctx, eval_ctx)?;
                }

                Op::Pick => {
                    let idx_i = at!(stack.pop_int())?;
                    if idx_i < 0 {
                        return Err(CagireError::new(
                            format!("pick index must be >= 0, got {idx_i}"),
                            span!(),
                        ));
                    }
                    let idx = idx_i as usize;
                    let mut quots: Vec<Value> = Vec::new();
                    while let Ok(val) = stack.pop() {
                        if matches!(&val, Value::Quotation(..)) {
                            quots.push(val);
                        } else {
                            stack.push(val, Span::default());
                            break;
                        }
                    }
                    quots.reverse();
                    if idx >= quots.len() {
                        return Err(CagireError::new(
                            format!(
                                "pick index {} out of range (have {} quotations)",
                                idx,
                                quots.len()
                            ),
                            span!(),
                        ));
                    }
                    run_quotation(
                        quots.swap_remove(idx),
                        stack,
                        events,
                        cmd,
                        self,
                        ctx,
                        eval_ctx,
                    )?;
                }

                Op::Mtof => {
                    let note = at!(stack.pop_float())?;
                    stack.push(
                        Value::Float(440.0 * 2.0_f64.powf((note - 69.0) / 12.0)),
                        span!(),
                    );
                }
                Op::Ftom => {
                    let freq = at!(stack.pop_float())?;
                    stack.push(Value::Float(69.0 + 12.0 * (freq / 440.0).log2()), span!());
                }

                Op::Degree(pattern) => {
                    if pattern.is_empty() {
                        return Err(CagireError::new("empty scale pattern", span!()));
                    }
                    let key = self.read_key();
                    let len = pattern.len() as i64;
                    at!(stack.ensure(1))?;
                    let values = std::mem::take(&mut stack.values);
                    let origins = std::mem::take(&mut stack.origins);
                    for (val, origin) in values.into_iter().zip(origins) {
                        let result = at!(lift_unary_int(val, |degree| {
                            let octave_offset = degree.div_euclid(len);
                            let idx = degree.rem_euclid(len) as usize;
                            key + octave_offset * 12 + pattern[idx]
                        }))?;
                        stack.push(result, origin);
                    }
                }

                Op::Chord(intervals) => {
                    let root = at!(stack.pop_int())?;
                    for &interval in *intervals {
                        stack.push(Value::Int(root + interval), span!());
                    }
                }

                Op::Transpose => {
                    let n = at!(stack.pop_int())?;
                    for val in stack.values.iter_mut() {
                        if let Value::Int(v) = val {
                            *v += n;
                        }
                    }
                }

                Op::Invert => {
                    at!(stack.ensure(2))?;
                    let start = stack
                        .values
                        .iter()
                        .rposition(|v| !matches!(v, Value::Int(_)))
                        .map_or(0, |i| i + 1);
                    let bottom = at!(stack.values[start].as_int())? + 12;
                    stack.remove(start);
                    stack.push(Value::Int(bottom), span!());
                }

                Op::DownInvert => {
                    at!(stack.ensure(2))?;
                    let top = at!(stack.pop_int())? - 12;
                    let start = stack
                        .values
                        .iter()
                        .rposition(|v| !matches!(v, Value::Int(_)))
                        .map_or(0, |i| i + 1);
                    stack.insert(start, Value::Int(top), span!());
                }

                Op::VoiceDrop2 => {
                    at!(stack.ensure(3))?;
                    let len = stack.len();
                    let note = at!(stack.values[len - 2].as_int())? - 12;
                    stack.remove(len - 2);
                    let start = stack
                        .values
                        .iter()
                        .rposition(|v| !matches!(v, Value::Int(_)))
                        .map_or(0, |i| i + 1);
                    stack.insert(start, Value::Int(note), span!());
                }

                Op::VoiceDrop3 => {
                    at!(stack.ensure(4))?;
                    let len = stack.len();
                    let note = at!(stack.values[len - 3].as_int())? - 12;
                    stack.remove(len - 3);
                    let start = stack
                        .values
                        .iter()
                        .rposition(|v| !matches!(v, Value::Int(_)))
                        .map_or(0, |i| i + 1);
                    stack.insert(start, Value::Int(note), span!());
                }

                Op::SetKey => {
                    let key = at!(stack.pop_int())?;
                    self.vars.insert("__key__".to_string(), Value::Int(key));
                }

                Op::DiatonicTriad(pattern) => {
                    if pattern.is_empty() {
                        return Err(CagireError::new("empty scale pattern", span!()));
                    }
                    let degree = at!(stack.pop_int())?;
                    let key = self.read_key();
                    let len = pattern.len() as i64;
                    for offset in [0, 2, 4] {
                        let d = degree + offset;
                        let octave_offset = d.div_euclid(len);
                        let idx = d.rem_euclid(len) as usize;
                        stack.push(Value::Int(key + octave_offset * 12 + pattern[idx]), span!());
                    }
                }

                Op::DiatonicSeventh(pattern) => {
                    if pattern.is_empty() {
                        return Err(CagireError::new("empty scale pattern", span!()));
                    }
                    let degree = at!(stack.pop_int())?;
                    let key = self.read_key();
                    let len = pattern.len() as i64;
                    for offset in [0, 2, 4, 6] {
                        let d = degree + offset;
                        let octave_offset = d.div_euclid(len);
                        let idx = d.rem_euclid(len) as usize;
                        stack.push(Value::Int(key + octave_offset * 12 + pattern[idx]), span!());
                    }
                }

                Op::Oct => {
                    let shift = at!(stack.pop())?;
                    let note = at!(stack.pop())?;
                    stack.push(at!(lift_binary(note, shift, |n, s| n + s * 12.0))?, span!());
                }

                Op::SetTempo => {
                    let tempo = at!(stack.pop_float())?;
                    let clamped = tempo.clamp(20.0, 300.0);
                    self.vars
                        .insert("__tempo__".to_string(), Value::Float(clamped));
                }

                Op::SetSpeed => {
                    let speed = at!(stack.pop_float())?;
                    let clamped = speed.clamp(0.125, 8.0);
                    self.vars
                        .insert("__speed__".to_string(), Value::Float(clamped));
                }

                Op::Loop => {
                    let steps = at!(stack.pop_float())?;
                    let dur = steps * ctx.step_duration;
                    cmd.set_param("fit", Value::Float(dur));
                    cmd.set_param("gate", Value::Float(dur));
                }

                Op::LinMap => {
                    let out_hi = at!(stack.pop_float())?;
                    let out_lo = at!(stack.pop_float())?;
                    let in_hi = at!(stack.pop_float())?;
                    let in_lo = at!(stack.pop_float())?;
                    let val = at!(stack.pop_float())?;
                    let t = if (in_hi - in_lo).abs() < f64::EPSILON {
                        0.0
                    } else {
                        (val - in_lo) / (in_hi - in_lo)
                    };
                    stack.push(Value::Float(out_lo + t * (out_hi - out_lo)), span!());
                }

                Op::ExpMap => {
                    let hi = at!(stack.pop_float())?;
                    let lo = at!(stack.pop_float())?;
                    let val = at!(stack.pop_float())?;
                    if lo <= 0.0 || hi <= 0.0 {
                        return Err(CagireError::new("expmap requires positive bounds", span!()));
                    }
                    stack.push(Value::Float(lo * (hi / lo).powf(val)), span!());
                }

                Op::Map => {
                    let quot = at!(stack.pop())?;
                    let items = std::mem::take(&mut stack.values);
                    let item_origins = std::mem::take(&mut stack.origins);
                    for (item, origin) in items.into_iter().zip(item_origins) {
                        stack.push(item, origin);
                        run_quotation(quot.clone(), stack, events, cmd, self, ctx, eval_ctx)?;
                    }
                }

                Op::At => {
                    at!(stack.ensure(1))?;
                    let deltas = std::mem::take(&mut stack.values);
                    stack.origins.clear();
                    cmd.set_deltas(deltas);
                }

                Op::AtLoop(body_ops, body_spans) => {
                    at!(stack.ensure(1))?;
                    let values = std::mem::take(&mut stack.values);
                    let origins = std::mem::take(&mut stack.origins);

                    // Single string → pattern mode (with gates + alternation)
                    if values.len() == 1 {
                        if let Value::Str(ref s) = values[0] {
                            let hits = at!(pattern::parse_pattern_annotated(s))?;
                            let origin = origins.first().copied().unwrap_or_default();
                            let n = hits.len();
                            for (i, hit) in hits.iter().enumerate() {
                                if let Some(alt) = &hit.alt {
                                    if ctx.runs % alt.count as usize != alt.index as usize {
                                        continue;
                                    }
                                }
                                let delta_secs = ctx.nudge_secs + hit.position * ctx.step_duration;

                                let iter_ctx = StepContext {
                                    step: ctx.step,
                                    beat: ctx.beat,
                                    tempo: ctx.tempo,
                                    phase: ctx.phase,
                                    slot: ctx.slot,
                                    runs: ctx.runs * n + i,
                                    iter: ctx.iter,
                                    speed: ctx.speed,
                                    step_duration: ctx.step_duration,
                                    frame_index: ctx.frame_index,
                                    nudge_secs: ctx.nudge_secs,
                                    default_device: ctx.default_device,
                                };

                                cmd.set_delta_secs(delta_secs);
                                cmd.set_param("gate", Value::Float(hit.gate * ctx.step_duration));
                                let highlight_span =
                                    self.pattern_hit_span(origin, hit.start, hit.end);
                                if let Some(span) = highlight_span {
                                    self.active_emit_annotations.push(span);
                                }
                                self.execute_ops(
                                    body_ops, body_spans, &iter_ctx, eval_ctx, stack, events, cmd,
                                )?;
                                if highlight_span.is_some() {
                                    self.active_emit_annotations.pop();
                                }
                                cmd.clear_params();
                                cmd.clear_sound();
                            }

                            pc += 1;
                            continue;
                        }
                    }

                    // Float mode (existing behavior)
                    let n = values.len();
                    for (i, delta_val) in values.iter().enumerate() {
                        let frac = at!(delta_val.as_float())?;
                        let delta_secs = ctx.nudge_secs + frac * ctx.step_duration;

                        let iter_ctx = StepContext {
                            step: ctx.step,
                            beat: ctx.beat,
                            tempo: ctx.tempo,
                            phase: ctx.phase,
                            slot: ctx.slot,
                            runs: ctx.runs * n + i,
                            iter: ctx.iter,
                            speed: ctx.speed,
                            step_duration: ctx.step_duration,
                            frame_index: ctx.frame_index,
                            nudge_secs: ctx.nudge_secs,
                            default_device: ctx.default_device,
                        };

                        cmd.set_delta_secs(delta_secs);
                        self.execute_ops(
                            body_ops, body_spans, &iter_ctx, eval_ctx, stack, events, cmd,
                        )?;
                        cmd.clear_params();
                        cmd.clear_sound();
                    }
                }

                Op::PatPush => {
                    let val = at!(stack.pop())?;
                    let s = at!(val.as_str().map(str::to_string))?;
                    let hits = at!(pattern::parse_pattern(&s))?;
                    let origin = span!();
                    for hit in hits.iter() {
                        if let Some(alt) = &hit.alt {
                            if ctx.runs % alt.count as usize != alt.index as usize {
                                continue;
                            }
                        }
                        stack.push(Value::Float(hit.position), origin);
                    }
                }

                Op::PatRot => {
                    let n = at!(stack.pop_int())?;
                    let val = at!(stack.pop())?;
                    let s = at!(val.as_str().map(str::to_string))?;
                    let result = at!(pattern::rotate_pattern(&s, n))?;
                    stack.push(Value::Str(Arc::from(result.as_str())), span!());
                }

                Op::PatRev => {
                    let val = at!(stack.pop())?;
                    let s = at!(val.as_str().map(str::to_string))?;
                    let result = at!(pattern::reverse_pattern(&s))?;
                    stack.push(Value::Str(Arc::from(result.as_str())), span!());
                }

                Op::PatInv => {
                    let val = at!(stack.pop())?;
                    let s = at!(val.as_str().map(str::to_string))?;
                    let result = at!(pattern::invert_pattern(&s))?;
                    stack.push(Value::Str(Arc::from(result.as_str())), span!());
                }

                Op::Adsr => {
                    let r = at!(stack.pop())?;
                    let s = at!(stack.pop())?;
                    let d = at!(stack.pop())?;
                    let a = at!(stack.pop())?;
                    cmd.set_param("attack", a);
                    cmd.set_param("decay", d);
                    cmd.set_param("sustain", s);
                    cmd.set_param("release", r);
                }

                Op::Ad => {
                    let d = at!(stack.pop())?;
                    let a = at!(stack.pop())?;
                    cmd.set_param("attack", a);
                    cmd.set_param("decay", d);
                    cmd.set_param("sustain", Value::Int(0));
                }

                Op::Apply => {
                    let quot = at!(stack.pop())?;
                    run_quotation(quot, stack, events, cmd, self, ctx, eval_ctx)?;
                }

                Op::Ramp => {
                    let curve = at!(stack.pop_float())?;
                    let freq = at!(stack.pop_float())?;
                    let phase = (freq * ctx.beat).fract();
                    let phase = if phase < 0.0 { phase + 1.0 } else { phase };
                    stack.push(Value::Float(phase.powf(curve)), span!());
                }
                Op::Triangle => {
                    let freq = at!(stack.pop_float())?;
                    let phase = (freq * ctx.beat).fract();
                    let phase = if phase < 0.0 { phase + 1.0 } else { phase };
                    stack.push(Value::Float(1.0 - (2.0 * phase - 1.0).abs()), span!());
                }
                Op::Range => {
                    let max = at!(stack.pop_float())?;
                    let min = at!(stack.pop_float())?;
                    let val = at!(stack.pop_float())?;
                    stack.push(Value::Float(min + val * (max - min)), span!());
                }
                Op::Perlin => {
                    let freq = at!(stack.pop_float())?;
                    stack.push(Value::Float(perlin_noise_1d(freq * ctx.beat)), span!());
                }

                Op::ClearCmd => {
                    cmd.clear();
                }

                Op::IntRange => {
                    let end = at!(stack.pop_int())?;
                    let start = at!(stack.pop_int())?;
                    let count = (end - start).unsigned_abs() + 1;
                    if count > 10_000 {
                        return Err(CagireError::new("range too large (max 10000)", span!()));
                    }
                    let sp = span!();
                    if start <= end {
                        for i in start..=end {
                            stack.push(Value::Int(i), sp);
                        }
                    } else {
                        for i in (end..=start).rev() {
                            stack.push(Value::Int(i), sp);
                        }
                    }
                }

                Op::StepRange => {
                    let step = at!(stack.pop_float())?;
                    let end = at!(stack.pop_float())?;
                    let start = at!(stack.pop_float())?;
                    if step == 0.0 {
                        return Err(CagireError::new("step cannot be zero", span!()));
                    }
                    let ascending = step > 0.0;
                    let mut val = start;
                    let mut count = 0u32;
                    let sp = span!();
                    loop {
                        if (ascending && val > end) || (!ascending && val < end) {
                            break;
                        }
                        count += 1;
                        if count > 10_000 {
                            return Err(CagireError::new("range too large (max 10000)", span!()));
                        }
                        stack.push(float_to_value(val), sp);
                        val += step;
                    }
                }

                Op::Generate => {
                    let count = at!(stack.pop_int())?;
                    let quot = at!(stack.pop())?;
                    if count < 0 {
                        return Err(CagireError::new("gen count must be >= 0", span!()));
                    }
                    let mut results = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        run_quotation(quot.clone(), stack, events, cmd, self, ctx, eval_ctx)?;
                        results.push(at!(stack.pop())?);
                    }
                    let sp = span!();
                    for val in results {
                        stack.push(val, sp);
                    }
                }

                Op::Times => {
                    let quot = at!(stack.pop())?;
                    let count = at!(stack.pop_int())?;
                    if count < 0 {
                        return Err(CagireError::new("times count must be >= 0", span!()));
                    }
                    for i in 0..count {
                        self.vars.insert("i".to_string(), Value::Int(i));
                        run_quotation(quot.clone(), stack, events, cmd, self, ctx, eval_ctx)?;
                    }
                }

                Op::GeomRange => {
                    let count = at!(stack.pop_int())?;
                    let ratio = at!(stack.pop_float())?;
                    let start = at!(stack.pop_float())?;
                    if count < 0 {
                        return Err(CagireError::new("geom.. count must be >= 0", span!()));
                    }
                    let sp = span!();
                    let mut val = start;
                    for _ in 0..count {
                        stack.push(float_to_value(val), sp);
                        val *= ratio;
                    }
                }

                Op::Euclid => {
                    let n = at!(stack.pop_int())?;
                    let k = at!(stack.pop_int())?;
                    if k < 0 || n < 0 {
                        return Err(CagireError::new("euclid: k and n must be >= 0", span!()));
                    }
                    let sp = span!();
                    for val in euclidean_rhythm(k as usize, n as usize, 0) {
                        stack.push(Value::Float(val), sp);
                    }
                }

                Op::EuclidRot => {
                    let r = at!(stack.pop_int())?;
                    let n = at!(stack.pop_int())?;
                    let k = at!(stack.pop_int())?;
                    if k < 0 || n < 0 || r < 0 {
                        return Err(CagireError::new(
                            "euclidrot: k, n, and r must be >= 0",
                            span!(),
                        ));
                    }
                    let sp = span!();
                    for val in euclidean_rhythm(k as usize, n as usize, r as usize) {
                        stack.push(Value::Float(val), sp);
                    }
                }

                Op::Subdivide => {
                    let n = at!(stack.pop_int())?;
                    if n < 1 {
                        return Err(CagireError::new("div: n must be >= 1", span!()));
                    }
                    if n > 10_000 {
                        return Err(CagireError::new("div: n too large (max 10000)", span!()));
                    }
                    let nf = n as f64;
                    let sp = span!();
                    for i in 0..n {
                        stack.push(Value::Float(i as f64 / nf), sp);
                    }
                }

                Op::Swing => {
                    let ratio = at!(stack.pop_float())?;
                    let n = at!(stack.pop_int())?;
                    if n < 1 {
                        return Err(CagireError::new("swing: n must be >= 1", span!()));
                    }
                    if n > 10_000 {
                        return Err(CagireError::new("swing: n too large (max 10000)", span!()));
                    }
                    let pair_len = 2.0 / n as f64;
                    let sp = span!();
                    for i in 0..n {
                        let pair_start = (i / 2) as f64 * pair_len;
                        let frac = if i % 2 == 0 {
                            pair_start
                        } else {
                            (pair_start + ratio * pair_len).min(1.0 - f64::EPSILON)
                        };
                        stack.push(Value::Float(frac), sp);
                    }
                }

                Op::ModLfo(shape) => {
                    let period = at!(stack.pop_float())? * ctx.step_duration;
                    let max = at!(stack.pop_float())?;
                    let min = at!(stack.pop_float())?;
                    let suffix = match shape {
                        1 => "t",
                        2 => "w",
                        3 => "q",
                        _ => "",
                    };
                    stack.push(
                        Value::Str(format!("{min}~{max}:{period}{suffix}").into()),
                        span!(),
                    );
                }
                Op::ModSlide(curve) => {
                    let dur = at!(stack.pop_float())? * ctx.step_duration;
                    let end = at!(stack.pop_float())?;
                    let start = at!(stack.pop_float())?;
                    let suffix = match curve {
                        1 => "e",
                        2 => "s",
                        3 => "i",
                        4 => "o",
                        5 => "p",
                        _ => "",
                    };
                    stack.push(
                        Value::Str(format!("{start}>{end}:{dur}{suffix}").into()),
                        span!(),
                    );
                }
                Op::ModRnd(dist) => {
                    let period = at!(stack.pop_float())? * ctx.step_duration;
                    let max = at!(stack.pop_float())?;
                    let min = at!(stack.pop_float())?;
                    let suffix = match dist {
                        1 => "s",
                        2 => "d",
                        _ => "",
                    };
                    stack.push(
                        Value::Str(format!("{min}?{max}:{period}{suffix}").into()),
                        span!(),
                    );
                }
                Op::ModEnv => {
                    let release = at!(stack.pop_float())? * ctx.step_duration;
                    let sustain = at!(stack.pop_float())?;
                    let decay = at!(stack.pop_float())? * ctx.step_duration;
                    let attack = at!(stack.pop_float())? * ctx.step_duration;
                    let max = at!(stack.pop_float())?;
                    let min = at!(stack.pop_float())?;
                    use std::fmt::Write;
                    let mut s = String::new();
                    let _ = write!(&mut s, "{min}^{max}:{attack}:{decay}:{sustain}:{release}");
                    stack.push(Value::Str(s.into()), span!());
                }
                Op::ModEnvAd => {
                    let decay = at!(stack.pop_float())? * ctx.step_duration;
                    let attack = at!(stack.pop_float())? * ctx.step_duration;
                    let max = at!(stack.pop_float())?;
                    let min = at!(stack.pop_float())?;
                    use std::fmt::Write;
                    let mut s = String::new();
                    let _ = write!(&mut s, "{min}^{max}:{attack}:{decay}:0:0");
                    stack.push(Value::Str(s.into()), span!());
                }
                Op::ModEnvAdr => {
                    let release = at!(stack.pop_float())? * ctx.step_duration;
                    let decay = at!(stack.pop_float())? * ctx.step_duration;
                    let attack = at!(stack.pop_float())? * ctx.step_duration;
                    let max = at!(stack.pop_float())?;
                    let min = at!(stack.pop_float())?;
                    use std::fmt::Write;
                    let mut s = String::new();
                    let _ = write!(&mut s, "{min}^{max}:{attack}:{decay}:0:{release}");
                    stack.push(Value::Str(s.into()), span!());
                }
                Op::Lpg => {
                    let depth = at!(stack.pop_float())?.clamp(0.0, 1.0);
                    let max = at!(stack.pop_float())?;
                    let min = at!(stack.pop_float())?;
                    let effective_max = min + (max - min) * depth;
                    let sd = ctx.step_duration;
                    let a = cmd.get_param_float("attack").unwrap_or(0.0) * sd;
                    let d = cmd.get_param_float("decay").unwrap_or(1.0) * sd;
                    let s = cmd.get_param_float("sustain").unwrap_or(0.0);
                    let r = cmd.get_param_float("release").unwrap_or(0.0) * sd;
                    use std::fmt::Write;
                    let mut mod_str = String::new();
                    let _ = write!(&mut mod_str, "{min}^{effective_max}:{a}:{d}:{s}:{r}");
                    cmd.set_param("lpf", Value::Str(mod_str.into()));
                }

                Op::GetMidiCC => {
                    let chan = at!(stack.pop_int())?;
                    let cc = at!(stack.pop_int())?;
                    let device_id = cmd
                        .params()
                        .iter()
                        .find(|(k, _)| *k == "device")
                        .and_then(|(_, v)| v.as_int().ok())
                        .map(|d| d.max(0) as usize)
                        .unwrap_or(ctx.default_device);
                    let cc_value = eval_ctx
                        .device_map
                        .get_input_cc(device_id, cc as i8, chan as i8)
                        .unwrap_or_default();
                    stack.push(Value::Int(cc_value), span!());
                }

                Op::MidiClock => {
                    let dev = get_cmd_dev(cmd, ctx);
                    self.push_event(
                        events,
                        ConcreteEvent::MidiClock(dev),
                        offset_micros(ctx, 0.0),
                    );
                }
                Op::MidiStart => {
                    let dev = get_cmd_dev(cmd, ctx);
                    self.push_event(
                        events,
                        ConcreteEvent::MidiStart(dev),
                        offset_micros(ctx, 0.0),
                    );
                }
                Op::MidiStop => {
                    let dev = get_cmd_dev(cmd, ctx);
                    self.push_event(
                        events,
                        ConcreteEvent::MidiStop(dev),
                        offset_micros(ctx, 0.0),
                    );
                }
                Op::MidiContinue => {
                    let dev = get_cmd_dev(cmd, ctx);
                    self.push_event(
                        events,
                        ConcreteEvent::MidiContinue(dev),
                        offset_micros(ctx, 0.0),
                    );
                }

                Op::Mark => {
                    marks.push(stack.len());
                }

                Op::Count(_) => {
                    let mark = at!(marks.pop().ok_or_else(|| "count without mark".to_string()))?;
                    let val = (stack.len() - mark) as i64;
                    stack.push(Value::Int(val), span!());
                }

                Op::EmitAll => {
                    if !cmd.params().is_empty() {
                        for (event, _) in events.iter_mut() {
                            if let ConcreteEvent::Generic(VariableValue::Map(args), ..) = event {
                                for (k, v) in cmd.params() {
                                    if *k == "device" {
                                        continue;
                                    }
                                    let param_str = v.to_param_string();
                                    if let Ok(f) = param_str.parse::<f64>() {
                                        if is_tempo_scaled_param(k) {
                                            args.insert(
                                                k.to_string(),
                                                VariableValue::Float(f * ctx.step_duration),
                                            );
                                        } else {
                                            args.insert(k.to_string(), VariableValue::Float(f));
                                        }
                                    } else {
                                        args.insert(k.to_string(), VariableValue::Str(param_str));
                                    }
                                }
                            }
                        }
                    }
                    cmd.commit_global();
                }

                Op::ClearGlobal => {
                    cmd.clear_global();
                }

                Op::Rec => {
                    let name = at!(stack.pop())?;
                    let path = format!("/doux/rec/{}", at!(name.as_str())?);
                    let message = OSCMessage::new(path, vec![]);
                    self.push_event(
                        events,
                        ConcreteEvent::Osc {
                            message,
                            device_id: 2,
                        },
                        offset_micros(ctx, 0.0),
                    );
                }

                Op::Overdub => {
                    let name = at!(stack.pop())?;
                    let path = format!("/doux/rec/{}/overdub/1", at!(name.as_str())?);
                    let message = OSCMessage::new(path, vec![]);
                    self.push_event(
                        events,
                        ConcreteEvent::Osc {
                            message,
                            device_id: 2,
                        },
                        offset_micros(ctx, 0.0),
                    );
                }

                Op::Orec => {
                    let orbit = at!(stack.pop_int())?;
                    let name = at!(stack.pop())?;
                    let path = format!("/doux/rec/{}/orbit/{}", at!(name.as_str())?, orbit);
                    let message = OSCMessage::new(path, vec![]);
                    self.push_event(
                        events,
                        ConcreteEvent::Osc {
                            message,
                            device_id: 2,
                        },
                        offset_micros(ctx, 0.0),
                    );
                }

                Op::Odub => {
                    let orbit = at!(stack.pop_int())?;
                    let name = at!(stack.pop())?;
                    let path = format!(
                        "/doux/rec/{}/overdub/1/orbit/{}",
                        at!(name.as_str())?,
                        orbit
                    );
                    let message = OSCMessage::new(path, vec![]);
                    self.push_event(
                        events,
                        ConcreteEvent::Osc {
                            message,
                            device_id: 2,
                        },
                        offset_micros(ctx, 0.0),
                    );
                }

                Op::Forget => {
                    let name = at!(stack.pop())?;
                    self.dict.remove(at!(name.as_str())?);
                }

                Op::Print => {
                    let val = at!(stack.pop())?;
                    self.push_event(events, ConcreteEvent::Print(val.to_param_string()), 0);
                }
            }
            pc += 1;
        }

        Ok(())
    }

    fn read_key(&self) -> i64 {
        self.vars
            .get("__key__")
            .and_then(|v| v.as_int().ok())
            .unwrap_or(60)
    }

    fn get_var(&self, name: &str, eval_ctx: &mut EvaluationContext) -> Value {
        let (scope, key) = parse_var_scope(name);
        match scope {
            VarScope::Instance => {
                if let Some(v) = self.vars.get(key) {
                    return v.clone();
                }
                let vv = eval_ctx.evaluate(&Variable::Instance(key.to_string()));
                Value::from_variable_value(&vv)
            }
            VarScope::Global => {
                let vv = eval_ctx.evaluate(&Variable::Global(key.to_string()));
                Value::from_variable_value(&vv)
            }
            VarScope::Line => {
                let vv = eval_ctx.evaluate(&Variable::Line(key.to_string()));
                Value::from_variable_value(&vv)
            }
            VarScope::Frame => {
                let vv = eval_ctx.evaluate(&Variable::Frame(key.to_string()));
                Value::from_variable_value(&vv)
            }
        }
    }

    fn set_var(&mut self, name: &str, val: Value, eval_ctx: &mut EvaluationContext) {
        let (scope, key) = parse_var_scope(name);
        match scope {
            VarScope::Instance => {
                if let Some(vv) = val.to_variable_value() {
                    self.vars.remove(key);
                    eval_ctx.redefine(&Variable::Instance(key.to_string()), vv);
                } else {
                    self.vars.insert(key.to_string(), val);
                }
            }
            VarScope::Global => {
                if let Some(vv) = val.to_variable_value() {
                    eval_ctx.redefine(&Variable::Global(key.to_string()), vv);
                }
            }
            VarScope::Line => {
                if let Some(vv) = val.to_variable_value() {
                    eval_ctx.redefine(&Variable::Line(key.to_string()), vv);
                }
            }
            VarScope::Frame => {
                if let Some(vv) = val.to_variable_value() {
                    eval_ctx.redefine(&Variable::Frame(key.to_string()), vv);
                }
            }
        }
    }

    fn emit_events(
        &mut self,
        cmd: &mut CmdRegister,
        ctx: &StepContext,
        events: &mut Vec<(ConcreteEvent, SyncTime)>,
    ) -> Result<(), String> {
        if let Some(dsecs) = cmd.take_delta_secs() {
            // AtLoop path: single delta, vary poly_idx
            let poly_count = compute_poly_count(cmd);
            for poly_idx in 0..poly_count {
                self.emit_single(cmd, ctx, events, poly_idx, dsecs)?;
            }
        } else {
            // Normal path: iterate deltas x poly
            let poly_count = compute_poly_count(cmd);
            let deltas: Vec<f64> = if cmd.deltas().is_empty() {
                vec![0.0]
            } else {
                cmd.deltas()
                    .iter()
                    .filter_map(|v| v.as_float().ok())
                    .collect()
            };

            for poly_idx in 0..poly_count {
                for &delta_frac in &deltas {
                    let delta_secs = ctx.nudge_secs + delta_frac * ctx.step_duration;
                    self.emit_single(cmd, ctx, events, poly_idx, delta_secs)?;
                }
            }
        }

        Ok(())
    }

    fn emit_single(
        &mut self,
        cmd: &CmdRegister,
        ctx: &StepContext,
        events: &mut Vec<(ConcreteEvent, SyncTime)>,
        poly_idx: usize,
        delta_secs: f64,
    ) -> Result<(), String> {
        let time = offset_micros(ctx, delta_secs);

        let (sound_opt, params) = match cmd.snapshot() {
            Some(s) => s,
            None => return Ok(()),
        };

        let resolved_sound = sound_opt.map(|sv| resolve_cycling(sv, poly_idx));

        let has_sound = resolved_sound.as_ref().is_some_and(|v| match v.as_ref() {
            Value::Str(s) => !s.is_empty(),
            Value::Int(_) | Value::Float(_) => true,
            _ => false,
        });

        let find_param = |name: &str| -> Option<&Value> {
            params
                .iter()
                .rev()
                .find(|(k, _)| *k == name)
                .or_else(|| cmd.global_params().iter().rev().find(|(k, _)| *k == name))
                .map(|(_, v)| v)
        };
        let get_int = |name: &str| -> Option<i64> {
            find_param(name).and_then(|v| {
                resolve_cycling(v, poly_idx)
                    .as_float()
                    .ok()
                    .map(|f| f as i64)
            })
        };
        let get_float = |name: &str| -> Option<f64> {
            find_param(name).and_then(|v| resolve_cycling(v, poly_idx).as_float().ok())
        };

        let dev = get_int("device")
            .unwrap_or(ctx.default_device as i64)
            .max(0) as usize;

        if has_sound {
            let sound_str = match &resolved_sound {
                Some(v) => match v.as_ref() {
                    Value::Str(s) => s.to_string(),
                    other => other.to_param_string(),
                },
                None => String::new(),
            };

            if sound_str.starts_with('/') {
                let mut osc_args = Vec::with_capacity(params.len() * 2);
                for (k, v) in cmd.global_params().iter().chain(params.iter()) {
                    if *k == "device" {
                        continue;
                    }
                    let resolved = resolve_cycling(v, poly_idx);
                    osc_args.push(VariableValue::Str(k.to_string()));
                    let param_str = resolved.to_param_string();
                    if let Ok(f) = param_str.parse::<f64>() {
                        osc_args.push(VariableValue::Float(f));
                    } else {
                        osc_args.push(VariableValue::Str(param_str));
                    }
                }
                let message = OSCMessage::new(sound_str, osc_args);
                self.push_event(
                    events,
                    ConcreteEvent::Osc {
                        message,
                        device_id: dev,
                    },
                    time,
                );
            } else {
                let mut args = HashMap::with_capacity(params.len() + 3);
                args.insert("sound".to_string(), VariableValue::Str(sound_str));

                for (k, v) in cmd.global_params().iter().chain(params.iter()) {
                    if *k == "device" {
                        continue;
                    }
                    let resolved = resolve_cycling(v, poly_idx);
                    let param_str = resolved.to_param_string();
                    if let Ok(f) = param_str.parse::<f64>() {
                        if is_tempo_scaled_param(k) {
                            args.insert(k.to_string(), VariableValue::Float(f * ctx.step_duration));
                        } else {
                            args.insert(k.to_string(), VariableValue::Float(f));
                        }
                    } else {
                        args.insert(k.to_string(), VariableValue::Str(param_str));
                    }
                }

                let has_gate = args.contains_key("gate");
                if !has_gate {
                    args.insert("gate".to_string(), VariableValue::Float(ctx.step_duration));
                }
                if !args.contains_key("release") && !has_gate {
                    args.insert(
                        "release".to_string(),
                        VariableValue::Float(ctx.step_duration),
                    );
                }
                if !args.contains_key("delaytime") {
                    args.insert(
                        "delaytime".to_string(),
                        VariableValue::Float(ctx.step_duration),
                    );
                }

                let gate_secs = match args.get("gate") {
                    Some(VariableValue::Float(f)) => *f,
                    Some(VariableValue::Integer(i)) => *i as f64,
                    _ => ctx.step_duration,
                };
                let dur_micros = (gate_secs * 1_000_000.0) as SyncTime;
                self.push_event(
                    events,
                    ConcreteEvent::Generic(args.into(), dur_micros, String::new(), dev),
                    time,
                );
            }
        } else {
            let chan = get_int("chan").unwrap_or(1).clamp(1, 16) as u64;

            if let (Some(cc), Some(val)) = (get_int("ccnum"), get_int("ccout")) {
                self.push_event(
                    events,
                    ConcreteEvent::MidiControl(
                        cc.clamp(0, 127) as u64,
                        val.clamp(0, 127) as u64,
                        chan,
                        dev,
                    ),
                    time,
                );
            } else if let Some(bend) = get_float("bend") {
                let bend_clamped = bend.clamp(-1.0, 1.0);
                let bend_14bit = ((bend_clamped + 1.0) * 8191.5) as u16;
                self.push_event(
                    events,
                    ConcreteEvent::MidiPitchBend(bend_14bit, chan, dev),
                    time,
                );
            } else if let Some(pressure) = get_int("pressure") {
                self.push_event(
                    events,
                    ConcreteEvent::MidiChannelPressure(pressure.clamp(0, 127) as u64, chan, dev),
                    time,
                );
            } else if let Some(program) = get_int("program") {
                self.push_event(
                    events,
                    ConcreteEvent::MidiProgram(program.clamp(0, 127) as u64, chan, dev),
                    time,
                );
            } else {
                let note = get_int("note").unwrap_or(60).clamp(0, 127) as u64;
                let velocity = get_int("velocity")
                    .or_else(|| get_int("vel"))
                    .unwrap_or(100)
                    .clamp(0, 127) as u64;
                let dur_frac = get_float("gate").unwrap_or(1.0);
                let dur_micros = (dur_frac * ctx.step_duration * 1_000_000.0) as SyncTime;
                self.push_event(
                    events,
                    ConcreteEvent::MidiNote(note, velocity, chan, dur_micros, dev),
                    time,
                );
            }
        }

        Ok(())
    }
}

enum VarScope {
    Instance,
    Global,
    Line,
    Frame,
}

fn parse_var_scope(name: &str) -> (VarScope, &str) {
    if let Some(key) = name.strip_prefix("G.") {
        (VarScope::Global, key)
    } else if let Some(key) = name.strip_prefix("L.") {
        (VarScope::Line, key)
    } else if let Some(key) = name.strip_prefix("F.") {
        (VarScope::Frame, key)
    } else {
        (VarScope::Instance, name)
    }
}

fn get_cmd_dev(cmd: &CmdRegister, ctx: &StepContext) -> usize {
    cmd.params()
        .iter()
        .find(|(k, _)| *k == "device")
        .and_then(|(_, v)| v.as_int().ok())
        .map(|d| d.max(0) as usize)
        .unwrap_or(ctx.default_device)
}

fn offset_micros(_ctx: &StepContext, delta_secs: f64) -> SyncTime {
    if delta_secs > 0.0 {
        (delta_secs * 1_000_000.0) as SyncTime
    } else {
        0
    }
}

fn is_tempo_scaled_param(name: &str) -> bool {
    matches!(
        name,
        "attack"
            | "decay"
            | "release"
            | "lpa"
            | "lpd"
            | "lpr"
            | "hpa"
            | "hpd"
            | "hpr"
            | "bpa"
            | "bpd"
            | "bpr"
            | "patt"
            | "pdec"
            | "prel"
            | "fma"
            | "fmd"
            | "fmr"
            | "glide"
            | "chorusdelay"
            | "duration"
    )
}

fn compute_poly_count(cmd: &CmdRegister) -> usize {
    let sound_len = match cmd.sound() {
        Some(Value::CycleList(items)) => items.len(),
        _ => 1,
    };
    let param_max = cmd
        .global_params()
        .iter()
        .chain(cmd.params().iter())
        .map(|(_, v)| match v {
            Value::CycleList(items) => items.len(),
            _ => 1,
        })
        .max()
        .unwrap_or(1);
    sound_len.max(param_max)
}

fn resolve_cycling(val: &Value, emit_idx: usize) -> Cow<'_, Value> {
    match val {
        Value::CycleList(items) if !items.is_empty() => {
            Cow::Owned(items[emit_idx % items.len()].clone())
        }
        other => Cow::Borrowed(other),
    }
}

fn run_quotation(
    quot: Value,
    stack: &mut Stack,
    events: &mut Vec<(ConcreteEvent, SyncTime)>,
    cmd: &mut CmdRegister,
    vm: &mut CagireVM,
    ctx: &StepContext,
    eval_ctx: &mut EvaluationContext,
) -> Result<(), CagireError> {
    match quot {
        Value::Quotation(quot_ops, quot_spans) => {
            vm.execute_ops(&quot_ops, &quot_spans, ctx, eval_ctx, stack, events, cmd)
        }
        _ => Err(CagireError::new("expected quotation", Span::default())),
    }
}

fn select_and_run(
    selected: Value,
    stack: &mut Stack,
    events: &mut Vec<(ConcreteEvent, SyncTime)>,
    cmd: &mut CmdRegister,
    vm: &mut CagireVM,
    ctx: &StepContext,
    eval_ctx: &mut EvaluationContext,
) -> Result<(), CagireError> {
    if matches!(selected, Value::Quotation(..)) {
        run_quotation(selected, stack, events, cmd, vm, ctx, eval_ctx)
    } else {
        stack.push(selected, Span::default());
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn drain_select_run(
    count: usize,
    idx: usize,
    stack: &mut Stack,
    events: &mut Vec<(ConcreteEvent, SyncTime)>,
    cmd: &mut CmdRegister,
    vm: &mut CagireVM,
    op_spans: &[Span],
    pc: usize,
    ctx: &StepContext,
    eval_ctx: &mut EvaluationContext,
) -> Result<(), CagireError> {
    stack
        .ensure(count)
        .map_err(|msg| CagireError::new(msg, op_spans.get(pc).copied().unwrap_or_default()))?;
    let start = stack.len() - count;
    let selected = stack.values[start + idx].clone();
    let origin = stack.origin(start + idx);
    if origin != Span::default() {
        vm.selected.push(origin);
    }
    stack.truncate(start);
    select_and_run(selected, stack, events, cmd, vm, ctx, eval_ctx)
}

fn drain_skip_quotations(stack: &mut Stack) -> Vec<Value> {
    let values = std::mem::take(&mut stack.values);
    let origins = std::mem::take(&mut stack.origins);
    let mut result = Vec::new();
    for (v, o) in values.into_iter().zip(origins) {
        if matches!(v, Value::Quotation(..)) {
            stack.push(v, o);
        } else {
            result.push(v);
        }
    }
    result
}

fn lift_unary<F>(val: Value, f: F) -> Result<Value, String>
where
    F: Fn(f64) -> f64,
{
    Ok(float_to_value(f(val.as_float()?)))
}

fn lift_unary_int<F>(val: Value, f: F) -> Result<Value, String>
where
    F: Fn(i64) -> i64,
{
    Ok(Value::Int(f(val.as_int()?)))
}

fn lift_binary<F>(a: Value, b: Value, f: F) -> Result<Value, String>
where
    F: Fn(f64, f64) -> f64,
{
    Ok(float_to_value(f(a.as_float()?, b.as_float()?)))
}

fn sort_paired(values: &mut [Value], origins: &mut [Span], reverse: bool) {
    let mut indices: Vec<usize> = (0..values.len()).collect();
    indices.sort_by(|&a, &b| {
        let af = values[a].as_float().unwrap_or(0.0);
        let bf = values[b].as_float().unwrap_or(0.0);
        if reverse {
            bf.partial_cmp(&af).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
    let sorted_vals: Vec<Value> = indices.iter().map(|&i| values[i].clone()).collect();
    let sorted_origins: Vec<Span> = indices.iter().map(|&i| origins[i]).collect();
    values.clone_from_slice(&sorted_vals);
    origins.copy_from_slice(&sorted_origins);
}

fn euclidean_hit(k: usize, n: usize, pos: usize) -> bool {
    if k == 0 {
        return false;
    }
    ((pos + 1) * k) / n != (pos * k) / n
}

fn euclidean_rhythm(k: usize, n: usize, rotation: usize) -> Vec<f64> {
    if k == 0 || n == 0 {
        return Vec::new();
    }
    let n_f = n as f64;
    if k >= n {
        let mut r: Vec<f64> = (0..n).map(|i| ((i + rotation) % n) as f64 / n_f).collect();
        r.sort_by(|a, b| a.partial_cmp(b).unwrap());
        return r;
    }
    let mut result = Vec::with_capacity(k);
    let mut prev: i64 = -1;
    for i in 0..n {
        let bucket = (i * k / n) as i64;
        if bucket != prev {
            let pos = (i + rotation) % n;
            result.push(pos as f64 / n_f);
        }
        prev = bucket;
    }
    result.sort_by(|a, b| a.partial_cmp(b).unwrap());
    result
}

fn perlin_grad(hash_input: i64) -> f64 {
    let mut h = (hash_input as u64)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    (h as i64 as f64) / (i64::MAX as f64)
}

fn perlin_noise_1d(x: f64) -> f64 {
    let x0 = x.floor() as i64;
    let t = x - x0 as f64;
    let s = t * t * (3.0 - 2.0 * t);
    let d0 = perlin_grad(x0) * t;
    let d1 = perlin_grad(x0 + 1) * (t - 1.0);
    (d0 + s * (d1 - d0)) * 0.5 + 0.5
}

#[cfg(test)]
mod tests {
    use rusty_link::SessionState;
    use sova_core::clock::{Clock, ClockServer};
    use sova_core::device_map::DeviceMap;
    use sova_core::error::ErrorQueue;
    use sova_core::vm::variable::VariableStore;
    use std::collections::VecDeque;
    use std::sync::Arc;

    use super::*;

    struct TestCtx {
        global: VariableStore,
        line: VariableStore,
        frame: VariableStore,
        instance: VariableStore,
        stack: VecDeque<VariableValue>,
        structure: Vec<Vec<f64>>,
        clock: Clock,
        device_map: DeviceMap,
        errors: ErrorQueue,
    }

    impl TestCtx {
        fn new() -> Self {
            let server = Arc::new(ClockServer::new(120.0, 4.0));
            let clock = Clock {
                server,
                session_state: SessionState::new(),
                drift: 0,
                system_time_offset: 0,
            };
            Self {
                global: VariableStore::new(),
                line: VariableStore::new(),
                frame: VariableStore::new(),
                instance: VariableStore::new(),
                stack: VecDeque::new(),
                structure: vec![],
                clock,
                device_map: DeviceMap::new(),
                errors: ErrorQueue::default(),
            }
        }

        fn eval_ctx(&mut self) -> EvaluationContext<'_> {
            EvaluationContext {
                logic_date: 0,
                global_vars: &mut self.global,
                line_vars: &mut self.line,
                frame_vars: &mut self.frame,
                instance_vars: &mut self.instance,
                stack: &mut self.stack,
                line_index: 0,
                line_iterations: 0,
                frame_index: 0,
                frame_len: 1.0,
                frame_triggers: 0,
                structure: &self.structure,
                clock: &self.clock,
                device_map: &self.device_map,
                errors: &self.errors,
            }
        }
    }

    fn eval(script: &str) -> Vec<(ConcreteEvent, SyncTime)> {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        let mut ctx = tctx.eval_ctx();
        vm.evaluate(script, &mut ctx).unwrap()
    }

    fn eval_vm(
        vm: &mut CagireVM,
        tctx: &mut TestCtx,
        script: &str,
    ) -> Vec<(ConcreteEvent, SyncTime)> {
        let mut ctx = tctx.eval_ctx();
        vm.evaluate(script, &mut ctx).unwrap()
    }

    #[test]
    fn test_arithmetic() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        let mut ctx = tctx.eval_ctx();
        let sctx = StepContext::from_eval_ctx(&ctx);
        let mut dict = Dictionary::new();
        let (ops, spans) = compile_script("3 4 + 10 *", &mut dict).unwrap();
        let mut stack = Stack::new();
        let mut events = Vec::new();
        let mut cmd = CmdRegister::new();
        vm.execute_ops(
            &ops,
            &spans,
            &sctx,
            &mut ctx,
            &mut stack,
            &mut events,
            &mut cmd,
        )
        .unwrap();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.values[0], Value::Int(70));
    }

    #[test]
    fn test_sound_emits_generic_event() {
        let events = eval("\"sine\" sound 440 freq .");
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0].0, ConcreteEvent::Generic(VariableValue::Map(args), ..)
                if args.get("sound") == Some(&VariableValue::Str("sine".into()))
            )
        );
    }

    #[test]
    fn test_numeric_sound_name() {
        let events = eval("808 snd 440 freq .");
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0].0, ConcreteEvent::Generic(VariableValue::Map(args), ..)
                if args.get("sound") == Some(&VariableValue::Str("808".into()))
            )
        );
    }

    #[test]
    fn test_midi_note_emit() {
        let events = eval("60 note 100 velocity .");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiNote(60, 100, 1, _, 1)
        ));
    }

    #[test]
    fn test_cyclelist_polyphony() {
        let events = eval("60 64 67 note .");
        assert_eq!(events.len(), 3);
        for ev in &events {
            assert!(matches!(&ev.0, ConcreteEvent::MidiNote(..)));
        }
    }

    #[test]
    fn test_variables() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        eval_vm(&mut vm, &mut tctx, "42 !x");
        let events = eval_vm(&mut vm, &mut tctx, "@x note .");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiNote(42, _, _, _, _)
        ));
    }

    #[test]
    fn test_colon_definition() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        eval_vm(&mut vm, &mut tctx, ": hi 60 note 100 velocity . ;");
        let events = eval_vm(&mut vm, &mut tctx, "hi");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiNote(60, 100, 1, _, 1)
        ));
    }

    #[test]
    fn test_if_then_true() {
        let events = eval("1 if 60 note . then");
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_if_then_false() {
        let events = eval("0 if 60 note . then");
        assert!(events.is_empty());
    }

    #[test]
    fn test_if_else_then() {
        let events = eval("0 if 60 note . else 72 note . then");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiNote(72, _, _, _, _)
        ));
    }

    #[test]
    fn test_curly_braces_ignored() {
        let events = eval("60 {} note .");
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_note_names() {
        let events = eval("c4 note .");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiNote(60, _, _, _, _)
        ));
    }

    #[test]
    fn test_empty_script_errors() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        let mut ctx = tctx.eval_ctx();
        assert!(vm.evaluate("", &mut ctx).is_err());
    }

    #[test]
    fn test_midi_cc() {
        let events = eval("10 ccnum 64 ccout .");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiControl(10, 64, 1, 1)
        ));
    }

    #[test]
    fn test_print() {
        let events = eval("42 print");
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0].0, ConcreteEvent::Print(s) if s == "42"));
    }

    #[test]
    fn test_global_var_write_read() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        eval_vm(&mut vm, &mut tctx, "42 !G.root");
        assert_eq!(tctx.global.get("root"), Some(&VariableValue::Integer(42)));
        let events = eval_vm(&mut vm, &mut tctx, "@G.root note .");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiNote(42, _, _, _, _)
        ));
    }

    #[test]
    fn test_line_var_write_read() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        eval_vm(&mut vm, &mut tctx, "7 !L.count");
        assert_eq!(tctx.line.get("count"), Some(&VariableValue::Integer(7)));
        let events = eval_vm(&mut vm, &mut tctx, "@L.count note .");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiNote(7, _, _, _, _)
        ));
    }

    #[test]
    fn test_frame_var_write_read() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        eval_vm(&mut vm, &mut tctx, "3.14 !F.pi");
        let pi = tctx.frame.get("pi").unwrap();
        assert!(matches!(pi, VariableValue::Float(f) if (*f - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn test_instance_var_via_ctx() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        eval_vm(&mut vm, &mut tctx, "99 !x");
        assert_eq!(tctx.instance.get("x"), Some(&VariableValue::Integer(99)));
    }

    #[test]
    fn test_quotation_stays_local() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        eval_vm(&mut vm, &mut tctx, "( 60 note . ) !myquot");
        assert!(vm.vars.contains_key("myquot"));
        assert!(tctx.instance.get("myquot").is_none());
    }

    #[test]
    fn test_setkeep_global() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        let events = eval_vm(&mut vm, &mut tctx, "60 ,G.root note .");
        assert_eq!(tctx.global.get("root"), Some(&VariableValue::Integer(60)));
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiNote(60, _, _, _, _)
        ));
    }

    #[test]
    fn test_osc_event() {
        let events = eval("\"/synth/play\" sound 440 freq 0.5 gain .");
        assert_eq!(events.len(), 1);
        match &events[0].0 {
            ConcreteEvent::Osc { message, device_id } => {
                assert_eq!(message.addr, "/synth/play");
                assert_eq!(*device_id, 1);
                assert_eq!(message.args.len(), 4);
                assert_eq!(message.args[0], VariableValue::Str("freq".into()));
                assert_eq!(message.args[1], VariableValue::Float(440.0));
                assert_eq!(message.args[2], VariableValue::Str("gain".into()));
                assert_eq!(message.args[3], VariableValue::Float(0.5));
            }
            other => panic!("expected Osc event, got {other:?}"),
        }
    }

    #[test]
    fn test_osc_no_params() {
        let events = eval("\"/trigger\" sound .");
        assert_eq!(events.len(), 1);
        match &events[0].0 {
            ConcreteEvent::Osc { message, .. } => {
                assert_eq!(message.addr, "/trigger");
                assert!(message.args.is_empty());
            }
            other => panic!("expected Osc event, got {other:?}"),
        }
    }

    #[test]
    fn test_osc_with_device() {
        let events = eval("3 device \"/fx/reverb\" sound 0.8 mix .");
        assert_eq!(events.len(), 1);
        match &events[0].0 {
            ConcreteEvent::Osc { message, device_id } => {
                assert_eq!(message.addr, "/fx/reverb");
                assert_eq!(*device_id, 3);
            }
            other => panic!("expected Osc event, got {other:?}"),
        }
    }

    #[test]
    fn test_getmidicc_default() {
        let events = eval("10 1 ccval note .");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0].0,
            ConcreteEvent::MidiNote(0, _, _, _, _)
        ));
    }

    // --- at + cycle tests (ported from standalone Cagire) ---

    fn eval_with_runs(script: &str, runs: usize) -> Vec<(ConcreteEvent, SyncTime)> {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        let mut eval_ctx = tctx.eval_ctx();
        eval_ctx.frame_triggers = runs;
        vm.evaluate(script, &mut eval_ctx).unwrap()
    }

    fn get_midi_notes(events: &[(ConcreteEvent, SyncTime)]) -> Vec<u64> {
        events
            .iter()
            .filter_map(|(ev, _)| match ev {
                ConcreteEvent::MidiNote(note, _, _, _, _) => Some(*note),
                _ => None,
            })
            .collect()
    }

    fn get_event_times(events: &[(ConcreteEvent, SyncTime)]) -> Vec<SyncTime> {
        events.iter().map(|(_, t)| *t).collect()
    }

    fn get_generic_param(ev: &ConcreteEvent, key: &str) -> Option<f64> {
        match ev {
            ConcreteEvent::Generic(VariableValue::Map(args), ..) => {
                args.get(key).and_then(|v| match v {
                    VariableValue::Float(f) => Some(*f),
                    VariableValue::Integer(i) => Some(*i as f64),
                    _ => None,
                })
            }
            _ => None,
        }
    }

    #[test]
    fn test_at_single_delta() {
        let events = eval("0.5 at sine snd 440 freq .");
        assert_eq!(events.len(), 1);
        assert!(
            events[0].1 > 0,
            "0.5 delta should produce non-zero time offset"
        );
    }

    #[test]
    fn test_at_list_deltas() {
        let events = eval("0 0.5 at sine snd 440 freq .");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].1, 0, "first delta=0 should have time 0");
        assert!(
            events[1].1 > 0,
            "second delta=0.5 should have non-zero time"
        );
    }

    #[test]
    fn test_at_loop_with_cycle_notes() {
        let events = eval_with_runs("0 0.25 0.5 0.75 at [ c4 e4 g4 b4 ] cycle note .", 0);
        assert_eq!(events.len(), 4);
        let notes = get_midi_notes(&events);
        assert_eq!(notes, vec![60, 64, 67, 71]);
    }

    #[test]
    fn test_at_loop_cycle_wraps() {
        let events = eval_with_runs("0 0.25 0.5 0.75 at [ c4 e4 ] cycle note .", 0);
        assert_eq!(events.len(), 4);
        let notes = get_midi_notes(&events);
        assert_eq!(notes, vec![60, 64, 60, 64]);
    }

    #[test]
    fn test_at_loop_rand_different_per_subdivision() {
        let events = eval("0 0.5 at sine snd 1 1000 rand freq .");
        assert_eq!(events.len(), 2);
        let f0 = get_generic_param(&events[0].0, "freq");
        let f1 = get_generic_param(&events[1].0, "freq");
        assert!(f0.is_some() && f1.is_some());
        assert_ne!(
            f0, f1,
            "rand should produce different values per at subdivision"
        );
    }

    #[test]
    fn test_at_loop_poly_cycling() {
        let events = eval("0 0.5 at sine snd c4 e4 note .");
        assert_eq!(events.len(), 4);
        // Each at iteration emits 2 poly voices (c4=60, e4=64)
        let notes: Vec<f64> = events
            .iter()
            .filter_map(|(ev, _)| get_generic_param(ev, "note"))
            .collect();
        assert_eq!(notes, vec![60.0, 64.0, 60.0, 64.0]);
    }

    #[test]
    fn test_at_loop_cycle_advances_across_runs() {
        for base_runs in 0..3 {
            let events = eval_with_runs("0 0.5 at [ c4 e4 g4 ] cycle note .", base_runs);
            assert_eq!(events.len(), 2, "base_runs={base_runs}");
            let notes = get_midi_notes(&events);
            let expected_0 = [60, 64, 67][(base_runs * 2) % 3];
            let expected_1 = [60, 64, 67][(base_runs * 2 + 1) % 3];
            assert_eq!(notes[0], expected_0, "runs={base_runs}: iter 0");
            assert_eq!(notes[1], expected_1, "runs={base_runs}: iter 1");
        }
    }

    #[test]
    fn test_at_loop_done_no_emit() {
        let events = eval("0 0.5 at [ 1 2 ] cycle drop done");
        assert!(events.is_empty());
    }

    #[test]
    fn test_at_loop_done_sets_variables() {
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        let events = eval_vm(
            &mut vm,
            &mut tctx,
            "0 0.5 at [ 10 20 ] cycle !x done @x note .",
        );
        assert_eq!(events.len(), 1);
        // Last iteration wins: cycle idx 1 -> 20
        let notes = get_midi_notes(&events);
        assert_eq!(notes[0], 20);
    }

    #[test]
    fn test_at_loop_timing_increases() {
        let events = eval("0 0.25 0.5 0.75 at sine snd 440 freq .");
        assert_eq!(events.len(), 4);
        let times = get_event_times(&events);
        assert_eq!(times[0], 0);
        assert!(times[1] > times[0]);
        assert!(times[2] > times[1]);
        assert!(times[3] > times[2]);
    }

    #[test]
    fn test_at_loop_midi_note_emit() {
        let events = eval("0 0.25 0.5 at 60 note .");
        assert_eq!(events.len(), 3);
        for (ev, _) in &events {
            assert!(matches!(ev, ConcreteEvent::MidiNote(60, _, _, _, _)));
        }
    }

    #[test]
    fn test_string_pattern_at_records_per_event_highlight_spans() {
        let script = "\"x--x..[xx.]\" at 60 note .";
        let mut vm = CagireVM::new();
        let mut tctx = TestCtx::new();
        let events = eval_vm(&mut vm, &mut tctx, script);
        assert_eq!(events.len(), 4);
        assert_eq!(
            vm.event_annotations,
            vec![
                vec![Span { start: 1, end: 4 }],
                vec![Span { start: 4, end: 5 }],
                vec![Span { start: 8, end: 9 }],
                vec![Span { start: 9, end: 10 }],
            ],
        );
    }
}
