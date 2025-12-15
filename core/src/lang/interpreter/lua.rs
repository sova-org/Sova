use rhai::packages::LanguageCorePackage;

use crate::{clock::SyncTime, lang::{event::ConcreteEvent, interpreter::Interpreter, EvaluationContext}};

pub struct LuaInterpreter {
    content: String    
}

impl Interpreter for LuaInterpreter {
    fn execute_next(&mut self, ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        todo!()
    }

    fn has_terminated(&self) -> bool {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}
