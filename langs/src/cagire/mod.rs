mod compiler;
mod factory;
mod interpreter;
mod ops;
mod theory;
mod types;
mod vm;
mod words;

pub use factory::CagireInterpreterFactory;

pub fn compiler_check(code: &str, dict: &mut std::collections::HashMap<String, Vec<ops::Op>>) -> Result<(), String> {
    compiler::compile_script(code, dict).map(|_| ())
}
