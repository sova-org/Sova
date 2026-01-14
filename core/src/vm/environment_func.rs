use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnvironmentFunc {
    GetTempo,
    RandomUInt(u64),
    RandomInt,
    RandomFloat,
    RandomDecInBounds(Box<Variable>, Box<Variable>),
    FrameLen(Box<Variable>, Box<Variable>),
}

use super::{
    EvaluationContext,
    variable::{Variable, VariableValue},
};

impl EnvironmentFunc {
    pub fn execute(&self, ctx: &mut EvaluationContext) -> VariableValue {
        match self {
            EnvironmentFunc::GetTempo => ctx.clock.session_state.tempo().into(),
            EnvironmentFunc::RandomUInt(n) => ((rand::random::<u64>() % n) as i64).into(),
            EnvironmentFunc::RandomInt => rand::random::<i64>().into(),
            EnvironmentFunc::RandomFloat => rand::random::<f64>().into(),
            EnvironmentFunc::RandomDecInBounds(min, max) => {
                let min = ctx.evaluate(min).as_float(ctx) as f32;
                let max = ctx.evaluate(max).as_float(ctx) as f32;
                let mut val : VariableValue = if min >= max {
                    (max as f64).into()
                } else {
                    let rand_val: f32 = rand::random_range(min..max);
                    (rand_val as f64).into()
                };
                val.cast_as_decimal(ctx);
                val
            },
            EnvironmentFunc::FrameLen(x, y) => {
                let line_i = ctx.evaluate(x).as_integer(ctx) as usize;
                let frame_i = ctx.evaluate(y).as_integer(ctx) as usize;
                let dur = ctx.structure.get(line_i).and_then(|l| l.get(frame_i));
                dur.cloned().unwrap_or(0.0).into()
            }
        }
    }
}
