use crate::device_map::DeviceMap;
use crate::{
    clock::Clock,
    scene::{line::Line, script::Script},
};
use std::sync::Arc;

use super::variable::{Variable, VariableStore, VariableValue};

pub struct EvaluationContext<'a> {
    pub global_vars: &'a mut VariableStore,
    pub frame_vars: &'a mut VariableStore,
    pub instance_vars: &'a mut VariableStore,
    pub stack: &'a mut Vec<VariableValue>,
    pub lines: &'a mut [Line],
    pub current_scene: usize,
    pub script: &'a Script,
    pub clock: &'a Clock,
    pub device_map: Arc<DeviceMap>,
}

impl<'a> EvaluationContext<'a> {
    pub fn line(&self) -> &Line {
        &self.lines[self.current_scene % self.lines.len()]
    }

    pub fn frame_len(&self) -> f64 {
        self.line().frame_len(self.script.index)
    }

    pub fn line_mut(&mut self) -> &mut Line {
        &mut self.lines[self.current_scene % self.lines.len()]
    }

    pub fn set_var(&mut self, var: &Variable, value: VariableValue) {
        match var {
            Variable::Global(n) => {
                self.global_vars
                    .insert(n.clone(), value, self.clock, self.frame_len())
            }
            Variable::Line(n) => {
                let frame_len = self.frame_len();
                let clock = self.clock;
                self.line_mut()
                    .vars
                    .insert(n.clone(), value, clock, frame_len)
            }
            Variable::Frame(n) => {
                self.frame_vars
                    .insert(n.clone(), value, self.clock, self.frame_len())
            }
            Variable::Instance(n) => {
                self.instance_vars
                    .insert(n.clone(), value, self.clock, self.frame_len())
            }
            _ => None,
        };
    }

    pub fn evaluate(&mut self, var: &Variable) -> VariableValue {
        let res = match var {
            Variable::Global(n) => self.global_vars.get(n),
            Variable::Line(n) => self.line().vars.get(n),
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
