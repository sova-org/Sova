use serde::{Deserialize, Serialize};

use crate::{clock::SyncTime, vm::variable::Variable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum GeneratorModifier {
    #[default]
    Loop,
    ScaleSpan(Variable),
    Truncate(Variable, Variable),
    Repeat(Variable),
    RandomPhase,
    Reverse
}

impl GeneratorModifier {
    pub fn get_phase(&self, start_date: SyncTime, date: SyncTime) -> f64 {
        todo!()
    }
}