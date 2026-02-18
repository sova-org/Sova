use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodePosition {
    pub line_start: usize,
    pub line_end: Option<usize>,
    pub col_start: Option<usize>,
    pub col_end: Option<usize>
}

impl CodePosition {

    pub fn line(i: usize) -> Self {
        CodePosition { line_start: i, ..Default::default() }
    }
    
}

impl Display for CodePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(end) = &self.line_end {
            write!(f, "Lines {}-{}", self.line_start, end)?;
        } else {
            write!(f, "Line {}", self.line_start)?;
        }
        if let (Some(start), Some(end)) = (self.col_start, self.col_end) {
            write!(f, ", chars : {}-{}", start, end)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    pub position: CodePosition,
    pub text: String
}