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

// SOON TO BE DEPRECATED !!!
// Define public keys for storing oscillator state in line vars
pub const SINE_PHASE_KEY: &str = "_sine_phase";
pub const SINE_LAST_BEAT_KEY: &str = "_sine_last_beat";
pub const SAW_PHASE_KEY: &str = "_saw_phase";
pub const SAW_LAST_BEAT_KEY: &str = "_saw_last_beat";
pub const TRI_PHASE_KEY: &str = "_triangle_phase";
pub const TRI_LAST_BEAT_KEY: &str = "_triangle_last_beat";
// Rename Ramp keys to ISaw
pub const ISAW_PHASE_KEY: &str = "_isaw_phase";
pub const ISAW_LAST_BEAT_KEY: &str = "_isaw_last_beat";
// Add keys for RandStep
pub const RANDSTEP_PHASE_KEY: &str = "_randstep_phase";
pub const RANDSTEP_LAST_BEAT_KEY: &str = "_randstep_last_beat";
pub const RANDSTEP_VALUE_KEY: &str = "_randstep_value"; // Key to store current held value

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
