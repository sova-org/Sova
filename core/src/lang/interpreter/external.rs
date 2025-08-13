use std::process::Child;

use crate::{clock::SyncTime, lang::{evaluation_context::EvaluationContext, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}}, scene::script::Script};

pub struct ExternalInterpreter {
    process: Child,
    terminated: bool, // Storing termination status in a variable in order for has_terminated to only require &self and not &mut self
}

impl Interpreter for ExternalInterpreter {

    fn execute_next(
        &mut self,
        ctx : &mut EvaluationContext
    ) -> (Option<ConcreteEvent>, Option<SyncTime>) {
        todo!()
    }

    fn has_terminated(&self) -> bool {
        self.terminated
    }

    fn stop(&mut self) {
        self.process.kill();
        self.terminated = true;
    }

}

pub struct ExternalInterpreterFactory {

}

impl InterpreterFactory for ExternalInterpreterFactory {

    fn name(&self) -> &str {
        "external"
    }

    fn make_instance(&self, script : &Script) -> Box<dyn Interpreter> {
        let executable = script.args.get("command");
        todo!()
    }

}