use std::sync::Arc;

use sova_core::compiler::CompilationError;
use sova_core::vm::variable::VariableValue;

use super::ops::Op;

#[derive(Clone, Debug)]
pub(crate) enum ResolvedValue {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl ResolvedValue {
    pub(crate) fn display(&self) -> String {
        match self {
            ResolvedValue::Int(i) => i.to_string(),
            ResolvedValue::Float(f) => format!("{f:.2}"),
            ResolvedValue::Bool(b) => if *b { "yes" } else { "no" }.into(),
        }
    }
}

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
        Self {
            message: message.into(),
            span,
        }
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

pub(crate) struct Stack {
    pub(super) values: Vec<Value>,
    pub(super) origins: Vec<Span>,
}

impl Stack {
    pub(super) fn new() -> Self {
        Self {
            values: Vec::with_capacity(16),
            origins: Vec::with_capacity(16),
        }
    }

    pub(super) fn push(&mut self, val: Value, origin: Span) {
        self.values.push(val);
        self.origins.push(origin);
    }

    pub(super) fn pop(&mut self) -> Result<Value, String> {
        self.origins.pop();
        self.values
            .pop()
            .ok_or_else(|| "stack underflow".to_string())
    }

    pub(super) fn pop_int(&mut self) -> Result<i64, String> {
        self.pop()?.as_int()
    }

    pub(super) fn pop_float(&mut self) -> Result<f64, String> {
        self.pop()?.as_float()
    }

    pub(super) fn pop_bool(&mut self) -> Result<bool, String> {
        Ok(self.pop()?.is_truthy())
    }

    pub(super) fn ensure(&self, n: usize) -> Result<(), String> {
        if self.values.len() < n {
            return Err("stack underflow".into());
        }
        Ok(())
    }

    pub(super) fn binary_op<F>(&mut self, f: F) -> Result<(), String>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = float_to_value(f(a.as_float()?, b.as_float()?));
        self.values.push(result);
        self.origins.push(Span::default());
        Ok(())
    }

    pub(super) fn cmp_op<F>(&mut self, f: F) -> Result<(), String>
    where
        F: Fn(f64, f64) -> bool,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = Value::Int(if f(a.as_float()?, b.as_float()?) {
            1
        } else {
            0
        });
        self.values.push(result);
        self.origins.push(Span::default());
        Ok(())
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub(super) fn last(&self) -> Option<&Value> {
        self.values.last()
    }

    pub(super) fn swap(&mut self, a: usize, b: usize) {
        self.values.swap(a, b);
        self.origins.swap(a, b);
    }

    pub(super) fn remove(&mut self, idx: usize) -> Value {
        self.origins.remove(idx);
        self.values.remove(idx)
    }

    pub(super) fn insert(&mut self, idx: usize, val: Value, origin: Span) {
        self.values.insert(idx, val);
        self.origins.insert(idx, origin);
    }

    pub(super) fn truncate(&mut self, len: usize) {
        self.values.truncate(len);
        self.origins.truncate(len);
    }

    pub(super) fn origin(&self, idx: usize) -> Span {
        self.origins.get(idx).copied().unwrap_or_default()
    }

    pub(super) fn pop_with_origin(&mut self) -> Result<(Value, Span), String> {
        let origin = self.origins.pop().unwrap_or_default();
        let val = self
            .values
            .pop()
            .ok_or_else(|| "stack underflow".to_string())?;
        Ok((val, origin))
    }
}

pub(super) fn float_to_value(result: f64) -> Value {
    if result.fract() == 0.0 && result.abs() < i64::MAX as f64 {
        Value::Int(result as i64)
    } else {
        Value::Float(result)
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct CmdRegister {
    sound: Option<Value>,
    params: Vec<(&'static str, Value)>,
    global_params: Vec<(&'static str, Value)>,
    delta_secs: Option<f64>,
}

impl CmdRegister {
    pub(super) fn new() -> Self {
        Self {
            sound: None,
            params: Vec::with_capacity(16),
            global_params: Vec::new(),
            delta_secs: None,
        }
    }

    pub(super) fn set_sound(&mut self, val: Value) {
        self.sound = Some(val);
    }

    pub(super) fn get_param_float(&self, key: &str) -> Option<f64> {
        self.params
            .iter()
            .find(|(k, _)| *k == key)?
            .1
            .as_float()
            .ok()
    }

    pub(super) fn set_param(&mut self, key: &'static str, val: Value) {
        self.params.push((key, val));
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
        self.delta_secs = None;
    }
}
