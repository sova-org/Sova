use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{vm::{EvaluationContext, variable::VariableValue}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum GeneratorModifier {
    #[default]
    Loop,
    StartAt(VariableValue),
    EndAt(VariableValue),
    Repeat(VariableValue),
    RandomPhase,
    Reverse
}

// Sine | TimeSpan 4' | Loop

impl GeneratorModifier {
    pub fn configure(&mut self, config: VariableValue) {
        match self {
            GeneratorModifier::Loop
            | GeneratorModifier::RandomPhase
            | GeneratorModifier::Reverse 
                => (),
            GeneratorModifier::EndAt(value) 
            | GeneratorModifier::StartAt(value) 
            | GeneratorModifier::Repeat(value) 
                => *value = config,
        }
    }

    pub fn reset(&self, _state: &mut VariableValue) {
        match self {
            GeneratorModifier::Loop |
            GeneratorModifier::StartAt(_) |
            GeneratorModifier::EndAt(_) |
            GeneratorModifier::RandomPhase |
            GeneratorModifier::Reverse 
                => (),
            GeneratorModifier::Repeat(_) => () // *state = 0.into()
        }
    }

    pub fn get_phase(&self, ctx: &EvaluationContext, _state: &mut VariableValue, rng: &mut impl Rng, incoming_phase: f64, span: f64) -> f64 {
        match self {
            GeneratorModifier::Loop => {
                incoming_phase % 1.0
            }
            GeneratorModifier::EndAt(d) => {
                let d = d.clone().as_dur(ctx).as_beats(ctx.clock, ctx.frame_len) / span;
                if incoming_phase > d {
                    1.0 + (incoming_phase - d)
                } else {
                    incoming_phase
                }
            }
            GeneratorModifier::StartAt(d) => {
                let d = d.clone().as_dur(ctx).as_beats(ctx.clock, ctx.frame_len) / span;
                incoming_phase + d
            }
            GeneratorModifier::Repeat(n) => {
                let to_do = n.clone().as_float(ctx);
                if incoming_phase < to_do {
                    incoming_phase % 1.0
                } else {
                    incoming_phase - to_do
                }
            }
            GeneratorModifier::RandomPhase => rng.random(),
            GeneratorModifier::Reverse => 1.0 - incoming_phase,
        }
    }
}