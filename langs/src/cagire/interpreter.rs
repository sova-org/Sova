use sova_core::clock::{NEVER, SyncTime};
use sova_core::vm::EvaluationContext;
use sova_core::vm::event::ConcreteEvent;
use sova_core::vm::interpreter::Interpreter;

use super::vm::CagireVM;

pub struct CagireInterpreter {
    source: String,
    vm: CagireVM,
    events: std::vec::IntoIter<(ConcreteEvent, SyncTime)>,
    executed: bool,
    terminated: bool,
}

impl CagireInterpreter {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            vm: CagireVM::new(),
            events: Vec::new().into_iter(),
            executed: false,
            terminated: false,
        }
    }
}

impl Interpreter for CagireInterpreter {
    fn execute_next(&mut self, ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        if !self.executed {
            self.executed = true;
            match self.vm.evaluate(&self.source, ctx) {
                Ok(evts) => self.events = evts.into_iter(),
                Err(e) => {
                    self.terminated = true;
                    return (Some(ConcreteEvent::Print(format!("cagire error: {e}"))), 0);
                }
            }
        }

        if let Some((event, time)) = self.events.next() {
            if self.events.len() == 0 {
                self.terminated = true;
            }
            (Some(event), time)
        } else {
            self.terminated = true;
            (None, NEVER)
        }
    }

    fn has_terminated(&self) -> bool {
        self.terminated
    }

    fn stop(&mut self) {
        self.terminated = true;
    }
}
