use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{compiler::CompilationError, lang::Program};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum CompilationState {
    #[default]
    NotCompiled,
    Compiling,
    Compiled(#[serde(skip)] Program),
    Parsed,
    Error(CompilationError)
}

impl CompilationState {
    pub fn is_compiled(&self) -> bool {
        match self {
            CompilationState::Compiled(_) => true,
            _ => false
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            CompilationState::Compiled(_) | CompilationState::Parsed 
                => true,
            _ => false
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            CompilationState::Error(_) => true,
            _ => false
        }
    }

    pub fn lightened(&self) -> Self {
        match self {
            Self::Compiled(_) => Self::Compiled(Default::default()),
            _ => self.clone()
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

impl Display for CompilationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationState::NotCompiled => write!(f, "Not compiled"),
            CompilationState::Compiling => write!(f, "Compiling..."),
            CompilationState::Compiled(_) => write!(f, "Compiled"),
            CompilationState::Error(err) => write!(f, "Error: {err}"),
            CompilationState::Parsed => write!(f, "Parsed"),
        }
    }
}