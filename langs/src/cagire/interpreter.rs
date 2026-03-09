use sova_core::clock::{NEVER, SyncTime};
use sova_core::error::SovaError;
use sova_core::vm::EvaluationContext;
use sova_core::vm::event::ConcreteEvent;
use sova_core::vm::interpreter::Interpreter;

use super::vm::CagireVM;

pub struct CagireInterpreter {
    source: String,
    vm: CagireVM,
    events: std::vec::IntoIter<(ConcreteEvent, SyncTime)>,
    pending: Option<ConcreteEvent>,
    executed: bool,
    terminated: bool,
    last_time: SyncTime,
}

impl CagireInterpreter {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            vm: CagireVM::new(),
            events: Vec::new().into_iter(),
            pending: None,
            executed: false,
            terminated: false,
            last_time: 0,
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
                    ctx.errors.throw(
                        SovaError::from(ctx).message(format!("cagire error: {e}"))
                    );
                    return (None, NEVER);
                }
            }
        }

        // Yield a pending event that was delayed by a time gap
        if let Some(event) = self.pending.take() {
            if self.events.len() == 0 {
                self.terminated = true;
            }
            return (Some(event), 0);
        }

        if let Some((event, time)) = self.events.next() {
            let delta = time.saturating_sub(self.last_time);
            self.last_time = time;

            if delta > 0 {
                // Time gap: hold the event, return a wait so the scheduler
                // calls us back at the right moment to dispatch it.
                self.pending = Some(event);
                return (None, delta);
            }

            if self.events.len() == 0 {
                self.terminated = true;
            }
            (Some(event), 0)
        } else {
            self.terminated = true;
            (None, NEVER)
        }
    }

    fn has_terminated(&self) -> bool {
        self.terminated && self.pending.is_none()
    }

    fn stop(&mut self) {
        self.terminated = true;
        self.pending = None;
    }
}
