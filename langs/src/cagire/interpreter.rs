use std::sync::{Arc, Mutex};

use sova_core::clock::{SyncTime, NEVER};
use sova_core::error::SovaError;
use sova_core::vm::event::ConcreteEvent;
use sova_core::vm::interpreter::{Annotation, CodePosition, Interpreter};
use sova_core::vm::EvaluationContext;

use super::compiler::Dictionary;
use super::types::Span;
use super::vm::CagireVM;

pub struct CagireInterpreter {
    source: String,
    vm: CagireVM,
    shared_dict: Arc<Mutex<Dictionary>>,
    events: std::vec::IntoIter<(ConcreteEvent, SyncTime, Vec<Span>)>,
    pending: Option<(ConcreteEvent, Vec<Span>)>,
    executed: bool,
    terminated: bool,
    last_time: SyncTime,
    current_event_annotations: Vec<Span>,
}

impl CagireInterpreter {
    pub fn new(source: &str, dict: Dictionary, shared_dict: Arc<Mutex<Dictionary>>) -> Self {
        Self {
            source: source.to_string(),
            vm: CagireVM::with_dict(dict),
            shared_dict,
            events: Vec::new().into_iter(),
            pending: None,
            executed: false,
            terminated: false,
            last_time: 0,
            current_event_annotations: Vec::new(),
        }
    }
}

impl Interpreter for CagireInterpreter {
    fn execute_next(&mut self, ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        if !self.executed {
            self.executed = true;
            match self.vm.evaluate(&self.source, ctx) {
                Ok(evts) => {
                    // Merge new word definitions back into the shared dictionary
                    let mut shared = self.shared_dict.lock().unwrap();
                    for (name, body) in &self.vm.dict {
                        shared.insert(name.clone(), body.clone());
                    }
                    let event_annotations = std::mem::take(&mut self.vm.event_annotations);
                    debug_assert_eq!(evts.len(), event_annotations.len());
                    self.events = evts
                        .into_iter()
                        .zip(event_annotations)
                        .map(|((event, time), annotations)| (event, time, annotations))
                        .collect::<Vec<_>>()
                        .into_iter();
                }
                Err(e) => {
                    self.terminated = true;
                    ctx.errors.throw(
                        SovaError::from(ctx).message(format!("cagire error: {}", e.message)),
                    );
                    return (None, NEVER);
                }
            }
        }

        // Yield a pending event that was delayed by a time gap
        if let Some((event, annotations)) = self.pending.take() {
            self.current_event_annotations = annotations;
            if self.events.len() == 0 {
                self.terminated = true;
            }
            return (Some(event), 0);
        }

        if let Some((event, time, annotations)) = self.events.next() {
            let delta = time.saturating_sub(self.last_time);
            self.last_time = time;

            if delta > 0 {
                // Time gap: hold the event, return a wait so the scheduler
                // calls us back at the right moment to dispatch it.
                self.pending = Some((event, annotations));
                return (None, delta);
            }

            self.current_event_annotations = annotations;
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
        self.current_event_annotations.clear();
    }

    fn annotations(&self) -> Vec<Annotation> {
        use std::collections::HashMap;

        // Keep only the last resolved value per span (at-loops produce duplicates)
        let mut last_resolved: HashMap<usize, (Span, String)> = HashMap::new();
        for (span, val) in &self.vm.resolved {
            last_resolved.insert(span.start, (*span, val.display()));
        }

        let mut out: Vec<Annotation> = last_resolved
            .into_values()
            .map(|(span, text)| {
                let pos = byte_offset_to_position(&self.source, span.end);
                Annotation::InsertText(format!("[{text}]"), pos)
            })
            .collect();

        // Keep only the last selected span per start position
        let mut last_selected: HashMap<usize, Span> = HashMap::new();
        for span in &self.vm.selected {
            last_selected.insert(span.start, *span);
        }
        for span in last_selected.into_values() {
            let start = byte_offset_to_position(&self.source, span.start);
            let end = byte_offset_to_position(&self.source, span.end);
            out.push(Annotation::Highlight(start, end));
        }

        let mut current_by_start: HashMap<usize, Span> = HashMap::new();
        for span in &self.current_event_annotations {
            current_by_start.insert(span.start, *span);
        }
        for span in current_by_start.into_values() {
            let start = byte_offset_to_position(&self.source, span.start);
            let end = byte_offset_to_position(&self.source, span.end);
            out.push(Annotation::Highlight(start, end));
        }
        out
    }
}

fn byte_offset_to_position(source: &str, offset: usize) -> CodePosition {
    let mut line = 0;
    let mut col = 0;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    CodePosition::at(line, col)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use rusty_link::SessionState;
    use sova_core::clock::{Clock, ClockServer};
    use sova_core::device_map::DeviceMap;
    use sova_core::error::ErrorQueue;
    use sova_core::vm::variable::{VariableStore, VariableValue};

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

    fn highlight_cols(annotations: &[Annotation]) -> Vec<(usize, usize)> {
        annotations
            .iter()
            .filter_map(|annotation| match annotation {
                Annotation::Highlight(start, end) if start.line == 0 && end.line == 0 => {
                    Some((start.col.unwrap_or(0), end.col.unwrap_or(0)))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn string_pattern_annotations_follow_event_progress() {
        let script = "\"x.x.\" at 60 note .";
        let mut interpreter = CagireInterpreter::new(
            script,
            Dictionary::new(),
            Arc::new(Mutex::new(Dictionary::new())),
        );
        let mut tctx = TestCtx::new();
        let mut ctx = tctx.eval_ctx();

        let (event, wait) = interpreter.execute_next(&mut ctx);
        assert!(event.is_some());
        assert_eq!(wait, 0);
        assert_eq!(highlight_cols(&interpreter.annotations()), vec![(1, 2)]);

        let (event, wait) = interpreter.execute_next(&mut ctx);
        assert!(event.is_none());
        assert!(wait > 0);
        assert_eq!(highlight_cols(&interpreter.annotations()), vec![(1, 2)]);

        let (event, wait) = interpreter.execute_next(&mut ctx);
        assert!(event.is_some());
        assert_eq!(wait, 0);
        assert_eq!(highlight_cols(&interpreter.annotations()), vec![(3, 4)]);
    }
}
