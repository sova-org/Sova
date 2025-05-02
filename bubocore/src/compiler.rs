//! Defines the core traits and structures for compiling source code into BuboCore programs.
//!
//! This module provides the [`Compiler`] trait, which abstracts the compilation
//! process for different languages, and the [`CompilationError`] type for
//! handling errors during compilation. It also includes an implementation for
//! invoking external compiler executables ([`ExternalCompiler`]).

use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    error, fmt,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    string::FromUtf8Error,
};

use crate::lang::Program;

pub mod bali;
pub mod boinx;
pub mod dummylang;

/// Represents an error that occurred during the compilation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationError {
    /// The name of the language or compiler stage where the error occurred.
    pub lang: String,
    /// A detailed message describing the error.
    pub info: String,
    /// The starting position in the source code related to the error, if applicable.
    pub from: usize,
    /// The ending position in the source code related to the error, if applicable.
    pub to: usize,
}

impl CompilationError {
    /// Creates a default error instance for a given language identifier.
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

/// Converts an I/O error into a `CompilationError`.
impl From<std::io::Error> for CompilationError {
    fn from(_: std::io::Error) -> Self {
        CompilationError::default_error("io".to_string())
    }
}

/// Converts a process output error (often from `wait_with_output`) into a `CompilationError`.
/// Note: Specific details from the process output are lost in this conversion.
impl From<std::process::Output> for CompilationError {
    fn from(_: std::process::Output) -> Self {
        CompilationError::default_error("process".to_string())
    }
}

/// Converts a UTF-8 conversion error into a `CompilationError`.
impl From<FromUtf8Error> for CompilationError {
    fn from(_: FromUtf8Error) -> Self {
        CompilationError::default_error("FromUtf8".to_string())
    }
}

/// Converts a Serde JSON deserialization error into a `CompilationError`.
impl From<serde_json::Error> for CompilationError {
    fn from(_: serde_json::Error) -> Self {
        CompilationError::default_error("serde_json".to_string())
    }
}

/// A trait for types that can compile source code text into a [`Program`].
///
/// Implementors define how source code for a specific language or system
/// is transformed into the intermediate BuboCore program representation.
/// Implementors must be safe to send and share across threads (`Send + Sync`).
pub trait Compiler: Send + Sync + std::fmt::Debug {
    /// Returns the unique name identifier for this compiler (e.g., "boinx", "bali").
    ///
    /// This name is used for identification and potentially for locating related resources
    /// like syntax definition files.
    fn name(&self) -> String;

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
    fn compile(&self, text: &str) -> Result<Program, CompilationError>;

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
pub struct ExternalCompiler(
    /// The path or name of the external compiler executable.
    pub String,
);

impl Compiler for ExternalCompiler {
    /// Returns the name of the compiler, which is the executable name/path.
    fn name(&self) -> String {
        self.0.clone()
    }

    /// Executes the external compiler process to compile the source code.
    ///
    /// Sends `text` to the process's stdin and expects a JSON representation of
    /// a [`Program`] on the process's stdout.
    fn compile(&self, text: &str) -> Result<Program, CompilationError> {
        let mut compiler = Command::new(&self.0)
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
    /// predefined relative path (`bubocore/src/static/syntaxes/`).
    ///
    /// # Notes
    ///
    /// *   This implementation assumes the application's working directory allows
    ///     access to `bubocore/src/static/syntaxes/`. A more robust approach
    ///     might involve locating assets relative to the executable or using
    ///     build script embedding.
    /// *   Returns `None` if the file is not found or cannot be read, potentially
    ///     logging the error internally (though currently commented out).
    fn syntax(&self) -> Option<Cow<'static, str>> {
        // Construct path relative to expected static assets directory
        // NOTE: This assumes the executable is run from the workspace root
        // or that the 'bubocore/src/static/syntaxes' path is accessible.
        // A more robust solution might involve configuration or finding the assets
        // directory relative to the executable.
        let syntax_file_name = format!("{}.sublime-syntax", self.name());
        let path = PathBuf::from("bubocore/src/static/syntaxes/").join(syntax_file_name);

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

/// A type alias for a collection mapping compiler names (Strings) to boxed [`Compiler`] trait objects.
///
/// This allows managing multiple compiler implementations dynamically.
pub type CompilerCollection = HashMap<String, Box<dyn Compiler>>;
