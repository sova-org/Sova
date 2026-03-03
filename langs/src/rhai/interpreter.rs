use sova_core::{
    clock::SyncTime,
    vm::{EvaluationContext, event::ConcreteEvent, interpreter::Interpreter},
};

use super::runtime::RhaiExecutor;

pub struct RhaiInterpreter {
    executor: RhaiExecutor,
}

impl RhaiInterpreter {
    pub fn new(executor: RhaiExecutor) -> Self {
        Self { executor }
    }
}

impl Interpreter for RhaiInterpreter {
    fn execute_next(&mut self, ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        self.executor.execute_next(ctx)
    }

    fn has_terminated(&self) -> bool {
        self.executor.has_terminated()
    }

    fn stop(&mut self) {
        self.executor.stop();
    }
}
