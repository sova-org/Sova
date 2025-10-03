use crate::device_map::DeviceMap;
use crate::clock::Clock;
use std::collections::VecDeque;
use std::sync::Arc;

use super::variable::{Variable, VariableStore, VariableValue};

pub struct EvaluationContext<'a> {
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
    pub device_map: Arc<DeviceMap>,
}

impl<'a> EvaluationContext<'a> {

    pub fn set_var(&mut self, var: &Variable, value: VariableValue) {
        match var {
            Variable::Global(n) => {
                self.global_vars
                    .insert(n.clone(), value, self.clock, self.frame_len)
            }
            Variable::Line(n) => {
                self.line_vars
                    .insert(n.clone(), value, self.clock, self.frame_len)
            }
            Variable::Frame(n) => {
                self.frame_vars
                    .insert(n.clone(), value, self.clock, self.frame_len)
            }
            Variable::Instance(n) => {
                self.instance_vars
                    .insert(n.clone(), value, self.clock, self.frame_len)
            }
            _ => None,
        };
    }

    pub fn evaluate(&self, var: &Variable) -> VariableValue {
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
        };
        res.cloned().unwrap_or(false.into())
    }
}

#[derive(Default)]
pub struct PartialContext<'a> {
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
    pub device_map: Option<&'a Arc<DeviceMap>>,
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
        self.global_vars.is_some() &&
            self.line_vars.is_some() &&
            self.frame_vars.is_some() &&
            self.instance_vars.is_some() &&
            self.stack.is_some() &&
            self.line_index.is_some() &&
            self.frame_index.is_some() &&
            self.frame_len.is_some() &&
            self.structure.is_some() &&
            self.clock.is_some() &&
            self.device_map.is_some()
    }

    pub fn child<'b>(&'b mut self) -> PartialContext<'b> 
        where 'a : 'b 
    {
        PartialContext { 
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
            device_map: self.device_map
        }
    }

}

impl<'a> From<PartialContext<'a>> for EvaluationContext<'a> {
    fn from(partial: PartialContext<'a>) -> Self {
        if !partial.is_complete() {
            panic!("Partial context is not complete !")
        }
        EvaluationContext { 
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
            device_map: Arc::clone(partial.device_map.unwrap())
        }
    }
}