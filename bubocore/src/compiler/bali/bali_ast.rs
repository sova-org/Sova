use crate::{lang::{Program, event::Event, Instruction, control_asm::ControlASM, variable::{Variable, VariableValue}, environment_func::EnvironmentFunc}, protocol::osc::{OSCMessage, Argument as OscArgument}};
use std::cmp::Ordering;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::cmp::min;
use rand::{self, Rng}; // Add rand import

pub type BaliProgram = Vec<Statement>;
pub type BaliPreparedProgram = Vec<TimeStatement>;

// TODO : définir les noms de variables temporaires ici et les commenter avec leurs types pour éviter les erreurs

// TODO :
// - (note [50 51 52]), (note <50 51 52>) - idem partout 
// - fonctions (func f [x y z] TopLevelEffectSet)
// - pour fspread, voir comment c'est fait. Passer un fspread_time dans les extend, qui vaut initialement 1 (durée de la frame) et est modifié à chaque loop (loop, eucloop, binloop) pour la durée du pas et à chaque spread/fspread pour la durée d'un élément

const DEBUG: bool = true;

const DEFAULT_VELOCITY: i64 = 90;
pub const DEFAULT_CHAN: i64 = 1;
pub const DEFAULT_DEVICE: i64 = 1;
const DEFAULT_DURATION: i64 = 2;
// Default frame duration if none is specified by context
lazy_static! {
    static ref DEFAULT_FRAME_DURATION: ConcreteFraction = ConcreteFraction { signe: 1, numerator: 1, denominator: 1 };
}

lazy_static! {
    static ref LOCAL_TARGET_VAR: Variable = Variable::Instance("_local_target".to_owned());
    static ref LOCAL_PICK_VAR: Variable = Variable::Instance("_local_pick".to_owned());
}

pub fn bali_as_asm(prog: BaliProgram) -> Program {

    let mut res: Program = Vec::new();

    if prog.len() == 0 {
        return res
    }

    //print!("Original prog {:?}\n", prog);
    //let prog = expend_loop(prog);
    //print!("Loopless prog {:?}\n", prog);
    let default_context = BaliContext{
        channel: Some(Expression::Value(Value::Number(DEFAULT_CHAN))),
        device: Some(Expression::Value(Value::Number(DEFAULT_DEVICE))),
        velocity: Some(Expression::Value(Value::Number(DEFAULT_VELOCITY))),
        duration: Some(Fraction{
            numerator: Box::new(Expression::Value(Value::Number(1))),
            denominator: Box::new(Expression::Value(Value::Number(DEFAULT_DURATION))),
        }),
        frame_duration: Some(DEFAULT_FRAME_DURATION.clone()), // Set default frame duration
        dirt_defaults: None, // Initialize to None
    };

    let mut choice_variables = ChoiceVariableGenerator::new("_choice".to_string(), "_target".to_string());
    let mut local_choice_variables = LocalChoiceVariableGenerator::new("_local_choice".to_string());

    let mut pick_variables = LocalChoiceVariableGenerator::new("_pick".to_string());

    let mut prog = expend_prog(prog, default_context, &mut choice_variables, &mut pick_variables);

    let mut set_pick_variables: Vec<bool> = Vec::new();
    for _i in 0..pick_variables.get_num_variables() {
        set_pick_variables.push(false);
    }

    if prog.len() == 0 {
        return res
    }

    // Initialize the variables for the choices with random values in the good range
    // Initialize the target variables as well TODO
    for var_pos in 0..choice_variables.variable_set.len() {
        res.push(Instruction::Control(ControlASM::Mov(Variable::Environment(EnvironmentFunc::RandomUInt(choice_variables.variable_bounds[var_pos] as u64)), choice_variables.variable_set[var_pos].clone())));
    }


    //print!("Choice variables {:?}\n", choice_variables);
    //print!("Pick variables {:?}\n", pick_variables);
    //print!("Expended prog {:?}\n", prog);
    prog.sort();
    //print!("Sorted prog {:?}\n", prog);

    let mut total_delay: f64 = if prog.len() > 0 {
        prog[0].get_time_as_f64()
    } else {
        0.0
    };

    let time_var = Variable::Instance("_time".to_owned());

    if total_delay > 0.0 {
        res.push(Instruction::Control(ControlASM::FloatAsFrames(total_delay.into(), time_var.clone())));
        res.push(Instruction::Effect(Event::Nop, time_var.clone()));
    }

    for i in 0..prog.len()-1 {
        //print!("{:?}\n", prog[i]);
        let delay = if total_delay >= 0.0 {
            prog[i+1].get_time_as_f64() - total_delay
        } else {
            prog[i+1].get_time_as_f64()
        };
        let delay = if delay < 0.0 {
            0.0
        } else {
            delay
        };
        total_delay = prog[i+1].get_time_as_f64();
        res.extend(prog[i].as_asm(&mut local_choice_variables, &mut set_pick_variables));
        if delay > 0.0 {
            res.push(Instruction::Control(ControlASM::FloatAsFrames(delay.into(), time_var.clone())));
            res.push(Instruction::Effect(Event::Nop, time_var.clone()));
        }
    }

    res.extend(prog[prog.len()-1].as_asm(&mut local_choice_variables, &mut set_pick_variables));


    // print program for debug
    if DEBUG {
        let mut count = 0;
        let info = "INTERNAL PROGRAM CONTENT";
        print!("BEGIN: {}\n", info);
        for inst in res.iter() {
            match inst {
                Instruction::Control(ControlASM::RelJump(x)) | Instruction::Control(ControlASM::RelJumpIf(_, x)) | Instruction::Control(ControlASM::RelJumpIfNot(_, x)) | Instruction::Control(ControlASM::RelJumpIfDifferent(_, _, x)) | Instruction::Control(ControlASM::RelJumpIfEqual(_, _, x)) | Instruction::Control(ControlASM::RelJumpIfLess(_, _, x)) | Instruction::Control(ControlASM::RelJumpIfLessOrEqual(_, _, x)) => print!("{}: {:?} ➡️  {}\n", count, inst, count + x),
                Instruction::Control(ControlASM::Jump(x)) | Instruction::Control(ControlASM::JumpIf(_, x)) | Instruction::Control(ControlASM::JumpIfNot(_, x)) | Instruction::Control(ControlASM::JumpIfDifferent(_, _, x)) | Instruction::Control(ControlASM::JumpIfEqual(_, _, x)) | Instruction::Control(ControlASM::JumpIfLess(_, _, x)) | Instruction::Control(ControlASM::JumpIfLessOrEqual(_, _, x)) => print!("{}: {:?} ➡️  {}\n", count, inst, x),
                _ => print!("{}: {:?}\n", count, inst),
            };
            count+=1;
        }
        print!("END: {}\n", info);
    }

    res
}


pub fn expend_prog(prog: BaliProgram, c: BaliContext, mut choice_vars: &mut ChoiceVariableGenerator, mut pick_variables: &mut LocalChoiceVariableGenerator) -> BaliPreparedProgram {
    prog.into_iter().map(|s| s.expend(&ConcreteFraction{signe: 1, numerator: 0, denominator: 1}, &ConcreteFraction{signe: 1, numerator: 1, denominator: 1}, c.clone(), Vec::new(), Vec::new(), &mut choice_vars, &mut pick_variables)).flatten().collect()
}

/*
pub fn set_context_prog(prog: BaliProgram, c: BaliContext) -> BaliProgram {
    prog.into_iter().map(|s| s.set_context(c.clone())).collect()
}
*/

#[derive(Debug)]
pub struct LocalChoiceVariableGenerator {
    current_variable_number: i64,
    choice_variable_base_name: String,
}

impl LocalChoiceVariableGenerator {

    pub fn new(choice_variable_base_name: String) -> LocalChoiceVariableGenerator {
        LocalChoiceVariableGenerator {
            current_variable_number: 0,
            choice_variable_base_name,
        }
    }

    pub fn get_variable(&mut self) -> Variable {
        let new_choice_variable_name = self.choice_variable_base_name.clone() + "_" + &self.current_variable_number.to_string();

        self.current_variable_number += 1;

        Variable::Instance(new_choice_variable_name)
    }

    pub fn get_variable_and_number(&mut self) -> (Variable, i64) {
        let number = self.current_variable_number;
        let variable = self.get_variable();

        (variable, number)
    }

    pub fn get_num_variables(&self) -> i64 {
        self.current_variable_number
    }

}

#[derive(Debug)]
pub struct ChoiceVariableGenerator {
    current_variable_number: i64,
    choice_variable_base_name: String,
    target_variable_base_name: String,
    variable_set: Vec<Variable>,
    variable_bounds: Vec<i64>,
}

impl ChoiceVariableGenerator {

    pub fn new(choice_variable_base_name: String, target_variable_base_name: String) -> ChoiceVariableGenerator {
        ChoiceVariableGenerator {
            current_variable_number: 0,
            choice_variable_base_name,
            target_variable_base_name,
            variable_set: Vec::new(),
            variable_bounds: Vec::new(), // gives the bound of each variable for random generation
        }
    }

    pub fn get_variables(&mut self, num_variables: i64, num_possibilities: i64) -> (Vec<Variable>, Vec<Variable>) {

        let mut choice_res = Vec::new();
        let mut target_res = Vec::new();

        if num_possibilities <= 0 {
            return (choice_res, target_res)
        }

        let num_variables = min(num_variables, num_possibilities);

        let new_choice_variable_base_name = self.choice_variable_base_name.clone() + "_" + &self.current_variable_number.to_string();
        let new_target_variable_base_name = self.target_variable_base_name.clone() + "_" + &self.current_variable_number.to_string();
        self.current_variable_number += 1;

        let mut current_bound = num_possibilities;

        for variable_num in 0..num_variables {
            let new_choice_variable_name = new_choice_variable_base_name.clone() + "_" + &variable_num.to_string();
            let new_choice_variable = Variable::Instance(new_choice_variable_name);

            self.variable_set.push(new_choice_variable.clone());
            choice_res.push(new_choice_variable);

            // bound for this variable
            self.variable_bounds.push(current_bound);
            current_bound -= 1;

            let new_target_variable_name = new_target_variable_base_name.clone() + "_" + &variable_num.to_string();
            let new_target_variable = Variable::Instance(new_target_variable_name);
            target_res.push(new_target_variable);
        }

        (choice_res, target_res)
    }

}

pub fn set_context_effect_set(set: Vec<TopLevelEffect>, c: BaliContext) -> Vec<TopLevelEffect> {
    set.into_iter().map(|e| e.set_context(c.clone())).collect()
}

#[derive(Debug, Clone)]
pub struct ChoiceInformation {
    pub variables: Vec<Variable>, // variables utilisée pour faire ce choix
    pub target_variables: Vec<Variable>, // variables utilisées pour stocker les valeurs visées pour les variables de choix
    //pub num_selectable: i64, // nombre d'éléments disponibles pour le choix
    pub position: usize, // position de cet élément particulier dans la liste des éléments du choix
}

#[derive(Debug, Clone)]
pub struct PickInformation {
    pub variable: Variable, // variable utilisée pour ce pick
    pub position: usize, // position de l'élément considéré dans le pick
    pub possibilities: usize, // nombre d'éléments dans le pick
    pub expression: Expression, // expression pour obtenir la valeur du pick
    pub num_variable: i64, // numéro de la variable dans l'ordre de génération
}

#[derive(Debug, Clone)]
pub struct LoopContext {
    pub negate: bool,
    pub reverse: bool,
    pub shift: Option<i64>,
}

impl LoopContext {
    pub fn new() -> LoopContext {
        LoopContext{
            negate: false,
            reverse: false,
            shift: None,
        }
    }

    pub fn update(self, above: LoopContext) -> LoopContext {
        let mut b = LoopContext::new();
        b.negate = self.negate || above.negate;
        b.reverse = self.reverse || above.reverse;
        b.shift = match self.shift {
            Some(_) => self.shift,
            None => above.shift,
        };
        b
    }
}

#[derive(Debug, Clone)]
pub struct BaliContext {
    pub channel: Option<Expression>,
    pub device: Option<Expression>,
    pub velocity: Option<Expression>,
    pub duration: Option<Fraction>,
    pub frame_duration: Option<ConcreteFraction>, // Added frame duration
    pub dirt_defaults: Option<HashMap<String, Fraction>>, // Added default dirt parameters
}

impl BaliContext {
    pub fn new() -> BaliContext {
        BaliContext{
            channel: None,
            device: None,
            velocity: None,
            duration: None,
            frame_duration: None, // Initialize to None
            dirt_defaults: None, // Initialize to None
        }
    }

    pub fn update(self, above: BaliContext) -> BaliContext {
        let mut b = BaliContext::new();
        b.channel = match self.channel {
            Some(_) => self.channel,
            None => above.channel,
        };
        b.device = match self.device {
            Some(_) => self.device,
            None => above.device,
        };
        b.velocity = match self.velocity {
            Some(_) => self.velocity,
            None => above.velocity,
        };
        b.duration = match self.duration {
            Some(_) => self.duration,
            None => above.duration,
        };
        // Update frame_duration: inner context overrides outer
        b.frame_duration = match self.frame_duration {
            Some(_) => self.frame_duration,
            None => above.frame_duration, // Inherit if not set locally
        };
        // Update dirt_defaults: Merge inner with outer, inner takes precedence
        b.dirt_defaults = match (self.dirt_defaults, above.dirt_defaults) {
            (Some(mut inner_map), Some(outer_map)) => {
                // Add outer defaults only if not present in inner
                for (key, value) in outer_map {
                    inner_map.entry(key).or_insert(value);
                }
                Some(inner_map)
            },
            (Some(inner_map), None) => Some(inner_map), // Only inner specified
            (None, Some(outer_map)) => Some(outer_map), // Only outer specified
            (None, None) => None, // Neither specified
        };
        b
    }
}

#[derive(Debug)]
pub enum TimeStatement {
    At(ConcreteFraction, TopLevelEffect, BaliContext, Vec<ChoiceInformation>, Vec<PickInformation>),
    JustBefore(ConcreteFraction, TopLevelEffect, BaliContext, Vec<ChoiceInformation>, Vec<PickInformation>),
    JustAfter(ConcreteFraction, TopLevelEffect, BaliContext, Vec<ChoiceInformation>, Vec<PickInformation>),
}

impl TimeStatement {

    pub fn get_time_as_f64(&self) -> f64 {
        match self {
            TimeStatement::At(x, _, _, _, _) | TimeStatement::JustBefore(x, _, _, _, _) | TimeStatement::JustAfter(x, _, _, _, _) => x.tof64(),
        }
    }

    pub fn get_time(&self) -> ConcreteFraction {
        match self {
            TimeStatement::At(x, _, _, _, _) | TimeStatement::JustBefore(x, _, _, _, _) | TimeStatement::JustAfter(x, _, _, _, _) => x.clone(),
        }
    }

    pub fn as_asm(&self,  local_choice_vars: &mut LocalChoiceVariableGenerator, set_pick_variables: &mut Vec<bool>) -> Vec<Instruction> {
        match self {
            TimeStatement::At(t, x, context, choices, picks) | TimeStatement::JustBefore(t, x, context, choices, picks) | TimeStatement::JustAfter(t, x, context, choices, picks) => {

                if choices.len() == 0 && picks.len() == 0 {
                    return x.as_asm(context.clone(), local_choice_vars);
                }

                // handle choices (? ...)
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
                    let prog = TimeStatement::At(t.clone(), x.clone(), context.clone(), choices, picks.to_vec()).as_asm(local_choice_vars, set_pick_variables);
                    res.push(Instruction::Control(ControlASM::RelJump((prog.len() + 1) as i64)));

                    res.extend(prog);

                    return res;
                }

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
                let prog = TimeStatement::At(t.clone(), x.clone(), context.clone(), choices.to_vec(), picks).as_asm(local_choice_vars, set_pick_variables);
                let num_prog_instruction = prog.len();
                res.push(Instruction::Control(ControlASM::RelJump((num_prog_instruction + 1) as i64)));
                    
                // add all of this to the previously constructed program
                res.extend(prog);
                
                res
                
            },
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
            return Ordering::Less
        }
        if v1 > v2 {
            return Ordering::Greater
        }
        match (self, other) {
            (TimeStatement::JustBefore(_, _, _, _, _), _) => Ordering::Less,
            (_, TimeStatement::JustAfter(_, _, _, _, _)) => Ordering::Less,
            (_, TimeStatement::JustBefore(_, _, _, _, _)) => Ordering::Greater,
            (TimeStatement::JustAfter(_, _, _, _, _), _) => Ordering::Greater,
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
            (TimeStatement::At(x, _, _, _, _), TimeStatement::At(y, _, _, _, _)) => x.numerator * y.denominator == y.numerator * x.denominator,
            (TimeStatement::JustBefore(x, _, _, _, _), TimeStatement::JustBefore(y, _, _, _, _)) => x.numerator * y.denominator == y.numerator * x.denominator,
            (TimeStatement::JustAfter(x, _, _, _, _), TimeStatement::JustAfter(y, _, _, _, _)) => x.numerator * y.denominator == y.numerator * x.denominator,
            _ => false
        }
    }
}

impl Eq for TimeStatement {}


/*
#[derive(Debug)]
pub enum TopLevelStatement {
    AtStatement(ConcreteFraction, Vec<Statement>),
    Statement(Vec<Statement>),
}

impl TopLevelStatement {

    pub fn expend(self) -> Vec<TimeStatement> {
        match self {
            TopLevelStatement::AtStatement(v, ss) => ss.into_iter().map(|s| s.expend(&v)).flatten().collect(),
            TopLevelStatement::Statement(ss) => ss.into_iter().map(|s| s.expend(&ConcreteFraction{numerator: 0, denominator: 1})).flatten().collect(),
        }
    }

}
*/

#[derive(Debug, Clone)]
pub enum TimingInformation {
    FrameRelative(ConcreteFraction),
    PositionRelative(ConcreteFraction),
}

impl TimingInformation {

    pub fn as_frames(&self, spread_time: &ConcreteFraction) -> ConcreteFraction {
        match self {
            TimingInformation::FrameRelative(time) => time.clone(),
            TimingInformation::PositionRelative(time) => time.mult(spread_time),
        }
    }

}

#[derive(Debug, Clone)]
pub enum Statement {
    AfterFrac(TimingInformation, Vec<Statement>, BaliContext),
    BeforeFrac(TimingInformation, Vec<Statement>, BaliContext),
    Loop(i64, TimingInformation, Vec<Statement>, BaliContext),
    Euclidean(i64, i64, LoopContext, TimingInformation, Vec<Statement>, BaliContext),
    Binary(i64, i64, LoopContext, TimingInformation, Vec<Statement>, BaliContext),
    After(Vec<TopLevelEffect>, BaliContext),
    Before(Vec<TopLevelEffect>, BaliContext),
    Effect(TopLevelEffect),
    With(Vec<Statement>, BaliContext),
    Choice(i64, i64, Vec<Statement>, BaliContext), 
    Spread(TimingInformation, Vec<Statement>, BaliContext), 
    Pick(Box<Expression>, Vec<Statement>, BaliContext), 
    Scatter(TimingInformation, Vec<Statement>, BaliContext), // Changed Option<ConcreteFraction> to TimingInformation
    WithDirt(HashMap<String, Fraction>, Vec<Statement>), 
}

impl Statement {

    /*
    pub fn set_context(self, c: BaliContext) -> Statement {
        match self {
            Statement::AfterFrac(v, es, cc) => Statement::AfterFrac(v, es, cc.update(c)),
            Statement::BeforeFrac(v, es, cc) => Statement::BeforeFrac(v, es, cc.update(c)),
            Statement::Loop(it, v, es, cc) => Statement::Loop(it, v, es, cc.update(c)),
            Statement::After(es, cc) => Statement::After(es, cc.update(c)),
            Statement::Before(es, cc) => Statement::Before(es, cc.update(c)),
            Statement::Effect(e, cc) => Statement::Effect(e, cc.update(c)),
        }
    }
    */

    fn is_simplifiable(seq: &Vec<Vec<i64>>) -> bool {
        if seq.len() < 2 {
            return false
        }

        return seq[seq.len() - 1].len() == seq[seq.len() - 2].len() && seq[seq.len() - 1].len() != seq[0].len()
    }

    fn get_euclidean(beats: i64, steps: i64, context: LoopContext) -> Vec<i64> {

        let mut seqs: Vec<Vec<i64>> = Vec::new();

        for _i in 0..beats {
            seqs.push(vec![1]);
        }

        let seqs_len = seqs.len();
        for j in 0..(steps - beats) {
            seqs[j as usize % seqs_len].push(0);
        }

        while Self::is_simplifiable(&seqs) {
            let mut in_pos = seqs.len() - 1;
            let mut out_pos = 0;
            let last = seqs[in_pos].len();
            while seqs[in_pos].len() == last {
                if let Some(elem) = seqs.pop() {
                    seqs[out_pos].extend(elem);
                    in_pos -= 1;
                    out_pos += 1;
                    if out_pos >= seqs.len() || seqs[out_pos].len() == last {
                        out_pos = 0;
                    }
                }
            }
        }

        let mut seq: Vec<i64> = seqs.into_iter().flatten().collect();

        Self::as_time_points(&mut seq, context)
    }

    fn get_binary(it: i64, steps: i64, context: LoopContext) -> Vec<i64> {
        let mut seq = Vec::new();
        let mut bin_seq = it;

        for _i in 0..7 {
            seq.push(bin_seq % 2);
            bin_seq = bin_seq/2;
        }
        seq.reverse();

        let mut res_seq = Vec::new();
        for i in 0..steps {
            res_seq.push(seq[(i % 7) as usize]);
        }

        Self::as_time_points(&mut res_seq, context)
    }

    fn as_time_points(seq: &mut Vec<i64>, context: LoopContext) -> Vec<i64> {
        
        //print!("{:?}\n", seq);

        if context.reverse {
            seq.reverse();
        }

        if context.negate {
            seq.iter_mut().for_each(|x| *x = 1 - *x);
        }

        if let Some(shift) = context.shift {
            seq.rotate_right(shift as usize);
        }

        //print!("{:?}\n", seq);

        let mut res = Vec::new();
        let mut count = 0;
        for i in 0..seq.len() {
            if seq[i] == 1 {
                res.push(count);
            } 
            count += 1;
        }

        //print!("{:?}\n", res);

        res
    }


    pub fn expend(self, val: &ConcreteFraction, spread_time: &ConcreteFraction, c: BaliContext, choices: Vec<ChoiceInformation>, picks: Vec<PickInformation>, choice_vars: &mut ChoiceVariableGenerator, pick_vars: &mut LocalChoiceVariableGenerator) -> Vec<TimeStatement> {
        /*let c = match self {
            Statement::AfterFrac(_, _, ref cc) | Statement::BeforeFrac(_, _, ref cc) | Statement::Loop(_, _, _, ref cc) | Statement::After(_, ref cc) | Statement::Before(_, ref cc) | Statement::Effect(_, ref cc) => cc.clone().update(c),
        };*/
        match self {
            Statement::AfterFrac(v, es, cc) => es.into_iter().map(|e| e.expend(&v.as_frames(spread_time).add(val), spread_time, cc.clone().update(c.clone()), choices.clone(), picks.clone(), choice_vars, pick_vars)).flatten().collect(),
            Statement::BeforeFrac(v, es, cc) => es.into_iter().map(|e| e.expend(&val.sub(&v.as_frames(spread_time)), spread_time, cc.clone().update(c.clone()), choices.clone(), picks.clone(), choice_vars, pick_vars)).flatten().collect(),
            Statement::Loop(it, v, es, cc) => {
                let mut res = Vec::new();
                let v = v.as_frames(spread_time).divbyint(it);
                for i in 0..it {
                    let content: Vec<TimeStatement> = es.clone().into_iter().map(|e| e.expend(&val.add(&v.multbyint(i)), &v, cc.clone().update(c.clone()), choices.clone(), picks.clone(), choice_vars, pick_vars)).flatten().collect();
                    res.extend(content);
                };
                res
            },
            Statement::Euclidean(beats, steps, loop_context, v, es, cc) => {
                let mut res = Vec::new();
                let euc = Self::get_euclidean(beats, steps, loop_context);
                let v = v.as_frames(spread_time).divbyint(steps);
                for i in 0..euc.len() {
                    let content: Vec<TimeStatement> = es.clone().into_iter().map(|e| e.expend(&val.add(&v.multbyint(euc[i])), &v, cc.clone().update(c.clone()), choices.clone(), picks.clone(), choice_vars, pick_vars)).flatten().collect();
                    res.extend(content);
                };
                res
            },
            Statement::Binary(it, steps, loop_context, v, es, cc) => {
                let mut res = Vec::new();
                let bin = Self::get_binary(it, steps, loop_context);
                let v = v.as_frames(spread_time).divbyint(steps);
                for i in 0..bin.len() {
                    let content: Vec<TimeStatement> = es.clone().into_iter().map(|e| e.expend(&val.add(&v.multbyint(bin[i])), &v, cc.clone().update(c.clone()), choices.clone(), picks.clone(), choice_vars, pick_vars)).flatten().collect();
                    res.extend(content);
                };
                res
            },
            Statement::After(es, cc) => es.into_iter().map(|e| TimeStatement::JustAfter(val.clone(), e, cc.clone().update(c.clone()), choices.clone(), picks.clone())).collect(),
            Statement::Before(es, cc) => es.into_iter().map(|e| TimeStatement::JustBefore(val.clone(), e, cc.clone().update(c.clone()), choices.clone(), picks.clone())).collect(),
            Statement::Effect(e) => vec![TimeStatement::At(val.clone(), e, c.clone(), choices.clone(), picks.clone())],
            Statement::With(es, cc) => es.into_iter().map(|e| e.expend(val, spread_time, cc.clone().update(c.clone()), choices.clone(), picks.clone(), choice_vars, pick_vars)).flatten().collect(),
            Statement::Choice(num_selected, num_selectable, es, cc) => {
                let mut res = Vec::new();

                if num_selected == 0 {
                    return res
                }

                if num_selected >= num_selectable {
                    return es.into_iter().map(|e| e.expend(val, spread_time, cc.clone().update(c.clone()), choices.clone(), picks.clone(), choice_vars, pick_vars)).flatten().collect()
                }

                let (choice_variables, target_variables) = choice_vars.get_variables(num_selected, num_selectable);
                for position in 0..es.len() {
                    let new_choice = ChoiceInformation {
                        variables: choice_variables.clone(),
                        target_variables: target_variables.clone(),
                        //num_selectable,
                        position,
                    };
                    let mut choices = choices.clone();
                    choices.push(new_choice);
                    res.extend(es[position].clone().expend(val, spread_time, cc.clone().update(c.clone()), choices, picks.clone(), choice_vars, pick_vars));
                };
                res
            },
            Statement::Spread(timing_info, es, cc) => { // Changed step_opt to timing_info
                let mut res = Vec::new();
                let effective_context = cc.clone().update(c.clone()); // Merged context
                let n = es.len() as i64;

                if n == 0 {
                    return res;
                }

                // Calculate the step as ConcreteFraction based on TimingInformation
                // Use the 'spread_time' argument from the outer function scope.
                let step = timing_info.as_frames(spread_time);

                // Iterate and expend each statement
                for i in 0..es.len() {
                    let time_offset = step.multbyint(i as i64);
                    let current_time = val.add(&time_offset);
                    // The child's spread_time should be the calculated step
                    let content: Vec<TimeStatement> = es[i as usize].clone().expend(
                        &current_time,
                        &step, // Child's spread_time is the step
                        effective_context.clone(),
                        choices.clone(),
                        picks.clone(),
                        choice_vars,
                        pick_vars
                    );
                    res.extend(content);
                }
                res // Return the accumulated results
            }, // Closing brace for the Spread arm
            Statement::Pick(pick_expression, es, cc) => {
        
                // If the pick contains only effects (no statements) consider it as a TopLevelEffect kind of pick as this is more intuitive
                let mut only_effects = true;
                let mut top_level_effects = Vec::new();
                for e in es.iter() {
                    if let Statement::Effect(effect) = e {
                        top_level_effects.push(effect.clone());
                    } else {
                        only_effects = false;
                        break
                    }
                }

                if only_effects {
                    return Statement::Effect(TopLevelEffect::Pick(pick_expression, top_level_effects, cc)).expend(val, spread_time, c.clone(), choices.clone(), picks.clone(), choice_vars, pick_vars)
                }

                // Else, handle the pick as a timed pick
                let mut res = Vec::new();
                let (pick_variable, num_pick_variable) = pick_vars.get_variable_and_number();
                for position in 0..es.len() {
                    let new_pick = PickInformation {
                        variable: pick_variable.clone(),
                        position,
                        possibilities: es.len(),
                        expression: *pick_expression.clone(),
                        num_variable: num_pick_variable,
                    };
                    let mut picks = picks.clone();
                    picks.push(new_pick);
                    res.extend(es[position].clone().expend(val, spread_time, cc.clone().update(c.clone()), choices.clone(), picks, choice_vars, pick_vars));
                };
                res
            },
            Statement::Scatter(timing_info, es, cc) => { // Changed duration_opt to timing_info
                let mut res = Vec::new();
                let effective_context = cc.clone().update(c.clone()); 
                let n = es.len() as i64;

                if n == 0 {
                    return res;
                }

                // Determine total duration for scattering directly from TimingInformation
                let total_duration = timing_info.as_frames(spread_time);

                let mut rng = rand::thread_rng(); // Use thread_rng for proper random generation

                for i in 0..n {
                    // Generate a random offset within the total duration
                    let random_offset_factor = rng.r#gen::<f64>(); // Reverted back to r#gen due to linter issues
                    let random_offset = total_duration.mult_by_float(random_offset_factor);
                    let current_time = val.add(&random_offset); // Changed current_offset to current_time for clarity

                    // Create the context for the child (similar to fspread logic if duration was context-based)
                    // The child's frame_duration is a fraction of the total scatter duration.
                    let child_frame_duration = total_duration.divbyint(n);
                    let child_context = BaliContext {
                        frame_duration: Some(child_frame_duration.clone()), // Clone needed here
                        ..effective_context.clone()
                    };

                    // Expand child using the random offset and child context
                    let content: Vec<TimeStatement> = es[i as usize].clone().expend(
                        &current_time, 
                        &child_frame_duration, // Pass child_frame_duration as spread_time
                        child_context, 
                        choices.clone(), 
                        picks.clone(), 
                        choice_vars, 
                        pick_vars
                    );
                    res.extend(content);
                }
                res
            },
            Statement::WithDirt(defaults, stmts) => {
                let mut res = Vec::new();
                // Calculate context for children
                // Start with the incoming context 'c'
                let outer_context = c.clone();

                // Merge the new defaults with any inherited defaults
                let inherited_defaults = outer_context.dirt_defaults.clone().unwrap_or_default();
                let mut merged_defaults = inherited_defaults;
                // The defaults from *this* WithDirt override inherited ones
                for (key, value) in defaults {
                    merged_defaults.insert(key, value);
                }

                // Create the child context, setting the merged defaults
                let child_context = BaliContext {
                    dirt_defaults: Some(merged_defaults),
                    ..outer_context // Inherit other fields (dev, chan, frame_duration, etc.)
                };

                // Expand child statements using the child context
                for stmt in stmts {
                    res.extend(stmt.expend(
                        val, 
                        spread_time, // Pass parent spread_time
                        child_context.clone(), 
                        choices.clone(), 
                        picks.clone(), 
                        choice_vars, 
                        pick_vars
                    ));
                }
                res
            },
        }
    }

}

#[derive(Debug, Clone)]
pub enum TopLevelEffect {
    Seq(Vec<TopLevelEffect>, BaliContext),
    For(Box<BooleanExpression>, Vec<TopLevelEffect>, BaliContext),
    If(Box<BooleanExpression>, Vec<TopLevelEffect>, BaliContext),
    Choice(i64, i64, Vec<TopLevelEffect>, BaliContext),
    Effect(Effect, BaliContext),
    Pick(Box<Expression>, Vec<TopLevelEffect>, BaliContext),
}

impl TopLevelEffect {

    pub fn set_context(self, c: BaliContext) -> TopLevelEffect {
        match self {
            TopLevelEffect::Seq(es, seq_context) => TopLevelEffect::Seq(es, seq_context.update(c)),
            TopLevelEffect::For(cond, es, for_context) => TopLevelEffect::For(cond, es, for_context.update(c)),
            TopLevelEffect::If(cond, es, if_context) => TopLevelEffect::If(cond, es, if_context.update(c)),
            TopLevelEffect::Choice(num_selected, num_selectable, es, choice_context) => TopLevelEffect::Choice(num_selected, num_selectable, es, choice_context.update(c)),
            TopLevelEffect::Pick(position, es, pick_context) => TopLevelEffect::Pick(position, es, pick_context.update(c)),
            TopLevelEffect::Effect(e, effect_context) => TopLevelEffect::Effect(e, effect_context.update(c)),
        }
    }

    pub fn as_asm(&self, context: BaliContext,  local_choice_vars: &mut LocalChoiceVariableGenerator) -> Vec<Instruction> {
        //let time_var = Variable::Instance("_time".to_owned());
        let bvar_out = Variable::Instance("_bres".to_owned());
        match self {
            TopLevelEffect::Seq(s, seq_context) => {
                let mut res = Vec::new();
                let context = seq_context.clone().update(context.clone());
                for i in 0..s.len() {
                    let to_add = s[i].as_asm(context.clone(), local_choice_vars);
                    res.extend(to_add);
                };
                res
            }
            TopLevelEffect::For(e, s, for_context) => {
                let mut res = Vec::new();

                // Compute and add condition
                let condition = e.as_asm();
                res.extend(condition);

                // Add for structure
                res.push(Instruction::Control(ControlASM::Pop(bvar_out.clone())));
                res.push(Instruction::Control(ControlASM::RelJumpIf(bvar_out.clone(), 2)));

                // Compute effects
                let context = for_context.clone().update(context.clone());
                let mut effects = Vec::new();
                for i in 0..s.len() {
                    let to_add = s[i].as_asm(context.clone(), local_choice_vars);
                    effects.extend(to_add);
                };

                // Add for structure (continued)
                let num_effect_instruction = effects.len() as i64;
                res.push(Instruction::Control(ControlASM::RelJump(num_effect_instruction + 2)));
                
                // Add effects
                res.extend(effects);

                // Add for structure (end)
                let num_instructions = res.len() as i64;
                res.push(Instruction::Control(ControlASM::RelJump(- num_instructions)));

                res
            },
            TopLevelEffect::If(e, s, if_context) => {
                let mut res = Vec::new();

                // Compute and add condition
                let condition = e.as_asm();
                res.extend(condition);

                // Add if structure
                res.push(Instruction::Control(ControlASM::Pop(bvar_out.clone())));
                res.push(Instruction::Control(ControlASM::RelJumpIf(bvar_out.clone(), 2)));

                // Compute effects
                let context = if_context.clone().update(context.clone());
                let mut effects = Vec::new();
                for i in 0..s.len() {
                    let to_add = s[i].as_asm(context.clone(), local_choice_vars);
                    effects.extend(to_add);
                };

                // Add if structure (continued)
                let num_effect_instruction = effects.len() as i64;
                res.push(Instruction::Control(ControlASM::RelJump(num_effect_instruction + 1)));
                
                // Add effects
                res.extend(effects);

                res
            }
            TopLevelEffect::Choice(num_selected, num_selectable, es, choice_context) => {

                let mut res = Vec::new();

                // If nothing is selected, generate no instructions
                let num_selected = *num_selected;
                if num_selected <= 0 {
                    return res
                }

                // If something in es cannot be selected, make it selectable
                let num_selectable = if *num_selectable < es.len() as i64 {
                    es.len() as i64
                } else {
                    *num_selectable
                };

                // If everything will be selected
                if num_selected >= num_selectable {
                    return TopLevelEffect::Seq(es.clone(), choice_context.clone()).as_asm(context, local_choice_vars)
                }

                // An actual selection will occur
                let mut choice_vars = Vec::new();
                let context = choice_context.clone().update(context.clone());

                // generate random values for the choice
                for selection_number in 0..num_selected {
                    let choice_variable = local_choice_vars.get_variable();
                    res.push(Instruction::Control(ControlASM::Mov(Variable::Environment(EnvironmentFunc::RandomUInt((num_selectable - selection_number) as u64)), choice_variable.clone())));
                    //position += 1;
                    choice_vars.push(choice_variable);
                }


                // generate the code for each effect in the set es
                for effect_pos in 0..es.len() {

                    // init targe variable to set effect position as selection value
                    res.push(Instruction::Control(ControlASM::Mov((effect_pos as i64).into(), LOCAL_TARGET_VAR.clone())));

                    // handle each possible choice for this effect
                    let num_instruction_for_first_choice = 1;
                    let num_instruction_for_other_choices = if effect_pos == 0 {
                        1
                    } else {
                        3
                    };
                    let num_instruction_between_choices_and_effects = 1;
                    let mut distance_to_effect = num_instruction_for_first_choice + num_instruction_for_other_choices * (num_selected - 1) + num_instruction_between_choices_and_effects;
                    for choice_number in 0..num_selected {

                        distance_to_effect = if choice_number == 0 {
                            distance_to_effect - num_instruction_for_first_choice
                        } else {
                            distance_to_effect - num_instruction_for_other_choices
                        };
                        
                        if choice_number > 0 && effect_pos > 0 {
                            res.push(Instruction::Control(ControlASM::RelJumpIfLessOrEqual( LOCAL_TARGET_VAR.clone(), choice_vars[choice_number as usize -1].clone(), 2)));
                            res.push(Instruction::Control(ControlASM::Sub(LOCAL_TARGET_VAR.clone(), 1.into(), LOCAL_TARGET_VAR.clone())));
                        }

                        res.push(Instruction::Control(ControlASM::RelJumpIfEqual(LOCAL_TARGET_VAR.clone(), choice_vars[choice_number as usize].clone(), distance_to_effect + 1)))
                    }

                    // jump over effects if the choice don't select them
                    let effect_prog = es[effect_pos].as_asm(context.clone(), local_choice_vars);
                    res.push(Instruction::Control(ControlASM::RelJump((effect_prog.len() + 1) as i64)));

                    // add the actual effects
                    res.extend(effect_prog);


                }

                res
            },
            TopLevelEffect::Pick(position, es, pick_context) => {

                // get context
                let context = pick_context.clone().update(context.clone());

                // compute the position
                let mut res = position.as_asm();
                res.push(Instruction::Control(ControlASM::Pop(LOCAL_PICK_VAR.clone())));
                res.push(Instruction::Control(ControlASM::Add(LOCAL_PICK_VAR.clone(), (es.len() as i64).into(), LOCAL_PICK_VAR.clone())));
                res.push(Instruction::Control(ControlASM::Sub(LOCAL_PICK_VAR.clone(), 1.into(), LOCAL_PICK_VAR.clone())));
                res.push(Instruction::Control(ControlASM::Mod(LOCAL_PICK_VAR.clone(), (es.len() as i64).into(), LOCAL_PICK_VAR.clone())));

                let mut effect_progs = Vec::new();

                // jump to the position
                let mut effect_number = 0;
                let num_pick_instruction_per_step = 1;
                let num_pick_instructions = (es.len() as i64) * num_pick_instruction_per_step;
                let mut distance_to_effect = num_pick_instructions - num_pick_instruction_per_step;
                let mut distance_to_end = 0;
                for e in es.iter() {
                    effect_progs.push(e.as_asm(context.clone(), local_choice_vars));
                    let new_effect_len = effect_progs[effect_number as usize].len() as i64 + 1; // +1 for the jumps that will be added later
                    distance_to_end += new_effect_len;

                    res.push(Instruction::Control(ControlASM::RelJumpIfEqual(LOCAL_PICK_VAR.clone(), effect_number.into(), distance_to_effect + 1)));

                    distance_to_effect = distance_to_effect - num_pick_instruction_per_step + new_effect_len; // +1 for the jumps after the effects

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
            },
            TopLevelEffect::Effect(ef, effect_context) => {
                let context = effect_context.clone().update(context.clone());
                ef.as_asm(context)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Effect {
    Definition(Value, Box<Expression>),
    Note(Box<Expression>, BaliContext),
    ProgramChange(Box<Expression>, BaliContext),
    ControlChange(Box<Expression>, Box<Expression>, BaliContext),
    Osc(String, Vec<Expression>, BaliContext),
    Dirt(Box<Expression>, Vec<(String, Fraction)>, BaliContext), // Changed Box<Expression> to Fraction
    Aftertouch(Box<Expression>, Box<Expression>, BaliContext),
    ChannelPressure(Box<Expression>, BaliContext),
}

impl Effect { // TODO : on veut que les durées soient des fractions
    pub fn as_asm(&self, context: BaliContext) -> Vec<Instruction> {
        //let time_var = Variable::Instance("_time".to_owned());
        let note_var = Variable::Instance("_note".to_owned());
        let velocity_var = Variable::Instance("_velocity".to_owned());
        let chan_var = Variable::Instance("_chan".to_owned());
        let duration_var = Variable::Instance("_duration".to_owned());
        let duration_time_var = Variable::Instance("_duration_time".to_owned());
        let program_var = Variable::Instance("_program".to_owned());
        let control_var = Variable::Instance("_control".to_owned());
        let value_var = Variable::Instance("_control_value".to_owned());
        let target_device_id_var = Variable::Instance("_target_device_id".to_string());

        let mut res = Vec::new();
        //let mut res = vec![Instruction::Control(ControlASM::FloatAsFrames(delay.into(), time_var.clone()))];
        
        match self {
            Effect::Definition(v, expr) => {
                res.extend(expr.as_asm());
                if let Value::Variable(v) = v {
                    res.push(Instruction::Control(ControlASM::Pop(Value::as_variable(v))));
                }
            },
            Effect::Note(n, c) => {
                let context = c.clone().update(context);
                res.extend(n.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(note_var.clone())));
                
                if let Some(v) = context.velocity {
                    res.extend(v.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(velocity_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_VELOCITY.into(), velocity_var.clone())))
                }
                
                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_CHAN.into(), chan_var.clone())))
                }
                
                if let Some(d) = context.duration {
                    res.extend(d.as_asm());
                } else {
                    res.extend(Fraction{
                        numerator: Box::new(Expression::Value(Value::Number(1))),
                        denominator: Box::new(Expression::Value(Value::Number(DEFAULT_DURATION))),
                    }.as_asm());
                }
                res.push(Instruction::Control(ControlASM::Pop(duration_var.clone())));
                res.push(Instruction::Control(ControlASM::FloatAsFrames(duration_var.clone(), duration_time_var.clone())));

                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(target_device_id_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_DEVICE.into(), target_device_id_var.clone())));
                }

                res.push(Instruction::Effect(Event::MidiNote(
                    note_var.clone(), velocity_var.clone(), chan_var.clone(),
                    duration_time_var.clone(), 
                    target_device_id_var.clone()
                ), 0.0.into()));
            },
            Effect::ProgramChange(p, c) => {
                let context = c.clone().update(context);
                res.extend(p.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(program_var.clone())));
                
                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_CHAN.into(), chan_var.clone())))
                }
                
                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(target_device_id_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_DEVICE.into(), target_device_id_var.clone())));
                }

                res.push(Instruction::Effect(Event::MidiProgram(
                    program_var.clone(), chan_var.clone(),
                    target_device_id_var.clone()
                ), 0.0.into()));
            },
            Effect::ControlChange(con, v, c) => {
                let context = c.clone().update(context);
                res.extend(con.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(control_var.clone())));
                res.extend(v.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(value_var.clone())));
                
                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_CHAN.into(), chan_var.clone())))
                }

                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(target_device_id_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_DEVICE.into(), target_device_id_var.clone())));
                }
                
                res.push(Instruction::Effect(Event::MidiControl(
                    control_var.clone(), value_var.clone(), chan_var.clone(),
                    target_device_id_var.clone()
                ), 0.0.into()));
            },
            Effect::Osc(addr, args, osc_context) => {
                let context = osc_context.clone().update(context);
                let target_device_id_var = Variable::Instance("_target_device_id".to_string());
                let mut osc_args: Vec<OscArgument> = Vec::new();
                let mut arg_instrs: Vec<Instruction> = Vec::new();

                // Generate instructions to evaluate dynamic arguments first
                // and store them in temporary variables.
                let mut temp_arg_vars: Vec<Variable> = Vec::new();
                for (i, arg_expr) in args.iter().enumerate() {
                    match arg_expr {
                        Expression::Value(Value::Number(_)) | Expression::Value(Value::String(_)) | Expression::Value(Value::Variable(_)) => {
                            // Literal or variable - handled below
                        }
                        _ => {
                            // Dynamic expression - evaluate it
                            let temp_var = Variable::Instance(format!("_osc_arg_{}", i));
                            arg_instrs.extend(arg_expr.as_asm());
                            arg_instrs.push(Instruction::Control(ControlASM::Pop(temp_var.clone())));
                            temp_arg_vars.push(temp_var);
                        }
                    }
                }
                res.extend(arg_instrs); // Add evaluation instructions

                // Determine target device ID
                if let Some(device_id_expr) = context.device {
                    res.extend(device_id_expr.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(target_device_id_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_DEVICE.into(), target_device_id_var.clone())));
                }

                // Build the OSC argument list directly
                let mut temp_var_idx = 0;
                for arg_expr in args.iter() {
                    match arg_expr {
                        Expression::Value(Value::Number(n)) => osc_args.push(OscArgument::Int(*n as i32)),
                        Expression::Value(Value::String(s)) => osc_args.push(OscArgument::String(s.clone())),
                        Expression::Value(Value::Variable(_)) => {
                            // Assume variable holds a number (int/float?) - treat as float for now
                            // This requires the Variable to be evaluated and pushed beforehand, which is complex.
                            // For now, let's treat simple variables like numbers if they represent notes.
                            // Or perhaps error out?
                            // Simplest: Treat as Int 0 for now if it's not a known note.
                            let val_as_var = if let Expression::Value(Value::Variable(var_name)) = arg_expr {
                                Value::as_variable(var_name)
                            } else { unreachable!() }; // Should be Variable

                            // We need to PUSH the variable value here!
                             res.push(Instruction::Control(ControlASM::Push(val_as_var.clone())));
                             let temp_var_for_var = Variable::Instance(format!("_osc_arg_var_{}", temp_var_idx));
                             temp_var_idx += 1;
                             res.push(Instruction::Control(ControlASM::Pop(temp_var_for_var.clone())));
                             // This variable now holds the value, but we can't easily get it back here
                             // to put into osc_args without complex VM interaction.
                             // Limitation: For now, only literal numbers/strings or pre-evaluated expressions work.
                             // Let's add a placeholder Float(0.0) and log a warning.
                             let var_name_str = match &val_as_var {
                                Variable::Global(name) => name.clone(),
                                Variable::Instance(name) => name.clone(),
                                Variable::Environment(func) => format!("Env::{:?}", func),
                                Variable::Line(name) => name.clone(), // Add Line
                                Variable::Frame(name) => name.clone(), // Add Frame
                                Variable::Constant(value) => format!("Const({:?})", value), // Format Constant value
                             };
                             eprintln!("[WARN] Bali OSC: Cannot directly use unevaluated variable '{}' as OSC argument. Using 0.0f32.", var_name_str);
                            osc_args.push(OscArgument::Float(0.0));
                        }
                        _ => {
                            // Dynamic expression: Use the pre-calculated temp variable
                            // We assume it's numeric (float). This is a limitation.
                            // We need to push the temp var back to the stack to use it in the Effect
                            // This is getting complicated. Let's simplify: only literal args for now.
                            eprintln!("[WARN] Bali OSC: Cannot use complex expression as OSC argument yet. Skipping.");
                            // For now, skip complex expressions
                            // temp_var_idx += 1; // Increment even if skipped?
                             // Instead of skipping, let's use the temp var we calculated
                             // Assume the temp var contains a float value
                             let _temp_var = temp_arg_vars.remove(0); // Get the corresponding temp var
                             // We can't directly get the f32 value here easily.
                             // Let's push a placeholder float.
                             osc_args.push(OscArgument::Float(0.0)); // Placeholder
                             eprintln!("[WARN] Bali OSC: Using placeholder 0.0f32 for dynamic expression argument.");
                        }
                    }
                }

                // Construct the OSC message
                let message = OSCMessage {
                    addr: addr.clone(),
                    args: osc_args,
                };

                // Create the Event::Osc (not ConcreteEvent)
                let event = Event::Osc {
                    message,
                    device_id: target_device_id_var.clone(), // Event::Osc takes Variable
                };

                // Add the final effect instruction using the event directly
                res.push(Instruction::Effect(event, 0.0.into())); 

                // Note: The current implementation for non-literal arguments is limited.
                // It pushes placeholders (0.0) due to difficulty retrieving evaluated values
                // from temporary variables back into this compile-time context.
                // A cleaner solution would involve extending the VM or event structure.
            },
            Effect::Dirt(sound_expr, explicit_params, dirt_context) => {
                let context = dirt_context.clone().update(context);
                let target_device_id_var = Variable::Instance("_target_device_id".to_string());
                let dirt_data_var = Variable::Instance("_dirt_data".to_string());
                let mut eval_instrs: Vec<Instruction> = Vec::new();

                // --- Instructions to build the data map ---
                // 1. Create an empty map variable
                let map_init_var = Variable::Instance("_dirt_map_init".to_string());
                eval_instrs.push(Instruction::Control(ControlASM::MapEmpty(map_init_var.clone())));

                // 2. Evaluate sound expression and add as "s"
                let sound_value_var = Variable::Instance("_dirt_sound_val".to_string());
                // --- Start Sound Handling Fix (Restored from previous version) ---
                match **sound_expr { // Dereference Box<Expression>
                    Expression::Value(Value::String(ref s)) => {
                        // Sound is a literal string, insert it directly
                        let string_const_var = Variable::Constant(VariableValue::Str(s.clone()));
                        eval_instrs.push(Instruction::Control(ControlASM::MapInsert(
                            map_init_var.clone(),
                            VariableValue::Str("s".to_string()), // Key "s"
                            string_const_var, // Pass the Constant Variable holding the string
                            map_init_var.clone() // Store back in the same map var
                        )));
                    }
                    _ => {
                        // Sound is a variable or complex expression, evaluate it
                        eval_instrs.extend(sound_expr.as_asm());
                        eval_instrs.push(Instruction::Control(ControlASM::Pop(sound_value_var.clone())));
                        eval_instrs.push(Instruction::Control(ControlASM::MapInsert(
                            map_init_var.clone(),
                            VariableValue::Str("s".to_string()), // Key "s"
                            sound_value_var, // Value (Variable holding evaluated sound)
                            map_init_var.clone() // Store back in the same map var
                        )));
                    }
                }
                // --- End Sound Handling Fix ---

                // --- Merge explicit params with context defaults ---
                let context_defaults = context.dirt_defaults.clone().unwrap_or_default();
                let mut final_params = context_defaults; // Start with defaults
                // Explicit params override defaults
                for (key, value_frac) in explicit_params.iter() {
                    final_params.insert(key.clone(), value_frac.clone());
                }
                // --- End merging --- 

                // 3. Evaluate parameters and add to map (use final_params)
                for (key, value_frac) in final_params.iter() { // Iterate over the final merged map
                    let param_value_var = Variable::Instance(format!("_dirt_param_{}_val", key));
                    eval_instrs.extend(value_frac.as_asm()); // Use Fraction's as_asm
                    eval_instrs.push(Instruction::Control(ControlASM::Pop(param_value_var.clone())));
                    eval_instrs.push(Instruction::Control(ControlASM::MapInsert(
                        map_init_var.clone(),
                        VariableValue::Str(key.clone()), // Key
                        param_value_var, // Value (Variable holding evaluated param)
                        map_init_var.clone() // Store back
                    )));
                }
                // --- End map building ---

                // 4. Push the final map onto the stack and pop into dirt_data_var
                eval_instrs.push(Instruction::Control(ControlASM::Push(map_init_var.clone())));
                eval_instrs.push(Instruction::Control(ControlASM::Pop(dirt_data_var.clone())));

                // 5. Evaluate device context
                if let Some(device_id_expr) = context.device {
                    eval_instrs.extend(device_id_expr.as_asm());
                    eval_instrs.push(Instruction::Control(ControlASM::Pop(target_device_id_var.clone())));
                } else {
                    eval_instrs.push(Instruction::Control(ControlASM::Mov(DEFAULT_DEVICE.into(), target_device_id_var.clone())));
                }

                // Add evaluation instructions first
                res.extend(eval_instrs);

                // 6. Create Event::Dirt using the variables holding the map and device ID
                let event = Event::Dirt {
                    data: dirt_data_var, // Variable holding the map
                    device_id: target_device_id_var, // Variable holding the device ID
                };

                // 7. Add the final effect instruction
                res.push(Instruction::Effect(event, 0.0.into()));
            },
            Effect::Aftertouch(note_expr, value_expr, c) => {
                let context = c.clone().update(context);
                let note_var = Variable::Instance("_at_note".to_owned());
                let value_var = Variable::Instance("_at_value".to_owned());

                res.extend(note_expr.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(note_var.clone())));
                res.extend(value_expr.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(value_var.clone())));

                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_CHAN.into(), chan_var.clone())))
                }

                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(target_device_id_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_DEVICE.into(), target_device_id_var.clone())));
                }

                res.push(Instruction::Effect(Event::MidiAftertouch(
                    note_var, value_var, chan_var.clone(),
                    target_device_id_var.clone()
                ), 0.0.into()));
            },
            Effect::ChannelPressure(value_expr, c) => {
                let context = c.clone().update(context);
                let value_var = Variable::Instance("_chanpress_value".to_owned());

                res.extend(value_expr.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(value_var.clone())));

                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_CHAN.into(), chan_var.clone())))
                }

                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(target_device_id_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_DEVICE.into(), target_device_id_var.clone())));
                }

                res.push(Instruction::Effect(Event::MidiChannelPressure(
                    value_var, chan_var.clone(),
                    target_device_id_var.clone()
                ), 0.0.into()));
            },
        }

        res
    }
}

#[derive(Debug, Clone)]
pub enum BooleanExpression {
    And(Box<BooleanExpression>, Box<BooleanExpression>),
    Or(Box<BooleanExpression>, Box<BooleanExpression>),
    Not(Box<BooleanExpression>),
    Lower(Box<Expression>, Box<Expression>),
    LowerOrEqual(Box<Expression>, Box<Expression>),
    Greater(Box<Expression>, Box<Expression>),
    GreaterOrEqual(Box<Expression>, Box<Expression>),
    Equal(Box<Expression>, Box<Expression>),
    Different(Box<Expression>, Box<Expression>),
}

impl BooleanExpression {
    pub fn as_asm(&self) -> Vec<Instruction> {
        let bvar_1 = Variable::Instance("_bexp1".to_owned());
        let bvar_2 = Variable::Instance("_bexp2".to_owned());
        let evar_1 = Variable::Instance("_exp1".to_owned());
        let evar_2 = Variable::Instance("_exp2".to_owned());
        let bvar_out = Variable::Instance("_bres".to_owned());
        let mut res = match self {
            BooleanExpression::And(e1, e2) | BooleanExpression::Or(e1, e2) => {
                let mut e1 = e1.as_asm();
                e1.extend(e2.as_asm());
                e1.push(Instruction::Control(ControlASM::Pop(bvar_2.clone())));
                e1.push(Instruction::Control(ControlASM::Pop(bvar_1.clone())));
                e1
            },
            BooleanExpression::Not(e) => {
                let mut e = e.as_asm();
                e.push(Instruction::Control(ControlASM::Pop(bvar_1.clone())));
                e
            }
            BooleanExpression::Lower(e1, e2) | BooleanExpression::LowerOrEqual(e1, e2) | BooleanExpression::Greater(e1, e2) | BooleanExpression::GreaterOrEqual(e1, e2) | BooleanExpression::Equal(e1, e2) | BooleanExpression::Different(e1, e2) => {
                let mut e1 = e1.as_asm();
                e1.extend(e2.as_asm());
                e1.push(Instruction::Control(ControlASM::Pop(evar_2.clone())));
                e1.push(Instruction::Control(ControlASM::Pop(evar_1.clone())));
                e1
            }
        };
        match self {
            BooleanExpression::And(_, _) => {
                res.push(Instruction::Control(ControlASM::And(bvar_1.clone(), bvar_2.clone(), bvar_out.clone())));
            },
            BooleanExpression::Or(_, _) => {
                res.push(Instruction::Control(ControlASM::Or(bvar_1.clone(), bvar_2.clone(), bvar_out.clone())));
            },
            BooleanExpression::Not(_) => {
                res.push(Instruction::Control(ControlASM::Not(bvar_1.clone(), bvar_out.clone())));
            },
            BooleanExpression::Lower(_, _) => {
               res.push(Instruction::Control(ControlASM::LowerThan(evar_1.clone(), evar_2.clone(), bvar_out.clone())))
            },
            BooleanExpression::LowerOrEqual(_, _) => {
                res.push(Instruction::Control(ControlASM::LowerOrEqual(evar_1.clone(), evar_2.clone(), bvar_out.clone())))
            },
            BooleanExpression::Greater(_, _) => {
                res.push(Instruction::Control(ControlASM::GreaterThan(evar_1.clone(), evar_2.clone(), bvar_out.clone())))
            },
            BooleanExpression::GreaterOrEqual(_, _) => {
                res.push(Instruction::Control(ControlASM::GreaterOrEqual(evar_1.clone(), evar_2.clone(), bvar_out.clone())))
            },
            BooleanExpression::Equal(_, _) => {
                res.push(Instruction::Control(ControlASM::Equal(evar_1.clone(), evar_2.clone(), bvar_out.clone())))
            },
            BooleanExpression::Different(_, _) => {
                res.push(Instruction::Control(ControlASM::Different(evar_1.clone(), evar_2.clone(), bvar_out.clone())))
            },
        };

        res.push(Instruction::Control(ControlASM::Push(bvar_out.clone())));
        res
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Subtraction(Box<Expression>, Box<Expression>),
    Division(Box<Expression>, Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),
    Scale(Box<Expression>, Box<Expression>, Box<Expression>, Box<Expression>, Box<Expression>), // value, old_min, old_max, new_min, new_max
    Clamp(Box<Expression>, Box<Expression>, Box<Expression>), // value, min, max
    Min(Box<Expression>, Box<Expression>),
    Max(Box<Expression>, Box<Expression>),
    Quantize(Box<Expression>, Box<Expression>), // value, step
    Sine(Box<Expression>), // speed
    Saw(Box<Expression>), // speed
    Triangle(Box<Expression>), // speed
    ISaw(Box<Expression>), // speed (inverted saw)
    RandStep(Box<Expression>), // speed (random step LFO)
    MidiCC(Box<Expression>, Option<Box<Expression>>, Option<Box<Expression>>), 
    Value(Value),
}

impl Expression {
    pub fn as_asm(&self) -> Vec<Instruction> {
        // Standard temporary variables for expression evaluation
        let var_1 = Variable::Instance("_exp1".to_owned());
        let var_2 = Variable::Instance("_exp2".to_owned());
        let var_3 = Variable::Instance("_exp3".to_owned());
        let var_4 = Variable::Instance("_exp4".to_owned());
        let var_5 = Variable::Instance("_exp5".to_owned());
        let speed_var = Variable::Instance("_osc_speed".to_owned());
        let var_out = Variable::Instance("_res".to_owned());

        let mut res_asm = match self {
            // Binary operations: Evaluate operands, pop into temps, execute operation into var_out
            Expression::Addition(e1, e2)
            | Expression::Multiplication(e1, e2)
            | Expression::Subtraction(e1, e2)
            | Expression::Division(e1, e2)
            | Expression::Modulo(e1, e2)
            | Expression::Min(e1, e2)
            | Expression::Max(e1, e2)
            | Expression::Quantize(e1, e2) => {
                let mut asm = e1.as_asm();
                asm.extend(e2.as_asm());
                asm.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                asm.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                match self {
                    Expression::Addition(_,_) => asm.push(Instruction::Control(ControlASM::Add(var_1.clone(), var_2.clone(), var_out.clone()))),
                    Expression::Multiplication(_,_) => asm.push(Instruction::Control(ControlASM::Mul(var_1.clone(), var_2.clone(), var_out.clone()))),
                    Expression::Subtraction(_,_) => asm.push(Instruction::Control(ControlASM::Sub(var_1.clone(), var_2.clone(), var_out.clone()))),
                    Expression::Division(_,_) => asm.push(Instruction::Control(ControlASM::Div(var_1.clone(), var_2.clone(), var_out.clone()))),
                    Expression::Modulo(_,_) => asm.push(Instruction::Control(ControlASM::Mod(var_1.clone(), var_2.clone(), var_out.clone()))),
                    Expression::Min(_,_) => asm.push(Instruction::Control(ControlASM::Min(var_1.clone(), var_2.clone(), var_out.clone()))),
                    Expression::Max(_,_) => asm.push(Instruction::Control(ControlASM::Max(var_1.clone(), var_2.clone(), var_out.clone()))),
                    Expression::Quantize(_,_) => asm.push(Instruction::Control(ControlASM::Quantize(var_1.clone(), var_2.clone(), var_out.clone()))),
                    _ => unreachable!(), // Should not happen due to outer match
                }
                asm
            },
            Expression::Scale(val, old_min, old_max, new_min, new_max) => {
                let mut asm = val.as_asm();
                asm.extend(old_min.as_asm());
                asm.extend(old_max.as_asm());
                asm.extend(new_min.as_asm());
                asm.extend(new_max.as_asm());
                asm.push(Instruction::Control(ControlASM::Pop(var_5.clone())));
                asm.push(Instruction::Control(ControlASM::Pop(var_4.clone())));
                asm.push(Instruction::Control(ControlASM::Pop(var_3.clone())));
                asm.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                asm.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                asm.push(Instruction::Control(ControlASM::Scale(var_1.clone(), var_2.clone(), var_3.clone(), var_4.clone(), var_5.clone(), var_out.clone())));
                asm
            }
            Expression::Clamp(val, min, max) => {
                let mut asm = val.as_asm();
                asm.extend(min.as_asm());
                asm.extend(max.as_asm());
                asm.push(Instruction::Control(ControlASM::Pop(var_3.clone())));
                asm.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                asm.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                asm.push(Instruction::Control(ControlASM::Clamp(var_1.clone(), var_2.clone(), var_3.clone(), var_out.clone())));
                asm
            }
            Expression::Sine(speed_expr)
            | Expression::Saw(speed_expr)
            | Expression::Triangle(speed_expr)
            | Expression::ISaw(speed_expr)
            | Expression::RandStep(speed_expr) => {
                let mut asm = speed_expr.as_asm();
                asm.push(Instruction::Control(ControlASM::Pop(speed_var.clone())));
                 match self {
                    Expression::Sine(_) => asm.push(Instruction::Control(ControlASM::GetSine(speed_var.clone(), var_out.clone()))),
                    Expression::Saw(_) => asm.push(Instruction::Control(ControlASM::GetSaw(speed_var.clone(), var_out.clone()))),
                    Expression::Triangle(_) => asm.push(Instruction::Control(ControlASM::GetTriangle(speed_var.clone(), var_out.clone()))),
                    Expression::ISaw(_) => asm.push(Instruction::Control(ControlASM::GetISaw(speed_var.clone(), var_out.clone()))),
                    Expression::RandStep(_) => asm.push(Instruction::Control(ControlASM::GetRandStep(speed_var.clone(), var_out.clone()))),
                    _ => unreachable!(),
                }
                asm
            }
            // MidiCC: Evaluate control expression, pop into midi_cc_ctrl_var, execute GetMidiCCFromContext into var_out
            Expression::MidiCC(ctrl_expr, device_expr_opt, channel_expr_opt) => {
                let mut asm = Vec::new();
                // Temporary variables for specific CC lookup, used only if provided
                let ccin_device_id_var = Variable::Instance("_ccin_device_id".to_owned());
                let ccin_chan_var = Variable::Instance("_ccin_chan".to_owned());
                // Variable for control number
                let ccin_ctrl_var = Variable::Instance("_ccin_ctrl".to_owned());
                // Placeholder variables to signal using context
                let use_context_device_var = Variable::Instance("_use_context_device".to_owned());
                let use_context_channel_var = Variable::Instance("_use_context_channel".to_owned());

                // 1. Evaluate the control number expression first
                asm.extend(ctrl_expr.as_asm());
                asm.push(Instruction::Control(ControlASM::Pop(ccin_ctrl_var.clone())));

                // 2. Determine and evaluate Device Variable
                let device_var_to_pass = if let Some(device_expr) = device_expr_opt {
                    // Evaluate specific device expression
                    asm.extend(device_expr.as_asm());
                    asm.push(Instruction::Control(ControlASM::Pop(ccin_device_id_var.clone())));
                    ccin_device_id_var // Pass the variable holding the evaluated result
                } else {
                    use_context_device_var // Pass the placeholder to signal using context
                };

                // 3. Determine and evaluate Channel Variable
                let channel_var_to_pass = if let Some(channel_expr) = channel_expr_opt {
                    // Evaluate specific channel expression
                    asm.extend(channel_expr.as_asm());
                    asm.push(Instruction::Control(ControlASM::Pop(ccin_chan_var.clone())));
                    ccin_chan_var // Pass the variable holding the evaluated result
                } else {
                    use_context_channel_var // Pass the placeholder to signal using context
                };

                // 4. Always generate the single GetMidiCC instruction
                asm.push(Instruction::Control(ControlASM::GetMidiCC(
                    device_var_to_pass,  // Either specific var or context placeholder
                    channel_var_to_pass, // Either specific var or context placeholder
                    ccin_ctrl_var.clone(),
                    var_out.clone()      // Standard result variable
                )));

                asm
            }
            Expression::Value(v) => {
                vec![
                    v.as_asm(), // Push the value onto stack
                    Instruction::Control(ControlASM::Pop(var_out.clone())) // Pop it into the result variable
                ]
            }
        };

        // Common final step for all expressions: Push the computed result (`var_out`)
        // onto the stack for the *next* operation or effect to use.
        res_asm.push(Instruction::Control(ControlASM::Push(var_out.clone())));
        res_asm
    }
}

#[derive(Debug, Clone)]
pub struct ConcreteFraction {
    pub signe: i64,
    pub numerator: i64,
    pub denominator: i64,
} 

impl ConcreteFraction {

    pub fn from_dec_string(dec: String) -> ConcreteFraction {
        let parts: Vec<&str> = dec.split('.').collect();
        let int_part = match parts[0].parse::<i64>() {
            Ok(n) => n,
            Err(_) => 0,
        };
        let dec_part = match parts[1].parse::<i64>() {
            Ok(n) => n,
            Err(_) => 0,
        };
        let num_dec = parts[1].len();
        let mut denominator = 1;
        for _i in 0..num_dec {
            denominator = denominator * 10;
        }
        let numerator = int_part * denominator + dec_part;
        ConcreteFraction{
            signe: 1,
            numerator,
            denominator,
        }.simplify()
    }

    pub fn tof64(&self) -> f64 {
        (self.signe * self.numerator) as f64 / self.denominator as f64
    }

    pub fn add(&self, other: &Self) -> ConcreteFraction {
        ConcreteFraction{
            signe: 1,
            numerator: self.signe * self.numerator * other.denominator + other.signe * other.numerator * self.denominator,
            denominator: self.denominator * other.denominator,
        }.simplify()
    }

    pub fn sub(&self, other: &Self) -> ConcreteFraction {
        ConcreteFraction{
            signe: 1,
            numerator: self.signe * self.numerator * other.denominator - other.signe * other.numerator * self.denominator,
            denominator: self.denominator * other.denominator,
        }.simplify()
    }

    pub fn mult(&self, other: &Self) -> ConcreteFraction {
        ConcreteFraction{
            signe: 1,
            numerator: self.signe * self.numerator * other.signe * other.numerator,
            denominator: self.denominator * other.denominator,
        }.simplify()
    }

    pub fn multbyint(&self, mult: i64) -> ConcreteFraction {
        ConcreteFraction{
            signe: 1,
            numerator: self.signe * self.numerator * mult,
            denominator: self.denominator,
        }.simplify()
    }

    pub fn divbyint(&self, div: i64) -> ConcreteFraction {
        ConcreteFraction{
            signe: 1,
            numerator: self.signe * self.numerator,
            denominator: self.denominator * div,
        }
    }

    fn simplify(&self) -> ConcreteFraction {
        let signe = if self.numerator * self.denominator < 0 {
            -1
        } else {
            1
        };
        let numerator = if self.numerator < 0 {
            -self.numerator
        } else {
            self.numerator
        };
        let denominator = if self.denominator < 0 {
            -self.denominator
        } else {
            self.denominator
        };
        let gcd = Self::gcd(numerator, denominator);
        let numerator = numerator / gcd;
        let denominator = denominator / gcd;
        ConcreteFraction{
            signe,
            numerator,
            denominator,
        }
    }

    fn gcd(a: i64, b: i64) -> i64 {
        let mut max = if a > b {
            a
        } else {
            b
        };

        let mut min = if a > b {
            b
        } else {
            a
        };

        while min != 0 {
            let r = max % min;
            max = min;
            min = r;
        };

        max
    }

    pub fn mult_by_float(&self, factor: f64) -> ConcreteFraction {
        // This is tricky due to potential precision loss. 
        // A simple approach is to convert to f64, multiply, then convert back.
        // More robust might involve large integer math or libraries like `num`.
        let current_f64 = self.tof64();
        let result_f64 = current_f64 * factor;
        
        // Simple conversion back from f64 - might lose precision for complex fractions
        // You might want a more sophisticated f64 -> Fraction conversion later.
        const MAX_DENOMINATOR: i64 = 1000000; // Limit denominator size
        let mut h1 = 1; let mut k1 = 0; // Make mutable
        let mut h2 = 0; let mut k2 = 1; // Make mutable
        let mut b = result_f64;
        loop {
            let a = b.floor();
            let aux = h1; h1 = a as i64 * h1 + h2; h2 = aux;
            let aux = k1; k1 = a as i64 * k1 + k2; k2 = aux;
            if (result_f64 - a) < 1.0e-8 || k1 > MAX_DENOMINATOR {
                break;
            }
            b = 1.0 / (b - a);
        }
        ConcreteFraction { signe: 1, numerator: h1, denominator: k1 }.simplify()
    }

}

#[derive(Debug, Clone)]
pub struct Fraction {
    pub numerator: Box<Expression>,
    pub denominator: Box<Expression>,
} 

impl Fraction {

    pub fn from_dec_string(dec: String) -> Fraction {
        let concrete = ConcreteFraction::from_dec_string(dec);
        Fraction{
            numerator: Box::new(Expression::Value(Value::Number(concrete.numerator))), 
            denominator: Box::new(Expression::Value(Value::Number(concrete.denominator)))
        }
    }

    pub fn as_asm(&self) -> Vec<Instruction> {
        let var_1 = Variable::Instance("_exp1_frac".to_owned());
        let var_2 = Variable::Instance("_exp2_frac".to_owned());
        let var_out = Variable::Instance("_res_frac".to_owned());
        let mut e1 = vec![
            Instruction::Control(ControlASM::Mov(0.0.into(), var_1.clone())),
            Instruction::Control(ControlASM::Mov(0.0.into(), var_2.clone())),
            Instruction::Control(ControlASM::Mov(0.0.into(), var_out.clone())),
        ];
        e1.extend(self.numerator.as_asm());
        e1.extend(self.denominator.as_asm());
        e1.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
        e1.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
        e1.push(Instruction::Control(ControlASM::Div(var_1.clone(), var_2.clone(), var_out.clone())));
        e1.push(Instruction::Control(ControlASM::Push(var_out.clone())));
        e1
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Variable(String),
    String(String), // Add String variant
}


impl Value {

    pub fn as_asm(&self) -> Instruction {
        match self {
            Value::Number(n) => Instruction::Control(ControlASM::Push((*n).into())),
            Value::Variable(s) => {
                match Self::as_note(s) {
                    None => Instruction::Control(ControlASM::Push(Self::as_variable(s))),
                    Some(n) => Instruction::Control(ControlASM::Push((*n).into())),
                }
            },
            Value::String(_s) => {
                // Pushing strings directly to the numeric/variable stack is problematic.
                // For the OSC command, we handle Value::String directly in Effect::as_asm.
                // If strings need general stack support, the VM/VariableType needs extension.
                // For now, generate a Nop or error if String is used outside OSC?
                // Let's generate a Push of 0 as a placeholder, assuming it won't be used elsewhere yet.
                eprintln!("[WARN] Bali VM: Pushing String as 0 to stack (Value::as_asm). String support is limited.");
                Instruction::Control(ControlASM::Push(0i64.into()))
            },
        }
    }

    pub fn _tostr(&self) -> String {
        match self {
            Value::Variable(s) => s.to_string(),
            _ => unreachable!(),
        }
    }

    pub fn as_note(name: &String) -> Option<&i64> {
        NOTE_MAP.get(name)
    }

    pub fn as_variable(name: &str) -> Variable {
        match name {
            "A" | "B" | "C" | "D" | "W" | "X" | "Y" | "Z" => Variable::Global(name.to_string()),
            "T" => Variable::Environment(EnvironmentFunc::GetTempo),
            "R" => Variable::Environment(EnvironmentFunc::RandomUInt(128)),
            _ => Variable::Instance(name.to_string()),
        }
    }

}

// Possible notes (auto-generated)
lazy_static! {
    static ref NOTE_MAP: HashMap<String, i64> = {
        let mut m = HashMap::new();
        m.insert("c-2".to_string(), 0);
        m.insert("c#-2".to_string(), 1);
        m.insert("c-2#".to_string(), 1);
        m.insert("db-2".to_string(), 1);
        m.insert("d-2b".to_string(), 1);
        m.insert("d-2".to_string(), 2);
        m.insert("d#-2".to_string(), 3);
        m.insert("d-2#".to_string(), 3);
        m.insert("eb-2".to_string(), 3);
        m.insert("e-2b".to_string(), 3);
        m.insert("e-2".to_string(), 4);
        m.insert("e#-2".to_string(), 5);
        m.insert("e-2#".to_string(), 5);
        m.insert("fb-2".to_string(), 4);
        m.insert("f-2b".to_string(), 4);
        m.insert("f-2".to_string(), 5);
        m.insert("f#-2".to_string(), 6);
        m.insert("f-2#".to_string(), 6);
        m.insert("gb-2".to_string(), 6);
        m.insert("g-2b".to_string(), 6);
        m.insert("g-2".to_string(), 7);
        m.insert("g#-2".to_string(), 8);
        m.insert("g-2#".to_string(), 8);
        m.insert("ab-2".to_string(), 8);
        m.insert("a-2b".to_string(), 8);
        m.insert("a-2".to_string(), 9);
        m.insert("a#-2".to_string(), 10);
        m.insert("a-2#".to_string(), 10);
        m.insert("bb-2".to_string(), 10);
        m.insert("b-2b".to_string(), 10);
        m.insert("b-2".to_string(), 11);
        m.insert("b#-2".to_string(), 12);
        m.insert("b-2#".to_string(), 12);
        m.insert("cb-1".to_string(), 11);
        m.insert("c-1b".to_string(), 11);
        m.insert("c-1".to_string(), 12);
        m.insert("c#-1".to_string(), 13);
        m.insert("c-1#".to_string(), 13);
        m.insert("db-1".to_string(), 13);
        m.insert("d-1b".to_string(), 13);
        m.insert("d-1".to_string(), 14);
        m.insert("d#-1".to_string(), 15);
        m.insert("d-1#".to_string(), 15);
        m.insert("eb-1".to_string(), 15);
        m.insert("e-1b".to_string(), 15);
        m.insert("e-1".to_string(), 16);
        m.insert("e#-1".to_string(), 17);
        m.insert("e-1#".to_string(), 17);
        m.insert("fb-1".to_string(), 16);
        m.insert("f-1b".to_string(), 16);
        m.insert("f-1".to_string(), 17);
        m.insert("f#-1".to_string(), 18);
        m.insert("f-1#".to_string(), 18);
        m.insert("gb-1".to_string(), 18);
        m.insert("g-1b".to_string(), 18);
        m.insert("g-1".to_string(), 19);
        m.insert("g#-1".to_string(), 20);
        m.insert("g-1#".to_string(), 20);
        m.insert("ab-1".to_string(), 20);
        m.insert("a-1b".to_string(), 20);
        m.insert("a-1".to_string(), 21);
        m.insert("a#-1".to_string(), 22);
        m.insert("a-1#".to_string(), 22);
        m.insert("bb-1".to_string(), 22);
        m.insert("b-1b".to_string(), 22);
        m.insert("b-1".to_string(), 23);
        m.insert("b#-1".to_string(), 24);
        m.insert("b-1#".to_string(), 24);
        m.insert("cb0".to_string(), 23);
        m.insert("c0b".to_string(), 23);
        m.insert("c0".to_string(), 24);
        m.insert("c#0".to_string(), 25);
        m.insert("c0#".to_string(), 25);
        m.insert("db0".to_string(), 25);
        m.insert("d0b".to_string(), 25);
        m.insert("d0".to_string(), 26);
        m.insert("d#0".to_string(), 27);
        m.insert("d0#".to_string(), 27);
        m.insert("eb0".to_string(), 27);
        m.insert("e0b".to_string(), 27);
        m.insert("e0".to_string(), 28);
        m.insert("e#0".to_string(), 29);
        m.insert("e0#".to_string(), 29);
        m.insert("fb0".to_string(), 28);
        m.insert("f0b".to_string(), 28);
        m.insert("f0".to_string(), 29);
        m.insert("f#0".to_string(), 30);
        m.insert("f0#".to_string(), 30);
        m.insert("gb0".to_string(), 30);
        m.insert("g0b".to_string(), 30);
        m.insert("g0".to_string(), 31);
        m.insert("g#0".to_string(), 32);
        m.insert("g0#".to_string(), 32);
        m.insert("ab0".to_string(), 32);
        m.insert("a0b".to_string(), 32);
        m.insert("a0".to_string(), 33);
        m.insert("a#0".to_string(), 34);
        m.insert("a0#".to_string(), 34);
        m.insert("bb0".to_string(), 34);
        m.insert("b0b".to_string(), 34);
        m.insert("b0".to_string(), 35);
        m.insert("b#0".to_string(), 36);
        m.insert("b0#".to_string(), 36);
        m.insert("cb1".to_string(), 35);
        m.insert("c1b".to_string(), 35);
        m.insert("c1".to_string(), 36);
        m.insert("c#1".to_string(), 37);
        m.insert("c1#".to_string(), 37);
        m.insert("db1".to_string(), 37);
        m.insert("d1b".to_string(), 37);
        m.insert("d1".to_string(), 38);
        m.insert("d#1".to_string(), 39);
        m.insert("d1#".to_string(), 39);
        m.insert("eb1".to_string(), 39);
        m.insert("e1b".to_string(), 39);
        m.insert("e1".to_string(), 40);
        m.insert("e#1".to_string(), 41);
        m.insert("e1#".to_string(), 41);
        m.insert("fb1".to_string(), 40);
        m.insert("f1b".to_string(), 40);
        m.insert("f1".to_string(), 41);
        m.insert("f#1".to_string(), 42);
        m.insert("f1#".to_string(), 42);
        m.insert("gb1".to_string(), 42);
        m.insert("g1b".to_string(), 42);
        m.insert("g1".to_string(), 43);
        m.insert("g#1".to_string(), 44);
        m.insert("g1#".to_string(), 44);
        m.insert("ab1".to_string(), 44);
        m.insert("a1b".to_string(), 44);
        m.insert("a1".to_string(), 45);
        m.insert("a#1".to_string(), 46);
        m.insert("a1#".to_string(), 46);
        m.insert("bb1".to_string(), 46);
        m.insert("b1b".to_string(), 46);
        m.insert("b1".to_string(), 47);
        m.insert("b#1".to_string(), 48);
        m.insert("b1#".to_string(), 48);
        m.insert("cb2".to_string(), 47);
        m.insert("c2b".to_string(), 47);
        m.insert("c2".to_string(), 48);
        m.insert("c#2".to_string(), 49);
        m.insert("c2#".to_string(), 49);
        m.insert("db2".to_string(), 49);
        m.insert("d2b".to_string(), 49);
        m.insert("d2".to_string(), 50);
        m.insert("d#2".to_string(), 51);
        m.insert("d2#".to_string(), 51);
        m.insert("eb2".to_string(), 51);
        m.insert("e2b".to_string(), 51);
        m.insert("e2".to_string(), 52);
        m.insert("e#2".to_string(), 53);
        m.insert("e2#".to_string(), 53);
        m.insert("fb2".to_string(), 52);
        m.insert("f2b".to_string(), 52);
        m.insert("f2".to_string(), 53);
        m.insert("f#2".to_string(), 54);
        m.insert("f2#".to_string(), 54);
        m.insert("gb2".to_string(), 54);
        m.insert("g2b".to_string(), 54);
        m.insert("g2".to_string(), 55);
        m.insert("g#2".to_string(), 56);
        m.insert("g2#".to_string(), 56);
        m.insert("ab2".to_string(), 56);
        m.insert("a2b".to_string(), 56);
        m.insert("a2".to_string(), 57);
        m.insert("a#2".to_string(), 58);
        m.insert("a2#".to_string(), 58);
        m.insert("bb2".to_string(), 58);
        m.insert("b2b".to_string(), 58);
        m.insert("b2".to_string(), 59);
        m.insert("b#2".to_string(), 60);
        m.insert("b2#".to_string(), 60);
        m.insert("cb3".to_string(), 59);
        m.insert("c3b".to_string(), 59);
        m.insert("cb".to_string(), 59);
        m.insert("c3".to_string(), 60);
        m.insert("c".to_string(), 60);
        m.insert("c#3".to_string(), 61);
        m.insert("c3#".to_string(), 61);
        m.insert("c#".to_string(), 61);
        m.insert("db3".to_string(), 61);
        m.insert("d3b".to_string(), 61);
        m.insert("db".to_string(), 61);
        m.insert("d3".to_string(), 62);
        m.insert("d".to_string(), 62);
        m.insert("d#3".to_string(), 63);
        m.insert("d3#".to_string(), 63);
        m.insert("d#".to_string(), 63);
        m.insert("eb3".to_string(), 63);
        m.insert("e3b".to_string(), 63);
        m.insert("eb".to_string(), 63);
        m.insert("e3".to_string(), 64);
        m.insert("e".to_string(), 64);
        m.insert("e#3".to_string(), 65);
        m.insert("e3#".to_string(), 65);
        m.insert("e#".to_string(), 65);
        m.insert("fb3".to_string(), 64);
        m.insert("f3b".to_string(), 64);
        m.insert("fb".to_string(), 64);
        m.insert("f3".to_string(), 65);
        m.insert("f".to_string(), 65);
        m.insert("f#3".to_string(), 66);
        m.insert("f3#".to_string(), 66);
        m.insert("f#".to_string(), 66);
        m.insert("gb3".to_string(), 66);
        m.insert("g3b".to_string(), 66);
        m.insert("gb".to_string(), 66);
        m.insert("g3".to_string(), 67);
        m.insert("g".to_string(), 67);
        m.insert("g#3".to_string(), 68);
        m.insert("g3#".to_string(), 68);
        m.insert("g#".to_string(), 68);
        m.insert("ab3".to_string(), 68);
        m.insert("a3b".to_string(), 68);
        m.insert("ab".to_string(), 68);
        m.insert("a3".to_string(), 69);
        m.insert("a".to_string(), 69);
        m.insert("a#3".to_string(), 70);
        m.insert("a3#".to_string(), 70);
        m.insert("a#".to_string(), 70);
        m.insert("bb3".to_string(), 70);
        m.insert("b3b".to_string(), 70);
        m.insert("bb".to_string(), 70);
        m.insert("b3".to_string(), 71);
        m.insert("b".to_string(), 71);
        m.insert("b#3".to_string(), 72);
        m.insert("b3#".to_string(), 72);
        m.insert("b#".to_string(), 72);
        m.insert("cb4".to_string(), 71);
        m.insert("c4b".to_string(), 71);
        m.insert("c4".to_string(), 72);
        m.insert("c#4".to_string(), 73);
        m.insert("c4#".to_string(), 73);
        m.insert("db4".to_string(), 73);
        m.insert("d4b".to_string(), 73);
        m.insert("d4".to_string(), 74);
        m.insert("d#4".to_string(), 75);
        m.insert("d4#".to_string(), 75);
        m.insert("eb4".to_string(), 75);
        m.insert("e4b".to_string(), 75);
        m.insert("e4".to_string(), 76);
        m.insert("e#4".to_string(), 77);
        m.insert("e4#".to_string(), 77);
        m.insert("fb4".to_string(), 76);
        m.insert("f4b".to_string(), 76);
        m.insert("f4".to_string(), 77);
        m.insert("f#4".to_string(), 78);
        m.insert("f4#".to_string(), 78);
        m.insert("gb4".to_string(), 78);
        m.insert("g4b".to_string(), 78);
        m.insert("g4".to_string(), 79);
        m.insert("g#4".to_string(), 80);
        m.insert("g4#".to_string(), 80);
        m.insert("ab4".to_string(), 80);
        m.insert("a4b".to_string(), 80);
        m.insert("a4".to_string(), 81);
        m.insert("a#4".to_string(), 82);
        m.insert("a4#".to_string(), 82);
        m.insert("bb4".to_string(), 82);
        m.insert("b4b".to_string(), 82);
        m.insert("b4".to_string(), 83);
        m.insert("b#4".to_string(), 84);
        m.insert("b4#".to_string(), 84);
        m.insert("cb5".to_string(), 83);
        m.insert("c5b".to_string(), 83);
        m.insert("c5".to_string(), 84);
        m.insert("c#5".to_string(), 85);
        m.insert("c5#".to_string(), 85);
        m.insert("db5".to_string(), 85);
        m.insert("d5b".to_string(), 85);
        m.insert("d5".to_string(), 86);
        m.insert("d#5".to_string(), 87);
        m.insert("d5#".to_string(), 87);
        m.insert("eb5".to_string(), 87);
        m.insert("e5b".to_string(), 87);
        m.insert("e5".to_string(), 88);
        m.insert("e#5".to_string(), 89);
        m.insert("e5#".to_string(), 89);
        m.insert("fb5".to_string(), 88);
        m.insert("f5b".to_string(), 88);
        m.insert("f5".to_string(), 89);
        m.insert("f#5".to_string(), 90);
        m.insert("f5#".to_string(), 90);
        m.insert("gb5".to_string(), 90);
        m.insert("g5b".to_string(), 90);
        m.insert("g5".to_string(), 91);
        m.insert("g#5".to_string(), 92);
        m.insert("g5#".to_string(), 92);
        m.insert("ab5".to_string(), 92);
        m.insert("a5b".to_string(), 92);
        m.insert("a5".to_string(), 93);
        m.insert("a#5".to_string(), 94);
        m.insert("a5#".to_string(), 94);
        m.insert("bb5".to_string(), 94);
        m.insert("b5b".to_string(), 94);
        m.insert("b5".to_string(), 95);
        m.insert("b#5".to_string(), 96);
        m.insert("b5#".to_string(), 96);
        m.insert("cb6".to_string(), 95);
        m.insert("c6b".to_string(), 95);
        m.insert("c6".to_string(), 96);
        m.insert("c#6".to_string(), 97);
        m.insert("c6#".to_string(), 97);
        m.insert("db6".to_string(), 97);
        m.insert("d6b".to_string(), 97);
        m.insert("d6".to_string(), 98);
        m.insert("d#6".to_string(), 99);
        m.insert("d6#".to_string(), 99);
        m.insert("eb6".to_string(), 99);
        m.insert("e6b".to_string(), 99);
        m.insert("e6".to_string(), 100);
        m.insert("e#6".to_string(), 101);
        m.insert("e6#".to_string(), 101);
        m.insert("fb6".to_string(), 100);
        m.insert("f6b".to_string(), 100);
        m.insert("f6".to_string(), 101);
        m.insert("f#6".to_string(), 102);
        m.insert("f6#".to_string(), 102);
        m.insert("gb6".to_string(), 102);
        m.insert("g6b".to_string(), 102);
        m.insert("g6".to_string(), 103);
        m.insert("g#6".to_string(), 104);
        m.insert("g6#".to_string(), 104);
        m.insert("ab6".to_string(), 104);
        m.insert("a6b".to_string(), 104);
        m.insert("a6".to_string(), 105);
        m.insert("a#6".to_string(), 106);
        m.insert("a6#".to_string(), 106);
        m.insert("bb6".to_string(), 106);
        m.insert("b6b".to_string(), 106);
        m.insert("b6".to_string(), 107);
        m.insert("b#6".to_string(), 108);
        m.insert("b6#".to_string(), 108);
        m.insert("cb7".to_string(), 107);
        m.insert("c7b".to_string(), 107);
        m.insert("c7".to_string(), 108);
        m.insert("c#7".to_string(), 109);
        m.insert("c7#".to_string(), 109);
        m.insert("db7".to_string(), 109);
        m.insert("d7b".to_string(), 109);
        m.insert("d7".to_string(), 110);
        m.insert("d#7".to_string(), 111);
        m.insert("d7#".to_string(), 111);
        m.insert("eb7".to_string(), 111);
        m.insert("e7b".to_string(), 111);
        m.insert("e7".to_string(), 112);
        m.insert("e#7".to_string(), 113);
        m.insert("e7#".to_string(), 113);
        m.insert("fb7".to_string(), 112);
        m.insert("f7b".to_string(), 112);
        m.insert("f7".to_string(), 113);
        m.insert("f#7".to_string(), 114);
        m.insert("f7#".to_string(), 114);
        m.insert("gb7".to_string(), 114);
        m.insert("g7b".to_string(), 114);
        m.insert("g7".to_string(), 115);
        m.insert("g#7".to_string(), 116);
        m.insert("g7#".to_string(), 116);
        m.insert("ab7".to_string(), 116);
        m.insert("a7b".to_string(), 116);
        m.insert("a7".to_string(), 117);
        m.insert("a#7".to_string(), 118);
        m.insert("a7#".to_string(), 118);
        m.insert("bb7".to_string(), 118);
        m.insert("b7b".to_string(), 118);
        m.insert("b7".to_string(), 119);
        m.insert("b#7".to_string(), 120);
        m.insert("b7#".to_string(), 120);
        m.insert("cb8".to_string(), 119);
        m.insert("c8b".to_string(), 119);
        m.insert("c8".to_string(), 120);
        m.insert("c#8".to_string(), 121);
        m.insert("c8#".to_string(), 121);
        m.insert("db8".to_string(), 121);
        m.insert("d8b".to_string(), 121);
        m.insert("d8".to_string(), 122);
        m.insert("d#8".to_string(), 123);
        m.insert("d8#".to_string(), 123);
        m.insert("eb8".to_string(), 123);
        m.insert("e8b".to_string(), 123);
        m.insert("e8".to_string(), 124);
        m.insert("e#8".to_string(), 125);
        m.insert("e8#".to_string(), 125);
        m.insert("fb8".to_string(), 124);
        m.insert("f8b".to_string(), 124);
        m.insert("f8".to_string(), 125);
        m.insert("f#8".to_string(), 126);
        m.insert("f8#".to_string(), 126);
        m.insert("gb8".to_string(), 126);
        m.insert("g8b".to_string(), 126);
        m.insert("g8".to_string(), 127);
        m
    };
}
