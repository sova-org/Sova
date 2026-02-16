use std::{cell::RefCell, collections::VecDeque, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::vm::{EvaluationContext, interpreter::CodePosition};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SovaError {
    pub line: usize,
    pub frame: usize,
    pub position: Option<CodePosition>,
    pub text: String
}

#[derive(Debug, Default)]
pub struct ErrorQueue {
    buffer: RefCell<VecDeque<SovaError>>
}

impl ErrorQueue {
    pub fn throw(&self, err: SovaError) {
        self.buffer.borrow_mut().push_back(err);
    }

    pub fn poll(&self) -> Option<SovaError> {
        self.buffer.borrow_mut().pop_front()
    }

    pub fn clear(&self) {
        self.buffer.borrow_mut().clear();
    }
}

impl SovaError {

    pub fn message<S>(mut self, msg: S) -> Self 
        where S : ToString
    {
        self.text = msg.to_string();
        self
    }

}

impl From<&EvaluationContext<'_>> for SovaError {
    fn from(ctx: &EvaluationContext) -> Self {
        SovaError { 
            line: ctx.line_index, 
            frame: ctx.frame_index,
            position: None,
            text: "Internal Sova Error".to_owned()
        }
    }
}

impl Display for SovaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}, frame {} : {}", self.line, self.frame, self.text)?;
        if let Some(pos) = &self.position {
            write!(f, "(at : {})", pos)?;
        }
        Ok(())
    }
}