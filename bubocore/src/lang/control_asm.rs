use serde::{Deserialize, Serialize};

use super::{evaluation_context::EvaluationContext, variable::{Variable, VariableValue}, Instruction, Program};

use crate::pattern::script::ReturnInfo;

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
    // AsBeats(Variable, Variable),
    // AsMicros(Variable, Variable),
    // AsSteps(Variable, Variable),
    // Memory manipulation
    //DeclareGlobale(String, Variable),
    //DeclareInstance(String, Variable),
    //DeclareSequence(String, Variable),
    //DeclareStep(String, Variable),
    Mov(Variable, Variable),
    // Jumps
    Jump(usize),
    JumpIf(Variable, usize),
    JumpIfDifferent(Variable, Variable, usize),
    JumpIfEqual(Variable, Variable, usize),
    JumpIfLess(Variable, Variable, usize),
    JumpIfLessOrEqual(Variable, Variable, usize),
    // Stack operations
    Push(Variable),
    Pop(Variable),
    // Calls and returns
    CallFunction(Variable),
    CallProcedure(usize),
    Return, // Only exit at the moment
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
                    VariableValue::Integer(_) => {y_value = y_value.cast_as_integer(ctx.clock);},
                    VariableValue::Float(_) => {y_value = y_value.cast_as_float(ctx.clock);},
                    VariableValue::Dur(_) => {y_value = y_value.cast_as_dur();},
                    _ => {
                        match y_value {
                            VariableValue::Integer(_) => {x_value = x_value.cast_as_integer(ctx.clock);},
                            VariableValue::Float(_) => {x_value = x_value.cast_as_float(ctx.clock);},
                            VariableValue::Dur(_) => {x_value = x_value.cast_as_dur();},
                            _ => {
                                x_value = x_value.cast_as_integer(ctx.clock);
                                y_value = x_value.cast_as_integer(ctx.clock);
                            },
                        }
                    }
                }

                // compute the result
                let res_value = match self {
                    ControlASM::Add(_, _, _) => x_value.add(y_value, ctx.clock),
                    ControlASM::Div(_, _, _) => x_value.div(y_value, ctx.clock),
                    ControlASM::Mod(_, _, _) => x_value.rem(y_value, ctx.clock),
                    ControlASM::Mul(_, _, _) => x_value.mul(y_value, ctx.clock),
                    ControlASM::Sub(_, _, _) => x_value.sub(y_value, ctx.clock),
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
                x_value = x_value.cast_as_bool(ctx.clock);
                y_value = y_value.cast_as_bool(ctx.clock);

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
                x_value = x_value.cast_as_bool(ctx.clock);

                // Compute the result
                let res_value = !x_value;

                ctx.set_var(z, res_value);

                ReturnInfo::None
            },
            // Bitwise operations (binary)
            ControlASM::BitAnd(x, y, z) | ControlASM::BitOr(x, y, z) | ControlASM::BitXor(x, y, z) | ControlASM::ShiftLeft(x, y, z) | ControlASM::ShiftRightA(x, y, z) | ControlASM::ShiftRightL(x, y, z) => {

                let mut x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                // Cast to correct types
                x_value = x_value.cast_as_integer(ctx.clock);
                y_value = y_value.cast_as_integer(ctx.clock);

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
                x_value = x_value.cast_as_integer(ctx.clock);

                // Compute the result
                let res_value = !x_value;

                ctx.set_var(z, res_value);

                ReturnInfo::None
            },
            // Memory manipulation
            ControlASM::Mov(x, z) => {
                let x_value = ctx.evaluate(x);
                ctx.set_var(z, x_value);
                ReturnInfo::None
            },
            // Jumps
            ControlASM::Jump(index) => ReturnInfo::IndexChange(*index),
            ControlASM::JumpIf(x, index) => {
                let mut x_value = ctx.evaluate(x);

                // Cast to correct type
                x_value = x_value.cast_as_bool(ctx.clock);

                if x_value.is_true(ctx.clock) {
                    return ReturnInfo::IndexChange(*index)
                }

                ReturnInfo::None
            },
            ControlASM::JumpIfDifferent(x, y, index) | ControlASM::JumpIfEqual(x, y, index) | ControlASM::JumpIfLess(x, y, index) | ControlASM::JumpIfLessOrEqual(x, y, index) => {
                let x_value = ctx.evaluate(x);
                let mut y_value = ctx.evaluate(y);

                match x_value {
                    VariableValue::Integer(_) => y_value = y_value.cast_as_integer(ctx.clock),
                    VariableValue::Bool(_) => y_value = y_value.cast_as_bool(ctx.clock),
                    VariableValue::Float(_) => y_value = y_value.cast_as_float(ctx.clock),
                    VariableValue::Str(_) => y_value = y_value.cast_as_str(ctx.clock),
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
        }
    }

}
