use std::sync::Arc;

use crate::{clock::{Clock, SyncTime}, lang::{evaluation_context::EvaluationContext, event::ConcreteEvent, variable::VariableStore}, scene::line::Line};

pub mod asm_interpreter;

pub trait Interpreter {

    fn execute_next(
        &mut self,
        ctx : &mut EvaluationContext
    ) -> (Option<ConcreteEvent>, Option<SyncTime>);

    fn has_terminated(&self) -> bool;

    fn stop(&mut self);

}

pub trait InterpreterFactory {

    fn name(&self) -> String;

    fn make_instance(&self, content : String) -> Box<dyn Interpreter>;

}