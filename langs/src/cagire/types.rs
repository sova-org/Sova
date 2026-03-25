use std::sync::Arc;

use sova_core::compiler::CompilationError;
use sova_core::vm::variable::VariableValue;

use super::ops::Op;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub(crate) struct CagireError {
    pub message: String,
    pub span: Span,
}

impl CagireError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self { message: message.into(), span }
    }

    pub fn into_compilation_error(self) -> CompilationError {
        CompilationError {
            lang: "cagire".into(),
            info: self.message,
            from: self.span.start,
            to: self.span.end,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(Arc<str>),
    Quotation(Arc<[Op]>, Arc<[Span]>),
    CycleList(Arc<[Value]>),
}

impl Value {
    pub fn as_float(&self) -> Result<f64, String> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as f64),
            _ => Err("expected number".into()),
        }
    }

    pub(super) fn as_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(i) => Ok(*i),
            Value::Float(f) => Ok(*f as i64),
            _ => Err("expected number".into()),
        }
    }

    pub(super) fn as_str(&self) -> Result<&str, String> {
        match self {
            Value::Str(s) => Ok(s),
            _ => Err("expected string".into()),
        }
    }

    pub(super) fn is_truthy(&self) -> bool {
        match self {
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Quotation(..) => true,
            Value::CycleList(items) => !items.is_empty(),
        }
    }

    pub(super) fn to_param_string(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Str(s) => s.to_string(),
            Value::Quotation(..) | Value::CycleList(_) => String::new(),
        }
    }

    pub(super) fn to_variable_value(&self) -> Option<VariableValue> {
        match self {
            Value::Int(i) => Some(VariableValue::Integer(*i)),
            Value::Float(f) => Some(VariableValue::Float(*f)),
            Value::Str(s) => Some(VariableValue::Str(s.to_string())),
            Value::Quotation(..) | Value::CycleList(_) => None,
        }
    }

    pub(super) fn from_variable_value(vv: &VariableValue) -> Self {
        match vv {
            VariableValue::Integer(i) => Value::Int(*i),
            VariableValue::Float(f) => Value::Float(*f),
            VariableValue::Str(s) => Value::Str(Arc::from(s.as_str())),
            VariableValue::Bool(b) => Value::Int(if *b { 1 } else { 0 }),
            _ => Value::Float(0.0),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct CmdRegister {
    sound: Option<Value>,
    params: Vec<(&'static str, Value)>,
    deltas: Vec<Value>,
    global_params: Vec<(&'static str, Value)>,
    delta_secs: Option<f64>,
}

impl CmdRegister {
    pub(super) fn new() -> Self {
        Self {
            sound: None,
            params: Vec::with_capacity(16),
            deltas: Vec::with_capacity(4),
            global_params: Vec::new(),
            delta_secs: None,
        }
    }

    pub(super) fn set_sound(&mut self, val: Value) {
        self.sound = Some(val);
    }

    pub(super) fn get_param_float(&self, key: &str) -> Option<f64> {
        self.params.iter().find(|(k, _)| *k == key)?.1.as_float().ok()
    }

    pub(super) fn set_param(&mut self, key: &'static str, val: Value) {
        self.params.push((key, val));
    }

    pub(super) fn set_deltas(&mut self, deltas: Vec<Value>) {
        self.deltas = deltas;
    }

    pub(super) fn deltas(&self) -> &[Value] {
        &self.deltas
    }

    pub(super) fn sound(&self) -> Option<&Value> {
        self.sound.as_ref()
    }

    pub(super) fn params(&self) -> &[(&'static str, Value)] {
        &self.params
    }

    #[allow(clippy::type_complexity)]
    pub(super) fn snapshot(&self) -> Option<(Option<&Value>, &[(&'static str, Value)])> {
        if self.sound.is_some() || !self.params.is_empty() {
            Some((self.sound.as_ref(), self.params.as_slice()))
        } else {
            None
        }
    }

    pub(super) fn global_params(&self) -> &[(&'static str, Value)] {
        &self.global_params
    }

    pub(super) fn commit_global(&mut self) {
        self.global_params.append(&mut self.params);
        self.sound = None;
        self.deltas.clear();
    }

    pub(super) fn clear_global(&mut self) {
        self.global_params.clear();
    }

    pub(super) fn set_global(&mut self, params: Vec<(&'static str, Value)>) {
        self.global_params = params;
    }

    pub(super) fn take_global(&mut self) -> Vec<(&'static str, Value)> {
        std::mem::take(&mut self.global_params)
    }

    pub(super) fn set_delta_secs(&mut self, secs: f64) {
        self.delta_secs = Some(secs);
    }

    pub(super) fn take_delta_secs(&mut self) -> Option<f64> {
        self.delta_secs.take()
    }

    pub(super) fn clear_sound(&mut self) {
        self.sound = None;
    }

    pub(super) fn clear_params(&mut self) {
        self.params.clear();
    }

    pub(super) fn clear(&mut self) {
        self.sound = None;
        self.params.clear();
        self.deltas.clear();
        self.delta_secs = None;
    }
}
