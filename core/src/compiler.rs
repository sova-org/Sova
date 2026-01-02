//! Defines the core traits and structures for compiling source code into Sova programs.
//!
//! This module provides the [`Compiler`] trait, which abstracts the compilation
//! process for different languages, and the [`CompilationError`] type for
//! handling errors during compilation. It also includes an implementation for
//! invoking external compiler executables ([`ExternalCompiler`]).

use std::{
    collections::BTreeMap,
    io::Write,
    process::{Command, Stdio}, sync::Arc,
};

use crate::vm::Program;

mod compilation_error;
pub use compilation_error::CompilationError;

mod compilation_state;
pub use compilation_state::CompilationState;

/// A trait for types that can compile source code text into a [`Program`].
///
/// Implementors define how source code for a specific language or system
/// is transformed into the intermediate Sova program representation.
/// Implementors must be safe to send and share across threads (`Send + Sync`).
pub trait Compiler: Send + Sync + std::fmt::Debug {
    /// Returns the unique name identifier for this compiler (e.g., "boinx", "bali").
    ///
    /// This name is used for identification and potentially for locating related resources
    /// like syntax definition files.
    fn name(&self) -> &str;

    /// Compiles the given source code text into a [`Program`].
    ///
    /// # Arguments
    ///
    /// * `text` - The source code string to compile.
    ///
    /// # Returns
    ///
    /// * `Ok(Program)` if compilation is successful.
    /// * `Err(CompilationError)` if any error occurs during compilation.
    fn compile(&self, text: &str, args: &BTreeMap<String, String>) -> Result<Program, CompilationError>;
}

/// A [`Compiler`] implementation that delegates compilation to an external executable.
///
/// It communicates with the external process via standard input (sending source code)
/// and standard output (receiving the compiled [`Program`] as JSON).
#[derive(Debug)]
pub struct ExternalCompiler {
    pub name: String,
    pub command: String,
}

impl ExternalCompiler {
    
    pub fn new(name: String, command: String) -> Self {
        Self { name, command }
    }

}

impl Compiler for ExternalCompiler {
    /// Returns the name of the compiler, which is the executable name/path.
    fn name(&self) -> &str {
        &self.name
    }

    /// Executes the external compiler process to compile the source code.
    ///
    /// Sends `text` to the process's stdin and expects a JSON representation of
    /// a [`Program`] on the process's stdout.
    fn compile(&self, text: &str, _args: &BTreeMap<String, String>) -> Result<Program, CompilationError> {
        let mut compiler = Command::new(&self.command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let Some(mut stdin) = compiler.stdin.take() else {
            return Err(CompilationError::default_error(
                format!("{} error: Unable to send script content to external process !",self.name.clone()),
            ));
        };
        stdin.write_all(text.as_bytes())?;
        let compiled = compiler.wait_with_output()?;
        let compiled = String::from_utf8(compiled.stdout)?;
        let prog: Program = serde_json::from_str(&compiled)?;
        Ok(prog)
    }
}

/// A type alias for a collection mapping compiler names (Strings) to Arc-ed [`Compiler`] trait objects.
///
/// This allows managing multiple compiler implementations dynamically.
pub type CompilerCollection = BTreeMap<String, Arc<dyn Compiler>>;
