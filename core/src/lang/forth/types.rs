use std::collections::VecDeque;

use crate::clock::SyncTime;
use crate::vm::event::ConcreteEvent;

pub type ForthValue = f64;

#[derive(Clone)]
pub enum Word {
    Builtin(BuiltinWord),
    UserDefined(Vec<String>),
}

#[derive(Clone, Copy)]
pub struct BuiltinWord(pub fn(&mut ForthState) -> Option<ForthAction>);

pub enum ForthAction {
    Emit(ConcreteEvent),
    Wait(SyncTime),
}

pub struct ForthState {
    pub data_stack: Vec<ForthValue>,
    pub return_stack: Vec<usize>,
    pub event_buffer: VecDeque<ConcreteEvent>,
    pub wait_time: SyncTime,
    pub channel: u64,
    pub device: usize,
    pub velocity: u64,
    pub duration_beats: f64,
}

impl Default for ForthState {
    fn default() -> Self {
        Self {
            data_stack: Vec::new(),
            return_stack: Vec::new(),
            event_buffer: VecDeque::new(),
            wait_time: 0,
            channel: 1,
            device: 1,
            velocity: 90,
            duration_beats: 0.25,
        }
    }
}

impl ForthState {
    pub fn push(&mut self, val: ForthValue) {
        self.data_stack.push(val);
    }

    pub fn pop(&mut self) -> ForthValue {
        self.data_stack.pop().unwrap_or(0.0)
    }

    pub fn peek(&self) -> ForthValue {
        self.data_stack.last().copied().unwrap_or(0.0)
    }
}
