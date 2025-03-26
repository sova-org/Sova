use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnvironmentFunc {
    GetTempo,
    Random(Box<Variable>, Box<Variable>)
}
pub use EnvironmentFunc::*;

use super::{evaluation_context::EvaluationContext, variable::{Variable, VariableValue}};

impl EnvironmentFunc {

    pub fn execute(&self, ctx : &EvaluationContext) -> VariableValue {
        match self {
            GetTempo => ctx.clock.session_state.tempo().into(),
            Random(x, y) => {
                todo!()
            }
        }
    }

}
