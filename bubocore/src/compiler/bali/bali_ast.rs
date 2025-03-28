use crate::{lang::{Program, event::Event, Instruction, control_asm::ControlASM, variable::Variable}};
use std::cmp::Ordering;

pub type BaliProgram = Vec<TopLevelStatement>;
pub type BaliPreparedProgram = Vec<TimeStatement>;

const MIDIDEVICE: &str = "BuboCoreOut";
//const MIDIDEVICE: &str = "log";
const DEFAULTVELOCITY: i64 = 90;
const DEFAULTCHAN: i64 = 1;

pub fn bali_as_asm(prog: BaliProgram) -> Program {
    //print!("Original prog {:?}\n", prog);
    let mut prog = expend_prog(prog);
    //print!("Expended prog {:?}\n", prog);
    prog.sort();
    //print!("Sorted prog {:?}\n", prog);

    let mut total_delay: f64 = if prog.len() > 0 {
        prog[0].get_time()
    } else {
        0.0
    };
    let mut res: Program = Vec::new();
    let time_var = Variable::Instance("_time".to_owned());

    if total_delay > 0.0 {
        res.push(Instruction::Control(ControlASM::FloatAsSteps(total_delay.into(), time_var.clone()))); // TODO: FloatAsSteps
        res.push(Instruction::Effect(Event::Nop, time_var.clone()));
    }

    for i in 0..prog.len()-1 {
        //print!("{:?}\n", prog[i]);
        let delay = if total_delay >= 0.0 {
            prog[i+1].get_time() - total_delay
        } else {
            prog[i+1].get_time()
        };
        let delay = if delay < 0.0 {
            0.0
        } else {
            delay
        };
        total_delay = prog[i+1].get_time();
        res.extend(prog[i].as_asm(delay));
    }

    res.extend(prog[prog.len()-1].as_asm(0.0));
    //print!("{:?}", res);


    res
}

pub fn expend_prog(prog: BaliProgram) -> BaliPreparedProgram {
    prog.into_iter().map(|tls| tls.expend()).flatten().collect()
}

#[derive(Debug)]
pub enum TimeStatement {
    At(f64, Effect),
    JustBefore(f64, Effect),
    JustAfter(f64, Effect),
}

impl TimeStatement {

    pub fn get_time(&self) -> f64 {
        match self {
            TimeStatement::At(x, _) | TimeStatement::JustBefore(x, _) | TimeStatement::JustAfter(x, _) => *x,
        }
    }

    pub fn as_asm(&self, delay: f64) -> Vec<Instruction> {
        match self {
            TimeStatement::At(_, x) | TimeStatement::JustBefore(_, x) | TimeStatement::JustAfter(_, x) => x.as_asm(delay),
        }
    }

}

impl Ord for TimeStatement {
    fn cmp(&self, other: &Self) -> Ordering {
        let t1 = self.get_time();
        let t2 = other.get_time();
        if t1 < t2 {
            return Ordering::Less
        }
        if t1 > t2 {
            return Ordering::Greater
        }
        match (self, other) {
            (TimeStatement::JustBefore(_, _), _) => Ordering::Less,
            (_, TimeStatement::JustAfter(_, _)) => Ordering::Less,
            (_, TimeStatement::JustBefore(_, _)) => Ordering::Greater,
            (TimeStatement::JustAfter(_, _), _) => Ordering::Greater,
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
            (TimeStatement::At(x, _), TimeStatement::At(y, _)) => x == y,
            (TimeStatement::JustBefore(x, _), TimeStatement::JustBefore(y, _)) => x == y,
            (TimeStatement::JustAfter(x, _), TimeStatement::JustAfter(y, _)) => x == y,
            _ => false
        }
    }
}

impl Eq for TimeStatement {}



#[derive(Debug)]
pub enum TopLevelStatement {
    AtStatement(ConcreteFraction, Vec<Statement>),
    Statement(Statement),
}

impl TopLevelStatement {

    pub fn expend(self) -> Vec<TimeStatement> {
        match self {
            TopLevelStatement::AtStatement(v, ss) => ss.into_iter().map(|s| s.expend(&v)).collect(),
            TopLevelStatement::Statement(s) => vec![s.expend(&ConcreteFraction{numerator: 0, denominator: 1})],
        }
    }

}

#[derive(Debug)]
pub enum Statement {
    AfterFrac(ConcreteFraction, Effect),
    After(Effect),
    BeforeFrac(ConcreteFraction, Effect),
    Before(Effect),
    Effect(Effect),
}

impl Statement {

    pub fn expend(self, val: &ConcreteFraction) -> TimeStatement {
        match self {
            Statement::AfterFrac(v, e) => TimeStatement::At(v.addf64(val), e),
            Statement::After(e) => TimeStatement::JustAfter(val.tof64(), e),
            Statement::BeforeFrac(v, e) => TimeStatement::At(val.subf64(&v), e),
            Statement::Before(e) => TimeStatement::JustBefore(val.tof64(), e),
            Statement::Effect(e) => TimeStatement::At(val.tof64(), e),
        }
    }

}

#[derive(Debug)]
pub enum Effect {
    Definition(Value, Box<Expression>),
    Note(Box<Expression>, Option<Box<Expression>>, Option<Box<Expression>>, Fraction),
    ProgramChange(Box<Expression>, Box<Expression>),
    ControlChange(Box<Expression>, Box<Expression>, Box<Expression>),
}

impl Effect { // TODO : on veut que les durées soient des fractions
    pub fn as_asm(&self, delay: f64) -> Vec<Instruction> {
        let time_var = Variable::Instance("_time".to_owned());
        let note_var = Variable::Instance("_note".to_owned());
        let velocity_var = Variable::Instance("_velocity".to_owned());
        let chan_var = Variable::Instance("_chan".to_owned());
        let duration_var = Variable::Instance("_duration".to_owned());
        let duration_time_var = Variable::Instance("_duration_time".to_owned());
        let program_var = Variable::Instance("_program".to_owned());
        let control_var = Variable::Instance("_control".to_owned());
        let value_var = Variable::Instance("_control_value".to_owned());
        let mut res = vec![Instruction::Control(ControlASM::FloatAsSteps(delay.into(), time_var.clone()))];
        
        match self {
            Effect::Definition(v, expr) => {
                res.extend(expr.as_asm());
                let v = v.tostr();
                res.push(Instruction::Control(ControlASM::Pop(Variable::Instance(v))));
                res.push(Instruction::Effect(Event::Nop, time_var.clone()));
            },
            Effect::Note(n, v, c, d) => {
                res.extend(n.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(note_var.clone())));
                if let Some(v) = v {
                    res.extend(v.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(velocity_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULTVELOCITY.into(), velocity_var.clone())))
                }
                if let Some(c) = c {
                    res.extend(c.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULTCHAN.into(), chan_var.clone())))
                }
                res.extend(d.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(duration_var.clone())));
                res.push(Instruction::Control(ControlASM::FloatAsSteps(duration_var.clone(), duration_time_var.clone())));
                res.push(Instruction::Effect(Event::MidiNote(
                    note_var.clone(), velocity_var.clone(), chan_var.clone(), 
                    duration_time_var.clone(), MIDIDEVICE.to_string().into()
                ), time_var.clone()));
            },
            Effect::ProgramChange(p, c) => {
                res.extend(p.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(program_var.clone())));
                res.extend(c.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                res.push(Instruction::Effect(Event::MidiProgram(
                    program_var.clone(), chan_var.clone(), MIDIDEVICE.to_string().into()
                ), time_var.clone()));
            },
            Effect::ControlChange(con, v, c) => {
                res.extend(con.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(control_var.clone())));
                res.extend(v.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(value_var.clone())));
                res.extend(c.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                res.push(Instruction::Effect(Event::MidiControl(
                    control_var.clone(), value_var.clone(), chan_var.clone(), MIDIDEVICE.to_string().into()
                ), time_var.clone()));
            },
        }

        res
    }
}

#[derive(Debug)]
pub enum Expression {
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Subtraction(Box<Expression>, Box<Expression>),
    Division(Box<Expression>, Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),
    Value(Value),
}

impl Expression {
    pub fn as_asm(&self) -> Vec<Instruction> {
        let var_1 = Variable::Instance("_exp1".to_owned());
        let var_2 = Variable::Instance("_exp2".to_owned());
        let var_out = Variable::Instance("_res".to_owned());
        let mut res = match self {
            Expression::Addition(e1, e2) | Expression::Multiplication(e1, e2) | Expression::Subtraction(e1, e2) | Expression::Division(e1, e2) | Expression::Modulo(e1, e2) => {
                let mut e1 = e1.as_asm();
                e1.extend(e2.as_asm());
                e1.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                e1.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                e1
            },
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
            Expression::Value(_) => 
                res.push(Instruction::Control(ControlASM::Pop(var_out.clone()))),
        };

        res.push(Instruction::Control(ControlASM::Push(var_out.clone())));
        res
    }
}

#[derive(Debug)]
pub struct ConcreteFraction {
    pub numerator: u8,
    pub denominator: u8,
} 

impl ConcreteFraction {

    pub fn tof64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    pub fn addf64(&self, other: &Self) -> f64 {
        self.tof64() + other.tof64()
    }

    pub fn subf64(&self, other: &Self) -> f64 {
        self.tof64() - other.tof64()
    }

}

#[derive(Debug)]
pub struct Fraction {
    pub numerator: Box<Expression>,
    pub denominator: Box<Expression>,
} 

impl Fraction {

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
        e1.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
        e1.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
        e1.push(Instruction::Control(ControlASM::Div(var_1.clone(), var_2.clone(), var_out.clone())));
        e1.push(Instruction::Control(ControlASM::Push(var_out.clone())));
        e1
    }
}

#[derive(Debug)]
pub enum Value {
    Number(u8),
    Variable(String),
}


impl Value {

    pub fn as_asm(&self) -> Instruction {
        match self {
            Value::Number(n) => Instruction::Control(ControlASM::Push((*n as i64).into())),
            Value::Variable(s) => Instruction::Control(ControlASM::Push(Variable::Instance(s.to_string()))),
        }
    }

    pub fn tostr(&self) -> String {
        match self {
            Value::Variable(s) => s.to_string(),
            _ => unreachable!(),
        }
    }

}
