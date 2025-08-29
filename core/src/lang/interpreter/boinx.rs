use crate::{clock::{SyncTime, TimeSpan}, lang::{evaluation_context::EvaluationContext, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}}, scene::script::Script};

mod boinx_ast;

use boinx_ast::*;

pub struct BoinxInterpreter {
    pub prog: BoinxProg,
    pub execution_lines: Vec<BoinxOutput>,
    pub time_span: TimeSpan,
    pub return_value: BoinxItem,
    pub start_date: SyncTime
}

impl Interpreter for BoinxInterpreter {

    fn execute_next(
        &mut self,
        ctx : &mut EvaluationContext
    ) -> (Option<ConcreteEvent>, Option<SyncTime>) { 
        for line in self.execution_lines.iter() {

        }
    }

    fn has_terminated(&self) -> bool {
        !self.execution_lines.is_empty()
    }

    fn stop(&mut self) {
        self.execution_lines.clear();
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