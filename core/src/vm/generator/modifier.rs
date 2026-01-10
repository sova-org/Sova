use serde::{Deserialize, Serialize};

use crate::{clock::{SyncTime, TimeSpan}, vm::{EvaluationContext, variable::{Variable, VariableValue}}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum GeneratorModifier {
    #[default]
    Loop,
    Truncate(VariableValue),
    StartAt(VariableValue),
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
            GeneratorModifier::Truncate(value) 
            | GeneratorModifier::StartAt(value) 
            | GeneratorModifier::Repeat(value) 
                => *value = config,
        }
    }

    pub fn reset(&self, state: &mut VariableValue) {

    }

    pub fn get_phase(&self, ctx: &EvaluationContext, state: &mut VariableValue, incoming_phase: f64) -> f64 {
        match self {
            GeneratorModifier::Loop => {
                incoming_phase % 1.0
            }
            GeneratorModifier::Truncate(d) => todo!(),
            GeneratorModifier::StartAt(d) => todo!(),
            GeneratorModifier::Repeat(n) => todo!(),
            GeneratorModifier::RandomPhase => rand::random(),
            GeneratorModifier::Reverse => 1.0 - incoming_phase,
        }
    }
}