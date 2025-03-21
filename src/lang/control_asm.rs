use serde::{Deserialize, Serialize};

use super::variable::{Variable, VariableValue, VariableStore};

use crate::{clock::Clock};

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
    // Calls and returns
    // CallFunction(Variable),
    CallProcedure(usize),
    Return, // Only exit at the moment
}


impl ControlASM {

    pub fn execute(&self, environment_vars: &mut VariableStore, global_vars: &mut VariableStore, sequence_vars: &mut VariableStore, step_vars: &mut VariableStore, instance_vars: &mut VariableStore, clock: &Clock, return_stack: &mut Vec<usize>, instruction_position: usize) -> Option<usize> {
        match self {
            // Arithmetic operations
            ControlASM::Add(x, y, z) | ControlASM::Div(x, y, z) | ControlASM::Mod(x, y, z) | ControlASM::Mul(x, y, z) | ControlASM::Sub(x, y, z) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                let mut y_value = y.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                
                // cast to correct types
                match x_value {
                    VariableValue::Integer(_) => {y_value = y_value.cast_as_integer(clock);},
                    VariableValue::Float(_) => {y_value = y_value.cast_as_float(clock);},
                    VariableValue::Dur(_) => {y_value = y_value.cast_as_dur();},
                    _ => {
                        match y_value {
                            VariableValue::Integer(_) => {x_value = x_value.cast_as_integer(clock);},
                            VariableValue::Float(_) => {x_value = x_value.cast_as_float(clock);},
                            VariableValue::Dur(_) => {x_value = x_value.cast_as_dur();},
                            _ => {
                                x_value = x_value.cast_as_integer(clock);
                                y_value = x_value.cast_as_integer(clock);
                            },
                        }
                    }
                }

                // compute the result
                let res_value = match self {
                    ControlASM::Add(_, _, _) => x_value.add(y_value, clock),
                    ControlASM::Div(_, _, _) => x_value.div(y_value, clock),
                    ControlASM::Mod(_, _, _) => x_value.rem(y_value, clock),
                    ControlASM::Mul(_, _, _) => x_value.mul(y_value, clock),
                    ControlASM::Sub(_, _, _) => x_value.sub(y_value, clock), 
                    _ => unreachable!(),
                };

                z.set(res_value, environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                
                None
            }
            // Boolean operations (binary)
            ControlASM::And(x, y, z) | ControlASM::Or(x, y, z) | ControlASM::Xor(x, y, z) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                let mut y_value = y.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                // Cast to correct types
                x_value = x_value.cast_as_bool(clock);
                y_value = y_value.cast_as_bool(clock);

                // Compute the result
                let res_value = match self {
                    ControlASM::And(_, _, _) => x_value.and(y_value),
                    ControlASM::Or(_, _, _) => x_value.or(y_value),
                    ControlASM::Xor(_, _, _) => x_value.xor(y_value),
                    _ => unreachable!(),
                };

                z.set(res_value, environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                None
            }
            // Boolean operations (unary)
            ControlASM::Not(x, z) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                // Cast to correct type
                x_value = x_value.cast_as_bool(clock);

                // Compute the result
                let res_value = !x_value;

                z.set(res_value, environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                None
            },
            // Bitwise operations (binary)
            ControlASM::BitAnd(x, y, z) | ControlASM::BitOr(x, y, z) | ControlASM::BitXor(x, y, z) | ControlASM::ShiftLeft(x, y, z) | ControlASM::ShiftRightA(x, y, z) | ControlASM::ShiftRightL(x, y, z) => {

                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                let mut y_value = y.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                // Cast to correct types
                x_value = x_value.cast_as_integer(clock);
                y_value = y_value.cast_as_integer(clock);

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

                z.set(res_value, environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                None
            }
            // Bitwise operations (unary)
            ControlASM::BitNot(x, z) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                // Cast to correct type
                x_value = x_value.cast_as_integer(clock);

                // Compute the result
                let res_value = !x_value;

                z.set(res_value, environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                None
            },
            // Memory manipulation
            ControlASM::Mov(x, z) => {
                print!("AVAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANT");
                let x_value = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                print!("PENDAAAAAAAAAAAAAAAAAAAAAAAAAAAAANT {:?}", x_value);
                z.set(x_value, environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                print!("APRÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈÈS");

                None
            },
            // Jumps
            ControlASM::Jump(index) => Some(*index),
            ControlASM::JumpIf(x, index) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                // Cast to correct type
                x_value = x_value.cast_as_bool(clock);

                if x_value.is_true(clock) {
                    return Some(*index)
                }

                None
            },
            ControlASM::JumpIfDifferent(x, y, index) | ControlASM::JumpIfEqual(x, y, index) | ControlASM::JumpIfLess(x, y, index) | ControlASM::JumpIfLessOrEqual(x, y, index) => {
                let x_value = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                let mut y_value = y.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);

                match x_value {
                    VariableValue::Integer(_) => y_value = y_value.cast_as_integer(clock),
                    VariableValue::Bool(_) => y_value = y_value.cast_as_bool(clock),
                    VariableValue::Float(_) => y_value = y_value.cast_as_float(clock),
                    VariableValue::Str(_) => y_value = y_value.cast_as_str(clock),
                    VariableValue::Dur(_) => y_value = y_value.cast_as_dur(),
                }

                match self {
                    ControlASM::JumpIfDifferent(_, _, _) => {
                        if x_value != y_value {
                            return Some(*index)
                        }
                    },
                    ControlASM::JumpIfEqual(_, _, _) => {
                        if x_value == y_value {
                            return Some(*index)
                        }
                    },
                    ControlASM::JumpIfLess(_, _, _) => {
                        if x_value < y_value {
                            return Some(*index)
                        }
                    },
                    ControlASM::JumpIfLessOrEqual(_, _, _) => {
                        if x_value <= y_value {
                            return Some(*index)
                        }
                    },
                    _ => unreachable!(),
                }

                None
            },
            // Calls and returns
            ControlASM::CallProcedure(proc_position) => {
                return_stack.push(instruction_position + 1);
                Some(*proc_position)
            },
            ControlASM::Return => {
                match return_stack.pop() {
                    Some(return_position) => Some(return_position),
                    None => Some(usize::MAX),
                }
            },
        }
    }

} 
