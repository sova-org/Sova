use std::{collections::HashMap, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};

use crate::{clock::{Clock, SyncTime}, lang::{control_asm::ControlASM, event::Event, variable::{Variable, VariableStore, VariableValue}, Instruction, Program}};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Script {
    pub content : String,
    pub compiled : Program,
    pub step_vars : Mutex<VariableStore>,
}

pub struct ScriptExecution {
    pub script : Arc<Script>,
    pub instance_vars : VariableStore,
    pub instruction_index : usize,
    pub scheduled_time : SyncTime
}

impl Script {

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    #[inline]
    pub fn is_compiled(&self) -> bool {
        !self.compiled.is_empty()
    }

}

impl From<Program> for Script {
    fn from(compiled: Program) -> Self {
        Script { compiled, ..Default::default() }
    }
}
impl From<String> for Script {
    fn from(content : String) -> Self {
        Script { content, ..Default::default() }
    }
}

impl ScriptExecution {

    pub fn execute_at(script : Arc<Script>, date : SyncTime) -> Self {
        ScriptExecution {
            script,
            instance_vars: HashMap::new(),
            instruction_index: 0,
            scheduled_time: date
        }
    }

    #[inline]
    pub fn stop(&mut self) {
        self.instruction_index = usize::MAX;
    }

    #[inline]
    pub fn has_terminated(&self) -> bool {
        self.instruction_index >= self.script.compiled.len()
    }

    #[inline]
    pub fn is_ready(&self, date : SyncTime) -> bool {
        self.scheduled_time <= date
    }

    #[inline]
    pub fn remaining_before(&self, date : SyncTime) -> SyncTime {
        if date >= self.scheduled_time {
            0
        } else {
            self.scheduled_time - date
        }
    }

    #[inline]
    pub fn current_instruction(&self) -> &Instruction {
        &self.script.compiled[self.instruction_index]
    }

    pub fn execute_next(&mut self, environment_vars : &mut VariableStore, global_vars : &mut VariableStore, sequence_vars : &mut VariableStore, clock : &Clock) -> Option<(Event, SyncTime)> {
        if self.has_terminated() {
            return None;
        }
        let current = &self.script.compiled[self.instruction_index];
        match current {
            Instruction::Control(_) => {
                self.execute_control(environment_vars, global_vars, sequence_vars, clock);
                None
            },
            Instruction::Effect(event, time_span) => {
                self.instruction_index += 1;
                let wait = time_span.as_micros(clock);
                let mut generated = event.clone();
                generated.map_values(environment_vars, global_vars, sequence_vars, &self.script.step_vars.lock().unwrap(), &self.instance_vars);
                let res = (generated, self.scheduled_time);
                self.scheduled_time += wait;
                Some(res)
            },
        }
    }

    fn execute_control(&mut self, environment_vars : &mut VariableStore, global_vars : &mut VariableStore, sequence_vars : &mut VariableStore, clock : &Clock) {
        let Instruction::Control(control) =  &self.script.compiled[self.instruction_index] else {
            return;
        };
        // Less performant than to do everything in one single check, but easier to read and write ?
        let mut step_vars = self.script.step_vars.lock().unwrap();
        let instance_vars = &mut self.instance_vars;
        if !ensure_executability(control, environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars) {
            return;
        }
        self.instruction_index += 1;
        match control {
            // Arithmetic operations
            ControlASM::Add(x, y, z) | ControlASM::Div(x, y, z) | ControlASM::Mod(x, y, z) | ControlASM::Mul(x, y, z) | ControlASM::Sub(x, y, z) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, &step_vars, instance_vars).unwrap();
                let mut y_value = y.evaluate(environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars).unwrap();
                
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
                let res_value = match control {
                    ControlASM::Add(_, _, _) => x_value.add(y_value, clock),
                    ControlASM::Div(_, _, _) => x_value.div(y_value, clock),
                    ControlASM::Mod(_, _, _) => x_value.rem(y_value, clock),
                    ControlASM::Mul(_, _, _) => x_value.mul(y_value, clock),
                    ControlASM::Sub(_, _, _) => x_value.sub(y_value, clock), 
                    _ => unreachable!(),
                };

                z.set(res_value, environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars)

            }
            // Boolean operations (binary)
            ControlASM::And(x, y, z) | ControlASM::Or(x, y, z) | ControlASM::Xor(x, y, z) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, &step_vars, instance_vars).unwrap();
                let mut y_value = y.evaluate(environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars).unwrap();

                // Cast to correct types
                x_value = x_value.cast_as_bool(clock);
                y_value = y_value.cast_as_bool(clock);

                // Compute the result
                let res_value = match control {
                    ControlASM::And(_, _, _) => x_value.and(y_value),
                    ControlASM::Or(_, _, _) => x_value.or(y_value),
                    ControlASM::Xor(_, _, _) => x_value.xor(y_value),
                    _ => unreachable!(),
                };

                z.set(res_value, environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars);
            }
            // Boolean operations (unary)
            ControlASM::Not(x, z) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, &step_vars, instance_vars).unwrap();

                // Cast to correct type
                x_value = x_value.cast_as_bool(clock);

                // Compute the result
                let res_value = !x_value;

                z.set(res_value, environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars);
            },
            // Bitwise operations (binary)
            ControlASM::BitAnd(x, y, z) | ControlASM::BitOr(x, y, z) | ControlASM::BitXor(x, y, z) | ControlASM::ShiftLeft(x, y, z) | ControlASM::ShiftRightA(x, y, z) | ControlASM::ShiftRightL(x, y, z) => {

                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, &step_vars, instance_vars).unwrap();
                let mut y_value = y.evaluate(environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars).unwrap();

                // Cast to correct types
                x_value = x_value.cast_as_integer(clock);
                y_value = y_value.cast_as_integer(clock);

                // Compute the result
                let res_value = match control {
                    ControlASM::BitAnd(_, _, _) => x_value & y_value,
                    ControlASM::BitOr(_, _, _) => x_value | y_value,
                    ControlASM::BitXor(_, _, _) => x_value ^ y_value,
                    ControlASM::ShiftLeft(_, _, _) => x_value << y_value,
                    ControlASM::ShiftRightA(_, _, _) => x_value >> y_value,
                    ControlASM::ShiftRightL(_, _, _) => x_value.logical_shift(y_value),
                    _ => unreachable!(),
                };

                z.set(res_value, environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars);
            }
            // Bitwise operations (unary)
            ControlASM::BitNot(x, z) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, &step_vars, instance_vars).unwrap();

                // Cast to correct type
                x_value = x_value.cast_as_integer(clock);

                // Compute the result
                let res_value = !x_value;

                z.set(res_value, environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars);
            },
            // Memory manipulation
            ControlASM::Mov(x, z) => {
                let x_value = x.evaluate(environment_vars, global_vars, sequence_vars, &step_vars, instance_vars).unwrap();
                z.set(x_value, environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars);
            },
            // Jumps
            ControlASM::Jump(index) => self.instruction_index = *index,
            ControlASM::JumpIf(x, index) => {
                let mut x_value = x.evaluate(environment_vars, global_vars, sequence_vars, &step_vars, instance_vars).unwrap();

                // Cast to correct type
                x_value = x_value.cast_as_bool(clock);

                if x_value.is_true(clock) {
                    self.instruction_index = *index;
                }
            },
            ControlASM::JumpIfDifferent(x, y, index) | ControlASM::JumpIfEqual(x, y, index) | ControlASM::JumpIfLess(x, y, index) | ControlASM::JumpIfLessOrEqual(x, y, index) => {
                let x_value = x.evaluate(environment_vars, global_vars, sequence_vars, &step_vars, instance_vars).unwrap();
                let mut y_value = y.evaluate(environment_vars, global_vars, sequence_vars, &step_vars, instance_vars).unwrap();

                match x_value {
                    VariableValue::Integer(_) => y_value = y_value.cast_as_integer(clock),
                    VariableValue::Bool(_) => y_value = y_value.cast_as_bool(clock),
                    VariableValue::Float(_) => y_value = y_value.cast_as_float(clock),
                    VariableValue::Str(_) => y_value = y_value.cast_as_str(clock),
                    VariableValue::Dur(_) => y_value = y_value.cast_as_dur(),
                }

                match control {
                    ControlASM::JumpIfDifferent(_, _, _) => {
                        if x_value != y_value {
                            self.instruction_index = *index;
                        }
                    },
                    ControlASM::JumpIfEqual(_, _, _) => {
                        if x_value == y_value {
                            self.instruction_index = *index;
                        }
                    },
                    ControlASM::JumpIfLess(_, _, _) => {
                        if x_value < y_value {
                            self.instruction_index = *index;
                        }
                    },
                    ControlASM::JumpIfLessOrEqual(_, _, _) => {
                        if x_value <= y_value {
                            self.instruction_index = *index;
                        }
                    },
                    _ => unreachable!(),
                }
            },
            // Calls and returns
            ControlASM::Return => {
                self.instruction_index = usize::MAX
            }
        }
    }

}

fn ensure_executability(
    control : &ControlASM,
    environment_vars : &mut VariableStore,
    global_vars : &mut VariableStore,
    sequence_vars : &mut VariableStore,
    step_vars : &mut VariableStore,
    instance_vars : &mut VariableStore
) -> bool {
    match control {
        ControlASM::Add(x, y, _) | ControlASM::Sub(x, y, _) |
        ControlASM::And(x, y, _) | ControlASM::Or(x, y, _) 
    => {
            Variable::ensure_existing(x, y, environment_vars, global_vars, sequence_vars, step_vars, instance_vars) && x.is_mutable()
        },
        ControlASM::JumpIfLess(x, y, _) => {
            Variable::ensure_existing(x, y, environment_vars, global_vars, sequence_vars, step_vars, instance_vars)
        },
        ControlASM::Mov(_, var) | ControlASM::JumpIf(var, _) | ControlASM::Not(var, _) => {
            var.exists(environment_vars, global_vars, sequence_vars, step_vars, instance_vars)
        },
        _ => true
    }
}
