use crate::{lang::{Program, event::Event, Instruction, control_asm::ControlASM, variable::Variable}};
use std::cmp::Ordering;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub type BaliProgram = Vec<TopLevelStatement>;
pub type BaliPreparedProgram = Vec<TimeStatement>;

const MIDIDEVICE: &str = "BuboCoreOut";
//const MIDIDEVICE: &str = "log";
const DEFAULT_VELOCITY: i64 = 90;
const DEFAULT_CHAN: i64 = 1;

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
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_VELOCITY.into(), velocity_var.clone())))
                }
                if let Some(c) = c {
                    res.extend(c.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(DEFAULT_CHAN.into(), chan_var.clone())))
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
            Value::Variable(s) => {
                match Value::as_note(s) {
                    None => Instruction::Control(ControlASM::Push(Variable::Instance(s.to_string()))),
                    Some(n) => Instruction::Control(ControlASM::Push((*n as i64).into())),
                }
            },
        }
    }

    pub fn tostr(&self) -> String {
        match self {
            Value::Variable(s) => s.to_string(),
            _ => unreachable!(),
        }
    }

    pub fn as_note(name: &String) -> Option<&i64> {
        NOTE_MAP.get(name)
    }

}
