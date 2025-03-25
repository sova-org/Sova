use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvironmentFunc {
    GetTempo,
}
pub use EnvironmentFunc::*;

use super::{evaluation_context::EvaluationContext, variable::VariableValue};

impl EnvironmentFunc {

    pub fn execute(&self, ctx : &EvaluationContext) -> VariableValue {
        match self {
            GetTempo => ctx.clock.session_state.tempo().into(),
        }
    }

}
