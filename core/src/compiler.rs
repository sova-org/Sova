//! Defines the core traits and structures for compiling source code into Sova programs.
//!
//! This module provides the [`Compiler`] trait, which abstracts the compilation
//! process for different languages, and the [`CompilationError`] type for
//! handling errors during compilation. It also includes an implementation for
//! invoking external compiler executables ([`ExternalCompiler`]).

use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio}, sync::Arc,
};

use crate::lang::Program;

mod compilation_error;
pub use compilation_error::CompilationError;

mod compilation_state;
pub use compilation_state::CompilationState;

pub mod bali;
pub mod dummylang;

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

    /// Returns the syntax definition content for the language, if available.
    ///
    /// This is typically used for syntax highlighting in editors.
    /// The content is expected to be in a format like Sublime Text's `.sublime-syntax`.
    /// Returns `None` if the syntax definition cannot be found or loaded.
    fn syntax(&self) -> Option<Cow<'static, str>>;
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
                "External language".to_string(),
            ));
        };
        stdin.write_all(text.as_bytes())?;
        let compiled = compiler.wait_with_output()?;
        let compiled = String::from_utf8(compiled.stdout)?;
        let prog: Program = serde_json::from_str(&compiled)?;
        Ok(prog)
    }

    /// Attempts to load the syntax definition file for the external language.
    ///
    /// It looks for a file named `{compiler_name}.sublime-syntax` within a
    /// predefined relative path (`sova/src/static/syntaxes/`).
    ///
    /// # Notes
    ///
    /// *   This implementation assumes the application's working directory allows
    ///     access to `sova/src/static/syntaxes/`. A more robust approach
    ///     might involve locating assets relative to the executable or using
    ///     build script embedding.
    /// *   Returns `None` if the file is not found or cannot be read, potentially
    ///     logging the error internally (though currently commented out).
    fn syntax(&self) -> Option<Cow<'static, str>> {
        // Construct path relative to expected static assets directory
        // NOTE: This assumes the executable is run from the workspace root
        // or that the 'sova/src/static/syntaxes' path is accessible.
        // A more robust solution might involve configuration or finding the assets
        // directory relative to the executable.
        let syntax_file_name = format!("{}.sublime-syntax", self.name());
        let path = PathBuf::from("sova/src/static/syntaxes/").join(syntax_file_name);

        match std::fs::read_to_string(path) {
            Ok(content) => Some(Cow::Owned(content)),
            Err(_) => {
                // Optional: Log error here (e.g., using the `log` crate)
                // eprintln!("Failed to load syntax file for {}: {}", self.name(), e);
                None // File not found or cannot be read
            }
        }
    }
}

/// A type alias for a collection mapping compiler names (Strings) to Arc-ed [`Compiler`] trait objects.
///
/// This allows managing multiple compiler implementations dynamically.
pub type CompilerCollection = BTreeMap<String, Arc<dyn Compiler>>;
