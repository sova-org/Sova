use serde::{Deserialize, Serialize};

mod shape;
pub use shape::*;

mod modifier;
pub use modifier::*;

mod state;
pub use state::*;

use crate::{
    clock::{SyncTime, TimeSpan},
    vm::{EvaluationContext, variable::VariableValue},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ValueGenerator {
    pub shape: GeneratorShape,
    pub modifiers: Vec<GeneratorModifier>,
    pub span: TimeSpan,
    pub state_id: usize,
}

impl ValueGenerator {
    pub fn of_shape(shape: GeneratorShape) -> Self {
        ValueGenerator {
            shape,
            ..Default::default()
        }
    }

    pub fn start(&mut self, _ctx: &EvaluationContext, _date: SyncTime) {
        //
    }

    pub fn seed(&mut self, _ctx: &EvaluationContext, _seed: VariableValue) {
        //let seed = seed.as_integer(ctx) as u64;
        //self.rng = Some(ChaCha20Rng::seed_from_u64(seed));
    }

    pub fn get_current(&self, ctx: &EvaluationContext) -> VariableValue {
        self.get(ctx, ctx.logic_date)
    }

    pub fn get(&self, ctx: &EvaluationContext, _date: SyncTime) -> VariableValue {
        let span = self.span.as_beats(ctx.clock, ctx.frame_len);
        if span == 0.0 {
            return VariableValue::default();
        }
        todo!()
    }

    pub fn save_state(&self) -> VariableValue {
        todo!()
    }

    pub fn set_state(&mut self, _state: VariableValue) {}
}
