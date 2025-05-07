use crate::lang::{
    Instruction,
    control_asm::ControlASM,
    variable::Variable,
    environment_func::EnvironmentFunc,
};
use crate::compiler::bali::bali_ast::constants::NOTE_MAP;

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
            Value::Variable(s) => match Self::as_note(s) {
                None => Instruction::Control(ControlASM::Push(Self::as_variable(s))),
                Some(n) => Instruction::Control(ControlASM::Push((*n).into())),
            },
            Value::String(_s) => {
                // Pushing strings directly to the numeric/variable stack is problematic.
                // For the OSC command, we handle Value::String directly in Effect::as_asm.
                // If strings need general stack support, the VM/VariableType needs extension.
                // For now, generate a Nop or error if String is used outside OSC?
                // Let's generate a Push of 0 as a placeholder, assuming it won't be used elsewhere yet.
                eprintln!(
                    "[WARN] Bali VM: Pushing String as 0 to stack (Value::as_asm). String support is limited."
                );
                Instruction::Control(ControlASM::Push(0i64.into()))
            }
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
