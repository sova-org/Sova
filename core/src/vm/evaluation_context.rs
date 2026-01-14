use serde::Serialize;

use crate::clock::Clock;
use crate::{clock::SyncTime, device_map::DeviceMap};
use std::collections::VecDeque;

use super::variable::{Variable, VariableStore, VariableValue};

/// Context that stores everything necessary for stateful script execution.
#[derive(Serialize)]
pub struct EvaluationContext<'a> {
    pub logic_date: SyncTime,
    pub global_vars: &'a mut VariableStore,
    pub line_vars: &'a mut VariableStore,
    pub frame_vars: &'a mut VariableStore,
    pub instance_vars: &'a mut VariableStore,
    pub stack: &'a mut VecDeque<VariableValue>,
    pub line_index: usize,
    pub frame_index: usize,
    pub frame_len: f64,
    pub structure: &'a Vec<Vec<f64>>,
    pub clock: &'a Clock,
    #[serde(skip)]
    pub device_map: &'a DeviceMap,
}

impl<'a> EvaluationContext<'a> {

    pub fn set_var<T : Into<VariableValue>>(&mut self, var: &Variable, value: T) {
        let value : VariableValue = value.into();
        match var {
            Variable::Global(n) => {
                self.global_vars
                    .insert(n.clone(), value);
            }
            Variable::Line(n) => {
                self.line_vars
                    .insert(n.clone(), value);
            }
            Variable::Frame(n) => {
                self.frame_vars
                    .insert(n.clone(), value);
            }
            Variable::Instance(n) => {
                self.instance_vars
                    .insert(n.clone(), value);
            }
            Variable::StackBack => {
                self.stack.push_back(value);
            }
            Variable::StackFront => {
                self.stack.push_front(value);
            }
            _ => (),
        };
    }

    pub fn evaluate(&mut self, var: &Variable) -> VariableValue {
        let res = match var {
            Variable::Global(n) => self.global_vars.get(n),
            Variable::Line(n) => self.line_vars.get(n),
            Variable::Frame(n) => self.frame_vars.get(n),
            Variable::Instance(n) => self.instance_vars.get(n),
            Variable::Environment(environment_func) => {
                return environment_func.execute(self);
            }
            Variable::Constant(x) => {
                return x.clone();
            }
            Variable::StackBack => {
                return self.stack.pop_back().unwrap_or_default()
            }
            Variable::StackFront => {
                return self.stack.pop_front().unwrap_or_default()
            }
        };
        if let Some(VariableValue::Generator(g)) = res {
            g.get_current(self)
        } else {
            res.cloned().unwrap_or_default()
        }
    }

    pub fn has_var(&self, var: &Variable) -> bool {
        match var {
            Variable::Global(n) => self.global_vars.has(n),
            Variable::Line(n) => self.line_vars.has(n),
            Variable::Frame(n) => self.frame_vars.has(n),
            Variable::Instance(n) => self.instance_vars.has(n),
            Variable::Environment(_) | Variable::Constant(_) => {
                true
            }
            Variable::StackBack | Variable::StackFront => {
                !self.stack.is_empty()
            }
        }
    }

    pub fn value_ref<'b>(&'b self, var: &'b Variable) -> Option<&'b VariableValue> {
        match var {
            Variable::Global(n) => self.global_vars.get(n),
            Variable::Line(n) => self.line_vars.get(n),
            Variable::Frame(n) => self.frame_vars.get(n),
            Variable::Instance(n) => self.instance_vars.get(n),
            Variable::Constant(x) => Some(x),
            Variable::StackBack => {
                self.stack.back()
            }
            Variable::StackFront => {
                self.stack.front()
            }
            Variable::Environment(_) => None,
        }
    }

    pub fn value_ref_mut(&mut self, var: &Variable) -> Option<&mut VariableValue> {
        match var {
            Variable::Global(n) => self.global_vars.get_mut(n),
            Variable::Line(n) => self.line_vars.get_mut(n),
            Variable::Frame(n) => self.frame_vars.get_mut(n),
            Variable::Instance(n) => self.instance_vars.get_mut(n),
            Variable::StackBack => {
                self.stack.back_mut()
            }
            Variable::StackFront => {
                self.stack.front_mut()
            }
            Variable::Constant(_) | Variable::Environment(_) => None,
        }
    }

    pub fn with_len(&'_ mut self, len: f64) -> EvaluationContext<'_> {
        EvaluationContext {
            logic_date: self.logic_date,
            global_vars: self.global_vars,
            line_vars: self.line_vars,
            frame_vars: self.frame_vars,
            instance_vars: self.instance_vars,
            stack: self.stack,
            line_index: self.line_index,
            frame_index: self.frame_index,
            frame_len: len,
            structure: self.structure,
            clock: self.clock,
            device_map: self.device_map,
        }
    }

    pub fn with_relative_len(&'a mut self, len: f64) -> EvaluationContext<'a> {
        self.with_len(self.frame_len * len)
    }
}

/// Used to partially construct a `EvaluationContext` step by step by rafining fields when known.
#[derive(Default)]
pub struct PartialContext<'a> {
    pub logic_date: SyncTime,
    pub global_vars: Option<&'a mut VariableStore>,
    pub line_vars: Option<&'a mut VariableStore>,
    pub frame_vars: Option<&'a mut VariableStore>,
    pub instance_vars: Option<&'a mut VariableStore>,
    pub stack: Option<&'a mut VecDeque<VariableValue>>,
    pub line_index: Option<usize>,
    pub frame_index: Option<usize>,
    pub frame_len: Option<f64>,
    pub structure: Option<&'a Vec<Vec<f64>>>,
    pub clock: Option<&'a Clock>,
    pub device_map: Option<&'a DeviceMap>,
}

impl<'a> PartialContext<'a> {
    pub fn to_context(self) -> Option<EvaluationContext<'a>> {
        if self.is_complete() {
            Some(self.into())
        } else {
            None
        }
    }

    pub fn is_complete(&self) -> bool {
        self.global_vars.is_some()
            && self.line_vars.is_some()
            && self.frame_vars.is_some()
            && self.instance_vars.is_some()
            && self.stack.is_some()
            && self.line_index.is_some()
            && self.frame_index.is_some()
            && self.frame_len.is_some()
            && self.structure.is_some()
            && self.clock.is_some()
            && self.device_map.is_some()
    }

    /// Creates another partial context sharing the same fields as its parent, but allowing override of some.
    pub fn child<'b>(&'b mut self) -> PartialContext<'b> 
        where 'a : 'b 
    {
        PartialContext {
            logic_date: self.logic_date,
            global_vars: self.global_vars.as_deref_mut(),
            line_vars: self.line_vars.as_deref_mut(),
            frame_vars: self.frame_vars.as_deref_mut(),
            instance_vars: self.instance_vars.as_deref_mut(),
            stack: self.stack.as_deref_mut(),
            line_index: self.line_index,
            frame_index: self.frame_index,
            frame_len: self.frame_len,
            structure: self.structure,
            clock: self.clock,
            device_map: self.device_map,
        }
    }
}

impl<'a> From<PartialContext<'a>> for EvaluationContext<'a> {
    fn from(partial: PartialContext<'a>) -> Self {
        if !partial.is_complete() {
            panic!("Partial context is not complete !")
        }
        EvaluationContext {
            logic_date: partial.logic_date,
            global_vars: partial.global_vars.unwrap(),
            line_vars: partial.line_vars.unwrap(),
            frame_vars: partial.frame_vars.unwrap(),
            instance_vars: partial.instance_vars.unwrap(),
            stack: partial.stack.unwrap(),
            line_index: partial.line_index.unwrap(),
            frame_index: partial.frame_index.unwrap(),
            frame_len: partial.frame_len.unwrap(),
            structure: partial.structure.unwrap(),
            clock: partial.clock.unwrap(),
            device_map: partial.device_map.unwrap(),
        }
    }
}
