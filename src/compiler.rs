use std::{collections::HashMap, io::Write, process::{Command, Stdio}, string::FromUtf8Error};

use crate::lang::Program;

pub mod dummyast;
pub mod boinx;

#[derive(Debug, Clone, Copy)]
pub struct CompilationError;
impl From<std::io::Error> for CompilationError {
    fn from(_ : std::io::Error) -> Self { CompilationError }
}
impl From<FromUtf8Error> for CompilationError {
    fn from(_ : FromUtf8Error) -> Self { CompilationError }
}
impl From<serde_json::Error> for CompilationError {
    fn from(_ : serde_json::Error) -> Self { CompilationError }
}

pub trait Compiler {
    fn compile(&self, text : &str) -> Result<Program, CompilationError>;
}

pub struct ExternalCompiler(pub String);

impl Compiler for ExternalCompiler {
    fn compile(&self, text : &str) -> Result<Program, CompilationError> {
        let mut compiler = Command::new(&self.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let Some(mut stdin) = compiler.stdin.take() else {
            return Err(CompilationError)
        };
        stdin.write_all(text.as_bytes())?;
        let compiled = compiler.wait_with_output()?;
        let compiled = String::from_utf8(compiled.stdout)?;
        let prog : Program = serde_json::from_str(&compiled)?;
        Ok(prog)
    }
}

pub type CompilerCollection = HashMap<String, dyn Compiler>;
