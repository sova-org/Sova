use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnvironmentFunc {
    GetTempo,
    RandomU8,
    RandomInt,
    RandomFloat,
    StepLen(Box<Variable>, Box<Variable>)
}
pub use EnvironmentFunc::*;

use super::{evaluation_context::EvaluationContext, variable::{Variable, VariableValue}};

impl EnvironmentFunc {

    pub fn execute(&self, ctx : &EvaluationContext) -> VariableValue {
        match self {
            GetTempo => ctx.clock.session_state.tempo().into(),
            RandomU8 => (rand::random::<u8>() as i64).into(),
            RandomInt => rand::random::<i64>().into(),
            RandomFloat => rand::random::<f64>().into(),
            StepLen(x, y) => {
                let seq_i = ctx.evaluate(x).as_integer(ctx) as usize;
                let step_i = ctx.evaluate(y).as_integer(ctx) as usize;
                ctx.sequences[seq_i % ctx.sequences.len()].step_len(step_i).into()
            },
        }
    }

}
