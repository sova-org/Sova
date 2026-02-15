use super::{
    EvaluationContext, Instruction, Program,
    variable::{Variable, VariableValue},
};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, mem};

use crate::{clock::TimeSpan, vm::{GeneratorModifier, GeneratorShape}};
use crate::log_eprintln;
use crate::scene::script::ReturnInfo;

use crate::protocol::ProtocolDevice;

pub const DEFAULT_DEVICE: i64 = 1;
pub const DEFAULT_CHAN: i64 = 1;

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
    BitOr(Variable, Variable, Variable),
    BitXor(Variable, Variable, Variable),
    // Shift Operations
    ShiftLeftA(Variable, Variable, Variable),
    ShiftLeftL(Variable, Variable, Variable),
    ShiftRightA(Variable, Variable, Variable),
    ShiftRightL(Variable, Variable, Variable),
    CircularShiftLeft(Variable, Variable, Variable),
    CircularShiftRight(Variable, Variable, Variable),
    LeadingZeros(Variable, Variable),
    // Sequence operations
    Concat(Variable, Variable, Variable),
    Len(Variable, Variable),
    Index(Variable, Variable, Variable),
    Insert(Variable, Variable, Variable, Variable),
    Remove(Variable, Variable, Variable, Variable),
    Contains(Variable, Variable, Variable),
    // Time manipulation
    FloatAsBeats(Variable, Variable),
    FloatAsFrames(Variable, Variable),
    // AsBeats(Variable, Variable),
    // AsMicros(Variable, Variable),
    // AsFrames(Variable, Variable),
    // Memory manipulation
    /// Moves 0 into 1 while erasing 1's type
    Mov(Variable, Variable),
    /// Moves 0 into 1 while preserving 1's type
    Redefine(Variable, Variable),
    IsSet(Variable, Variable),
    // Stack operations
    Push(Variable),
    Pop(Variable),
    PushFront(Variable),
    PopFront(Variable),
    // Vec operations
    VecPush(Variable, Variable, Variable),
    VecPop(Variable, Variable, Variable),
    // Generators
    GenStart(Variable),
    GenCopy(Variable, Variable),
    GenSetShape(GeneratorShape, Variable),
    GenAddModifier(GeneratorModifier, Variable, Variable),
    GenRemoveModifier(Variable, Variable),
    GenConfigureShape(Variable, Variable),
    GenConfigureModifier(Variable, Variable, Variable),
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
    // Midi
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
            VariableValue::Decimal(d) => d.into(),
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
            | ControlASM::Sub(x, y, z) 
            | ControlASM::Concat(x, y, z) => {
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
                    ControlASM::Concat(_, _, _) => x_value.concat(y_value, ctx),
                    _ => unreachable!(),
                };

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Arithmetic operations (unary)
            ControlASM::Neg(x, z) => {
                let x_value = ctx.evaluate(x);

                let res_value = x_value.neg(ctx);
                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Boolean operations (binary)
            ControlASM::And(x, y, z) 
            | ControlASM::Or(x, y, z) 
            | ControlASM::Xor(x, y, z) => {
                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                // Cast to correct types
                x_value.cast_as_bool(ctx);
                y_value.cast_as_bool(ctx);

                // Compute the result
                let res_value = match self {
                    ControlASM::And(_, _, _) => x_value.and(y_value, ctx),
                    ControlASM::Or(_, _, _) => x_value.or(y_value, ctx),
                    ControlASM::Xor(_, _, _) => x_value.xor(y_value, ctx),
                    _ => unreachable!(),
                };

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Boolean operations (unary)
            ControlASM::Not(x, z) => {
                let mut x_value = ctx.evaluate(x);

                // Cast to correct type
                match &mut x_value {
                    VariableValue::Integer(_) 
                    | VariableValue::Bool(_) => (),
                    x => x.cast_as_integer(ctx),
                }

                // Compute the result
                let res_value = x_value.not(ctx);

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
                    ControlASM::LowerThan(_, _, _) => x_value.lt(&y_value, ctx),
                    ControlASM::LowerOrEqual(_, _, _) => x_value.leq(&y_value, ctx),
                    ControlASM::GreaterThan(_, _, _) => x_value.gt(&y_value, ctx),
                    ControlASM::GreaterOrEqual(_, _, _) => x_value.geq(&y_value, ctx),
                    ControlASM::Equal(_, _, _) => x_value.eq(&y_value, ctx),
                    ControlASM::Different(_, _, _) => x_value.neq(&y_value, ctx),
                    _ => unreachable!(),
                };

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            // Bitwise operations (binary)
            ControlASM::BitAnd(x, y, z)
            | ControlASM::BitOr(x, y, z)
            | ControlASM::BitXor(x, y, z)
            | ControlASM::ShiftLeftA(x, y, z)
            | ControlASM::ShiftLeftL(x, y, z)
            | ControlASM::ShiftRightA(x, y, z) 
            | ControlASM::ShiftRightL(x, y, z)
            | ControlASM::CircularShiftLeft(x, y, z)
            | ControlASM::CircularShiftRight(x, y, z) => {
                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                // For maps, apply bitwise ops directly (union/intersection/symmetric diff)
                // For other types, cast to integer first

                match (&mut x_value, &mut y_value) {
                    (VariableValue::Map(_), VariableValue::Map(_))
                    | (VariableValue::Integer(_), VariableValue::Integer(_)) => (),
                    (x,y) => {
                        x.cast_as_integer(ctx);
                        y.cast_as_integer(ctx);
                    }
                }

                let res_value = match self {
                    ControlASM::BitAnd(_, _, _) => x_value.bitand(y_value, ctx),
                    ControlASM::BitOr(_, _, _) => x_value.bitor(y_value, ctx),
                    ControlASM::BitXor(_, _, _) => x_value.bitxor(y_value, ctx),
                    ControlASM::ShiftLeftA(_, _, _) => x_value.arithmetic_shl(y_value, ctx),
                    ControlASM::ShiftLeftL(_, _, _) => x_value.shl(y_value, ctx),
                    ControlASM::ShiftRightA(_, _, _) => x_value.arithmetic_shr(y_value, ctx), 
                    ControlASM::ShiftRightL(_, _, _) => x_value.shr(y_value, ctx), 
                    _ => unreachable!(),
                };

                ctx.set_var(z, res_value);

                ReturnInfo::None
            }
            ControlASM::LeadingZeros(x, z) => {
                let x_value = ctx.evaluate(x);
                let x_int = x_value.as_integer(ctx);
                let lz = (x_int as u64).leading_zeros() as i64;
                ctx.set_var(z, VariableValue::Integer(lz));
                ReturnInfo::None
            }
            // Time manipulation
            ControlASM::FloatAsBeats(x, z) => {
                let x_value = ctx.evaluate(x);
                let res_value = VariableValue::Dur(TimeSpan::Beats(
                    x_value.as_float(ctx),
                ));
                ctx.set_var(z, res_value);
                ReturnInfo::None
            }
            ControlASM::FloatAsFrames(x, z) => {
                let x_value = ctx.evaluate(x);
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
            ControlASM::Redefine(x, z) => {
                let x_value = ctx.evaluate(x);
                ctx.redefine(z, x_value);
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
            ControlASM::Insert(cont, key, val, res) => {
                let container = ctx.evaluate(cont);
                let key = ctx.evaluate(key);
                let val_value = ctx.evaluate(val);

                match container {
                    VariableValue::Map(mut hash_map) => {
                        hash_map.insert(key.as_str(ctx), val_value);
                        ctx.set_var(res, VariableValue::Map(hash_map));
                    }
                    VariableValue::Vec(mut v) => {
                        let index = key.as_integer(ctx) as usize;
                        if index > v.len() {
                            todo!()
                        }
                        v.insert(index, val_value);
                        ctx.set_var(res, VariableValue::Vec(v));
                    }
                    VariableValue::Str(mut s) => {
                        let index = key.as_integer(ctx) as usize;
                        if index > s.len() {
                            todo!()
                        }
                        s.insert_str(index, val_value.as_str(ctx).as_str());
                        ctx.set_var(res, VariableValue::Str(s));
                    }
                    _ => {
                        log_eprintln!(
                            "[!] Runtime Error: Insert expected a container variable for {:?}, got {:?}",
                            cont,
                            container
                        );
                        ctx.set_var(res, VariableValue::Vec(Vec::new()));
                        todo!()
                    }
                }

                ReturnInfo::None
            }
            ControlASM::Index(cont, key, res) => {
                let container = ctx.evaluate(cont);
                let key = ctx.evaluate(key);

                match container {
                    VariableValue::Map(hash_map) => {
                        let value = hash_map.get(key.as_str(ctx).as_str()).cloned();
                        ctx.set_var(res, value.unwrap_or_default());
                    }
                    VariableValue::Vec(v) => {
                        let index = key.as_integer(ctx) as usize;
                        let value = v.get(index).cloned();
                        ctx.set_var(res, value.unwrap_or_default());
                    }
                    VariableValue::Blob(v) => {
                        let index = key.as_integer(ctx) as usize;
                        let value = v.get(index).cloned();
                        ctx.set_var(res, value.unwrap_or_default() as i64);
                    }
                    VariableValue::Str(s) => {
                        let index = key.as_integer(ctx) as usize;
                        let value = s.chars().nth(index).map(|c| c.to_string());
                        ctx.set_var(res, value.unwrap_or_default());
                    }
                    _ => {
                        log_eprintln!(
                            "[!] Runtime Error: Index expected a container variable for {:?}, got {:?}",
                            cont,
                            container
                        );
                        ctx.set_var(res, VariableValue::Vec(Vec::new()));
                        todo!()
                    }
                }

                ReturnInfo::None
            }
            ControlASM::Contains(cont, key, res) => {
                let container = ctx.evaluate(cont);
                let key = ctx.evaluate(key);

                match container {
                    VariableValue::Map(hash_map) => {
                        let key = key.as_str(ctx);
                        ctx.set_var(res, hash_map.contains_key(&key));
                    }
                    VariableValue::Vec(v) => {
                        let contains = v.contains(&key);
                        ctx.set_var(res, contains);
                    }
                    VariableValue::Str(s) => {
                        let index = key.as_integer(ctx) as usize;
                        let value = s.chars().nth(index).map(|c| c.to_string());
                        ctx.set_var(res, value.unwrap_or_default());
                    }
                    _ => {
                        log_eprintln!(
                            "[!] Runtime Error: Contains expected a container variable for {:?}, got {:?}",
                            cont,
                            container
                        );
                        ctx.set_var(res, VariableValue::Vec(Vec::new()));
                        todo!()
                    }
                }

                ReturnInfo::None
            }
            ControlASM::Len(src, dest) => {
                let val = ctx.value_ref(src);
                let len = match val {
                    Some(VariableValue::Map(m)) => m.len() as i64,
                    Some(VariableValue::Vec(v)) => v.len() as i64,
                    Some(VariableValue::Str(s)) => s.len() as i64,
                    Some(VariableValue::Blob(b)) => b.len() as i64,
                    _ => 0,
                };
                ctx.set_var(dest, len);
                ReturnInfo::None
            }
            ControlASM::Remove(cont, key, res, removed) => {
                let container = ctx.evaluate(cont);
                let key = ctx.evaluate(key);

                match container {
                    VariableValue::Map(mut hash_map) => {
                        let value = hash_map.remove(&key.as_str(ctx));
                        ctx.set_var(res, VariableValue::Map(hash_map));
                        ctx.set_var(removed, value.unwrap_or_default());
                    }
                    VariableValue::Vec(mut v) => {
                        let index = key.as_integer(ctx) as usize;
                        let value = if index < v.len() {
                            v.remove(index)
                        } else {
                            Default::default()
                        };
                        ctx.set_var(res, VariableValue::Vec(v));
                        ctx.set_var(removed, value);
                    }
                    VariableValue::Blob(mut v) => {
                        let index = key.as_integer(ctx) as usize;
                        let value = if index < v.len() {
                            v.remove(index)
                        } else {
                            Default::default()
                        };
                        ctx.set_var(res, VariableValue::Blob(v));
                        ctx.set_var(removed, value as i64);
                    }
                    VariableValue::Str(mut s) => {
                        let index = key.as_integer(ctx) as usize;
                        let value = if index < s.len() {
                            s.remove(index).to_string()
                        } else {
                            Default::default()
                        };
                        ctx.set_var(res, VariableValue::Str(s));
                        ctx.set_var(removed, value);
                    }
                    _ => {
                        log_eprintln!(
                            "[!] Runtime Error: Insert expected a container variable for {:?}, got {:?}",
                            cont,
                            container
                        );
                        ctx.set_var(res, VariableValue::Vec(Vec::new()));
                        todo!()
                    }
                }

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
                        vec,
                        vec_value
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
                    log_eprintln!(
                        "[!] Runtime Error: VecPop from a variable that is not a vec ! {:?}",
                        vec_value
                    );
                    (VariableValue::Vec(Vec::new()), VariableValue::default())
                };

                ctx.set_var(res, vec);
                ctx.set_var(removed, value);
                ReturnInfo::None
            }
            // Generators
            ControlASM::GenStart(g) => {
                let date = ctx.logic_date;
                let Some(VariableValue::Generator(g)) = ctx.value_ref_mut(g) else {
                    log_eprintln!("Unable to start non-generator variable !");
                    return ReturnInfo::None;
                };
                g.start(date);
                ReturnInfo::None
            }
            ControlASM::GenCopy(g, z) => {
                let Some(VariableValue::Generator(g)) = ctx.value_ref(g) else {
                    log_eprintln!("Unable to copy non-generator variable !");
                    return ReturnInfo::None;
                };
                ctx.redefine(z, g.clone());
                ReturnInfo::None
            }
            ControlASM::GenSetShape(shape, g) => {
                let Some(VariableValue::Generator(g)) = ctx.value_ref_mut(g) else {
                    log_eprintln!("Unable to set shape of non-generator variable !");
                    return ReturnInfo::None;
                };
                g.shape = shape.clone();
                ReturnInfo::None
            }
            ControlASM::GenAddModifier(modif, index, g) => {
                let index = ctx.evaluate(index).as_integer(ctx) as usize;
                let Some(VariableValue::Generator(g)) = ctx.value_ref_mut(g) else {
                    log_eprintln!("Unable to set shape of non-generator variable !");
                    return ReturnInfo::None;
                };
                if g.modifiers.len() < index {
                    log_eprintln!("Modifier index out of bounds for insertion ! Ignoring...");
                    return ReturnInfo::None;
                }
                g.modifiers.insert(index, modif.clone());
                ReturnInfo::None
            }
            ControlASM::GenRemoveModifier(index, g) => {
                let index = ctx.evaluate(index).as_integer(ctx) as usize;
                let Some(VariableValue::Generator(g)) = ctx.value_ref_mut(g) else {
                    log_eprintln!("Unable to set shape of non-generator variable !");
                    return ReturnInfo::None;
                };
                g.modifiers.remove(index);
                ReturnInfo::None
            }
            ControlASM::GenConfigureShape(config, g) => {
                let config = ctx.evaluate(config);
                let Some(VariableValue::Generator(g_value)) = ctx.value_ref_mut(g) else {
                    log_eprintln!("Unable to set shape of non-generator variable !");
                    return ReturnInfo::None;
                };
                let mut shape = mem::take(&mut g_value.shape);
                shape.configure(config, ctx);
                let Some(VariableValue::Generator(g_value)) = ctx.value_ref_mut(g) else {
                    unreachable!()
                };
                g_value.shape = shape;
                ReturnInfo::None
            }
            ControlASM::GenConfigureModifier(config, index, g) => {
                let config = ctx.evaluate(config);
                let index = ctx.evaluate(index).as_integer(ctx) as usize;
                let Some(VariableValue::Generator(g_value)) = ctx.value_ref_mut(g) else {
                    log_eprintln!("Unable to set shape of non-generator variable !");
                    return ReturnInfo::None;
                };
                if g_value.modifiers.len() <= index {
                    log_eprintln!("Modifier index out of bounds for configuration ! Ignoring...");
                    return ReturnInfo::None;
                }
                let mut modif = mem::take(&mut g_value.modifiers[index]);
                modif.configure(config, ctx);
                let Some(VariableValue::Generator(g_value)) = ctx.value_ref_mut(g) else {
                    unreachable!()
                };
                g_value.modifiers[index] = modif;
                ReturnInfo::None
            }
            // Jumps
            ControlASM::Jump(index) => ReturnInfo::IndexChange(*index),
            ControlASM::RelJump(index_change) => ReturnInfo::RelIndexChange(*index_change),
            ControlASM::JumpIf(x, _) | ControlASM::RelJumpIf(x, _) => {
                let mut x_value = ctx.evaluate(x);

                // Cast to correct type
                x_value.cast_as_bool(ctx);

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
                x_value.cast_as_bool(ctx);

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
                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                x_value.compatible_cast(&mut y_value, ctx);

                match self {
                    ControlASM::JumpIfDifferent(_, _, _)
                    | ControlASM::RelJumpIfDifferent(_, _, _) => {
                        if x_value.neq(&y_value, ctx).is_true(ctx) {
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
                        if x_value.eq(&y_value, ctx).is_true(ctx) {
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
                        if x_value.lt(&y_value, ctx).is_true(ctx) {
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
                        if x_value.leq(&y_value, ctx).is_true(ctx) {
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

    pub fn volatile_execution(&self, ctx: &mut EvaluationContext) -> ReturnInfo {
        self.execute(ctx, &mut Vec::new(), 0, &mut Vec::new())
    }
}
