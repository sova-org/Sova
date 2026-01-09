use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{clock::TimeSpan, vm::variable::{Variable, VariableValue}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum GeneratorShape {
    #[default]
    Sine,
    Saw,
    Triangle,
    Square(f64),
    Stairs(f64),
    RandFloat,
    RandInt,
    Table(Vec<VariableValue>),
}

impl GeneratorShape {
    pub fn get_value(&self, _internal: &mut VariableValue, phase: f64) -> VariableValue {
        match self {
            Self::Sine => (phase * 2.0 * PI).sin().into(),
            Self::Saw => phase.into(),
            Self::Triangle => todo!(),
            Self::Square(duty) => {
                if phase < *duty {
                    (1.0).into()
                } else {
                    (0.0).into()
                }
            }
            Self::RandFloat => rand::random::<f64>().into(),
            Self::RandInt => rand::random::<i64>().into(),
            Self::Table(values) => {
                if values.is_empty() {
                    return VariableValue::default();
                }
                let index = (phase * values.len() as f64) as usize;
                values[index].clone()
            }
            Self::Stairs(n) => {
                let step_len = 1.0 / n;
                let current_step = (phase / step_len).floor();
                (current_step * step_len).into()
            }
        }
    }
}
