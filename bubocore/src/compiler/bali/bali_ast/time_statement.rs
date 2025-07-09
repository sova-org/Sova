use std::cmp::Ordering;
use crate::compiler::bali::bali_ast::{
    concrete_fraction::ConcreteFraction,
    toplevel_effect::TopLevelEffect,
    bali_context::BaliContext,
    LocalChoiceVariableGenerator,
    AltVariableGenerator,
    constants::LOCAL_TARGET_VAR,
    information::Information,
    function::FunctionContent,
};
use crate::lang::{
    Instruction,
    control_asm::ControlASM,
    variable::Variable,
};

use std::collections::HashMap;

#[derive(Debug)]
pub enum TimeStatement {
    At(
        ConcreteFraction,
        TopLevelEffect,
        BaliContext,
        Vec<Information>,
    ),
    JustBefore(
        ConcreteFraction,
        TopLevelEffect,
        BaliContext,
        Vec<Information>,
    ),
    JustAfter(
        ConcreteFraction,
        TopLevelEffect,
        BaliContext,
        Vec<Information>,
    ),
}

impl TimeStatement {
    pub fn get_time_as_f64(&self) -> f64 {
        match self {
            TimeStatement::At(x, _, _, _)
            | TimeStatement::JustBefore(x, _, _, _)
            | TimeStatement::JustAfter(x, _, _, _) => x.tof64(),
        }
    }

    pub fn get_time(&self) -> ConcreteFraction {
        match self {
            TimeStatement::At(x, _, _, _)
            | TimeStatement::JustBefore(x, _, _, _)
            | TimeStatement::JustAfter(x, _, _, _) => x.clone(),
        }
    }

    pub fn as_asm(
        &self,
        local_choice_vars: &mut LocalChoiceVariableGenerator,
        set_pick_variables: &mut Vec<bool>,
        local_alt_vars: &mut AltVariableGenerator,
        set_alt_variables: &mut Vec<bool>,
        functions: &HashMap<String, FunctionContent>,
    ) -> Vec<Instruction> {
        match self {
            TimeStatement::At(t, x, context, infos)
            | TimeStatement::JustBefore(t, x, context, infos)
            | TimeStatement::JustAfter(t, x, context, infos) => {
                if infos.len() == 0 {
                    return x.as_asm(context.clone(), local_choice_vars, local_alt_vars, &functions);
                }

                // handle choices (? ...), picks (pick ...), and alt (alt ...)
                let mut infos = infos.clone();
                let current_info = infos.pop();
                let current_info = current_info.unwrap();
                //print!("ONE POP: {:?}\n", current_info);

                match current_info {
                    Information::Choice(current_choice) => {
                        let mut res = Vec::new();

                        res.push(Instruction::Control(ControlASM::Mov(
                            (current_choice.position as i64).into(),
                            LOCAL_TARGET_VAR.clone(),
                        )));

                        // handle choice structure
                        let num_instruction_for_first_choice = 1;
                        let num_instruction_for_other_choices =
                            if current_choice.position == 0 { 1 } else { 3 };
                        let num_instruction_between_choices_and_effects = 1;
                        let mut distance_to_prog = num_instruction_for_first_choice
                            + num_instruction_for_other_choices
                                * (current_choice.variables.len() - 1)
                            + num_instruction_between_choices_and_effects;

                        for choice_step in 0..current_choice.variables.len() {
                            distance_to_prog = if choice_step == 0 {
                                distance_to_prog - num_instruction_for_first_choice
                            } else {
                                distance_to_prog - num_instruction_for_other_choices
                            };

                            if choice_step > 0 && current_choice.position > 0 {
                                res.push(Instruction::Control(ControlASM::RelJumpIfLessOrEqual(
                                    LOCAL_TARGET_VAR.clone(),
                                    current_choice.variables[choice_step as usize - 1].clone(),
                                    2,
                                )));
                                res.push(Instruction::Control(ControlASM::Sub(
                                    LOCAL_TARGET_VAR.clone(),
                                    1.into(),
                                    LOCAL_TARGET_VAR.clone(),
                                )));
                            }

                            res.push(Instruction::Control(ControlASM::RelJumpIfEqual(
                                LOCAL_TARGET_VAR.clone(),
                                current_choice.variables[choice_step].clone(),
                                (distance_to_prog + 1) as i64,
                            )));
                        }

                        // jump after prog if choice is not successful
                        let prog = TimeStatement::At(t.clone(), x.clone(), context.clone(), infos)
                            .as_asm(
                                local_choice_vars,
                                set_pick_variables,
                                local_alt_vars,
                                set_alt_variables,
                                &functions,
                            );
                        res.push(Instruction::Control(ControlASM::RelJump(
                            (prog.len() + 1) as i64,
                        )));

                        res.extend(prog);

                        //print!("END CHOICE\n");

                        res
                    }
                    Information::Pick(current_pick) => {
                        let mut res = Vec::new();

                        // if this is the first element (in time) of this pick, evaluate the pick expression and store the result
                        // in the pick variable
                        if !set_pick_variables[current_pick.num_variable as usize] {
                            res.extend(current_pick.expression.as_asm(&functions));
                            res.push(Instruction::Control(ControlASM::Pop(
                                current_pick.variable.clone(),
                            )));
                            res.push(Instruction::Control(ControlASM::Add(
                                current_pick.variable.clone(),
                                (current_pick.possibilities as i64).into(),
                                current_pick.variable.clone(),
                            )));
                            res.push(Instruction::Control(ControlASM::Sub(
                                current_pick.variable.clone(),
                                1.into(),
                                current_pick.variable.clone(),
                            )));
                            res.push(Instruction::Control(ControlASM::Mod(
                                current_pick.variable.clone(),
                                (current_pick.possibilities as i64).into(),
                                current_pick.variable.clone(),
                            )));
                            set_pick_variables[current_pick.num_variable as usize] = true;
                        }

                        // in any case, add the conditional structure for the pick
                        res.push(Instruction::Control(ControlASM::RelJumpIfEqual(
                            current_pick.variable.clone(),
                            (current_pick.position as i64).into(),
                            2,
                        )));

                        // jump over effects if the pick is not successful
                        let prog = TimeStatement::At(t.clone(), x.clone(), context.clone(), infos)
                            .as_asm(
                                local_choice_vars,
                                set_pick_variables,
                                local_alt_vars,
                                set_alt_variables,
                                &functions,
                            );
                        let num_prog_instruction = prog.len();
                        res.push(Instruction::Control(ControlASM::RelJump(
                            (num_prog_instruction + 1) as i64,
                        )));

                        // add all of this to the previously constructed program
                        res.extend(prog);

                        //print!("END PICK\n");

                        res
                    }
                    Information::Alt(current_alt) => {
                        let mut res = Vec::new();

                        // if this is the first element (in time) of this alt, get the
                        // value of the frame variable, then increase it by one
                        if !set_alt_variables[current_alt.num_variable as usize] {
                            res.push(Instruction::Control(ControlASM::Mov(
                                current_alt.frame_variable.clone(),
                                current_alt.instance_variable.clone(),
                            )));
                            res.push(Instruction::Control(ControlASM::Add(
                                current_alt.frame_variable.clone(),
                                1.into(),
                                current_alt.frame_variable.clone(),
                            ))); // there is a race condition here, it could be solved by implementing the Atomic instruction in ControlASM
                            res.push(Instruction::Control(ControlASM::Mod(
                                current_alt.instance_variable.clone(),
                                (current_alt.possibilities as i64).into(),
                                current_alt.instance_variable.clone(),
                            )));
                            set_alt_variables[current_alt.num_variable as usize] = true;
                        }

                        // in any case, add the conditional structure for the alt
                        res.push(Instruction::Control(ControlASM::RelJumpIfEqual(
                            current_alt.instance_variable.clone(),
                            (current_alt.position as i64).into(),
                            2,
                        )));

                        // jump over the effects if the alt is not selected
                        let prog = TimeStatement::At(t.clone(), x.clone(), context.clone(), infos)
                            .as_asm(
                                local_choice_vars,
                                set_pick_variables,
                                local_alt_vars,
                                set_alt_variables,
                                &functions,
                            );
                        let num_prog_instruction = prog.len();
                        res.push(Instruction::Control(ControlASM::RelJump(
                            (num_prog_instruction + 1) as i64,
                        )));

                        res.extend(prog);

                        //print!("END ALT\n");

                        res
                    }
                    Information::Ramp(current_ramp) => {
                        let mut res = Vec::new();

                        // set the ramp variable
                        res.push(Instruction::Control(ControlASM::Mov(
                            current_ramp.variable_value.into(),
                            Variable::Instance(current_ramp.variable_name.into()),
                        )));

                        // add the program
                        res.extend(
                            TimeStatement::At(t.clone(), x.clone(), context.clone(), infos).as_asm(
                                local_choice_vars,
                                set_pick_variables,
                                local_alt_vars,
                                set_alt_variables,
                                &functions,
                            ),
                        );

                        res
                    }
                }

                /*
                if choices.len() > 0 {
                    let mut choices = choices.clone();
                    let current_choice = choices.pop();
                    let current_choice = current_choice.unwrap();

                    let mut res = Vec::new();

                    res.push(Instruction::Control(ControlASM::Mov((current_choice.position as i64).into(), LOCAL_TARGET_VAR.clone())));

                    // handle choice structure
                    let num_instruction_for_first_choice = 1;
                    let num_instruction_for_other_choices = if current_choice.position == 0 {
                        1
                    } else {
                        3
                    };
                    let num_instruction_between_choices_and_effects = 1;
                    let mut distance_to_prog = num_instruction_for_first_choice + num_instruction_for_other_choices * (current_choice.variables.len() - 1) + num_instruction_between_choices_and_effects;

                    for choice_step in 0..current_choice.variables.len() {

                        distance_to_prog = if choice_step == 0 {
                            distance_to_prog - num_instruction_for_first_choice
                        } else {
                            distance_to_prog - num_instruction_for_other_choices
                        };

                        if choice_step > 0 && current_choice.position > 0 {
                            res.push(Instruction::Control(ControlASM::RelJumpIfLessOrEqual(LOCAL_TARGET_VAR.clone(), current_choice.variables[choice_step as usize -1].clone(), 2)));
                            res.push(Instruction::Control(ControlASM::Sub(LOCAL_TARGET_VAR.clone(), 1.into(), LOCAL_TARGET_VAR.clone())));
                        }

                        res.push(Instruction::Control(ControlASM::RelJumpIfEqual(LOCAL_TARGET_VAR.clone(), current_choice.variables[choice_step].clone(), (distance_to_prog + 1) as i64)));
                    }

                    // jump after prog if choice is not successful
                    let prog = TimeStatement::At(t.clone(), x.clone(), context.clone(), choices, picks.to_vec(), alts.to_vec()).as_asm(local_choice_vars, set_pick_variables, local_alt_vars, set_alt_variables);
                    res.push(Instruction::Control(ControlASM::RelJump((prog.len() + 1) as i64)));

                    res.extend(prog);

                    return res;
                }
                */

                /*
                // handle picks (pick ...)
                // here there is no choice to handle
                let mut picks = picks.clone();
                let current_pick = picks.pop();
                let current_pick = current_pick.unwrap();

                let mut res = Vec::new();

                // if this is the first element (in time) of this pick, evaluate the pick expression and store the result
                // in the pick variable
                if !set_pick_variables[current_pick.num_variable as usize] {
                    res.extend(current_pick.expression.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(current_pick.variable.clone())));
                    res.push(Instruction::Control(ControlASM::Add(current_pick.variable.clone(), (current_pick.possibilities as i64).into(), current_pick.variable.clone())));
                    res.push(Instruction::Control(ControlASM::Sub(current_pick.variable.clone(), 1.into(), current_pick.variable.clone())));
                    res.push(Instruction::Control(ControlASM::Mod(current_pick.variable.clone(), (current_pick.possibilities as i64).into(), current_pick.variable.clone())));
                    set_pick_variables[current_pick.num_variable as usize] = true;
                }

                // in any case, add the conditional structure for the pick
                res.push(Instruction::Control(ControlASM::RelJumpIfEqual(current_pick.variable.clone(), (current_pick.position as i64).into(), 2)));

                // jump over effects if the pick is not successful
                let prog = TimeStatement::At(t.clone(), x.clone(), context.clone(), choices.to_vec(), picks, alts.to_vec()).as_asm(local_choice_vars, set_pick_variables, local_alt_vars, set_alt_variables);
                let num_prog_instruction = prog.len();
                res.push(Instruction::Control(ControlASM::RelJump((num_prog_instruction + 1) as i64)));

                // add all of this to the previously constructed program
                res.extend(prog);

                res
                */
            }
        }
    }
}

impl Ord for TimeStatement {
    fn cmp(&self, other: &Self) -> Ordering {
        let t1 = self.get_time();
        let t2 = other.get_time();
        let v1 = t1.signe * t1.numerator * t2.denominator;
        let v2 = t2.signe * t2.numerator * t1.denominator;
        if v1 < v2 {
            return Ordering::Less;
        }
        if v1 > v2 {
            return Ordering::Greater;
        }
        match (self, other) {
            (TimeStatement::JustBefore(_, _, _, _), _) => Ordering::Less,
            (_, TimeStatement::JustAfter(_, _, _, _)) => Ordering::Less,
            (_, TimeStatement::JustBefore(_, _, _, _)) => Ordering::Greater,
            (TimeStatement::JustAfter(_, _, _, _), _) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for TimeStatement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TimeStatement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TimeStatement::At(x, _, _, _), TimeStatement::At(y, _, _, _)) => {
                x.numerator * y.denominator == y.numerator * x.denominator
            }
            (TimeStatement::JustBefore(x, _, _, _), TimeStatement::JustBefore(y, _, _, _)) => {
                x.numerator * y.denominator == y.numerator * x.denominator
            }
            (TimeStatement::JustAfter(x, _, _, _), TimeStatement::JustAfter(y, _, _, _)) => {
                x.numerator * y.denominator == y.numerator * x.denominator
            }
            _ => false,
        }
    }
}

impl Eq for TimeStatement {}

