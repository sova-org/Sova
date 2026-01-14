pub struct GeneratorState {
    pub rng: Option<ChaCha20Rng>,
    pub seed: Box<VariableValue>,
    pub start_date: SyncTime,
    pub shape_state: Box<VariableValue>,
    pub modifier_states: Vec<VariableValue>
}