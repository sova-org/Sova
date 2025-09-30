use std::{cmp, collections::VecDeque};

use crate::{
    clock::{SyncTime, TimeSpan},
    lang::{
        evaluation_context::EvaluationContext,
        event::ConcreteEvent,
        interpreter::{Interpreter, InterpreterFactory},
    },
    scene::script::Script,
};

mod boinx_ast;

use boinx_ast::*;

pub struct BoinxLine {
    pub start_date: SyncTime,
    pub time_span: TimeSpan,
    pub output: BoinxOutput,
    pub finished: bool,
    next_date: SyncTime,
    out_buffer: VecDeque<ConcreteEvent>,
    previous: Option<BoinxItem>,
}

impl BoinxLine {
    pub fn new(start_date: SyncTime, time_span: TimeSpan, output: BoinxOutput) -> Self {
        BoinxLine {
            start_date,
            time_span,
            output,
            finished: false,
            next_date: 0,
            out_buffer: VecDeque::new(),
            previous: None,
        }
    }

    pub fn execute_item(
        &mut self,
        ctx: &mut EvaluationContext,
        item: BoinxItem,
        dur: TimeSpan,
    ) -> Vec<ConcreteEvent> {
        if let BoinxItem::Previous = item {
            if let Some(prev) = &self.previous {
                return self.execute_item(ctx, prev.clone(), dur);
            }
            return Vec::new();
        };
        self.previous = Some(item.clone());
        match item {
            BoinxItem::Stop => {
                self.finished = true;
                return Vec::new();
            }
            BoinxItem::Note(n) => {
                vec![ConcreteEvent::MidiNote((), (), (), (), ())]
            }
            BoinxItem::Number(_) => {
                todo!()
            },
            BoinxItem::External(prog) => vec![ConcreteEvent::StartProgram(prog)],
            _ => Vec::new(),
        }
    }

    pub fn update(&mut self, ctx: &mut EvaluationContext) -> Vec<BoinxLine> {
        if !self.ready(ctx) {
            return Vec::new();
        }
        let item = self.output.compo.yield_item(ctx);
        let date = ctx.clock.micros();
        let len = self.time_span.as_beats(&ctx.clock, ctx.frame_len());
        let items = item.at(ctx, len, date);
        let mut new_lines = Vec::new();
        for (item, dur) in items {
            if let BoinxItem::SubProg(prog) = item {
                let mut prog_lines = prog.start(self.next_date, dur, ctx);
                new_lines.append(&mut prog_lines);
                continue;
            };
            let vec = self.execute_item(ctx, item, dur);
            self.out_buffer.append(&mut vec.into());
        }
        new_lines
    }

    pub fn get_event(&mut self) -> Option<ConcreteEvent> {
        self.out_buffer.pop_front()
    }

    pub fn ready(&self, ctx: &EvaluationContext) -> bool {
        self.next_date <= ctx.clock.micros()
    }
}

pub struct BoinxInterpreter {
    pub prog: BoinxProg,
    pub execution_lines: Vec<BoinxLine>,
}

impl Interpreter for BoinxInterpreter {
    fn execute_next(
        &mut self,
        ctx: &mut EvaluationContext,
    ) -> (Option<ConcreteEvent>, Option<SyncTime>) {
        let mut new_lines = Vec::new();
        let mut event = None;
        let mut wait = SyncTime::MAX;
        for line in self.execution_lines.iter_mut() {
            let mut lines = line.update(ctx);
            new_lines.append(&mut lines);
            if event.is_none() {
                event = line.get_event();
            }
            wait = cmp::min(wait, line.next_date);
        }
        self.execution_lines.append(&mut new_lines);
        let wait = if event.is_some() { None } else { Some(wait) };
        (event, wait)
    }

    fn has_terminated(&self) -> bool {
        !self.execution_lines.is_empty()
    }

    fn stop(&mut self) {
        self.execution_lines.clear();
    }
}

pub struct BoinxInterpreterFactory {}

impl InterpreterFactory for BoinxInterpreterFactory {
    fn name(&self) -> &str {
        "boinx"
    }

    fn make_instance(&self, script: &Script) -> Box<dyn Interpreter> {
        todo!()
    }
}
