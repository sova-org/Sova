use std::{cell::RefCell, collections::VecDeque, fmt::Display, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::vm::{EvaluationContext, interpreter::CodePosition};

/// Standard execution-time error that contains a text message
/// and the position it was triggered from.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SovaError {
    pub line: usize,
    pub frame: usize,
    pub position: Option<CodePosition>,
    pub text: String,
}

/// A wrapper allowing to queue errors from an immutable reference.
#[derive(Debug, Default)]
pub struct ErrorQueue {
    buffer: RefCell<VecDeque<SovaError>>,
}

impl ErrorQueue {
    /// Queues an error in the buffer.
    pub fn throw(&self, err: SovaError) {
        self.buffer.borrow_mut().push_back(err);
    }

    /// Tries to pop the oldest error in the buffer.
    pub fn poll(&self) -> Option<SovaError> {
        self.buffer.borrow_mut().pop_front()
    }

    /// Clears the buffer.
    pub fn clear(&self) {
        self.buffer.borrow_mut().clear();
    }
}

impl SovaError {
    /// Adds the given message to the error.
    pub fn message<S>(mut self, msg: S) -> Self
    where
        S: ToString,
    {
        self.text = msg.to_string();
        self
    }
}

impl<'a, T: Deref<Target = EvaluationContext<'a>>> From<T> for SovaError {
    /// Instantiates a [SovaError]
    /// at the location specified in the [EvaluationContext].
    fn from(ctx: T) -> Self {
        SovaError {
            line: ctx.line_index,
            frame: ctx.frame_index,
            position: None,
            text: "Internal Sova Error".to_owned(),
        }
    }
}

impl Display for SovaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Line {}, frame {} : {}",
            self.line, self.frame, self.text
        )?;
        if let Some(pos) = &self.position {
            write!(f, "(at : {})", pos)?;
        }
        Ok(())
    }
}
