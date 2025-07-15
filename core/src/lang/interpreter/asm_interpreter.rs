use std::sync::Arc;

use crate::{clock::{SyncTime}, lang::{evaluation_context::EvaluationContext, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}, variable::VariableStore, Program}, scene::line::Line, transcoder::Transcoder};

pub struct ASMInterpreter {
    compiled: Program,
    instruction_index: usize,
}

impl Interpreter for ASMInterpreter {

    fn execute_next(
        &mut self,
        ctx : &mut EvaluationContext
    ) -> Option<(ConcreteEvent, SyncTime)> {
        todo!()
    }

}

pub struct ASMInterpreterFactory {
    pub transcoder : Transcoder
}

impl ASMInterpreterFactory {

    pub fn new(transcoder : Transcoder) -> Self {
        ASMInterpreterFactory { transcoder }
    }

}

impl InterpreterFactory for ASMInterpreterFactory {

    fn name(&self) -> String {
        todo!()
    }

    fn make_instance(&self) -> Box<dyn Interpreter> {
        todo!()
    }

}