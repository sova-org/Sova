use std::collections::HashMap;

use rhai::Expr;
use sova_core::{
    clock::{NEVER, SyncTime, TimeSpan},
    error::SovaError,
    vm::{EvaluationContext, event::ConcreteEvent, variable::VariableValue},
};

use super::value::RhaiValue;

pub const DEFAULT_RHAI_INSTRUCTION_BATCH_SIZE: usize = 16;

#[derive(Debug, Clone)]
pub struct LoweredProgram {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    EvalExpr(Expr),
    SetVar {
        name: String,
        expr: Expr,
    },
    SetVarOp {
        name: String,
        op: String,
        expr: Expr,
    },
    SetIndex {
        name: String,
        indices: Vec<Expr>,
        expr: Expr,
    },
    SetIndexOp {
        name: String,
        indices: Vec<Expr>,
        op: String,
        expr: Expr,
    },
    JumpIfFalse {
        cond: Expr,
        target: usize,
    },
    Jump {
        target: usize,
    },
    Emit {
        args: Expr,
        dur: Option<Expr>,
        chan: Option<Expr>,
        dev: Option<Expr>,
    },
    Delay {
        dur: Expr,
    },
    ForInit {
        id: usize,
        iterable: Expr,
        iter_var: String,
        counter_var: Option<String>,
        exit_target: usize,
    },
    ForNext {
        id: usize,
        body_start: usize,
    },
    ForCleanup {
        id: usize,
    },
}

#[derive(Debug, Clone)]
struct ForState {
    items: Vec<VariableValue>,
    next_index: usize,
    iter_var: String,
    counter_var: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum ScopeTarget {
    Global,
    Line,
    Frame,
    Instance,
}

impl ScopeTarget {
    fn resolve(name: &str) -> (Self, &str) {
        if let Some(rest) = name.strip_prefix("g_") {
            return (Self::Global, rest);
        }
        if let Some(rest) = name.strip_prefix("l_") {
            return (Self::Line, rest);
        }
        if let Some(rest) = name.strip_prefix("f_") {
            return (Self::Frame, rest);
        }
        (Self::Instance, name)
    }
}

#[derive(Debug, Clone)]
pub struct RhaiExecutor {
    instructions: Vec<Instruction>,
    pc: usize,
    terminated: bool,
    pub instruction_batch_size: usize,
    for_states: HashMap<usize, ForState>,
}

impl RhaiExecutor {
    pub fn new(program: LoweredProgram) -> Self {
        Self {
            instructions: program.instructions,
            pc: 0,
            terminated: false,
            instruction_batch_size: DEFAULT_RHAI_INSTRUCTION_BATCH_SIZE,
            for_states: HashMap::new(),
        }
    }

    pub fn has_terminated(&self) -> bool {
        self.terminated
    }

    pub fn stop(&mut self) {
        self.terminated = true;
        self.pc = self.instructions.len();
        self.for_states.clear();
    }

    pub fn execute_next(
        &mut self,
        ctx: &mut EvaluationContext,
    ) -> (Option<ConcreteEvent>, SyncTime) {
        for _ in 0..self.instruction_batch_size {
            if self.pc >= self.instructions.len() {
                self.terminated = true;
                return (None, NEVER);
            }

            let instruction = self.instructions[self.pc].clone();
            let result = self.execute_instruction(instruction, ctx);
            let Ok(action) = result else {
                let err = result.err().unwrap_or_else(|| "unknown error".to_string());
                ctx.errors
                    .throw(SovaError::from(&*ctx).message(format!("rhai runtime error: {err}")));
                self.terminated = true;
                return (None, NEVER);
            };

            match action {
                StepResult::Continue => {}
                StepResult::YieldEvent(event) => return (Some(event), 0),
                StepResult::YieldDelay(wait) => return (None, wait),
            }
        }

        (None, 0)
    }

    fn execute_instruction(
        &mut self,
        instruction: Instruction,
        ctx: &mut EvaluationContext,
    ) -> Result<StepResult, String> {
        match instruction {
            Instruction::EvalExpr(expr) => {
                let _ = self.eval_expr(&expr, ctx)?;
                self.pc += 1;
                Ok(StepResult::Continue)
            }
            Instruction::SetVar { name, expr } => {
                let value = self.eval_expr(&expr, ctx)?;
                self.write_var(&name, value, ctx);
                self.pc += 1;
                Ok(StepResult::Continue)
            }
            Instruction::SetVarOp { name, op, expr } => {
                let rhs = self.eval_expr(&expr, ctx)?;
                let lhs = self.read_var(&name, ctx);
                let value = self.apply_binary_operator(&op, lhs, rhs, ctx)?;
                self.write_var(&name, value, ctx);
                self.pc += 1;
                Ok(StepResult::Continue)
            }
            Instruction::SetIndex {
                name,
                indices,
                expr,
            } => {
                let rhs = self.eval_expr(&expr, ctx)?;
                self.write_indexed_value(&name, &indices, rhs, None, ctx)?;
                self.pc += 1;
                Ok(StepResult::Continue)
            }
            Instruction::SetIndexOp {
                name,
                indices,
                op,
                expr,
            } => {
                let rhs = self.eval_expr(&expr, ctx)?;
                self.write_indexed_value(&name, &indices, rhs, Some(op.as_str()), ctx)?;
                self.pc += 1;
                Ok(StepResult::Continue)
            }
            Instruction::JumpIfFalse { cond, target } => {
                let condition = self.eval_expr(&cond, ctx)?;
                if !condition.truthy(ctx) {
                    self.pc = target;
                } else {
                    self.pc += 1;
                }
                Ok(StepResult::Continue)
            }
            Instruction::Jump { target } => {
                self.pc = target;
                Ok(StepResult::Continue)
            }
            Instruction::Emit {
                args,
                dur,
                chan,
                dev,
            } => {
                let args = self.eval_expr(&args, ctx)?.into_variable();
                let duration = if let Some(dur_expr) = dur {
                    let value = self.eval_expr(&dur_expr, ctx)?;
                    let Some(duration) = value.duration() else {
                        return Err(format!(
                            "EMIT duration must be a duration, got {}",
                            value.type_name()
                        ));
                    };
                    duration.as_micros(ctx.clock, ctx.frame_len)
                } else {
                    0
                };

                let channel = if let Some(chan_expr) = chan {
                    self.eval_expr(&chan_expr, ctx)?.as_string(ctx)
                } else {
                    String::new()
                };

                let device = if let Some(dev_expr) = dev {
                    let raw = self.eval_expr(&dev_expr, ctx)?.as_i64(ctx);
                    usize::try_from(raw).unwrap_or(1)
                } else {
                    1
                };

                self.pc += 1;
                Ok(StepResult::YieldEvent(ConcreteEvent::Generic(
                    args, duration, channel, device,
                )))
            }
            Instruction::Delay { dur } => {
                let value = self.eval_expr(&dur, ctx)?;
                let Some(duration) = value.duration() else {
                    return Err(format!(
                        "DELAY expects a duration, got {}",
                        value.type_name()
                    ));
                };
                self.pc += 1;
                Ok(StepResult::YieldDelay(
                    duration.as_micros(ctx.clock, ctx.frame_len),
                ))
            }
            Instruction::ForInit {
                id,
                iterable,
                iter_var,
                counter_var,
                exit_target,
            } => {
                let iterable_value = self.eval_expr(&iterable, ctx)?;
                let items = iterable_value.into_variable().as_vec(ctx);
                if items.is_empty() {
                    self.for_states.remove(&id);
                    self.pc = exit_target;
                    return Ok(StepResult::Continue);
                }

                let first = items[0].clone();
                let state = ForState {
                    items,
                    next_index: 1,
                    iter_var: iter_var.clone(),
                    counter_var: counter_var.clone(),
                };
                self.for_states.insert(id, state);
                self.write_var(&iter_var, RhaiValue::from_variable(first), ctx);
                if let Some(counter_var) = counter_var {
                    self.write_var(&counter_var, RhaiValue::from_variable(0.into()), ctx);
                }
                self.pc += 1;
                Ok(StepResult::Continue)
            }
            Instruction::ForNext { id, body_start } => {
                if let Some(state) = self.for_states.get_mut(&id) {
                    if state.next_index < state.items.len() {
                        let item_index = state.next_index;
                        state.next_index += 1;

                        let iter_var = state.iter_var.clone();
                        let counter_var = state.counter_var.clone();
                        let next_value = state.items[item_index].clone();

                        self.write_var(&iter_var, RhaiValue::from_variable(next_value), ctx);
                        if let Some(counter_var) = counter_var {
                            self.write_var(
                                &counter_var,
                                RhaiValue::from_variable((item_index as i64).into()),
                                ctx,
                            );
                        }
                        self.pc = body_start;
                        return Ok(StepResult::Continue);
                    }
                }

                self.for_states.remove(&id);
                self.pc += 1;
                Ok(StepResult::Continue)
            }
            Instruction::ForCleanup { id } => {
                self.for_states.remove(&id);
                self.pc += 1;
                Ok(StepResult::Continue)
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr, ctx: &mut EvaluationContext) -> Result<RhaiValue, String> {
        match expr {
            Expr::DynamicConstant(value, ..) => Ok(RhaiValue::from_dynamic((**value).clone())),
            Expr::BoolConstant(value, ..) => Ok(RhaiValue::from_variable((*value).into())),
            Expr::IntegerConstant(value, ..) => {
                Ok(RhaiValue::from_variable((*value as i64).into()))
            }
            Expr::FloatConstant(value, ..) => {
                Ok(RhaiValue::from_variable(f64::from(*value).into()))
            }
            Expr::CharConstant(value, ..) => Ok(RhaiValue::from_variable(value.to_string().into())),
            Expr::StringConstant(value, ..) => {
                Ok(RhaiValue::from_variable(value.to_string().into()))
            }
            Expr::InterpolatedString(parts, ..) => {
                let mut out = String::new();
                for part in parts {
                    out.push_str(&self.eval_expr(part, ctx)?.as_string(ctx));
                }
                Ok(RhaiValue::from_variable(out.into()))
            }
            Expr::Array(items, ..) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.eval_expr(item, ctx)?.into_variable());
                }
                Ok(RhaiValue::from_variable(out.into()))
            }
            Expr::Map(map, ..) => {
                let mut out = HashMap::new();
                for (key, value) in map.0.iter() {
                    out.insert(key.to_string(), self.eval_expr(value, ctx)?.into_variable());
                }
                Ok(RhaiValue::from_variable(out.into()))
            }
            Expr::Unit(..) => Ok(RhaiValue::Unit),
            Expr::Variable(var, ..) => Ok(self.read_var(var.1.as_str(), ctx)),
            Expr::FnCall(call, ..) => self.eval_fn_call(call, ctx),
            Expr::Index(binary, ..) => {
                let base = self.eval_expr(&binary.lhs, ctx)?;
                let index = self.eval_expr(&binary.rhs, ctx)?;
                self.read_index(base, index, ctx)
            }
            Expr::And(items, ..) => {
                for item in items {
                    if !self.eval_expr(item, ctx)?.truthy(ctx) {
                        return Ok(RhaiValue::from_variable(false.into()));
                    }
                }
                Ok(RhaiValue::from_variable(true.into()))
            }
            Expr::Or(items, ..) => {
                for item in items {
                    if self.eval_expr(item, ctx)?.truthy(ctx) {
                        return Ok(RhaiValue::from_variable(true.into()));
                    }
                }
                Ok(RhaiValue::from_variable(false.into()))
            }
            Expr::Coalesce(items, ..) => {
                for item in items {
                    let value = self.eval_expr(item, ctx)?;
                    if !matches!(value, RhaiValue::Unit) {
                        return Ok(value);
                    }
                }
                Ok(RhaiValue::Unit)
            }
            _ => Err("unsupported expression encountered at runtime".to_string()),
        }
    }

    fn eval_fn_call(
        &mut self,
        call: &rhai::FnCallExpr,
        ctx: &mut EvaluationContext,
    ) -> Result<RhaiValue, String> {
        let name = call.name.as_str();

        match name {
            "beats" => {
                if call.args.len() != 1 {
                    return Err("beats expects exactly one argument".to_string());
                }
                let amount = self.eval_expr(&call.args[0], ctx)?.as_f64(ctx);
                return Ok(RhaiValue::from_variable(VariableValue::Dur(
                    TimeSpan::Beats(amount),
                )));
            }
            "frames" => {
                if call.args.len() != 1 {
                    return Err("frames expects exactly one argument".to_string());
                }
                let amount = self.eval_expr(&call.args[0], ctx)?.as_f64(ctx);
                return Ok(RhaiValue::from_variable(VariableValue::Dur(
                    TimeSpan::Frames(amount),
                )));
            }
            "micros" => {
                if call.args.len() != 1 {
                    return Err("micros expects exactly one argument".to_string());
                }
                let amount = self.eval_expr(&call.args[0], ctx)?.as_i64(ctx).max(0) as u64;
                return Ok(RhaiValue::from_variable(VariableValue::Dur(
                    TimeSpan::Micros(amount),
                )));
            }
            "EMIT" | "DELAY" => {
                return Err(format!("{name} cannot be used as an expression"));
            }
            _ => {}
        }

        match call.args.len() {
            1 => {
                let value = self.eval_expr(&call.args[0], ctx)?;
                self.apply_unary_operator(name, value, ctx)
            }
            2 => {
                let lhs = self.eval_expr(&call.args[0], ctx)?;
                let rhs = self.eval_expr(&call.args[1], ctx)?;
                self.apply_binary_operator(name, lhs, rhs, ctx)
            }
            _ => Err(format!(
                "unsupported function call '{}' with {} args",
                name,
                call.args.len()
            )),
        }
    }

    fn apply_unary_operator(
        &self,
        op: &str,
        value: RhaiValue,
        ctx: &EvaluationContext,
    ) -> Result<RhaiValue, String> {
        let var = value.into_variable();
        let out = match op {
            "+" => var,
            "-" => var.neg(ctx),
            "!" | "~" => var.not(ctx),
            _ => return Err(format!("unsupported unary operator '{op}'")),
        };
        Ok(RhaiValue::from_variable(out))
    }

    fn apply_binary_operator(
        &self,
        op: &str,
        lhs: RhaiValue,
        rhs: RhaiValue,
        ctx: &EvaluationContext,
    ) -> Result<RhaiValue, String> {
        if op == "??" {
            return Ok(if matches!(lhs, RhaiValue::Unit) {
                rhs
            } else {
                lhs
            });
        }

        let left = lhs.into_variable();
        let right = rhs.into_variable();

        let out = match op {
            "+" => left.add(right, ctx),
            "-" => left.sub(right, ctx),
            "*" => left.mul(right, ctx),
            "/" => left.div(right, ctx),
            "%" => left.rem(right, ctx),
            "**" => left.pow(right, ctx),
            "==" => left.eq(&right, ctx),
            "!=" => left.neq(&right, ctx),
            "<" => left.lt(&right, ctx),
            "<=" => left.leq(&right, ctx),
            ">" => left.gt(&right, ctx),
            ">=" => left.geq(&right, ctx),
            "&&" => left.and(right, ctx),
            "||" => left.or(right, ctx),
            "&" => left.bitand(right, ctx),
            "|" => left.bitor(right, ctx),
            "^" => left.bitxor(right, ctx),
            "<<" => left.shl(right, ctx),
            ">>" => left.shr(right, ctx),
            _ => return Err(format!("unsupported binary operator '{op}'")),
        };

        Ok(RhaiValue::from_variable(out))
    }

    fn read_index(
        &self,
        base: RhaiValue,
        index: RhaiValue,
        ctx: &EvaluationContext,
    ) -> Result<RhaiValue, String> {
        match base.into_variable() {
            VariableValue::Vec(items) => {
                let idx = index.as_i64(ctx);
                if idx < 0 {
                    return Ok(RhaiValue::Unit);
                }
                Ok(items
                    .get(idx as usize)
                    .cloned()
                    .map(RhaiValue::from_variable)
                    .unwrap_or(RhaiValue::Unit))
            }
            VariableValue::Map(map) => {
                let key = index.as_string(ctx);
                Ok(map
                    .get(&key)
                    .cloned()
                    .map(RhaiValue::from_variable)
                    .unwrap_or(RhaiValue::Unit))
            }
            _ => Ok(RhaiValue::Unit),
        }
    }

    fn write_indexed_value(
        &mut self,
        name: &str,
        index_exprs: &[Expr],
        rhs: RhaiValue,
        op: Option<&str>,
        ctx: &mut EvaluationContext,
    ) -> Result<(), String> {
        let mut indices = Vec::with_capacity(index_exprs.len());
        for index_expr in index_exprs {
            indices.push(self.eval_expr(index_expr, ctx)?);
        }

        let mut base = self.read_var(name, ctx).into_variable();
        self.write_nested_index(&mut base, &indices, &rhs, op, ctx)?;
        self.write_var(name, RhaiValue::from_variable(base), ctx);
        Ok(())
    }

    fn write_nested_index(
        &self,
        container: &mut VariableValue,
        indices: &[RhaiValue],
        rhs: &RhaiValue,
        op: Option<&str>,
        ctx: &EvaluationContext,
    ) -> Result<(), String> {
        if indices.is_empty() {
            *container = rhs.clone().into_variable();
            return Ok(());
        }

        let index = &indices[0];
        let is_leaf = indices.len() == 1;

        match container {
            VariableValue::Vec(items) => {
                let idx = index.as_i64(ctx);
                if idx < 0 {
                    return Err("array index cannot be negative".to_string());
                }
                let idx = idx as usize;
                while items.len() <= idx {
                    items.push(VariableValue::default());
                }
                if is_leaf {
                    if let Some(op) = op {
                        let current = RhaiValue::from_variable(items[idx].clone());
                        items[idx] = self
                            .apply_binary_operator(op, current, rhs.clone(), ctx)?
                            .into_variable();
                    } else {
                        items[idx] = rhs.clone().into_variable();
                    }
                    Ok(())
                } else {
                    self.write_nested_index(&mut items[idx], &indices[1..], rhs, op, ctx)
                }
            }
            VariableValue::Map(map) => {
                let key = index.as_string(ctx);
                if is_leaf {
                    if let Some(op) = op {
                        let current =
                            RhaiValue::from_variable(map.get(&key).cloned().unwrap_or_default());
                        let new_value = self
                            .apply_binary_operator(op, current, rhs.clone(), ctx)?
                            .into_variable();
                        map.insert(key, new_value);
                    } else {
                        map.insert(key, rhs.clone().into_variable());
                    }
                    Ok(())
                } else {
                    let entry = map.entry(key).or_insert_with(VariableValue::default);
                    self.write_nested_index(entry, &indices[1..], rhs, op, ctx)
                }
            }
            _ => {
                let mut replacement = if index.prefers_array_index() {
                    VariableValue::Vec(Vec::new())
                } else {
                    VariableValue::Map(HashMap::new())
                };
                self.write_nested_index(&mut replacement, indices, rhs, op, ctx)?;
                *container = replacement;
                Ok(())
            }
        }
    }

    fn read_var(&self, name: &str, ctx: &EvaluationContext) -> RhaiValue {
        let (target, key) = ScopeTarget::resolve(name);
        let value = match target {
            ScopeTarget::Global => ctx.global_vars.get(key),
            ScopeTarget::Line => ctx.line_vars.get(key),
            ScopeTarget::Frame => ctx.frame_vars.get(key),
            ScopeTarget::Instance => ctx.instance_vars.get(key),
        }
        .cloned()
        .unwrap_or_default();

        RhaiValue::from_variable(value)
    }

    fn write_var(&self, name: &str, value: RhaiValue, ctx: &mut EvaluationContext) {
        let (target, key) = ScopeTarget::resolve(name);
        let value = value.into_variable();
        match target {
            ScopeTarget::Global => {
                ctx.global_vars.insert(key.to_owned(), value);
            }
            ScopeTarget::Line => {
                ctx.line_vars.insert(key.to_owned(), value);
            }
            ScopeTarget::Frame => {
                ctx.frame_vars.insert(key.to_owned(), value);
            }
            ScopeTarget::Instance => {
                ctx.instance_vars.insert(key.to_owned(), value);
            }
        }
    }
}

enum StepResult {
    Continue,
    YieldEvent(ConcreteEvent),
    YieldDelay(SyncTime),
}
