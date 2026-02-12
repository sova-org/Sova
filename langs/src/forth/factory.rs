use sova_core::compiler::CompilationState;
use sova_core::scene::script::Script;
use sova_core::vm::Language;
use sova_core::vm::interpreter::{Interpreter, InterpreterFactory};

use super::interpreter::ForthInterpreter;

pub struct ForthInterpreterFactory;

impl Language for ForthInterpreterFactory {
    fn name(&self) -> &str {
        "forth"
    }
}

impl InterpreterFactory for ForthInterpreterFactory {
    
    fn make_instance(&self, script: &Script) -> Result<Box<dyn Interpreter>, String> {
        Ok(Box::new(ForthInterpreter::new(script.content())))
    }

    fn check(&self, _script: &Script) -> CompilationState {
        // Parsed(None) indicates "checked and valid" without caching anything
        CompilationState::Parsed(None)
    }
}
