use std::sync::Arc;

use script::Script;

use crate::lang::variable::VariableStore;

pub mod script;

#[derive(Debug, Clone)]
pub struct Sequence {
    pub steps : Vec<f64>,  // Each step is defined by its length in beats
    pub sequence_vars : VariableStore,
    pub scripts : Vec<Arc<Script>>,
    pub speed_factor : f64
}

#[derive(Debug, Default)]
pub struct Pattern {
    pub sequences : Vec<Sequence>,
    pub sequence_index : usize
}

impl Pattern {

    pub fn current_sequence(&self) -> Option<&Sequence> {
        self.sequences.get(self.sequence_index)
    }

    pub fn current_sequence_mut(&mut self) -> Option<&mut Sequence> {
        self.sequences.get_mut(self.sequence_index)
    }

}
