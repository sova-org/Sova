use serde::{Deserialize, Serialize};

use super::variable::Variable;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlASM {
    Mov(Variable, Variable),
    JumpIfLess(Variable, Variable, usize),
    JumpIf(Variable, usize),
    Add(Variable, Variable),
    Sub(Variable, Variable),
    And(Variable, Variable),
    Or(Variable, Variable),
    Cmp(Variable, Variable),
    Not(Variable),
    Goto(usize),
    Exit
}
