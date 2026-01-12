use super::{
    Instruction, Program,
    EvaluationContext,
    variable::{Variable, VariableValue},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{clock::TimeSpan, vm::{GeneratorModifier, GeneratorShape}};
use crate::log_eprintln;
use crate::scene::script::ReturnInfo;

use std::collections::HashMap;
use std::f64::consts::PI;

// Import state keys
use crate::vm::environment_func::{
    ISAW_LAST_BEAT_KEY, ISAW_PHASE_KEY, RANDSTEP_LAST_BEAT_KEY, RANDSTEP_PHASE_KEY,
    RANDSTEP_VALUE_KEY, SAW_LAST_BEAT_KEY, SAW_PHASE_KEY, SINE_LAST_BEAT_KEY, SINE_PHASE_KEY,
    TRI_LAST_BEAT_KEY, TRI_PHASE_KEY,
};
use crate::protocol::ProtocolDevice;

pub const DEFAULT_DEVICE : i64 = 1;
pub const DEFAULT_CHAN : i64 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ControlASM {
    #[default]
    Nop,
    // Atomic operations
    // Atomic(Vec<ControlASM>) // executes in one step of the scheduler all the instructions in the vector
    // Arithmetic operations
    Add(Variable, Variable, Variable),
    Div(Variable, Variable, Variable),
    Mod(Variable, Variable, Variable),
    Mul(Variable, Variable, Variable),
    Sub(Variable, Variable, Variable),
    Neg(Variable, Variable),
    // Boolean operations
    And(Variable, Variable, Variable),
    Not(Variable, Variable),
    Or(Variable, Variable, Variable),
    Xor(Variable, Variable, Variable),
    LowerThan(Variable, Variable, Variable),
    LowerOrEqual(Variable, Variable, Variable),
    GreaterThan(Variable, Variable, Variable),
    GreaterOrEqual(Variable, Variable, Variable),
    Equal(Variable, Variable, Variable),
    Different(Variable, Variable, Variable),
    Scale(Variable, Variable, Variable, Variable, Variable, Variable),
    Clamp(Variable, Variable, Variable, Variable),
    Min(Variable, Variable, Variable),
    Max(Variable, Variable, Variable),
    Quantize(Variable, Variable, Variable),
    // Bitwise operations
    BitAnd(Variable, Variable, Variable),
    BitNot(Variable, Variable),
    BitOr(Variable, Variable, Variable),
    BitXor(Variable, Variable, Variable),
    ShiftLeft(Variable, Variable, Variable),
    ShiftRightA(Variable, Variable, Variable),
    ShiftRightL(Variable, Variable, Variable),
    // String operations
    //Concat(Variable, Variable, Variable),
    // Time manipulation
    FloatAsBeats(Variable, Variable),
    FloatAsFrames(Variable, Variable),
    // AsBeats(Variable, Variable),
    // AsMicros(Variable, Variable),
    // AsFrames(Variable, Variable),
    // Memory manipulation
    Mov(Variable, Variable),
    IsSet(Variable, Variable),
    // Stack operations
    Push(Variable),
    Pop(Variable),
    PushFront(Variable),
    PopFront(Variable),
    // Map operations
    MapInsert(Variable, Variable, Variable, Variable),
    MapGet(Variable, Variable, Variable),
    MapHas(Variable, Variable, Variable),
    MapRemove(Variable, Variable, Variable, Variable),
    // Vec operations
    VecPush(Variable, Variable, Variable),
    VecPop(Variable, Variable, Variable),
    VecLen(Variable, Variable),
    VecInsert(Variable, Variable, Variable, Variable),
    VecGet(Variable, Variable, Variable),
    VecRemove(Variable, Variable, Variable, Variable),
    // Generators
    GenStart(Variable),
    GenGet(Variable, Variable),
    GenSetShape(GeneratorShape, Variable),
    GenAddModifier(GeneratorModifier, Variable, Variable),
    GenRemoveModifier(Variable, Variable),
    GenConfigureShape(Variable, Variable),
    GenConfigureModifier(Variable, Variable, Variable),
    GenSeed(Variable, Variable),
    GenSave(Variable, Variable),
    GenRestore(Variable, Variable),
    // Jumps
    Jump(usize),
    JumpIf(Variable, usize),
    JumpIfNot(Variable, usize),
    JumpIfDifferent(Variable, Variable, usize),
    JumpIfEqual(Variable, Variable, usize),
    JumpIfLess(Variable, Variable, usize),
    JumpIfLessOrEqual(Variable, Variable, usize),
    RelJump(i64),
    RelJumpIf(Variable, i64),
    RelJumpIfNot(Variable, i64),
    RelJumpIfDifferent(Variable, Variable, i64),
    RelJumpIfEqual(Variable, Variable, i64),
    RelJumpIfLess(Variable, Variable, i64),
    RelJumpIfLessOrEqual(Variable, Variable, i64),
    // Calls and returns
    CallFunction(Variable),
    CallProcedure(usize),
    Return, // Only exit at the moment
    // Add Oscillator Getters
    GetSine(Variable, Variable),
    GetSaw(Variable, Variable),
    GetTriangle(Variable, Variable),
    GetISaw(Variable, Variable),
    GetRandStep(Variable, Variable),
    GetMidiCC(Variable, Variable, Variable, Variable), // device_var | _use_context_device, channel_var | _use_context_channel, ctrl_var, result_dest_var
}

impl ControlASM {
    fn evaluate_var_as_int_or(
        &self,
        ctx: &mut EvaluationContext,
        var: &Variable,
        default: i64,
    ) -> i64 {
        let value = ctx.evaluate(var); // Pass mutable borrow to evaluate
        match value {
            VariableValue::Integer(i) => i,
            VariableValue::Float(f) => f.round() as i64,
            VariableValue::Decimal(sign, num, den) => (sign as i64) * ((num / den) as i64),
            _ => default,
        }
    }

    pub fn execute(
        &self,
        ctx: &mut EvaluationContext,
        return_stack: &mut Vec<ReturnInfo>,
        instruction_position: usize,
        current_prog: &Program,
    ) -> ReturnInfo {
        match self {
            ControlASM::Nop => ReturnInfo::None,
            // Arithmetic operations
            ControlASM::Add(x, y, z)
            | ControlASM::Div(x, y, z)
            | ControlASM::Mod(x, y, z)
            | ControlASM::Mul(x, y, z)
            | ControlASM::Sub(x, y, z) => {
                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                // cast to correct types
                x_value.compatible_cast(&mut y_value, ctx);

                // compute the result
                let res_value = match self {
                    ControlASM::Add(_, _, _) => x_value.add(y_value, ctx),
                    ControlASM::Div(_, _, _) => x_value.div(y_value, ctx),
                    ControlASM::Mod(_, _, _) => x_value.rem(y_value, ctx),
                    ControlASM::Mul(_, _, _) => x_value.mul(y_value, ctx),
                    ControlASM::Sub(_, _, _) => x_value.sub(y_value, ctx),
                    _ => unreachable!(),
                };

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Arithmetic operations (unary)
            ControlASM::Neg(x, z) => {
                let x_value = ctx.evaluate(x);

                let res_value = -x_value;
                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Boolean operations (binary)
            ControlASM::And(x, y, z) | ControlASM::Or(x, y, z) | ControlASM::Xor(x, y, z) => {
                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                // Cast to correct types
                x_value = x_value.cast_as_bool(ctx);
                y_value = y_value.cast_as_bool(ctx);

                // Compute the result
                let res_value = match self {
                    ControlASM::And(_, _, _) => x_value.and(y_value),
                    ControlASM::Or(_, _, _) => x_value.or(y_value),
                    ControlASM::Xor(_, _, _) => x_value.xor(y_value),
                    _ => unreachable!(),
                };

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Boolean operations (unary)
            ControlASM::Not(x, z) => {
                let mut x_value = ctx.evaluate(x);

                // Cast to correct type
                x_value = x_value.cast_as_bool(ctx);

                // Compute the result
                let res_value = !x_value;

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Boolean operations (numeric operators)
            ControlASM::LowerThan(x, y, z)
            | ControlASM::LowerOrEqual(x, y, z)
            | ControlASM::GreaterThan(x, y, z)
            | ControlASM::GreaterOrEqual(x, y, z)
            | ControlASM::Equal(x, y, z)
            | ControlASM::Different(x, y, z) => {
                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                // cast to correct types
                x_value.compatible_cast(&mut y_value, ctx);

                let res_value = match self {
                    ControlASM::LowerThan(_, _, _) => x_value.lt(y_value),
                    ControlASM::LowerOrEqual(_, _, _) => x_value.leq(y_value),
                    ControlASM::GreaterThan(_, _, _) => x_value.gt(y_value),
                    ControlASM::GreaterOrEqual(_, _, _) => x_value.geq(y_value),
                    ControlASM::Equal(_, _, _) => x_value.eq(y_value),
                    ControlASM::Different(_, _, _) => x_value.neq(y_value),
                    _ => unreachable!(),
                };

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Bitwise operations (binary)
            ControlASM::BitAnd(x, y, z)
            | ControlASM::BitOr(x, y, z)
            | ControlASM::BitXor(x, y, z)
            | ControlASM::ShiftLeft(x, y, z)
            | ControlASM::ShiftRightA(x, y, z)
            | ControlASM::ShiftRightL(x, y, z) => {
                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                // Cast to correct types
                x_value = x_value.cast_as_integer(ctx);
                y_value = y_value.cast_as_integer(ctx);

                // Compute the result
                let res_value = match self {
                    ControlASM::BitAnd(_, _, _) => x_value & y_value,
                    ControlASM::BitOr(_, _, _) => x_value | y_value,
                    ControlASM::BitXor(_, _, _) => x_value ^ y_value,
                    ControlASM::ShiftLeft(_, _, _) => x_value << y_value,
                    ControlASM::ShiftRightA(_, _, _) => x_value >> y_value,
                    ControlASM::ShiftRightL(_, _, _) => x_value.logical_shift(y_value),
                    _ => unreachable!(),
                };

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Bitwise operations (unary)
            ControlASM::BitNot(x, z) => {
                let mut x_value = ctx.evaluate(x);

                // Cast to correct type
                x_value = x_value.cast_as_integer(ctx);

                // Compute the result
                let res_value = !x_value;

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Time manipulation
            ControlASM::FloatAsBeats(x, z) => {
                let x_value = ctx.evaluate(x);
                let x_value = x_value.cast_as_float(ctx);
                let res_value =
                    VariableValue::Dur(TimeSpan::Beats(x_value.as_float(ctx)));
                ctx.set_var(z, res_value);
                ReturnInfo::None
            }
            ControlASM::FloatAsFrames(x, z) => {
                let x_value = ctx.evaluate(x);
                let x_value = x_value.cast_as_float(ctx);
                let res_value = VariableValue::Dur(TimeSpan::Frames(
                    x_value.as_float(ctx),
                ));
                ctx.set_var(z, res_value);
                ReturnInfo::None
            }
            // Memory manipulation
            ControlASM::Mov(x, z) => {
                let x_value = ctx.evaluate(x);
                ctx.set_var(z, x_value);
                ReturnInfo::None
            }
            ControlASM::IsSet(x, z) => {
                let exists = ctx.has_var(x);
                ctx.set_var(z, exists);
                ReturnInfo::None
            }
            ControlASM::Push(x) => {
                let value = ctx.evaluate(x);
                ctx.stack.push_back(value);
                ReturnInfo::None
            }
            ControlASM::Pop(x) => {
                if let Some(value) = ctx.stack.pop_back() {
                    ctx.set_var(x, value);
                } else {
                    log_eprintln!("[!] Runtime Error: Pop from empty stack into Var {:?}", x);
                }
                ReturnInfo::None
            }
            ControlASM::PushFront(x) => {
                let value = ctx.evaluate(x);
                ctx.stack.push_front(value);
                ReturnInfo::None
            }
            ControlASM::PopFront(x) => {
                if let Some(value) = ctx.stack.pop_front() {
                    ctx.set_var(x, value);
                } else {
                    log_eprintln!("[!] Runtime Error: Pop from empty stack into Var {:?}", x);
                }
                ReturnInfo::None
            }
            ControlASM::MapInsert(map, key, val, res) => {
                let map_value = ctx.evaluate(map);
                let key_as_string = ctx.evaluate(key).as_str(ctx);
                let val_value = ctx.evaluate(val);

                if let VariableValue::Map(mut hash_map) = map_value {
                    hash_map.insert(key_as_string, val_value);
                    ctx.set_var(res, VariableValue::Map(hash_map));
                } else {
                    log_eprintln!(
                        "[!] Runtime Error: MapInsert expected a Map variable for {:?}, got {:?}",
                        map,
                        map_value
                    );
                    ctx.set_var(res, VariableValue::Map(HashMap::new()));
                }
                ReturnInfo::None
            }
            ControlASM::MapGet(map, key, res) => {
                let key_value = ctx.evaluate(key).as_str(ctx);
                let map_value = ctx.value_ref(map);
                
                let value = if let Some(VariableValue::Map(map)) = map_value {
                    map.get(&key_value).cloned().unwrap_or_default()
                } else {
                    log_eprintln!(
                        "[!] Runtime Error: MapGet from a variable that is not a map ! {:?}",
                        map_value
                    );
                    VariableValue::default()
                };

                ctx.set_var(res, value);
                ReturnInfo::None
            }
            ControlASM::MapHas(map, key, res) => {
                let key_value = ctx.evaluate(key).as_str(ctx);
                let map_value = ctx.value_ref(map);

                let value = if let Some(VariableValue::Map(map)) = map_value {
                    map.contains_key(&key_value)
                } else {
                    false
                };

                ctx.set_var(res, value);
                ReturnInfo::None
            }
            ControlASM::MapRemove(map, key, res, removed) => {
                let map_value = ctx.evaluate(map);
                let key_value = ctx.evaluate(key).as_str(ctx);

                let (map, value) = if let VariableValue::Map(mut map) = map_value {
                    let value = map.remove(&key_value).unwrap_or_default();
                    (VariableValue::Map(map), value)
                } else {
                    log_eprintln!("[!] Runtime Error: MapRemove from a variable that is not a map ! {:?}", map_value);
                    (VariableValue::Map(HashMap::new()), VariableValue::default())
                };

                ctx.set_var(res, map);
                ctx.set_var(removed, value);
                ReturnInfo::None
            }
            ControlASM::VecPush(vec, val, res) => {
                let vec_value = ctx.evaluate(vec);
                let val_value = ctx.evaluate(val);

                if let VariableValue::Vec(mut vec) = vec_value {
                    vec.push(val_value);
                    ctx.set_var(res, VariableValue::Vec(vec));
                } else {
                    log_eprintln!(
                        "[!] Runtime Error: VecPush expected a Vec variable for {:?}, got {:?}",
                        vec, vec_value
                    );
                    ctx.set_var(res, VariableValue::Vec(Vec::new()));
                }
                ReturnInfo::None
            }
            ControlASM::VecPop(vec, res, removed) => {
                let vec_value = ctx.evaluate(vec);

                let (vec, value) = if let VariableValue::Vec(mut vec) = vec_value {
                    if !vec.is_empty() {
                        let value = vec.pop().unwrap();
                        (VariableValue::Vec(vec), value)
                    } else {
                        log_eprintln!("[!] Runtime Error: VecPop from empty vector !");
                        (VariableValue::Vec(vec), Default::default())
                    }
                } else {
                    log_eprintln!("[!] Runtime Error: VecPop from a variable that is not a vec ! {:?}", vec_value);
                    (VariableValue::Vec(Vec::new()), VariableValue::default())
                };

                ctx.set_var(res, vec);
                ctx.set_var(removed, value);
                ReturnInfo::None
            }
            ControlASM::VecLen(vec, res) => {
                let vec_value = ctx.value_ref(vec);
                let len = if let Some(VariableValue::Vec(vec)) = vec_value {
                    vec.len() as i64
                } else {
                    log_eprintln!("[!] Runtime Error: VecLen from a variable that is not a vec ! {:?}", vec_value);
                    0
                };
                ctx.set_var(res, len);
                ReturnInfo::None
            }
            ControlASM::VecInsert(vec, at, val, res) => {
                let vec_value = ctx.evaluate(vec);
                let at_index = ctx.evaluate(at).as_integer(ctx) as usize;
                let val_value = ctx.evaluate(val);

                if let VariableValue::Vec(mut vec) = vec_value {
                    let index = std::cmp::min(vec.len(), at_index);
                    vec.insert(index, val_value);
                    ctx.set_var(res, VariableValue::Vec(vec));
                } else {
                    log_eprintln!(
                        "[!] Runtime Error: VecInsert expected a Vec variable for {:?}, got {:?}",
                        vec, vec_value
                    );
                    ctx.set_var(res, VariableValue::Vec(Vec::new()));
                }
                ReturnInfo::None
            }
            ControlASM::VecGet(vec, at, res) => {
                let key_value = ctx.evaluate(at).as_integer(ctx) as usize;
                let vec_value = ctx.value_ref(vec);
                
                let value = if let Some(VariableValue::Vec(vec)) = vec_value {
                    vec.get(key_value).cloned().unwrap_or_default()
                } else {
                    log_eprintln!("[!] Runtime Error: VecGet from a variable that is not a vec ! {:?}", vec_value);
                    VariableValue::default()
                };

                ctx.set_var(res, value);
                ReturnInfo::None
            }
            ControlASM::VecRemove(vec, at, res, removed) => {
                let vec_value = ctx.evaluate(vec);
                let key_value = ctx.evaluate(at).as_integer(ctx) as usize;

                let (vec, value) = if let VariableValue::Vec(mut vec) = vec_value {
                    if key_value <= vec.len() {
                        let value = vec.remove(key_value);
                        (VariableValue::Vec(vec), value)
                    } else {
                        log_eprintln!("[!] Runtime Error: VecRemove index out of bounds ! {} > {}", key_value, vec.len());
                        (VariableValue::Vec(vec), Default::default())
                    }
                } else {
                    log_eprintln!("[!] Runtime Error: VecRemove from a variable that is not a vec ! {:?}", vec_value);
                    (VariableValue::Vec(Vec::new()), VariableValue::default())
                };

                ctx.set_var(res, vec);
                ctx.set_var(removed, value);
                ReturnInfo::None
            }
            // Jumps
            ControlASM::Jump(index) => ReturnInfo::IndexChange(*index),
            ControlASM::RelJump(index_change) => ReturnInfo::RelIndexChange(*index_change),
            ControlASM::JumpIf(x, _) | ControlASM::RelJumpIf(x, _) => {
                let mut x_value = ctx.evaluate(x);

                // Cast to correct type
                x_value = x_value.cast_as_bool(ctx);

                if x_value.is_true(ctx) {
                    match self {
                        ControlASM::JumpIf(_, index) => return ReturnInfo::IndexChange(*index),
                        ControlASM::RelJumpIf(_, index_change) => {
                            return ReturnInfo::RelIndexChange(*index_change);
                        }
                        _ => unreachable!(),
                    }
                }

                ReturnInfo::None
            }
            ControlASM::JumpIfNot(x, _) | ControlASM::RelJumpIfNot(x, _) => {
                let mut x_value = ctx.evaluate(x);

                // Cast to correct type
                x_value = x_value.cast_as_bool(ctx);

                if !x_value.is_true(ctx) {
                    match self {
                        ControlASM::JumpIfNot(_, index) => return ReturnInfo::IndexChange(*index),
                        ControlASM::RelJumpIfNot(_, index_change) => {
                            return ReturnInfo::RelIndexChange(*index_change);
                        }
                        _ => unreachable!(),
                    }
                }

                ReturnInfo::None
            }
            ControlASM::JumpIfDifferent(x, y, _)
            | ControlASM::JumpIfEqual(x, y, _)
            | ControlASM::JumpIfLess(x, y, _)
            | ControlASM::JumpIfLessOrEqual(x, y, _)
            | ControlASM::RelJumpIfDifferent(x, y, _)
            | ControlASM::RelJumpIfEqual(x, y, _)
            | ControlASM::RelJumpIfLess(x, y, _)
            | ControlASM::RelJumpIfLessOrEqual(x, y, _) => {
                let x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                y_value.as_type(&x_value, ctx);

                match self {
                    ControlASM::JumpIfDifferent(_, _, _)
                    | ControlASM::RelJumpIfDifferent(_, _, _) => {
                        if x_value != y_value {
                            match self {
                                ControlASM::JumpIfDifferent(_, _, index) => {
                                    return ReturnInfo::IndexChange(*index);
                                }
                                ControlASM::RelJumpIfDifferent(_, _, index_change) => {
                                    return ReturnInfo::RelIndexChange(*index_change);
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    ControlASM::JumpIfEqual(_, _, _) | ControlASM::RelJumpIfEqual(_, _, _) => {
                        if x_value == y_value {
                            match self {
                                ControlASM::JumpIfEqual(_, _, index) => {
                                    return ReturnInfo::IndexChange(*index);
                                }
                                ControlASM::RelJumpIfEqual(_, _, index_change) => {
                                    return ReturnInfo::RelIndexChange(*index_change);
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    ControlASM::JumpIfLess(_, _, _) | ControlASM::RelJumpIfLess(_, _, _) => {
                        if x_value < y_value {
                            match self {
                                ControlASM::JumpIfLess(_, _, index) => {
                                    return ReturnInfo::IndexChange(*index);
                                }
                                ControlASM::RelJumpIfLess(_, _, index_change) => {
                                    return ReturnInfo::RelIndexChange(*index_change);
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    ControlASM::JumpIfLessOrEqual(_, _, _)
                    | ControlASM::RelJumpIfLessOrEqual(_, _, _) => {
                        if x_value <= y_value {
                            match self {
                                ControlASM::JumpIfLessOrEqual(_, _, index) => {
                                    return ReturnInfo::IndexChange(*index);
                                }
                                ControlASM::RelJumpIfLessOrEqual(_, _, index_change) => {
                                    return ReturnInfo::RelIndexChange(*index_change);
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    _ => unreachable!(),
                }

                ReturnInfo::None
            }
            // Calls and returns
            ControlASM::CallFunction(f) => {
                return_stack.push(ReturnInfo::ProgChange(
                    instruction_position + 1,
                    current_prog.clone(),
                ));

                let f_value = ctx.evaluate(f);
                let next_prog = match f_value {
                    VariableValue::Func(p) => p,
                    _ => vec![Instruction::Control(ControlASM::Return)],
                };
                ReturnInfo::ProgChange(0, next_prog)
            }
            ControlASM::CallProcedure(proc_position) => {
                return_stack.push(ReturnInfo::IndexChange(instruction_position + 1));
                ReturnInfo::IndexChange(*proc_position)
            }
            ControlASM::Return => match return_stack.pop() {
                Some(return_info) => return_info,
                None => ReturnInfo::IndexChange(usize::MAX),
            },
            // Updated Range implementation
            ControlASM::Scale(val, old_min, old_max, new_min, new_max, dest) => {
                let val_f = ctx.evaluate(val).as_float(ctx);
                let old_min_f = ctx.evaluate(old_min).as_float(ctx);
                let old_max_f = ctx.evaluate(old_max).as_float(ctx);
                let new_min_f = ctx.evaluate(new_min).as_float(ctx);
                let new_max_f = ctx.evaluate(new_max).as_float(ctx);

                let old_range = old_max_f - old_min_f;

                let result = if old_range.abs() < f64::EPSILON {
                    new_min_f // Return new_min if old range is zero
                } else {
                    let normalized = (val_f - old_min_f) / old_range;
                    new_min_f + normalized * (new_max_f - new_min_f)
                };

                // Clamp the result to the new range
                let clamped_result = result
                    .max(new_min_f.min(new_max_f))
                    .min(new_min_f.max(new_max_f));

                ctx.set_var(dest, VariableValue::Float(clamped_result));
                ReturnInfo::None
            }
            // Clamp implementation remains the same
            ControlASM::Clamp(val, min, max, dest) => {
                let val_f = ctx.evaluate(val).as_float(ctx);
                let min_f = ctx.evaluate(min).as_float(ctx);
                let max_f = ctx.evaluate(max).as_float(ctx);

                let clamped_value = val_f.max(min_f).min(max_f);
                ctx.set_var(dest, VariableValue::Float(clamped_value));
                ReturnInfo::None
            }
            ControlASM::Min(v1, v2, dest) => {
                let v1_f = ctx.evaluate(v1).as_float(ctx);
                let v2_f = ctx.evaluate(v2).as_float(ctx);
                ctx.set_var(dest, VariableValue::Float(v1_f.min(v2_f)));
                ReturnInfo::None
            }
            ControlASM::Max(v1, v2, dest) => {
                let v1_f = ctx.evaluate(v1).as_float(ctx);
                let v2_f = ctx.evaluate(v2).as_float(ctx);
                ctx.set_var(dest, VariableValue::Float(v1_f.max(v2_f)));
                ReturnInfo::None
            }
            ControlASM::Quantize(val, step, dest) => {
                let val_f = ctx.evaluate(val).as_float(ctx);
                let step_f = ctx.evaluate(step).as_float(ctx);

                let result = if step_f.abs() < f64::EPSILON {
                    val_f // Return original value if step is zero
                } else {
                    (val_f / step_f).round() * step_f
                };
                ctx.set_var(dest, VariableValue::Float(result));
                ReturnInfo::None
            }
            // Stateful Oscillators using Line Variables and Beat Delta
            ControlASM::GetSine(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat();

                let last_beat = ctx
                    .line_vars
                    .get(SINE_LAST_BEAT_KEY)
                    .map_or(current_beat, |v| v.as_float(ctx));
                let current_phase = ctx
                    .line_vars
                    .get(SINE_PHASE_KEY)
                    .map_or(0.0, |v| v.as_float(ctx));

                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();

                ctx.line_vars
                    .insert(SINE_PHASE_KEY.to_string(), VariableValue::Float(new_phase));
                ctx.line_vars.insert(
                    SINE_LAST_BEAT_KEY.to_string(),
                    VariableValue::Float(current_beat),
                );

                let raw_value = (new_phase * 2.0 * PI).sin(); // Raw value [-1, 1]
                let normalized_value = (raw_value + 1.0) / 2.0; // Normalize to [0, 1]
                let scaled_to_midi = 1.0 + normalized_value * 126.0; // Scale to [1, 127]
                let midi_int = scaled_to_midi.round().max(1.0).min(127.0) as i64;
                ctx.set_var(dest_var, VariableValue::Integer(midi_int));
                ReturnInfo::None
            }
            ControlASM::GetSaw(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat();
                let last_beat = ctx
                    .line_vars
                    .get(SAW_LAST_BEAT_KEY)
                    .map_or(current_beat, |v| v.as_float(ctx));
                let current_phase = ctx
                    .line_vars
                    .get(SAW_PHASE_KEY)
                    .map_or(0.0, |v| v.as_float(ctx));
                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();

                ctx.line_vars
                    .insert(SAW_PHASE_KEY.to_string(), VariableValue::Float(new_phase));
                ctx.line_vars.insert(
                    SAW_LAST_BEAT_KEY.to_string(),
                    VariableValue::Float(current_beat),
                );

                let raw_value = new_phase * 2.0 - 1.0; // Raw value [-1, 1]
                let normalized_value = (raw_value + 1.0) / 2.0; // Normalize to [0, 1]
                let scaled_to_midi = 1.0 + normalized_value * 126.0; // Scale to [1, 127]
                let midi_int = scaled_to_midi.round().max(1.0).min(127.0) as i64;
                ctx.set_var(dest_var, VariableValue::Integer(midi_int));
                ReturnInfo::None
            }
            ControlASM::GetTriangle(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat();
                let last_beat = ctx
                    .line_vars
                    .get(TRI_LAST_BEAT_KEY)
                    .map_or(current_beat, |v| v.as_float(ctx));
                let current_phase = ctx
                    .line_vars
                    .get(TRI_PHASE_KEY)
                    .map_or(0.0, |v| v.as_float(ctx));
                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();

                ctx.line_vars
                    .insert(TRI_PHASE_KEY.to_string(), VariableValue::Float(new_phase));
                ctx.line_vars.insert(
                    TRI_LAST_BEAT_KEY.to_string(),
                    VariableValue::Float(current_beat),
                );

                let raw_value = 1.0 - (new_phase * 2.0 - 1.0).abs() * 2.0; // Raw value [-1, 1]
                let normalized_value = (raw_value + 1.0) / 2.0; // Normalize to [0, 1]
                let scaled_to_midi = 1.0 + normalized_value * 126.0; // Scale to [1, 127]
                let midi_int = scaled_to_midi.round().max(1.0).min(127.0) as i64;
                ctx.set_var(dest_var, VariableValue::Integer(midi_int));
                ReturnInfo::None
            }
            ControlASM::GetISaw(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat();
                let last_beat = ctx
                    .line_vars
                    .get(ISAW_LAST_BEAT_KEY)
                    .map_or(current_beat, |v| v.as_float(ctx));
                let current_phase = ctx
                    .line_vars
                    .get(ISAW_PHASE_KEY)
                    .map_or(0.0, |v| v.as_float(ctx));
                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();

                ctx.line_vars
                    .insert(ISAW_PHASE_KEY.to_string(), VariableValue::Float(new_phase));
                ctx.line_vars.insert(
                    ISAW_LAST_BEAT_KEY.to_string(),
                    VariableValue::Float(current_beat),
                );

                let raw_value = 1.0 - (new_phase * 2.0); // Raw value [1, -1] (inverted saw)
                let normalized_value = (raw_value + 1.0) / 2.0; // Normalize to [0, 1]
                let scaled_to_midi = 1.0 + normalized_value * 126.0; // Scale to [1, 127]
                let midi_int = scaled_to_midi.round().max(1.0).min(127.0) as i64;
                ctx.set_var(dest_var, VariableValue::Integer(midi_int));
                ReturnInfo::None
            }
            ControlASM::GetRandStep(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat();
                let last_beat = ctx
                    .line_vars
                    .get(RANDSTEP_LAST_BEAT_KEY)
                    .map_or(current_beat, |v| v.as_float(ctx));
                let current_phase = ctx
                    .line_vars
                    .get(RANDSTEP_PHASE_KEY)
                    .map_or(0.0, |v| v.as_float(ctx));

                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();

                let mut current_value = ctx
                    .line_vars
                    .get(RANDSTEP_VALUE_KEY)
                    .map_or(0, |v| v.as_integer(ctx)); // Default to 0 if not set

                // Check if phase wrapped around (cycle completed) or if it's the first run
                if new_phase < current_phase || current_value == 0 {
                    current_value = (rand::random::<u8>() % 127) as i64 + 1; // Generate new random value [1, 127]

                    ctx.line_vars.insert(
                        RANDSTEP_VALUE_KEY.to_string(),
                        VariableValue::Integer(current_value),
                    ); // Store it
                }

                ctx.line_vars.insert(
                    RANDSTEP_PHASE_KEY.to_string(),
                    VariableValue::Float(new_phase),
                );
                ctx.line_vars.insert(
                    RANDSTEP_LAST_BEAT_KEY.to_string(),
                    VariableValue::Float(current_beat),
                );

                ctx.set_var(dest_var, VariableValue::Integer(current_value)); // Return the current held value
                ReturnInfo::None
            }
            ControlASM::GetMidiCC(device_var, channel_var, ctrl_var, result_var) => {
                // Resolve Device ID
                let device_id = match device_var {
                    Variable::Instance(name) if name == "_use_context_device" => {
                        // Fetch from implicit context variable (_target_device_id)
                        let context_device_var =
                            Variable::Instance("_target_device_id".to_string());
                        self.evaluate_var_as_int_or(ctx, &context_device_var, DEFAULT_DEVICE)
                            as usize
                    }
                    _ => {
                        // Evaluate the provided device_var
                        self.evaluate_var_as_int_or(ctx, device_var, DEFAULT_DEVICE) as usize
                    }
                };

                // Resolve Channel
                let channel_val = match channel_var {
                    Variable::Instance(name) if name == "_use_context_channel" => {
                        // Fetch from implicit context variable (_chan)
                        let context_chan_var = Variable::Instance("_chan".to_string());
                        self.evaluate_var_as_int_or(ctx, &context_chan_var, DEFAULT_CHAN)
                    }
                    _ => {
                        // Evaluate the provided channel_var
                        self.evaluate_var_as_int_or(ctx, channel_var, DEFAULT_CHAN)
                    }
                };

                // Evaluate Control Number
                let control_val = ctx.evaluate(ctrl_var).as_integer(ctx);

                // Look up device and get CC value
                let mut cc_value = 0i64; // Default value

                if let Some(device_name) = ctx.device_map.get_name_for_slot(device_id) {
                    let input_connections = ctx.device_map.input_connections.lock().unwrap();
                    if let Some(device_arc) = input_connections.get(&device_name) {
                        if let ProtocolDevice::MIDIInDevice(midi_in) = &**device_arc {
                            if let Ok(memory_guard) = midi_in.memory.lock() {
                                let midi_chan_0_based =
                                    (channel_val.saturating_sub(1).max(0).min(15)) as i8;
                                let control_i8 = (control_val.max(0).min(127)) as i8;
                                cc_value = memory_guard.get(midi_chan_0_based, control_i8) as i64;
                                // Optional Debug: println!("[VM GetMidiCC] Resolved Dev: {}, Chan: {}, Ctrl: {}, Result: {}", device_id, channel_val, control_val, cc_value);
                            } else {
                                log_eprintln!(
                                    "[!] GetMidiCC Error: Failed to lock MidiInMemory for device '{}'",
                                    device_name
                                );
                            }
                        } else {
                            log_eprintln!(
                                "[!] GetMidiCC Warning: Device '{}' in slot {} is not a MIDI Input device.",
                                device_name,
                                device_id
                            );
                        }
                    } else {
                        log_eprintln!(
                            "[!] GetMidiCC Warning: Device name '{}' (from slot {}) not found in registered input connections.",
                            device_name,
                            device_id
                        );
                    }
                } else if device_id != DEFAULT_DEVICE as usize {
                    // Only warn if specific non-default device requested
                    log_eprintln!(
                        "[!] GetMidiCC Warning: No device assigned to slot {}.",
                        device_id
                    );
                }

                // Store the result
                ctx.set_var(result_var, VariableValue::Integer(cc_value));
                ReturnInfo::None
            }
        }
    }
}
