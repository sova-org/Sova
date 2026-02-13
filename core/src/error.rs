use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::vm::{EvaluationContext, interpreter::CodePosition};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SovaError {
    pub line: usize,
    pub frame: usize,
    pub position: Option<CodePosition>,
    pub text: String
}

pub struct ErrorQueue {
    pub buffer: VecDeque<SovaError>
}

impl ErrorQueue {

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