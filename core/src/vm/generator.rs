mod shape;
use serde::{Deserialize, Serialize};
pub use shape::*;

mod modifier;
pub use modifier::*;

use crate::{clock::{SyncTime, TimeSpan}, vm::{EvaluationContext, variable::VariableValue}};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ValueGenerator {
    pub shape: GeneratorShape,
    pub modifiers: Vec<(GeneratorModifier, Box<VariableValue>)>,
    pub shape_state: Box<VariableValue>,
    pub start_date: SyncTime,
    pub span: TimeSpan
}

impl ValueGenerator {
    pub fn of_shape(shape: GeneratorShape) -> Self {
        ValueGenerator {
            shape, ..Default::default()
        }
    }

    pub fn get(&mut self, ctx: &EvaluationContext, date: SyncTime) -> VariableValue {
        let relative = date.saturating_sub(self.start_date);
        let mut phase = 0.0;
        for (modif, m_state) in self.modifiers.iter_mut().rev() {
            phase = modif.get_phase(ctx, m_state, phase);
        }
        self.shape.get_value(ctx, &mut self.shape_state, phase)
    }

    pub fn save_state(&self) -> VariableValue {
        let mut state = vec![*self.shape_state.clone()];
        for (_, m_state) in self.modifiers.iter() {
            state.push(*m_state.clone());
        }
        state.into()
    }

    pub fn set_state(&mut self, state: VariableValue) {
        let mut state = state.as_vec();
        for (i, (_, m_state)) in self.modifiers.iter_mut().enumerate().rev() {
            if (i + 1) < state.len() {
                **m_state = state.pop().unwrap();
            }
        }
        *self.shape_state = state.pop().unwrap_or_default();
    }
}