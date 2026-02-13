use std::f64::consts::PI;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{vm::{EvaluationContext, variable::VariableValue}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum GeneratorShape {
    #[default]
    Sine,
    Saw,
    Triangle(Box<VariableValue>),
    Square(Box<VariableValue>),
    Stairs(Box<VariableValue>),
    RandFloat,
    RandInt,
    RandUInt(Box<VariableValue>),
    Table(Vec<VariableValue>),
}

impl GeneratorShape {
    pub fn configure(&mut self, value: VariableValue, ctx: &EvaluationContext) {
        match self {
            GeneratorShape::Sine 
            | GeneratorShape::Saw 
            | GeneratorShape::RandFloat 
            | GeneratorShape::RandInt 
                => (),
            GeneratorShape::RandUInt(x)
            | GeneratorShape::Triangle(x) 
            | GeneratorShape::Square(x) 
            | GeneratorShape::Stairs(x) 
                => *x = Box::new(value),
            | GeneratorShape::Table(values) => *values = value.as_vec(ctx)
        }
    }

    pub fn get_value(&self, ctx: &EvaluationContext, rng: &mut impl Rng, phase: f64) -> VariableValue {
        match self {
            Self::Sine => (phase * 2.0 * PI).sin().into(),
            Self::Saw => phase.into(),
            Self::Triangle(duty) => {
                let duty = duty.yield_float(ctx);
                if duty == 0.0 {
                    return 0.0.into();
                }
                if phase < duty {
                    (phase * (1.0 / duty)).into()
                } else {
                    (1.0 - (phase - duty) * (1.0 / (1.0 - duty))).into()
                }
            }
            Self::Square(duty) => {
                let duty = duty.yield_float(ctx);
                if phase < duty {
                    (1.0).into()
                } else {
                    (0.0).into()
                }
            }
            Self::RandFloat => rng.random::<f64>().into(),
            Self::RandInt => rng.random::<i64>().into(),
            Self::RandUInt(n) => { 
                let n = n.yield_integer(ctx) as u64;
                ((rng.random::<u64>() % n) as i64).into()
            }
            Self::Table(values) => {
                if values.is_empty() {
                    return VariableValue::default();
                }
                let index = (phase * values.len() as f64) as usize;
                values.get(index).cloned().unwrap_or_default()
            }
            Self::Stairs(n) => {
                let n = n.yield_float(ctx);
                let step_len = 1.0 / n;
                let current_step = (phase / step_len).floor();
                (current_step * step_len).into()
            }
        }
    }
}
