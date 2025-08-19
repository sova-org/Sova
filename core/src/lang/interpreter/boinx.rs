use crate::{clock::{SyncTime, TimeSpan}, lang::{evaluation_context::EvaluationContext, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}}, scene::script::Script};

mod boinx_ast;

use boinx_ast::*;

pub struct BoinxInterpreter {
    pub prog: BoinxProg,
    pub execution_lines: Vec<BoinxCompo>,
    pub time_span: TimeSpan,
    pub return_value: BoinxItem
}

impl Interpreter for BoinxInterpreter {

    fn execute_next(
        &mut self,
        ctx : &mut EvaluationContext
    ) -> (Option<ConcreteEvent>, Option<SyncTime>) {
        todo!()
    }

    fn has_terminated(&self) -> bool {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }

}

pub struct BoinxInterpreterFactory {

}

impl InterpreterFactory for BoinxInterpreterFactory {
    
    fn name(&self) -> &str {
        "boinx"
    }

    fn make_instance(&self, script : &Script) -> Box<dyn Interpreter> {
        todo!()
    }

}