use crate::compiler::bali::bali_ast::{concrete_fraction::ConcreteFraction, constants::NOTE_MAP};
use crate::lang::{
    Instruction,
    control_asm::ControlASM,
    environment_func::EnvironmentFunc,
    variable::{Variable, VariableValue},
};

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Decimal(String),
    Variable(String),
    String(String), // Add String variant
}

impl Value {
    pub fn as_asm(&self) -> Instruction {
        match self {
            Value::Number(n) => {
                let (signe, n) = if *n < 0 { (-1, -*n) } else { (1, *n) };
                Instruction::Control(ControlASM::Push(Variable::Constant(
                    VariableValue::Decimal(signe, n as u64, 1),
                )))
            }
            Value::Decimal(d) => {
                let frac = ConcreteFraction::from_dec_string(d.clone());
                Instruction::Control(ControlASM::Push(Variable::Constant(
                    VariableValue::Decimal(
                        frac.signe as i8,
                        frac.numerator as u64,
                        frac.denominator as u64,
                    ),
                )))
            }
            Value::Variable(s) => match Self::as_note(s) {
                None => Instruction::Control(ControlASM::Push(Self::as_variable(s))),
                Some(n) => Value::Number(*n).as_asm(),
            },
            Value::String(s) => Instruction::Control(ControlASM::Push(s.clone().into())),
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            Value::Variable(s) => s.to_string(),
            Value::String(s) => s.to_string(),
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
