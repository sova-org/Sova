use crate::{clock::Clock, pattern::{script::Script, Sequence}};

use super::variable::{Variable, VariableStore, VariableValue};

pub struct EvaluationContext<'a> {
    pub global_vars : &'a mut VariableStore,
    pub step_vars : &'a mut VariableStore,
    pub instance_vars : &'a mut VariableStore,
    pub sequence : &'a mut Sequence,
    pub script : &'a Script,
    pub clock : &'a Clock
}

impl<'a> EvaluationContext<'a> {

    pub fn set_var(&mut self, var : &Variable, value : VariableValue) {
        match var {
            Variable::Global(n) => self.global_vars.insert(n.clone(), value),
            Variable::Sequence(n) => self.sequence.sequence_vars.insert(n.clone(), value),
            Variable::Step(n) => self.step_vars.insert(n.clone(), value),
            Variable::Instance(n) => self.instance_vars.insert(n.clone(), value),
            _ => None
        };
    }

    pub fn evaluate(&self, var : &Variable) -> VariableValue {
        let res = match var {
            Variable::Global(n) => self.global_vars.get(n),
            Variable::Sequence(n) => self.sequence.sequence_vars.get(n),
            Variable::Step(n) => self.step_vars.get(n),
            Variable::Instance(n) => self.instance_vars.get(n),
            Variable::Environment(environment_func) => {
                return environment_func.execute(self);
            },
            Variable::Constant(x) => {
                return x.clone();
            },
        };
        res.map(VariableValue::clone).unwrap_or(false.into())
    }

}
