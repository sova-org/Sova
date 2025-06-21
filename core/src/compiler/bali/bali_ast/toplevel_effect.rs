use crate::{
    compiler::bali::bali_ast::{
        AltVariableGenerator, LOCAL_ALT_VAR, LOCAL_PICK_VAR, LOCAL_TARGET_VAR,
        LocalChoiceVariableGenerator, bali_context::BaliContext, boolean::BooleanExpression,
        effect::Effect, expression::Expression, function::FunctionContent,
    },
    lang::{
        Instruction, control_asm::ControlASM, environment_func::EnvironmentFunc, variable::Variable,
    },
};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum TopLevelEffect {
    Seq(Vec<TopLevelEffect>, BaliContext),
    With(Vec<TopLevelEffect>, BaliContext),
    For(Box<BooleanExpression>, Vec<TopLevelEffect>, BaliContext),
    If(Box<BooleanExpression>, Vec<TopLevelEffect>, BaliContext),
    Choice(i64, i64, Vec<TopLevelEffect>, BaliContext),
    Effect(Effect, BaliContext),
    Pick(Box<Expression>, Vec<TopLevelEffect>, BaliContext),
    Alt(Vec<TopLevelEffect>, Variable, BaliContext),
}

impl TopLevelEffect {
    pub fn set_context(self, c: BaliContext) -> TopLevelEffect {
        match self {
            TopLevelEffect::Seq(es, seq_context) => TopLevelEffect::Seq(es, seq_context.update(c)),
            TopLevelEffect::With(es, with_context) => {
                TopLevelEffect::With(es, with_context.update(c))
            }
            TopLevelEffect::For(cond, es, for_context) => {
                TopLevelEffect::For(cond, es, for_context.update(c))
            }
            TopLevelEffect::If(cond, es, if_context) => {
                TopLevelEffect::If(cond, es, if_context.update(c))
            }
            TopLevelEffect::Choice(num_selected, num_selectable, es, choice_context) => {
                TopLevelEffect::Choice(num_selected, num_selectable, es, choice_context.update(c))
            }
            TopLevelEffect::Pick(position, es, pick_context) => {
                TopLevelEffect::Pick(position, es, pick_context.update(c))
            }
            TopLevelEffect::Effect(e, effect_context) => {
                TopLevelEffect::Effect(e, effect_context.update(c))
            }
            TopLevelEffect::Alt(es, var, alt_context) => {
                TopLevelEffect::Alt(es, var, alt_context.update(c))
            }
        }
    }

    pub fn as_asm(
        &self,
        context: BaliContext,
        local_choice_vars: &mut LocalChoiceVariableGenerator,
        local_alt_vars: &mut AltVariableGenerator,
        functions: &HashMap<String, FunctionContent>,
    ) -> Vec<Instruction> {
        //let time_var = Variable::Instance("_time".to_owned());
        let bvar_out = Variable::Instance("_bres".to_owned());
        match self {
            TopLevelEffect::Seq(s, seq_context) | TopLevelEffect::With(s, seq_context) => {
                let mut res = Vec::new();
                let context = seq_context.clone().update(context.clone());
                for i in 0..s.len() {
                    let to_add = s[i].as_asm(
                        context.clone(),
                        local_choice_vars,
                        local_alt_vars,
                        &functions,
                    );
                    res.extend(to_add);
                }
                res
            }
            TopLevelEffect::For(e, s, for_context) => {
                let mut res = Vec::new();

                // Compute and add condition
                let condition = e.as_asm(&functions);
                res.extend(condition);

                // Add for structure
                res.push(Instruction::Control(ControlASM::Pop(bvar_out.clone())));
                res.push(Instruction::Control(ControlASM::RelJumpIf(
                    bvar_out.clone(),
                    2,
                )));

                // Compute effects
                let context = for_context.clone().update(context.clone());
                let mut effects = Vec::new();
                for i in 0..s.len() {
                    let to_add = s[i].as_asm(
                        context.clone(),
                        local_choice_vars,
                        local_alt_vars,
                        &functions,
                    );
                    effects.extend(to_add);
                }

                // Add for structure (continued)
                let num_effect_instruction = effects.len() as i64;
                res.push(Instruction::Control(ControlASM::RelJump(
                    num_effect_instruction + 2,
                )));

                // Add effects
                res.extend(effects);

                // Add for structure (end)
                let num_instructions = res.len() as i64;
                res.push(Instruction::Control(ControlASM::RelJump(-num_instructions)));

                res
            }
            TopLevelEffect::If(e, s, if_context) => {
                let mut res = Vec::new();

                // Compute and add condition
                let condition = e.as_asm(&functions);
                res.extend(condition);

                // Add if structure
                res.push(Instruction::Control(ControlASM::Pop(bvar_out.clone())));
                res.push(Instruction::Control(ControlASM::RelJumpIf(
                    bvar_out.clone(),
                    2,
                )));

                // Compute effects
                let context = if_context.clone().update(context.clone());
                let mut effects = Vec::new();
                for i in 0..s.len() {
                    let to_add = s[i].as_asm(
                        context.clone(),
                        local_choice_vars,
                        local_alt_vars,
                        &functions,
                    );
                    effects.extend(to_add);
                }

                // Add if structure (continued)
                let num_effect_instruction = effects.len() as i64;
                res.push(Instruction::Control(ControlASM::RelJump(
                    num_effect_instruction + 1,
                )));

                // Add effects
                res.extend(effects);

                res
            }
            TopLevelEffect::Choice(num_selected, num_selectable, es, choice_context) => {
                let mut res = Vec::new();

                // If nothing is selected, generate no instructions
                let num_selected = *num_selected;
                if num_selected <= 0 {
                    return res;
                }

                // If something in es cannot be selected, make it selectable
                let num_selectable = if *num_selectable < es.len() as i64 {
                    es.len() as i64
                } else {
                    *num_selectable
                };

                // If everything will be selected
                if num_selected >= num_selectable {
                    return TopLevelEffect::Seq(es.clone(), choice_context.clone()).as_asm(
                        context,
                        local_choice_vars,
                        local_alt_vars,
                        &functions,
                    );
                }

                // An actual selection will occur
                let mut choice_vars = Vec::new();
                let context = choice_context.clone().update(context.clone());

                // generate random values for the choice
                for selection_number in 0..num_selected {
                    let choice_variable = local_choice_vars.get_variable();
                    res.push(Instruction::Control(ControlASM::Mov(
                        Variable::Environment(EnvironmentFunc::RandomUInt(
                            (num_selectable - selection_number) as u64,
                        )),
                        choice_variable.clone(),
                    )));
                    //position += 1;
                    choice_vars.push(choice_variable);
                }

                // generate the code for each effect in the set es
                for effect_pos in 0..es.len() {
                    // init targe variable to set effect position as selection value
                    res.push(Instruction::Control(ControlASM::Mov(
                        (effect_pos as i64).into(),
                        LOCAL_TARGET_VAR.clone(),
                    )));

                    // handle each possible choice for this effect
                    let num_instruction_for_first_choice = 1;
                    let num_instruction_for_other_choices = if effect_pos == 0 { 1 } else { 3 };
                    let num_instruction_between_choices_and_effects = 1;
                    let mut distance_to_effect = num_instruction_for_first_choice
                        + num_instruction_for_other_choices * (num_selected - 1)
                        + num_instruction_between_choices_and_effects;
                    for choice_number in 0..num_selected {
                        distance_to_effect = if choice_number == 0 {
                            distance_to_effect - num_instruction_for_first_choice
                        } else {
                            distance_to_effect - num_instruction_for_other_choices
                        };

                        if choice_number > 0 && effect_pos > 0 {
                            res.push(Instruction::Control(ControlASM::RelJumpIfLessOrEqual(
                                LOCAL_TARGET_VAR.clone(),
                                choice_vars[choice_number as usize - 1].clone(),
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
                            choice_vars[choice_number as usize].clone(),
                            distance_to_effect + 1,
                        )))
                    }

                    // jump over effects if the choice don't select them
                    let effect_prog = es[effect_pos].as_asm(
                        context.clone(),
                        local_choice_vars,
                        local_alt_vars,
                        &functions,
                    );
                    res.push(Instruction::Control(ControlASM::RelJump(
                        (effect_prog.len() + 1) as i64,
                    )));

                    // add the actual effects
                    res.extend(effect_prog);
                }

                res
            }
            TopLevelEffect::Pick(position, es, pick_context) => {
                // get context
                let context = pick_context.clone().update(context.clone());

                // compute the position
                let mut res = position.as_asm(&functions);
                res.push(Instruction::Control(ControlASM::Pop(
                    LOCAL_PICK_VAR.clone(),
                )));
                res.push(Instruction::Control(ControlASM::Add(
                    LOCAL_PICK_VAR.clone(),
                    (es.len() as i64).into(),
                    LOCAL_PICK_VAR.clone(),
                )));
                res.push(Instruction::Control(ControlASM::Sub(
                    LOCAL_PICK_VAR.clone(),
                    1.into(),
                    LOCAL_PICK_VAR.clone(),
                )));
                res.push(Instruction::Control(ControlASM::Mod(
                    LOCAL_PICK_VAR.clone(),
                    (es.len() as i64).into(),
                    LOCAL_PICK_VAR.clone(),
                )));

                let mut effect_progs = Vec::new();

                // jump to the position
                let mut effect_number = 0;
                let num_pick_instruction_per_step = 1;
                let num_pick_instructions = (es.len() as i64) * num_pick_instruction_per_step;
                let mut distance_to_effect = num_pick_instructions - num_pick_instruction_per_step;
                let mut distance_to_end = 0;
                for e in es.iter() {
                    effect_progs.push(e.as_asm(
                        context.clone(),
                        local_choice_vars,
                        local_alt_vars,
                        &functions,
                    ));
                    let new_effect_len = effect_progs[effect_number as usize].len() as i64 + 1; // +1 for the jumps that will be added later
                    distance_to_end += new_effect_len;

                    res.push(Instruction::Control(ControlASM::RelJumpIfEqual(
                        LOCAL_PICK_VAR.clone(),
                        effect_number.into(),
                        distance_to_effect + 1,
                    )));

                    distance_to_effect =
                        distance_to_effect - num_pick_instruction_per_step + new_effect_len; // +1 for the jumps after the effects

                    effect_number += 1;
                }

                // add the effects and jumps to avoir other effects
                for ep in effect_progs.iter() {
                    res.extend(ep.clone());

                    distance_to_end -= (ep.len() as i64) + 1;
                    if distance_to_end != 0 {
                        res.push(Instruction::Control(ControlASM::RelJump(distance_to_end)));
                    }
                }

                res
            }
            TopLevelEffect::Alt(es, frame_variable, alt_context) => {
                let mut res = Vec::new();

                // get context
                let context = alt_context.clone().update(context.clone());

                // no alt if only one effect
                if es.len() == 1 {
                    return es[0].as_asm(
                        context.clone(),
                        local_choice_vars,
                        local_alt_vars,
                        &functions,
                    );
                }

                let alt_variable = frame_variable;

                // Store the value of the frame variable locally to avoid strange behaviors with several scripts
                // running at the same time for the same frame
                res.push(Instruction::Control(ControlASM::Mov(
                    alt_variable.clone(),
                    LOCAL_ALT_VAR.clone(),
                )));
                res.push(Instruction::Control(ControlASM::Mod(
                    LOCAL_ALT_VAR.clone(),
                    (es.len() as i64).into(),
                    LOCAL_ALT_VAR.clone(),
                )));

                let mut effect_progs = Vec::new();
                let mut distance_to_end = 0;
                for pos in 0..es.len() {
                    effect_progs.push(Vec::new());

                    let this_effect_prog = es[pos].as_asm(
                        context.clone(),
                        local_choice_vars,
                        local_alt_vars,
                        &functions,
                    );
                    let distance_to_next_effect = this_effect_prog.len() as i64 + 1; // +1 for the jump after the effects

                    // Jump after the effects if they are not selected
                    effect_progs[pos].push(Instruction::Control(ControlASM::RelJumpIfDifferent(
                        (pos as i64).into(),
                        LOCAL_ALT_VAR.clone(),
                        distance_to_next_effect + 1,
                    )));

                    // Record the effects
                    effect_progs[pos].extend(this_effect_prog);

                    // update the total distance to the end of the effets
                    distance_to_end += effect_progs[pos].len() + 1; // +1 for the jump after the effects
                }

                // Add the effects and the jump to the end after them
                for prog in effect_progs.iter() {
                    distance_to_end -= prog.len() + 1;

                    res.extend(prog.to_vec());
                    res.push(Instruction::Control(ControlASM::RelJump(
                        distance_to_end as i64 + 1,
                    )));
                }

                // Update the frame variable
                res.push(Instruction::Control(ControlASM::Add(
                    alt_variable.clone(),
                    1.into(),
                    alt_variable.clone(),
                )));

                res
            }
            TopLevelEffect::Effect(ef, effect_context) => {
                let context = effect_context.clone().update(context.clone());
                ef.as_asm(context, &functions)
            }
        }
    }
}
