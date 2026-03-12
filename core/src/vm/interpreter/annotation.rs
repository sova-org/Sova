use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodePosition {
    pub line: usize,
    pub col: Option<usize>,
}

impl CodePosition {

    pub fn line(line: usize) -> Self {
        CodePosition { line, ..Default::default() }
    }

    pub fn at(line: usize, col: usize) -> Self {
        CodePosition { line, col: Some(col) }
    }
    
}

impl Display for CodePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}", self.line)?;
        if let Some(col) = self.col {
            write!(f, ", at : {}", col)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Annotation {
    Highlight(CodePosition, CodePosition),
    InsertText(String, CodePosition),
    ExplainSection(String, CodePosition, CodePosition)
}