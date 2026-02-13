use crate::{compiler::CompilationState, scene::script::Script, vm::Language};

use super::Interpreter;

pub trait InterpreterFactory : Language + Send + Sync {

    fn make_instance(&self, script : &Script) -> Result<Box<dyn Interpreter>, String>;

    fn check(&self, script: &Script) -> CompilationState;

}