use crate::{clock::SyncTime, lang::{evaluation_context::EvaluationContext, event::ConcreteEvent}};

mod factory;
mod directory;
pub mod asm_interpreter;

pub mod boinx;
pub mod external;

pub use factory::InterpreterFactory;
pub use directory::InterpreterDirectory;

pub trait Interpreter : Send + Sync {

    fn execute_next(
        &mut self,
        ctx : &mut EvaluationContext
    ) -> (Option<ConcreteEvent>, Option<SyncTime>);

    fn has_terminated(&self) -> bool;

    fn stop(&mut self);

}

