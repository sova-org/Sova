// TODO : définir les noms de variables temporaires ici et les commenter avec leurs types pour éviter les erreurs

/*
3. fonctions avec une valeur de retour toujours + définissables une seule fois
4. ramp avec fonction appliquée sur variable
5. rajouter des variables d'environnement
6. (jump 2.5)
7. le nombre d'éléments sélectionnés dans choice devrait être une expression si possible
8. ajouter la possibilité de listes dans les contextes
*/

use crate::compiler::bali::bali_ast::constants::{
    DEBUG_TIME_STATEMENTS,
    DEBUG_INSTRUCTIONS,
    DEFAULT_CHAN,
    DEFAULT_DEVICE,
    DEFAULT_VELOCITY,
    DEFAULT_DURATION,
    LOCAL_TARGET_VAR,
    LOCAL_PICK_VAR,
    LOCAL_ALT_VAR,
};
use crate::lang::{
    Instruction, Program,
    control_asm::ControlASM,
    environment_func::EnvironmentFunc,
    event::Event,
    variable::Variable,
};

pub type BaliProgram = Vec<Statement>;
pub type BaliPreparedProgram = Vec<TimeStatement>;

pub mod fraction;
pub mod bali_context;
pub mod concrete_fraction;
pub mod expression;
pub mod loop_context;
pub mod time_statement;
pub mod information;
pub mod statement;
pub mod effect;
pub mod boolean;
pub mod value;
pub mod toplevel_effect;
pub mod constants;
pub mod variable_generators;
pub mod abstract_effect;
pub mod args;
pub mod abstract_statement;

pub use fraction::Fraction;
pub use variable_generators::{
    ChoiceVariableGenerator,
    LocalChoiceVariableGenerator,
    AltVariableGenerator,
};
pub use bali_context::BaliContext;
pub use concrete_fraction::ConcreteFraction;
pub use expression::Expression;
pub use loop_context::LoopContext;
pub use time_statement::TimeStatement;
pub use information::TimingInformation;
pub use statement::Statement;
pub use effect::Effect;
pub use boolean::BooleanExpression;
pub use value::Value;
pub use toplevel_effect::TopLevelEffect;
pub fn bali_as_asm(prog: BaliProgram) -> Program {
    let mut res: Program = Vec::new();

    if prog.len() == 0 {
        return res;
    }

    //print!("Original prog {:?}\n", prog);
    //let prog = expend_loop(prog);
    //print!("Loopless prog {:?}\n", prog);
    let default_context = BaliContext {
        channel: Some(Expression::Value(Value::Number(DEFAULT_CHAN))),
        device: Some(Expression::Value(Value::Number(DEFAULT_DEVICE))),
        velocity: Some(Expression::Value(Value::Number(DEFAULT_VELOCITY))),
        duration: Some(Expression::Value(Value::Number(DEFAULT_DURATION))),
    };

    let mut choice_variables =
        ChoiceVariableGenerator::new("_choice".to_string(), "_target".to_string());
    let mut local_choice_variables = LocalChoiceVariableGenerator::new("_local_choice".to_string());
    let mut pick_variables = LocalChoiceVariableGenerator::new("_pick".to_string());
    let mut local_alt_variables = AltVariableGenerator::new("_local_alt".to_string());
    let mut alt_variables = AltVariableGenerator::new("_instance_alt".to_string());

    let mut prog = expend_prog(
        prog,
        default_context,
        &mut choice_variables,
        &mut pick_variables,
        &mut alt_variables,
    );

    let mut set_pick_variables: Vec<bool> = Vec::new();
    for _i in 0..pick_variables.get_num_variables() {
        set_pick_variables.push(false);
    }

    let mut set_alt_variables: Vec<bool> = Vec::new();
    for _i in 0..alt_variables.get_num_variables() {
        set_alt_variables.push(false);
    }

    if prog.len() == 0 {
        return res;
    }

    // Set expected types for all variables
    res.push(Instruction::Control(ControlASM::Mov(
        0.into(),
        LOCAL_ALT_VAR.clone(),
    )));

    // Initialize the variables for the choices with random values in the good range
    for var_pos in 0..choice_variables.variable_set.len() {
        res.push(Instruction::Control(ControlASM::Mov(
            Variable::Environment(EnvironmentFunc::RandomUInt(
                choice_variables.variable_bounds[var_pos] as u64,
            )),
            choice_variables.variable_set[var_pos].clone(),
        )));
    }

    //print!("Choice variables {:?}\n", choice_variables);
    //print!("Pick variables {:?}\n", pick_variables);
    if DEBUG_TIME_STATEMENTS {
        let info = "EXPENDED PROG";
        print!("BEGIN: {}\n", info);
        for ts in prog.iter() {
            print!("{:?}\n", ts);
        }
        print!("END: {}\n", info);
    }
    prog.sort();
    //print!("Sorted prog {:?}\n", prog);

    let mut total_delay: f64 = if prog.len() > 0 {
        prog[0].get_time_as_f64()
    } else {
        0.0
    };

    let time_var = Variable::Instance("_time".to_owned());

    if total_delay > 0.0 {
        res.push(Instruction::Control(ControlASM::FloatAsFrames(
            total_delay.into(),
            time_var.clone(),
        )));
        res.push(Instruction::Effect(Event::Nop, time_var.clone()));
    }

    for i in 0..prog.len() - 1 {
        //print!("{:?}\n", prog[i]);
        let delay = if total_delay >= 0.0 {
            prog[i + 1].get_time_as_f64() - total_delay
        } else {
            prog[i + 1].get_time_as_f64()
        };
        let delay = if delay < 0.0 { 0.0 } else { delay };
        total_delay = prog[i + 1].get_time_as_f64();
        res.extend(prog[i].as_asm(
            &mut local_choice_variables,
            &mut set_pick_variables,
            &mut local_alt_variables,
            &mut set_alt_variables,
        ));
        if delay > 0.0 {
            res.push(Instruction::Control(ControlASM::FloatAsFrames(
                delay.into(),
                time_var.clone(),
            )));
            res.push(Instruction::Effect(Event::Nop, time_var.clone()));
        }

        //print!("NEW TIME STATEMENT!\n");
    }

    res.extend(prog[prog.len() - 1].as_asm(
        &mut local_choice_variables,
        &mut set_pick_variables,
        &mut local_alt_variables,
        &mut set_alt_variables,
    ));

    // print program for debug
    if DEBUG_INSTRUCTIONS {
        let mut count = 0;
        let info = "INTERNAL PROGRAM CONTENT";
        print!("BEGIN: {}\n", info);
        for inst in res.iter() {
            match inst {
                Instruction::Control(ControlASM::RelJump(x))
                | Instruction::Control(ControlASM::RelJumpIf(_, x))
                | Instruction::Control(ControlASM::RelJumpIfNot(_, x))
                | Instruction::Control(ControlASM::RelJumpIfDifferent(_, _, x))
                | Instruction::Control(ControlASM::RelJumpIfEqual(_, _, x))
                | Instruction::Control(ControlASM::RelJumpIfLess(_, _, x))
                | Instruction::Control(ControlASM::RelJumpIfLessOrEqual(_, _, x)) => {
                    print!("{}: {:?} ➡️  {}\n", count, inst, count + x)
                }
                Instruction::Control(ControlASM::Jump(x))
                | Instruction::Control(ControlASM::JumpIf(_, x))
                | Instruction::Control(ControlASM::JumpIfNot(_, x))
                | Instruction::Control(ControlASM::JumpIfDifferent(_, _, x))
                | Instruction::Control(ControlASM::JumpIfEqual(_, _, x))
                | Instruction::Control(ControlASM::JumpIfLess(_, _, x))
                | Instruction::Control(ControlASM::JumpIfLessOrEqual(_, _, x)) => {
                    print!("{}: {:?} ➡️  {}\n", count, inst, x)
                }
                _ => print!("{}: {:?}\n", count, inst),
            };
            count += 1;
        }
        print!("END: {}\n", info);
    }

    res
}

pub fn expend_prog(
    prog: BaliProgram,
    c: BaliContext,
    mut choice_vars: &mut ChoiceVariableGenerator,
    mut pick_variables: &mut LocalChoiceVariableGenerator,
    mut alt_variables: &mut AltVariableGenerator,
) -> BaliPreparedProgram {
    prog.into_iter()
        .map(|s| {
            s.expend(
                &ConcreteFraction {
                    signe: 1,
                    numerator: 0,
                    denominator: 1,
                },
                &ConcreteFraction {
                    signe: 1,
                    numerator: 1,
                    denominator: 1,
                },
                c.clone(),
                Vec::new(),
                &mut choice_vars,
                &mut pick_variables,
                &mut alt_variables,
            )
        })
        .flatten()
        .collect()
}

pub fn set_context_effect_set(set: Vec<TopLevelEffect>, c: BaliContext) -> Vec<TopLevelEffect> {
    set.into_iter().map(|e| e.set_context(c.clone())).collect()
}
