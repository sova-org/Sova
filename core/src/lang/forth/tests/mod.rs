use crate::lang::forth::ForthInterpreter;

mod arithmetic;
mod comparison;
mod control_flow;
mod stack;

pub fn run_forth(source: &str) -> Vec<f64> {
    let mut interp = ForthInterpreter::new(source);
    interp.run();
    interp.stack().to_vec()
}
