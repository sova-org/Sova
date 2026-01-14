use rand_chacha::ChaCha20Rng;

use crate::{clock::SyncTime, vm::variable::VariableValue};

pub struct GeneratorState {
    pub rng: Option<ChaCha20Rng>,
    pub seed: VariableValue,
    pub start_date: SyncTime,
    pub shape_state: VariableValue,
    pub modifier_states: Vec<VariableValue>
}