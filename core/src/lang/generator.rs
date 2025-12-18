use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::lang::variable::VariableValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ValueGenerator {
    #[default]
    Sine,
    Saw,
    Triangle,
    Square(f64),
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
            Self::Reversed(inner) => inner.get_value(1.0 - phase),
        }
    }
}
