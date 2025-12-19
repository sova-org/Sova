use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::vm::variable::VariableValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ValueGenerator {
    #[default]
    Sine,
    Saw,
    Triangle,
    Square(f64),
    Stairs(f64),
    RandFloat,
    RandInt,
    Reversed(Box<ValueGenerator>),
    Table(Vec<VariableValue>),
}

impl ValueGenerator {
    pub fn get_value(&self, phase: f64) -> VariableValue {
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
            Self::Reversed(inner) => inner.get_value(1.0 - phase),
        }
    }
}
