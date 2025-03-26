use crate::{clock::Clock, pattern::{script::Script, Sequence}};

use super::variable::{Variable, VariableStore, VariableValue};

pub struct EvaluationContext<'a> {
    pub global_vars : &'a mut VariableStore,
    pub step_vars : &'a mut VariableStore,
    pub instance_vars : &'a mut VariableStore,
    pub sequences : &'a mut [Sequence],
    pub current_sequence : usize,
    pub script : &'a Script,
    pub clock : &'a Clock
}

impl<'a> EvaluationContext<'a> {

    pub fn sequence(&self) -> &Sequence {
        &self.sequences[self.current_sequence % self.sequences.len()]
    }

    pub fn sequence_mut(&mut self) -> &mut Sequence {
        &mut self.sequences[self.current_sequence % self.sequences.len()]
    }

    pub fn set_var(&mut self, var : &Variable, value : VariableValue) {
        match var {
            Variable::Global(n) => self.global_vars.insert(n.clone(), value),
            Variable::Sequence(n) => self.sequence_mut().vars.insert(n.clone(), value),
            Variable::Step(n) => self.step_vars.insert(n.clone(), value),
            Variable::Instance(n) => self.instance_vars.insert(n.clone(), value),
            _ => None
        };
    }

    pub fn evaluate(&self, var : &Variable) -> VariableValue {
        let res = match var {
            Variable::Global(n) => self.global_vars.get(n),
            Variable::Sequence(n) => self.sequence().vars.get(n),
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
