use sova_core::HOST_PROXY_SLOT;
use sova_core::clock::{NEVER, SyncTime};
use sova_core::compiler::CompilationState;
use sova_core::scene::script::Script;
use sova_core::vm::EvaluationContext;
use sova_core::vm::Language;
use sova_core::vm::event::ConcreteEvent;
use sova_core::vm::interpreter::{Interpreter, InterpreterFactory};
use sova_core::vm::language::{LanguageDocumentation, LanguageSyntax};
use sova_core::vm::variable::VariableValue;

use super::syntax;

const HYDRA_EVAL_ROUTE: &str = "hydra/eval";

pub struct HydraInterpreterFactory;

impl Language for HydraInterpreterFactory {
    fn name(&self) -> &str {
        "hydra"
    }

    fn version(&self) -> (usize, usize, usize) {
        (0, 1, 0)
    }

    fn documentation(&self) -> LanguageDocumentation {
        let mut doc = LanguageDocumentation::default();
        doc.articles.push((
            "Introduction".into(),
            include_str!("../../docs/hydra/intro.md").into(),
        ));
        doc.articles.push((
            "Chaining".into(),
            include_str!("../../docs/hydra/chaining.md").into(),
        ));
        doc.articles.push((
            "Sources".into(),
            include_str!("../../docs/hydra/sources.md").into(),
        ));
        doc.articles.push((
            "Geometry".into(),
            include_str!("../../docs/hydra/geometry.md").into(),
        ));
        doc.articles.push((
            "Color".into(),
            include_str!("../../docs/hydra/color.md").into(),
        ));
        doc.articles.push((
            "Blending".into(),
            include_str!("../../docs/hydra/blending.md").into(),
        ));
        doc.articles.push((
            "Modulation".into(),
            include_str!("../../docs/hydra/modulation.md").into(),
        ));
        doc.articles.push((
            "Buffers".into(),
            include_str!("../../docs/hydra/buffers.md").into(),
        ));
        doc.articles.push((
            "Feedback".into(),
            include_str!("../../docs/hydra/feedback.md").into(),
        ));
        doc.articles.push((
            "Animation".into(),
            include_str!("../../docs/hydra/animation.md").into(),
        ));
        doc.articles.push((
            "Text".into(),
            include_str!("../../docs/hydra/text.md").into(),
        ));
        doc.articles.push((
            "Differences".into(),
            include_str!("../../docs/hydra/differences.md").into(),
        ));
        doc
    }

    fn syntax(&self) -> Option<LanguageSyntax> {
        Some(syntax::syntax())
    }
}

impl InterpreterFactory for HydraInterpreterFactory {
    fn make_instance(&self, script: &Script) -> Result<Box<dyn Interpreter>, String> {
        Ok(Box::new(HydraInterpreter::new(script.content().to_owned())))
    }

    fn check(&self, _script: &Script) -> CompilationState {
        CompilationState::Parsed(None)
    }
}

pub struct HydraInterpreter {
    source: String,
    terminated: bool,
}

impl HydraInterpreter {
    pub fn new(source: String) -> Self {
        Self {
            source,
            terminated: false,
        }
    }
}

impl Interpreter for HydraInterpreter {
    fn execute_next(&mut self, _ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        if self.terminated {
            return (None, NEVER);
        }
        self.terminated = true;
        let event = ConcreteEvent::Generic(
            VariableValue::Str(std::mem::take(&mut self.source)),
            0,
            HYDRA_EVAL_ROUTE.to_string(),
            HOST_PROXY_SLOT,
        );
        (Some(event), NEVER)
    }

    fn has_terminated(&self) -> bool {
        self.terminated
    }

    fn stop(&mut self) {
        self.terminated = true;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Arc;

    use rusty_link::SessionState;
    use sova_core::clock::{Clock, ClockServer};
    use sova_core::device_map::DeviceMap;
    use sova_core::error::ErrorQueue;
    use sova_core::vm::variable::VariableStore;

    use super::*;

    struct TestCtx {
        global: VariableStore,
        line: VariableStore,
        frame: VariableStore,
        instance: VariableStore,
        stack: VecDeque<VariableValue>,
        structure: Vec<Vec<f64>>,
        clock: Clock,
        device_map: DeviceMap,
        errors: ErrorQueue,
    }

    impl TestCtx {
        fn new() -> Self {
            let server = Arc::new(ClockServer::new(120.0, 4.0));
            let clock = Clock {
                server,
                session_state: SessionState::new(),
                drift: 0,
                system_time_offset: 0,
            };
            Self {
                global: VariableStore::new(),
                line: VariableStore::new(),
                frame: VariableStore::new(),
                instance: VariableStore::new(),
                stack: VecDeque::new(),
                structure: vec![],
                clock,
                device_map: DeviceMap::new(),
                errors: ErrorQueue::default(),
            }
        }

        fn eval_ctx(&mut self) -> EvaluationContext<'_> {
            EvaluationContext {
                logic_date: 0,
                global_vars: &mut self.global,
                line_vars: &mut self.line,
                frame_vars: &mut self.frame,
                instance_vars: &mut self.instance,
                stack: &mut self.stack,
                line_index: 0,
                line_iterations: 0,
                frame_index: 0,
                frame_len: 1.0,
                frame_triggers: 0,
                structure: &self.structure,
                clock: &self.clock,
                device_map: &self.device_map,
                errors: &self.errors,
            }
        }
    }

    #[test]
    fn emits_one_generic_then_terminates() {
        let mut interp = HydraInterpreter::new("osc().out()".to_string());
        let mut tctx = TestCtx::new();
        let mut ctx = tctx.eval_ctx();

        let (event, wait) = interp.execute_next(&mut ctx);
        let Some(ConcreteEvent::Generic(value, _, channel, device_id)) = event else {
            panic!("first call should yield a Generic event, got {:?}", event);
        };
        assert_eq!(channel, HYDRA_EVAL_ROUTE);
        assert_eq!(device_id, HOST_PROXY_SLOT);
        assert_eq!(value, VariableValue::Str("osc().out()".to_string()));
        assert_eq!(wait, NEVER);
        assert!(interp.has_terminated());

        let (event, wait) = interp.execute_next(&mut ctx);
        assert!(event.is_none());
        assert_eq!(wait, NEVER);
    }

    #[test]
    fn carries_source_verbatim() {
        let source = "  osc(60)\n  .modulate(noise(3))\n  .out()  ";
        let mut interp = HydraInterpreter::new(source.to_string());
        let mut tctx = TestCtx::new();
        let mut ctx = tctx.eval_ctx();
        let (event, _) = interp.execute_next(&mut ctx);
        let Some(ConcreteEvent::Generic(VariableValue::Str(out), ..)) = event else {
            panic!("expected Generic with Str payload");
        };
        assert_eq!(out, source);
    }
}
