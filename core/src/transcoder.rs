/// A compiler is a trait that defines any piece of software that can compile
/// a textual representation of a program into a program.
use crate::compiler::{CompilationError, Compiler, CompilerCollection};
use crate::lang::Program;
use crate::scene::script::Script;
use std::{error, fmt};

/// Represents errors that can occur within the Transcoder operations.
#[derive(Debug)]
pub enum TranscoderError {
    /// No compiler was found for the requested language identifier.
    CompilerNotFound(String),

    /// An error occurred during the compilation process itself.
    CompilationFailed(CompilationError),
}

/// Manual implementation of Display trait
impl fmt::Display for TranscoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranscoderError::CompilerNotFound(lang) => {
                write!(f, "Compiler not found for language: {}", lang)
            }
            TranscoderError::CompilationFailed(err) => {
                write!(f, "Script compilation failed: {}", err)
            }
        }
    }
}

/// Manual implementation of Error trait
impl error::Error for TranscoderError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            TranscoderError::CompilerNotFound(_) => None,
            TranscoderError::CompilationFailed(e) => Some(e),
        }
    }
}

/// The transcoder is a repository of compilers. It allows to add, remove and
/// compile programs in different languages.
#[derive(Debug, Default)]
pub struct Transcoder {
    pub compilers: CompilerCollection,
    pub active_compiler: Option<String>,
}

impl Transcoder {
    /// Create a new transcoder with a set of compilers and an active compiler.
    /// If the active compiler is not in the set of compilers, it will be set to None.
    /// If no active compiler is set, the first added compiler will be set as active.
    ///
    /// # Arguments
    ///
    /// * `compilers` - A set of compilers to add to the transcoder.
    /// * `active_compiler` - The active compiler to set.
    ///
    /// # Returns
    ///
    /// A new transcoder with the set of compilers and active compiler.
    pub fn new(compilers: CompilerCollection, active_compiler: Option<String>) -> Self {
        let validated_active = active_compiler.filter(|name| compilers.contains_key(name));
        Self {
            compilers,
            active_compiler: validated_active,
        }
    }

    /// Add a compiler to the transcoder.
    /// If no active compiler is set, the new compiler will be set as active.
    ///
    /// # Arguments
    ///
    /// * `compiler` - The compiler to add to the transcoder.
    ///
    /// # Returns
    ///
    /// The transcoder with the new compiler added.
    pub fn add_compiler(&mut self, compiler: impl Compiler + 'static) {
        let name : String = compiler.name().into();
        self.compilers.insert(name.clone(), Box::new(compiler));
        if self.active_compiler.is_none() {
            self.active_compiler = Some(name.into());
        }
    }

    /// Remove a compiler from the transcoder.
    /// If the removed compiler was the active compiler, the active compiler will be set to None.
    ///
    /// # Arguments
    ///
    /// * `lang` - The language of the compiler to remove.
    ///
    /// # Returns
    ///
    /// The removed compiler, or None if the compiler was not found.
    pub fn remove_compiler(&mut self, lang: &str) -> Option<Box<dyn Compiler>> {
        let removed = self.compilers.remove(lang);
        if removed.is_some() && self.active_compiler.as_deref() == Some(lang) {
            self.active_compiler = None;
        }
        removed
    }

    /// Compile a program from a string using the active compiler.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the program to compile.
    /// * `lang` - The language of the compiler to use.
    ///
    /// # Returns
    ///
    /// The compiled program, or an error if the compiler was not found or the compilation failed.
    pub fn compile(&self, content: &str, lang: &str) -> Result<Program, TranscoderError> {
        let compiler = self
            .compilers
            .get(lang)
            .ok_or_else(|| TranscoderError::CompilerNotFound(lang.to_string()))?;
        compiler
            .compile(content)
            .map_err(TranscoderError::CompilationFailed)
    }

    pub fn compile_script(&self, script : &mut Script) -> bool {
        if let Ok(prog) = self.compile(script.content(), script.lang()) {
            script.compiled = prog;
            true
        } else {
            false
        }
    }

    /// Set the active compiler.
    ///
    /// # Arguments
    ///
    /// * `lang` - The language of the compiler to set as active.
    ///
    /// # Returns
    ///
    /// The transcoder with the active compiler set.
    pub fn set_active_compiler(&mut self, lang: &str) -> Result<(), TranscoderError> {
        if self.compilers.contains_key(lang) {
            self.active_compiler = Some(lang.to_string());
            Ok(())
        } else {
            Err(TranscoderError::CompilerNotFound(lang.to_string()))
        }
    }

    /// Compile a program from a string using the active compiler.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the program to compile.
    ///
    /// # Returns
    ///
    /// The compiled program, or an error if the compiler was not found or the compilation failed.
    pub fn compile_active(&self, content: &str) -> Result<Program, TranscoderError> {
        let active_lang = self.active_compiler.clone().ok_or_else(|| {
            TranscoderError::CompilerNotFound("No active compiler set".to_string())
        })?;
        self.compile(content, &active_lang)
    }

    /// Returns a list of names of the available compilers.
    pub fn available_compilers(&self) -> Vec<String> {
        self.compilers.keys().cloned().collect()
    }

    pub fn has_compiler(&self, lang : &str) -> bool {
        self.compilers.contains_key(lang)
    }

}
