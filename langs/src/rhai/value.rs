use rhai::{Array as RhaiArray, Dynamic, FLOAT, INT, ImmutableString, Map as RhaiMap};
use sova_core::{
    clock::TimeSpan,
    vm::{EvaluationContext, variable::VariableValue},
};

#[derive(Debug, Clone, PartialEq)]
pub enum RhaiValue {
    Unit,
    Value(VariableValue),
}

impl RhaiValue {
    pub fn from_variable(value: VariableValue) -> Self {
        Self::Value(value)
    }

    pub fn from_dynamic(dynamic: Dynamic) -> Self {
        dynamic_to_variable(dynamic)
            .map(Self::Value)
            .unwrap_or(Self::Unit)
    }

    pub fn into_variable(self) -> VariableValue {
        match self {
            Self::Unit => VariableValue::default(),
            Self::Value(value) => value,
        }
    }

    pub fn truthy(&self, ctx: &EvaluationContext) -> bool {
        match self {
            Self::Unit => false,
            Self::Value(value) => value.yield_bool(ctx),
        }
    }

    pub fn as_i64(&self, ctx: &EvaluationContext) -> i64 {
        match self {
            Self::Unit => 0,
            Self::Value(value) => value.yield_integer(ctx),
        }
    }

    pub fn as_f64(&self, ctx: &EvaluationContext) -> f64 {
        match self {
            Self::Unit => 0.0,
            Self::Value(value) => value.yield_float(ctx),
        }
    }

    pub fn as_string(&self, ctx: &EvaluationContext) -> String {
        match self {
            Self::Unit => String::new(),
            Self::Value(value) => value.clone().as_str(ctx),
        }
    }

    pub fn duration(&self) -> Option<TimeSpan> {
        match self {
            Self::Value(VariableValue::Dur(d)) => Some(*d),
            _ => None,
        }
    }

    pub fn prefers_array_index(&self) -> bool {
        matches!(
            self,
            Self::Value(
                VariableValue::Integer(_)
                    | VariableValue::Float(_)
                    | VariableValue::Decimal(_)
                    | VariableValue::Bool(_)
            )
        )
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Value(value) => variable_type_name(value),
        }
    }
}

fn dynamic_to_variable(dynamic: Dynamic) -> Option<VariableValue> {
    if dynamic.is_unit() {
        return None;
    }
    if dynamic.is::<INT>() {
        return Some((dynamic.clone_cast::<INT>() as i64).into());
    }
    if dynamic.is::<FLOAT>() {
        return Some((dynamic.clone_cast::<FLOAT>() as f64).into());
    }
    if dynamic.is::<bool>() {
        return Some(dynamic.clone_cast::<bool>().into());
    }
    if dynamic.is::<ImmutableString>() {
        return Some(dynamic.clone_cast::<ImmutableString>().to_string().into());
    }
    if dynamic.is::<String>() {
        return Some(dynamic.clone_cast::<String>().into());
    }
    if dynamic.is::<RhaiArray>() {
        let values: RhaiArray = dynamic.clone_cast::<RhaiArray>();
        return Some(
            values
                .into_iter()
                .map(|value| dynamic_to_variable(value).unwrap_or_else(VariableValue::default))
                .collect::<Vec<_>>()
                .into(),
        );
    }
    if dynamic.is::<RhaiMap>() {
        let map: RhaiMap = dynamic.clone_cast::<RhaiMap>();
        return Some(
            map.into_iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        dynamic_to_variable(v).unwrap_or_else(VariableValue::default),
                    )
                })
                .collect::<std::collections::HashMap<_, _>>()
                .into(),
        );
    }

    None
}

fn variable_type_name(value: &VariableValue) -> &'static str {
    match value {
        VariableValue::Decimal(_) => "decimal",
        VariableValue::Func(_) => "function",
        VariableValue::Blob(_) => "blob",
        VariableValue::Generator(_) => "generator",
        VariableValue::Integer(_) => "int",
        VariableValue::Float(_) => "float",
        VariableValue::Bool(_) => "bool",
        VariableValue::Str(_) => "string",
        VariableValue::Dur(_) => "duration",
        VariableValue::Map(_) => "map",
        VariableValue::Vec(_) => "array",
    }
}
