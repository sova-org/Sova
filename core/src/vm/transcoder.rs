/// A compiler is a trait that defines any piece of software that can compile
/// a textual representation of a program into a program.
use crate::compiler::{Compiler, CompilerCollection};
use std::sync::Arc;

/// The transcoder is a repository of compilers. It allows to add, remove and
/// compile programs in different languages.
#[derive(Debug, Default)]
pub struct Transcoder {
    pub compilers: CompilerCollection,
}

impl Transcoder {
    /// Create a new transcoder with a set of compilers and an active compiler.
    /// If the active compiler is not in the set of compilers, it will be set to None.
    /// If no active compiler is set, the first added compiler will be set as active.
    ///
    /// # Arguments
    ///
    /// * `compilers` - A set of compilers to add to the transcoder.
    ///
    /// # Returns
    ///
    /// A new transcoder with the set of compilers.
    pub fn new(compilers: CompilerCollection) -> Self {
        Self { compilers }
    }

    /// Add a compiler to the transcoder.
    ///
    /// # Arguments
    ///
    /// * `compiler` - The compiler to add to the transcoder.
    ///
    /// # Returns
    ///
    /// The transcoder with the new compiler added.
    pub fn add_compiler(&mut self, compiler: impl Compiler + 'static) {
        let name: String = compiler.name().into();
        self.compilers.insert(name.clone(), Arc::new(compiler));
    }

    /// Remove a compiler from the transcoder.
    ///
    /// # Arguments
    ///
    /// * `lang` - The language of the compiler to remove.
    ///
    /// # Returns
    ///
    /// The removed compiler, or None if the compiler was not found.
    pub fn remove_compiler(&mut self, lang: &str) -> Option<Arc<dyn Compiler>> {
        self.compilers.remove(lang)
    }

    pub fn get_compiler(&self, lang: &str) -> Option<Arc<dyn Compiler>> {
        self.compilers.get(lang).map(Arc::clone)
    }

    /// Returns a list of names of the available compilers.
    pub fn available_compilers(&self) -> impl Iterator<Item = &str> {
        self.compilers.keys().map(String::as_str)
    }

    pub fn has_compiler(&self, lang: &str) -> bool {
        self.compilers.contains_key(lang)
    }
}
