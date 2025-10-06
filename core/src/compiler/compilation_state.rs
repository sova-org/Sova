use serde::{Deserialize, Serialize};

use crate::{compiler::CompilationError, lang::Program};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum CompilationState {
    #[default]
    NotCompiled,
    Compiling,
    Compiled(Program),
    Error(CompilationError)
}

impl CompilationState {
    pub fn is_compiled(&self) -> bool {
        match self {
            CompilationState::Compiled(_) => true,
            _ => false
        }
    }

    pub fn has_not_been_compiled(&self) -> bool {
        matches!(self, Self::NotCompiled)
    }

    pub fn clear(&mut self) {
        *self = Self::NotCompiled
    }

    pub fn program(&self) -> Option<&Program> {
        match self {
            CompilationState::Compiled(prog) => Some(prog),
            _ => None
        }
    }
}