use crate::compiler::CompilationState;
use crate::scene::script::Script;
use crate::vm::interpreter::{Interpreter, InterpreterFactory};

use super::interpreter::ForthInterpreter;

pub struct ForthInterpreterFactory;

impl InterpreterFactory for ForthInterpreterFactory {
    fn name(&self) -> &str {
        "forth"
    }

    fn make_instance(&self, script: &Script) -> Result<Box<dyn Interpreter>, String> {
        Ok(Box::new(ForthInterpreter::new(script.content())))
    }

    fn check(&self, _script: &Script) -> CompilationState {
        // Parsed(None) indicates "checked and valid" without caching anything
        CompilationState::Parsed(None)
    }
}
