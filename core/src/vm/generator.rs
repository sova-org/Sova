use serde::{Deserialize, Serialize};

mod shape;
pub use shape::*;

mod modifier;
pub use modifier::*;

mod state;
pub use state::*;

use crate::{clock::{SyncTime, TimeSpan}, vm::{EvaluationContext, variable::VariableValue}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ValueGenerator {
    pub shape: GeneratorShape,
    pub modifiers: Vec<GeneratorModifier>,
    pub span: TimeSpan,
    pub started: SyncTime,
    pub state_id: usize
}

impl ValueGenerator {
    pub fn of_shape(shape: GeneratorShape) -> Self {
        ValueGenerator {
            shape, ..Default::default()
        }
    }

    pub fn start(&mut self, ctx: &EvaluationContext, date: SyncTime) {
        self.started = date;
    }

    pub fn seed(&mut self, ctx: &EvaluationContext, seed: VariableValue) {
        //let seed = seed.as_integer(ctx) as u64;
        //self.rng = Some(ChaCha20Rng::seed_from_u64(seed));
    }

    pub fn get_current(&self, ctx: &EvaluationContext) -> VariableValue {
        self.get(ctx, ctx.logic_date)
    }

    pub fn get(&self, ctx: &EvaluationContext, date: SyncTime) -> VariableValue {
        let span = self.span.as_beats(ctx.clock, ctx.frame_len);
        if span == 0.0 {
            return VariableValue::default();
        }
        todo!()
        // if self.rng.is_none() {
        //     self.rng = Some(ChaCha20Rng::from_rng(&mut rand::rng()));
        // }
        // let rng = self.rng.as_mut().unwrap();
        // let phase = date.saturating_sub(self.start_date);
        // let mut phase = ctx.clock.micros_to_beats(phase) / span;
        // for (modif, m_state) in self.modifiers.iter_mut().rev() {
        //     phase = modif.get_phase(ctx, m_state, rng, phase, span);
        // }
        // if phase < 0.0 || phase > 1.0 {
        //     return Default::default();
        // }
        // self.shape.get_value(ctx, &mut self.shape_state, rng, phase)
    }

    pub fn get_at(&self, ctx: &EvaluationContext, index: i64) -> VariableValue {
        todo!()
    }

    pub fn save_state(&self) -> VariableValue {
        // let mut state = vec![*self.seed.clone(), *self.shape_state.clone()];
        // for (_, m_state) in self.modifiers.iter() {
        //     state.push(*m_state.clone());
        // }
        // state.into()
        todo!()
    }

    pub fn set_state(&mut self, state: VariableValue) {
        // let mut state = state.as_vec();
        // for (i, (_, m_state)) in self.modifiers.iter_mut().enumerate().rev() {
        //     if (i + 2) < state.len() {
        //         **m_state = state.pop().unwrap();
        //     }
        // }
        // *self.shape_state = state.pop().unwrap_or_default();
        // *self.seed = state.pop().unwrap_or_default();
    }
}