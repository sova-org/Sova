use crate::bob::BobCompiler;
use sova_core::compiler::Compiler;
use sova_core::vm::runner::execute_program;
use std::collections::BTreeMap;

mod basics;
mod brace_syntax;
mod control_flow;
mod emit_dispatch;
mod euclidean;
mod expressions;
mod functions;
mod list_expansion;
mod lists;
mod maps;
mod midi;
mod operators;
mod osc;
mod selection;
mod sysex;

pub fn compile_and_run(source: &str) -> sova_core::vm::runner::ExecutionResult {
    let compiler = BobCompiler;
    let prog = compiler
        .compile(source, &BTreeMap::new())
        .expect("compilation failed");
    execute_program(prog)
}

#[allow(dead_code)]
pub fn compile_and_run_debug(source: &str) -> sova_core::vm::runner::ExecutionResult {
    let compiler = BobCompiler;
    let prog = compiler
        .compile(source, &BTreeMap::new())
        .expect("compilation failed");
    eprintln!("=== Compiled program ({} instructions) ===", prog.len());
    for (i, instr) in prog.iter().enumerate() {
        eprintln!("{:3}: {:?}", i, instr);
    }
    eprintln!("=== End program ===");
    execute_program(prog)
}
