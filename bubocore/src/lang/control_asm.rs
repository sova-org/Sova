use serde::{Deserialize, Serialize};

use super::{evaluation_context::EvaluationContext, variable::{Variable, VariableValue}, Instruction, Program};

use crate::scene::script::ReturnInfo;
use crate::clock::TimeSpan;

use std::f64::consts::PI;

// Import state keys
use crate::lang::environment_func::{SINE_PHASE_KEY, SINE_LAST_BEAT_KEY, SAW_PHASE_KEY, SAW_LAST_BEAT_KEY, TRI_PHASE_KEY, TRI_LAST_BEAT_KEY, ISAW_PHASE_KEY, ISAW_LAST_BEAT_KEY, RANDSTEP_PHASE_KEY, RANDSTEP_LAST_BEAT_KEY, RANDSTEP_VALUE_KEY};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlASM {
    // Arithmetic operations
    Add(Variable, Variable, Variable),
    Div(Variable, Variable, Variable),
    Mod(Variable, Variable, Variable),
    Mul(Variable, Variable, Variable),
    Sub(Variable, Variable, Variable),
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
    //DeclareGlobale(String, Variable),
    //DeclareInstance(String, Variable),
    //DeclareLine(String, Variable),
    //DeclareFrame(String, Variable),
    Mov(Variable, Variable),    
    // Stack operations
    Push(Variable),
    Pop(Variable),
    // Jumps
    Jump(usize),
    JumpIf(Variable, usize),
    JumpIfNot(Variable, usize),
    JumpIfDifferent(Variable, Variable, usize),
    JumpIfEqual(Variable, Variable, usize),
    JumpIfLess(Variable, Variable, usize),
    JumpIfLessOrEqual(Variable, Variable, usize),
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
}


impl ControlASM {

    pub fn execute(&self, ctx : &mut EvaluationContext, return_stack: &mut Vec<ReturnInfo>, instruction_position: usize, current_prog: &Program) -> ReturnInfo {
        match self {
            // Arithmetic operations
            ControlASM::Add(x, y, z) | ControlASM::Div(x, y, z) | ControlASM::Mod(x, y, z) | ControlASM::Mul(x, y, z) | ControlASM::Sub(x, y, z) => {
                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                // cast to correct types
                match x_value {
                    VariableValue::Integer(_) => {y_value = y_value.cast_as_integer(ctx);},
                    VariableValue::Float(_) => {y_value = y_value.cast_as_float(ctx);},
                    VariableValue::Dur(_) => {y_value = y_value.cast_as_dur();},
                    _ => {
                        match y_value {
                            VariableValue::Integer(_) => {x_value = x_value.cast_as_integer(ctx);},
                            VariableValue::Float(_) => {x_value = x_value.cast_as_float(ctx);},
                            VariableValue::Dur(_) => {x_value = x_value.cast_as_dur();},
                            _ => {
                                x_value = x_value.cast_as_integer(ctx);
                                y_value = x_value.cast_as_integer(ctx);
                            },
                        }
                    }
                }

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
            },
            // Boolean operations (numeric operators)
            ControlASM::LowerThan(x, y, z) | ControlASM::LowerOrEqual(x, y, z) | ControlASM::GreaterThan(x, y, z) | ControlASM::GreaterOrEqual(x, y, z) | ControlASM::Equal(x, y, z) | ControlASM::Different(x, y, z) => {
                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                // cast to correct types
                match x_value {
                    VariableValue::Integer(_) => {y_value = y_value.cast_as_integer(ctx);},
                    VariableValue::Float(_) => {y_value = y_value.cast_as_float(ctx);},
                    VariableValue::Dur(_) => {y_value = y_value.cast_as_dur();},
                    _ => {
                        match y_value {
                            VariableValue::Integer(_) => {x_value = x_value.cast_as_integer(ctx);},
                            VariableValue::Float(_) => {x_value = x_value.cast_as_float(ctx);},
                            VariableValue::Dur(_) => {x_value = x_value.cast_as_dur();},
                            _ => {
                                x_value = x_value.cast_as_integer(ctx);
                                y_value = x_value.cast_as_integer(ctx);
                            },
                        }
                    }
                }

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
            ControlASM::BitAnd(x, y, z) | ControlASM::BitOr(x, y, z) | ControlASM::BitXor(x, y, z) | ControlASM::ShiftLeft(x, y, z) | ControlASM::ShiftRightA(x, y, z) | ControlASM::ShiftRightL(x, y, z) => {

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
            },
            // Time manipulation
            ControlASM::FloatAsBeats(x, z) => {
                let x_value = ctx.evaluate(x);
                let x_value = x_value.cast_as_float(ctx);
                let res_value = VariableValue::Dur(TimeSpan::Beats(x_value.as_float(ctx)));
                ctx.set_var(z, res_value);
                ReturnInfo::None
            }
            ControlASM::FloatAsFrames(val_var, dest_var) => {
                let val_float = ctx.evaluate(val_var).as_float(ctx);
                ctx.set_var(dest_var, VariableValue::Dur(TimeSpan::Frames(val_float)));
                ReturnInfo::None
            },
            // Memory manipulation
            ControlASM::Mov(x, z) => {
                let x_value = ctx.evaluate(x);
                ctx.set_var(z, x_value);
                ReturnInfo::None
            },
            ControlASM::Push(x) => {
                let value = ctx.evaluate(x);
                ctx.stack.push(value);
                ReturnInfo::None
            },
            ControlASM::Pop(x) => {
                let value = ctx.stack.pop().unwrap_or(false.into());
                ctx.set_var(x, value);
                ReturnInfo::None
            },
            // Jumps
            ControlASM::Jump(index) => ReturnInfo::IndexChange(*index),
            ControlASM::JumpIf(x, index) => {
                let mut x_value = ctx.evaluate(x);

                // Cast to correct type
                x_value = x_value.cast_as_bool(ctx);

                if x_value.is_true(ctx) {
                    return ReturnInfo::IndexChange(*index)
                }

                ReturnInfo::None
            },            
            ControlASM::JumpIfNot(x, index) => {
                let mut x_value = ctx.evaluate(x);

                // Cast to correct type
                x_value = x_value.cast_as_bool(ctx);

                if !x_value.is_true(ctx) {
                    return ReturnInfo::IndexChange(*index)
                }

                ReturnInfo::None
            },
            ControlASM::JumpIfDifferent(x, y, index) | ControlASM::JumpIfEqual(x, y, index) | ControlASM::JumpIfLess(x, y, index) | ControlASM::JumpIfLessOrEqual(x, y, index) => {
                let x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                match x_value {
                    VariableValue::Integer(_) => y_value = y_value.cast_as_integer(ctx),
                    VariableValue::Bool(_) => y_value = y_value.cast_as_bool(ctx),
                    VariableValue::Float(_) => y_value = y_value.cast_as_float(ctx),
                    VariableValue::Str(_) => y_value = y_value.cast_as_str(ctx),
                    VariableValue::Dur(_) => y_value = y_value.cast_as_dur(),
                    VariableValue::Func(_) => todo!(),
                }

                match self {
                    ControlASM::JumpIfDifferent(_, _, _) => {
                        if x_value != y_value {
                            return ReturnInfo::IndexChange(*index)
                        }
                    },
                    ControlASM::JumpIfEqual(_, _, _) => {
                        if x_value == y_value {
                            return ReturnInfo::IndexChange(*index)
                        }
                    },
                    ControlASM::JumpIfLess(_, _, _) => {
                        if x_value < y_value {
                            return ReturnInfo::IndexChange(*index)
                        }
                    },
                    ControlASM::JumpIfLessOrEqual(_, _, _) => {
                        if x_value <= y_value {
                            return ReturnInfo::IndexChange(*index)
                        }
                    },
                    _ => unreachable!(),
                }

                ReturnInfo::None
            },
            // Calls and returns
            ControlASM::CallFunction(f) => {
                return_stack.push(ReturnInfo::ProgChange(instruction_position + 1, current_prog.clone()));

                let f_value = ctx.evaluate(f);
                let next_prog = match f_value {
                    VariableValue::Func(p) => p,
                    _ => vec![Instruction::Control(ControlASM::Return)],
                };
                ReturnInfo::ProgChange(0, next_prog)
            },
            ControlASM::CallProcedure(proc_position) => {
                return_stack.push(ReturnInfo::IndexChange(instruction_position + 1));
                ReturnInfo::IndexChange(*proc_position)
            },
            ControlASM::Return => {
                match return_stack.pop() {
                    Some(return_info) => return_info,
                    None => ReturnInfo::IndexChange(usize::MAX),
                }
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
                let clamped_result = result.max(new_min_f.min(new_max_f)).min(new_min_f.max(new_max_f));

                ctx.set_var(dest, VariableValue::Float(clamped_result));
                ReturnInfo::None
            },
            // Clamp implementation remains the same
            ControlASM::Clamp(val, min, max, dest) => {
                let val_f = ctx.evaluate(val).as_float(ctx);
                let min_f = ctx.evaluate(min).as_float(ctx);
                let max_f = ctx.evaluate(max).as_float(ctx);

                let clamped_value = val_f.max(min_f).min(max_f);
                ctx.set_var(dest, VariableValue::Float(clamped_value));
                ReturnInfo::None
            },
            ControlASM::Min(v1, v2, dest) => {
                let v1_f = ctx.evaluate(v1).as_float(ctx);
                let v2_f = ctx.evaluate(v2).as_float(ctx);
                ctx.set_var(dest, VariableValue::Float(v1_f.min(v2_f)));
                ReturnInfo::None
            },
            ControlASM::Max(v1, v2, dest) => {
                let v1_f = ctx.evaluate(v1).as_float(ctx);
                let v2_f = ctx.evaluate(v2).as_float(ctx);
                ctx.set_var(dest, VariableValue::Float(v1_f.max(v2_f)));
                ReturnInfo::None
            },
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
            },
            // Stateful Oscillators using Line Variables and Beat Delta
            ControlASM::GetSine(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat(); 
                
                let last_beat = ctx.line().vars
                    .get(SINE_LAST_BEAT_KEY) 
                    .map_or(current_beat, |v| v.as_float(ctx)); 
                let current_phase = ctx.line().vars
                    .get(SINE_PHASE_KEY)
                    .map_or(0.0, |v| v.as_float(ctx));

                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();

                ctx.line_mut().vars.insert(SINE_PHASE_KEY.to_string(), VariableValue::Float(new_phase));
                ctx.line_mut().vars.insert(SINE_LAST_BEAT_KEY.to_string(), VariableValue::Float(current_beat)); 

                let raw_value = (new_phase * 2.0 * PI).sin(); // Raw value [-1, 1]
                let normalized_value = (raw_value + 1.0) / 2.0; // Normalize to [0, 1]
                let scaled_to_midi = 1.0 + normalized_value * 126.0; // Scale to [1, 127]
                let midi_int = scaled_to_midi.round().max(1.0).min(127.0) as i64;
                ctx.set_var(dest_var, VariableValue::Integer(midi_int)); 
                ReturnInfo::None
            },
            ControlASM::GetSaw(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat();
                let last_beat = ctx.line().vars.get(SAW_LAST_BEAT_KEY).map_or(current_beat, |v| v.as_float(ctx));
                let current_phase = ctx.line().vars.get(SAW_PHASE_KEY).map_or(0.0, |v| v.as_float(ctx));
                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();
                ctx.line_mut().vars.insert(SAW_PHASE_KEY.to_string(), VariableValue::Float(new_phase));
                ctx.line_mut().vars.insert(SAW_LAST_BEAT_KEY.to_string(), VariableValue::Float(current_beat));
                let raw_value = new_phase * 2.0 - 1.0; // Raw value [-1, 1]
                let normalized_value = (raw_value + 1.0) / 2.0; // Normalize to [0, 1]
                let scaled_to_midi = 1.0 + normalized_value * 126.0; // Scale to [1, 127]
                let midi_int = scaled_to_midi.round().max(1.0).min(127.0) as i64;
                ctx.set_var(dest_var, VariableValue::Integer(midi_int));
                ReturnInfo::None
            },
            ControlASM::GetTriangle(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat();
                let last_beat = ctx.line().vars.get(TRI_LAST_BEAT_KEY).map_or(current_beat, |v| v.as_float(ctx));
                let current_phase = ctx.line().vars.get(TRI_PHASE_KEY).map_or(0.0, |v| v.as_float(ctx));
                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();
                ctx.line_mut().vars.insert(TRI_PHASE_KEY.to_string(), VariableValue::Float(new_phase));
                ctx.line_mut().vars.insert(TRI_LAST_BEAT_KEY.to_string(), VariableValue::Float(current_beat));
                let raw_value = 1.0 - (new_phase * 2.0 - 1.0).abs() * 2.0; // Raw value [-1, 1]
                let normalized_value = (raw_value + 1.0) / 2.0; // Normalize to [0, 1]
                let scaled_to_midi = 1.0 + normalized_value * 126.0; // Scale to [1, 127]
                let midi_int = scaled_to_midi.round().max(1.0).min(127.0) as i64;
                ctx.set_var(dest_var, VariableValue::Integer(midi_int));
                ReturnInfo::None
            },
            ControlASM::GetISaw(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat();
                let last_beat = ctx.line().vars.get(ISAW_LAST_BEAT_KEY).map_or(current_beat, |v| v.as_float(ctx));
                let current_phase = ctx.line().vars.get(ISAW_PHASE_KEY).map_or(0.0, |v| v.as_float(ctx));
                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();
                ctx.line_mut().vars.insert(ISAW_PHASE_KEY.to_string(), VariableValue::Float(new_phase));
                ctx.line_mut().vars.insert(ISAW_LAST_BEAT_KEY.to_string(), VariableValue::Float(current_beat));
                let raw_value = 1.0 - (new_phase * 2.0); // Raw value [1, -1] (inverted saw)
                let normalized_value = (raw_value + 1.0) / 2.0; // Normalize to [0, 1]
                let scaled_to_midi = 1.0 + normalized_value * 126.0; // Scale to [1, 127]
                let midi_int = scaled_to_midi.round().max(1.0).min(127.0) as i64;
                ctx.set_var(dest_var, VariableValue::Integer(midi_int));
                ReturnInfo::None
            },
            ControlASM::GetRandStep(speed_var, dest_var) => {
                let speed_factor = ctx.evaluate(speed_var).as_float(ctx);
                let current_beat = ctx.clock.beat();
                let last_beat = ctx.line().vars.get(RANDSTEP_LAST_BEAT_KEY).map_or(current_beat, |v| v.as_float(ctx));
                let current_phase = ctx.line().vars.get(RANDSTEP_PHASE_KEY).map_or(0.0, |v| v.as_float(ctx));
                
                let delta_beats = current_beat - last_beat;
                let phase_increment = delta_beats * speed_factor;
                let new_phase = (current_phase + phase_increment).fract();

                let mut current_value = ctx.line().vars
                    .get(RANDSTEP_VALUE_KEY)
                    .map_or(0, |v| v.as_integer(ctx)); // Default to 0 if not set

                // Check if phase wrapped around (cycle completed) or if it's the first run
                if new_phase < current_phase || current_value == 0 {
                    current_value = (rand::random::<u8>() % 127) as i64 + 1; // Generate new random value [1, 127]
                    ctx.line_mut().vars.insert(RANDSTEP_VALUE_KEY.to_string(), VariableValue::Integer(current_value)); // Store it
                } 

                ctx.line_mut().vars.insert(RANDSTEP_PHASE_KEY.to_string(), VariableValue::Float(new_phase));
                ctx.line_mut().vars.insert(RANDSTEP_LAST_BEAT_KEY.to_string(), VariableValue::Float(current_beat));

                ctx.set_var(dest_var, VariableValue::Integer(current_value)); // Return the current held value
                ReturnInfo::None
            },
        }
    }

}
