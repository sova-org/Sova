use std::{collections::HashMap, error, fmt, io::Write, process::{Command, Stdio}, string::FromUtf8Error};

use crate::lang::Program;

pub mod dummylang;
pub mod boinx;
pub mod bali;

#[derive(Debug, Clone)]
pub struct CompilationError {
    pub lang: String,
    pub info: String, 
    pub from: usize,
    pub to: usize,
}

impl CompilationError {
    fn default_error(lang: String) -> Self {
        Self {
            lang,
            info: "unknown error (todo)".to_string(),
            from: 0,
            to: 0,
        }
    }
}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} error: {}", self.lang, self.info)
    }
}

impl error::Error for CompilationError {}
impl From<std::io::Error> for CompilationError {
    fn from(_ : std::io::Error) -> Self { CompilationError::default_error("io".to_string()) }
}
impl From<std::process::Output> for CompilationError {
    fn from(_ : std::process::Output) -> Self { CompilationError::default_error("process".to_string()) }
}
impl From<FromUtf8Error> for CompilationError {
    fn from(_ : FromUtf8Error) -> Self { CompilationError::default_error("FromUtf8".to_string()) }
}
impl From<serde_json::Error> for CompilationError {
    fn from(_ : serde_json::Error) -> Self { CompilationError::default_error("serde_json".to_string()) }
}

/// A trait for types that can compile source code text into a `Program`.
/// Implementors must be safe to send and share across threads.
pub trait Compiler: Send + Sync + std::fmt::Debug {
    /// Returns the name identifier for this compiler (e.g., "boinx", "bali").
    fn name(&self) -> String;

    fn compile(&self, text : &str) -> Result<Program, CompilationError>;
}

#[derive(Debug)]
pub struct ExternalCompiler(pub String);

impl Compiler for ExternalCompiler {
    fn name(&self) -> String {
        self.0.clone()
    }

    fn compile(&self, text : &str) -> Result<Program, CompilationError> {
        let mut compiler = Command::new(&self.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let Some(mut stdin) = compiler.stdin.take() else {
            return Err(CompilationError::default_error("External language".to_string()))
        };
        stdin.write_all(text.as_bytes())?;
        let compiled = compiler.wait_with_output()?;
        let compiled = String::from_utf8(compiled.stdout)?;
        let prog : Program = serde_json::from_str(&compiled)?;
        Ok(prog)
    }

}

pub type CompilerCollection = HashMap<String, Box<dyn Compiler>>;