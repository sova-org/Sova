use crate::{lang::{Program, event::Event, Instruction, control_asm::ControlASM, variable::Variable, environment_func::EnvironmentFunc}};
use std::cmp::Ordering;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub type BaliProgram = Vec<Statement>;
pub type BaliPreparedProgram = Vec<TimeStatement>;

// TODO : définir les noms de variables temporaires ici et les commenter avec leurs types pour éviter les erreurs

const DEFAULT_VELOCITY: i64 = 90;
const DEFAULT_CHAN: i64 = 1;
const DEFAULT_DEVICE: i64 = 1;
const DEFAULT_DURATION: i64 = 2;

pub fn bali_as_asm(prog: BaliProgram) -> Program {
    //print!("Original prog {:?}\n", prog);
    //let prog = expend_loop(prog);
    //print!("Loopless prog {:?}\n", prog);
    let default_context = BaliContext{
        channel: Some(Expression::Value(Value::Number(DEFAULT_CHAN))),
        device: Some(DEFAULT_DEVICE),
        velocity: Some(Expression::Value(Value::Number(DEFAULT_VELOCITY))),
        duration: Some(Fraction{
            numerator: Box::new(Expression::Value(Value::Number(1))),
            denominator: Box::new(Expression::Value(Value::Number(DEFAULT_DURATION))),
        }),
    };


    let mut prog = expend_prog(prog, default_context);
    //print!("Expended prog {:?}\n", prog);
    prog.sort();
    //print!("Sorted prog {:?}\n", prog);

    let mut total_delay: f64 = if prog.len() > 0 {
        prog[0].get_time_as_f64()
    } else {
        0.0
    };
    let mut res: Program = Vec::new();
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
        res.extend(prog[i].as_asm(delay, res.len()));
    }

    res.extend(prog[prog.len()-1].as_asm(0.0, res.len()));
    //print!("{:?}", res);


    res
}


pub fn expend_prog(prog: BaliProgram, c: BaliContext) -> BaliPreparedProgram {
    prog.into_iter().map(|s| s.expend(&ConcreteFraction{signe: 1, numerator: 0, denominator: 1}, c.clone())).flatten().collect()
}

/*
pub fn set_context_prog(prog: BaliProgram, c: BaliContext) -> BaliProgram {
    prog.into_iter().map(|s| s.set_context(c.clone())).collect()
}
*/

pub fn set_context_effect_set(set: Vec<TopLevelEffect>, c: BaliContext) -> Vec<TopLevelEffect> {
    set.into_iter().map(|e| e.set_context(c.clone())).collect()
}

#[derive(Debug, Clone)]
pub struct BaliContext {
    pub channel: Option<Expression>,
    pub device: Option<i64>,
    pub velocity: Option<Expression>,
    pub duration: Option<Fraction>,
}

impl BaliContext {
    pub fn new() -> BaliContext {
        BaliContext{
            channel: None,
            device: None,
            velocity: None,
            duration: None,
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
        b
    }
}

#[derive(Debug)]
pub enum TimeStatement {
    At(ConcreteFraction, TopLevelEffect, BaliContext),
    JustBefore(ConcreteFraction, TopLevelEffect, BaliContext),
    JustAfter(ConcreteFraction, TopLevelEffect, BaliContext),
}

impl TimeStatement {

    pub fn get_time_as_f64(&self) -> f64 {
        match self {
            TimeStatement::At(x, _, _) | TimeStatement::JustBefore(x, _, _) | TimeStatement::JustAfter(x, _, _) => x.tof64(),
        }
    }

    pub fn get_time(&self) -> ConcreteFraction {
        match self {
            TimeStatement::At(x, _, _) | TimeStatement::JustBefore(x, _, _) | TimeStatement::JustAfter(x, _, _) => x.clone(),
        }
    }

    pub fn as_asm(&self, delay: f64, position: usize) -> Vec<Instruction> {
        match self {
            TimeStatement::At(_, x, context) | TimeStatement::JustBefore(_, x, context) | TimeStatement::JustAfter(_, x, context) => x.as_asm(delay, position, context.clone()),
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
            (TimeStatement::JustBefore(_, _, _), _) => Ordering::Less,
            (_, TimeStatement::JustAfter(_, _, _)) => Ordering::Less,
            (_, TimeStatement::JustBefore(_, _, _)) => Ordering::Greater,
            (TimeStatement::JustAfter(_, _, _), _) => Ordering::Greater,
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
            (TimeStatement::At(x, _, _), TimeStatement::At(y, _, _)) => x.numerator * y.denominator == y.numerator * x.denominator,
            (TimeStatement::JustBefore(x, _, _), TimeStatement::JustBefore(y, _, _)) => x.numerator * y.denominator == y.numerator * x.denominator,
            (TimeStatement::JustAfter(x, _, _), TimeStatement::JustAfter(y, _, _)) => x.numerator * y.denominator == y.numerator * x.denominator,
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
pub enum Statement {
    AfterFrac(ConcreteFraction, Vec<Statement>, BaliContext),
    BeforeFrac(ConcreteFraction, Vec<Statement>, BaliContext),
    Loop(i64, ConcreteFraction, Vec<Statement>, BaliContext),
    Euclidean(i64, i64, Option<i64>, bool, ConcreteFraction, Vec<Statement>, BaliContext),
    Binary(i64, i64, Option<i64>, bool, ConcreteFraction, Vec<Statement>, BaliContext),
    After(Vec<TopLevelEffect>, BaliContext),
    Before(Vec<TopLevelEffect>, BaliContext),
    Effect(TopLevelEffect, BaliContext),
    With(Vec<Statement>, BaliContext),
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

    fn get_euclidean(beats: i64, steps: i64, shift: Option<i64>, reverse: bool) -> Vec<i64> {

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

        Self::as_time_points(&mut seq, shift, reverse)
    }

    fn get_binary(it: i64, steps: i64, shift: Option<i64>, reverse: bool) -> Vec<i64> {
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

        Self::as_time_points(&mut res_seq, shift, reverse)
    }

    fn as_time_points(seq: &mut Vec<i64>, shift: Option<i64>, reverse: bool) -> Vec<i64> {
        
        //print!("{:?}\n", seq);

        if reverse {
            seq.reverse();
        }

        if let Some(shift) = shift {
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

    pub fn expend(self, val: &ConcreteFraction, c: BaliContext) -> Vec<TimeStatement> {
        /*let c = match self {
            Statement::AfterFrac(_, _, ref cc) | Statement::BeforeFrac(_, _, ref cc) | Statement::Loop(_, _, _, ref cc) | Statement::After(_, ref cc) | Statement::Before(_, ref cc) | Statement::Effect(_, ref cc) => cc.clone().update(c),
        };*/
        match self {
            Statement::AfterFrac(v, es, cc) => es.into_iter().map(|e| e.expend(&v.add(val), cc.clone().update(c.clone()))).flatten().collect(),
            Statement::BeforeFrac(v, es, cc) => es.into_iter().map(|e| e.expend(&val.sub(&v), cc.clone().update(c.clone()))).flatten().collect(),
            Statement::Loop(it, v, es, cc) => {
                let mut res = Vec::new();
                for i in 0..it {
                    let content: Vec<TimeStatement> = es.clone().into_iter().map(|e| e.expend(&val.add(&v.multbyint(i)), cc.clone().update(c.clone()))).flatten().collect();
                    res.extend(content);
                };
                res
            },
            Statement::Euclidean(beats, steps, shift, reverse, v, es, cc) => {
                let mut res = Vec::new();
                let euc = Self::get_euclidean(beats, steps, shift, reverse);
                for i in 0..euc.len() {
                    let content: Vec<TimeStatement> = es.clone().into_iter().map(|e| e.expend(&val.add(&v.multbyint(euc[i])), cc.clone().update(c.clone()))).flatten().collect();
                    res.extend(content);
                };
                res
            },
            Statement::Binary(it, steps, shift, reverse, v, es, cc) => {
                let mut res = Vec::new();
                let bin = Self::get_binary(it, steps, shift, reverse);
                for i in 0..bin.len() {
                    let content: Vec<TimeStatement> = es.clone().into_iter().map(|e| e.expend(&val.add(&v.multbyint(bin[i])), cc.clone().update(c.clone()))).flatten().collect();
                    res.extend(content);
                };
                res
            },
            Statement::After(es, cc) => es.into_iter().map(|e| TimeStatement::JustAfter(val.clone(), e, cc.clone().update(c.clone()))).collect(),
            Statement::Before(es, cc) => es.into_iter().map(|e| TimeStatement::JustBefore(val.clone(), e, cc.clone().update(c.clone()))).collect(),
            Statement::Effect(e, cc) => vec![TimeStatement::At(val.clone(), e, cc.clone().update(c.clone()))],
            Statement::With(es, cc) => es.into_iter().map(|e| e.expend(val, cc.clone().update(c.clone()))).flatten().collect(),
        }
    }

}

#[derive(Debug, Clone)]
pub enum TopLevelEffect {
    Seq(Vec<TopLevelEffect>, BaliContext),
    For(Box<BooleanExpression>, Vec<TopLevelEffect>, BaliContext),
    If(Box<BooleanExpression>, Vec<TopLevelEffect>, BaliContext),
    Effect(Effect, BaliContext),
}

impl TopLevelEffect {

    pub fn set_context(self, c: BaliContext) -> TopLevelEffect {
        match self {
            TopLevelEffect::Seq(es, seq_context) => TopLevelEffect::Seq(es, seq_context.update(c)),
            TopLevelEffect::For(cond, es, for_context) => TopLevelEffect::For(cond, es, for_context.update(c)),
            TopLevelEffect::If(cond, es, if_context) => TopLevelEffect::If(cond, es, if_context.update(c)),
            TopLevelEffect::Effect(e, effect_context) => TopLevelEffect::Effect(e, effect_context.update(c)),
        }
    }

    pub fn as_asm(&self, delay: f64, position: usize, context: BaliContext) -> Vec<Instruction> {
        let time_var = Variable::Instance("_time".to_owned());
        let bvar_out = Variable::Instance("_bres".to_owned());
        match self {
            TopLevelEffect::Seq(s, seq_context) => {
                let mut res = Vec::new();
                let mut position = position;
                let context = seq_context.clone().update(context.clone());
                for i in 0..s.len() {
                    let true_delay = if i < s.len() - 1 {
                        0.0
                    } else {
                        delay
                    };
                    let to_add = s[i].as_asm(true_delay, position, context.clone());
                    position += to_add.len();
                    res.extend(to_add);
                };
                res
            }
            TopLevelEffect::For(e, s, for_context) => {
                let mut res = Vec::new();

                let condition_position = position;

                // Compute and add condition
                let condition = e.as_asm();
                let mut position = position + condition.len();
                res.extend(condition);

                // Add for structure
                position += 5;
                res.push(Instruction::Control(ControlASM::Pop(bvar_out.clone())));
                res.push(Instruction::Control(ControlASM::JumpIf(bvar_out.clone(), position)));
                res.push(Instruction::Control(ControlASM::FloatAsFrames(delay.into(), time_var.clone())));
                res.push(Instruction::Effect(Event::Nop, time_var.clone()));

                // Compute effects
                let context = for_context.clone().update(context.clone());
                let mut effects = Vec::new();
                for i in 0..s.len() {
                    let to_add = s[i].as_asm(0.0, position, context.clone());
                    position += to_add.len();
                    effects.extend(to_add);
                };

                // Add for structure (continued)
                position += 1;
                res.push(Instruction::Control(ControlASM::Jump(position)));
                
                // Add effects
                res.extend(effects);

                // Add for structure (end)
                res.push(Instruction::Control(ControlASM::Jump(condition_position)));

                res
            },
            TopLevelEffect::If(e, s, if_context) => {
                let mut res = Vec::new();

                // Compute and add condition
                let condition = e.as_asm();
                let mut position = position + condition.len();
                res.extend(condition);

                // Add if structure
                position += 5;
                res.push(Instruction::Control(ControlASM::Pop(bvar_out.clone())));
                res.push(Instruction::Control(ControlASM::JumpIf(bvar_out.clone(), position)));
                res.push(Instruction::Control(ControlASM::FloatAsFrames(delay.into(), time_var.clone())));
                res.push(Instruction::Effect(Event::Nop, time_var.clone()));

                // Compute effects
                let context = if_context.clone().update(context.clone());
                let mut effects = Vec::new();
                for i in 0..s.len() {
                    let true_delay = if i < s.len() - 1 {
                        0.0
                    } else {
                        delay
                    };
                    let to_add = s[i].as_asm(true_delay, position, context.clone());
                    position += to_add.len();
                    effects.extend(to_add);
                };

                // Add if structure (continued)
                res.push(Instruction::Control(ControlASM::Jump(position)));
                
                // Add effects
                res.extend(effects);

                res
            }
            TopLevelEffect::Effect(ef, effect_context) => {
                let context = effect_context.clone().update(context.clone());
                ef.as_asm(delay, context)
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
}

impl Effect { // TODO : on veut que les durées soient des fractions
    pub fn as_asm(&self, delay: f64, context: BaliContext) -> Vec<Instruction> {
        let time_var = Variable::Instance("_time".to_owned());
        let note_var = Variable::Instance("_note".to_owned());
        let velocity_var = Variable::Instance("_velocity".to_owned());
        let chan_var = Variable::Instance("_chan".to_owned());
        let duration_var = Variable::Instance("_duration".to_owned());
        let duration_time_var = Variable::Instance("_duration_time".to_owned());
        let program_var = Variable::Instance("_program".to_owned());
        let control_var = Variable::Instance("_control".to_owned());
        let value_var = Variable::Instance("_control_value".to_owned());
        let target_device_id_var = Variable::Instance("_target_device_id".to_string());

        let mut res = vec![Instruction::Control(ControlASM::FloatAsFrames(delay.into(), time_var.clone()))];
        
        match self {
            Effect::Definition(v, expr) => {
                res.extend(expr.as_asm());
                if let Value::Variable(v) = v {
                    res.push(Instruction::Control(ControlASM::Pop(Value::as_variable(v))));
                }
                if delay > 0.0 && res.len() == 1 { 
                    res.push(Instruction::Effect(Event::Nop, time_var.clone()));
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

                let device_id = context.device.unwrap_or(DEFAULT_DEVICE);
                res.push(Instruction::Control(ControlASM::Mov(device_id.into(), target_device_id_var.clone())));

                res.push(Instruction::Effect(Event::MidiNote(
                    note_var.clone(), velocity_var.clone(), chan_var.clone(),
                    duration_time_var.clone(), 
                    target_device_id_var.clone()
                ), time_var.clone()));
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
                
                let device_id = context.device.unwrap_or(DEFAULT_DEVICE);
                res.push(Instruction::Control(ControlASM::Mov(device_id.into(), target_device_id_var.clone())));

                res.push(Instruction::Effect(Event::MidiProgram(
                    program_var.clone(), chan_var.clone(),
                    target_device_id_var.clone()
                ), time_var.clone()));
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

                let device_id = context.device.unwrap_or(DEFAULT_DEVICE);
                res.push(Instruction::Control(ControlASM::Mov(device_id.into(), target_device_id_var.clone())));
                
                res.push(Instruction::Effect(Event::MidiControl(
                    control_var.clone(), value_var.clone(), chan_var.clone(),
                    target_device_id_var.clone()
                ), time_var.clone()));
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
    Value(Value),
}

impl Expression {
    pub fn as_asm(&self) -> Vec<Instruction> {
        let var_1 = Variable::Instance("_exp1".to_owned());
        let var_2 = Variable::Instance("_exp2".to_owned());
        let var_3 = Variable::Instance("_exp3".to_owned());
        let var_4 = Variable::Instance("_exp4".to_owned());
        let var_5 = Variable::Instance("_exp5".to_owned());
        let speed_var = Variable::Instance("_osc_speed".to_owned()); 
        let var_out = Variable::Instance("_res".to_owned());
        let mut res = match self {
            Expression::Addition(e1, e2)
            | Expression::Multiplication(e1, e2)
            | Expression::Subtraction(e1, e2)
            | Expression::Division(e1, e2)
            | Expression::Modulo(e1, e2) 
            | Expression::Min(e1, e2) 
            | Expression::Max(e1, e2) 
            | Expression::Quantize(e1, e2) => {
                let mut e1_asm = e1.as_asm();
                e1_asm.extend(e2.as_asm());
                e1_asm.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                e1_asm.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                e1_asm
            },
            Expression::Scale(val, old_min, old_max, new_min, new_max) => {
                let mut val_asm = val.as_asm();
                val_asm.extend(old_min.as_asm());
                val_asm.extend(old_max.as_asm());
                val_asm.extend(new_min.as_asm());
                val_asm.extend(new_max.as_asm());
                val_asm.push(Instruction::Control(ControlASM::Pop(var_5.clone())));
                val_asm.push(Instruction::Control(ControlASM::Pop(var_4.clone())));
                val_asm.push(Instruction::Control(ControlASM::Pop(var_3.clone())));
                val_asm.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                val_asm.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                val_asm
            }
            Expression::Clamp(val, min, max) => {
                let mut val_asm = val.as_asm();
                val_asm.extend(min.as_asm());
                val_asm.extend(max.as_asm());
                val_asm.push(Instruction::Control(ControlASM::Pop(var_3.clone())));
                val_asm.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                val_asm.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                val_asm
            }
            Expression::Sine(speed_expr)
            | Expression::Saw(speed_expr)
            | Expression::Triangle(speed_expr)
            | Expression::ISaw(speed_expr) 
            | Expression::RandStep(speed_expr) => {
                let mut speed_asm = speed_expr.as_asm();
                speed_asm.push(Instruction::Control(ControlASM::Pop(speed_var.clone()))); 
                speed_asm
            }
            Expression::Value(v) => {
                vec![v.as_asm()]
            }
        };
        match self {
            Expression::Addition(_, _) => {
                res.push(Instruction::Control(ControlASM::Add(var_1.clone(), var_2.clone(), var_out.clone())));
            },
            Expression::Multiplication(_, _) =>
                res.push(Instruction::Control(ControlASM::Mul(var_1.clone(), var_2.clone(), var_out.clone()))),
            Expression::Subtraction(_, _) =>
                res.push(Instruction::Control(ControlASM::Sub(var_1.clone(), var_2.clone(), var_out.clone()))),
            Expression::Division(_, _) =>
                res.push(Instruction::Control(ControlASM::Div(var_1.clone(), var_2.clone(), var_out.clone()))),
            Expression::Modulo(_, _) =>
                res.push(Instruction::Control(ControlASM::Mod(var_1.clone(), var_2.clone(), var_out.clone()))),
            Expression::Scale(_, _, _, _, _) => {
                res.push(Instruction::Control(ControlASM::Scale(var_1.clone(), var_2.clone(), var_3.clone(), var_4.clone(), var_5.clone(), var_out.clone())));
            },
            Expression::Clamp(_, _, _) => {
                res.push(Instruction::Control(ControlASM::Clamp(var_1.clone(), var_2.clone(), var_3.clone(), var_out.clone())));
            },
            Expression::Min(_, _) => {
                res.push(Instruction::Control(ControlASM::Min(var_1.clone(), var_2.clone(), var_out.clone())));
            },
            Expression::Max(_, _) => {
                res.push(Instruction::Control(ControlASM::Max(var_1.clone(), var_2.clone(), var_out.clone())));
            },
            Expression::Quantize(_, _) => {
                res.push(Instruction::Control(ControlASM::Quantize(var_1.clone(), var_2.clone(), var_out.clone())));
            },
            Expression::Sine(_) => {
                res.push(Instruction::Control(ControlASM::GetSine(speed_var.clone(), var_out.clone())));
            },
            Expression::Saw(_) => {
                res.push(Instruction::Control(ControlASM::GetSaw(speed_var.clone(), var_out.clone())));
            },
            Expression::Triangle(_) => {
                res.push(Instruction::Control(ControlASM::GetTriangle(speed_var.clone(), var_out.clone())));
            },
            Expression::ISaw(_) => {
                res.push(Instruction::Control(ControlASM::GetISaw(speed_var.clone(), var_out.clone())));
            },
            Expression::RandStep(_) => {
                res.push(Instruction::Control(ControlASM::GetRandStep(speed_var.clone(), var_out.clone())));
            },
            Expression::Value(_) =>
                res.push(Instruction::Control(ControlASM::Pop(var_out.clone()))),
        };

        res.push(Instruction::Control(ControlASM::Push(var_out.clone())));
        res
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

    pub fn multbyint(&self, mult: i64) -> ConcreteFraction {
        ConcreteFraction{
            signe: 1,
            numerator: self.signe * self.numerator * mult,
            denominator: self.denominator,
        }.simplify()
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
            "R" => Variable::Environment(EnvironmentFunc::RandomU8),
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
