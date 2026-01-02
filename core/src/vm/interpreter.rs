use crate::{
    clock::SyncTime,
    vm::{EvaluationContext, event::ConcreteEvent},
};

pub mod asm_interpreter;
mod directory;
mod factory;

pub mod external;

pub use directory::InterpreterDirectory;
pub use factory::InterpreterFactory;

pub trait Interpreter {
    fn execute_next(&mut self, ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime);

    fn has_terminated(&self) -> bool;

    fn stop(&mut self);
}
