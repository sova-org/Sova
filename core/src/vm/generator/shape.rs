use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{vm::{EvaluationContext, variable::VariableValue}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum GeneratorShape {
    #[default]
    Sine,
    Saw,
    Triangle,
    Square(Box<VariableValue>),
    Stairs(Box<VariableValue>),
    RandFloat,
    RandInt,
    RandUInt(Box<VariableValue>),
    Table(Vec<VariableValue>),
}

impl GeneratorShape {
    pub fn configure(&mut self, value: VariableValue) {
        match self {
            GeneratorShape::Sine 
            | GeneratorShape::Saw 
            | GeneratorShape::Triangle 
            | GeneratorShape::RandFloat
            | GeneratorShape::RandInt 
                => (),
            GeneratorShape::RandUInt(x)
            | GeneratorShape::Square(x) 
            | GeneratorShape::Stairs(x) 
                => *x = Box::new(value),
            GeneratorShape::Table(variable_values) => todo!(),
        }
    }

    pub fn get_value(&self, ctx: &EvaluationContext, _internal: &mut VariableValue, phase: f64) -> VariableValue {
        match self {
            Self::Sine => (phase * 2.0 * PI).sin().into(),
            Self::Saw => phase.into(),
            Self::Triangle => todo!(),
            Self::Square(duty) => {
                let duty = duty.as_float(ctx.clock, ctx.frame_len);
                if phase < duty {
                    (1.0).into()
                } else {
                    (0.0).into()
                }
            }
            Self::RandFloat => rand::random::<f64>().into(),
            Self::RandInt => rand::random::<i64>().into(),
            Self::RandUInt(n) => { 
                let n = n.as_integer(ctx.clock, ctx.frame_len) as u64;
                ((rand::random::<u64>() % n) as i64).into()
            }
            Self::Table(values) => {
                if values.is_empty() {
                    return VariableValue::default();
                }
                let index = (phase * values.len() as f64) as usize;
                values.get(index).cloned().unwrap_or_default()
            }
            Self::Stairs(n) => {
                let n = n.as_float(ctx.clock, ctx.frame_len);
                let step_len = 1.0 / n;
                let current_step = (phase / step_len).floor();
                (current_step * step_len).into()
            }
        }
    }
}
