use crate::lang::forth::ForthInterpreter;
use crate::vm::runner::execute_interpreter;

mod stack;
mod arithmetic;
mod comparison;
mod control_flow;
mod midi;

pub fn run_forth(source: &str) -> crate::vm::runner::ExecutionResult {
    let interp = Box::new(ForthInterpreter::new(source));
    execute_interpreter(interp)
}
