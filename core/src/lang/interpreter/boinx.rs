use std::{cmp, collections::VecDeque, mem};

use crate::{
    clock::{SyncTime, TimeSpan, NEVER},
    lang::{
        evaluation_context::EvaluationContext,
        event::ConcreteEvent,
        interpreter::{Interpreter, InterpreterFactory},
    },
    scene::script::Script,
};

mod ast;
mod parser;

use ast::*;

pub use parser::parse_boinx;

pub struct BoinxLine {
    pub start_date: SyncTime,
    pub time_span: TimeSpan,
    pub output: BoinxOutput,
    pub finished: bool,
    pub position: BoinxPosition,
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
            position: BoinxPosition::Undefined,
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
                //vec![ConcreteEvent::MidiNote((), (), (), (), ())]
                todo!()
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
        let item = self.output.compo.yield_compiled(ctx);
        let date = ctx.clock.micros();
        let len = self.time_span.as_beats(&ctx.clock, ctx.frame_len);
        let (pos, next_wait) = item.position(ctx, len, date.saturating_sub(self.start_date));
        self.next_date += next_wait;
        let old_pos = mem::replace(&mut self.position, pos);
        let delta = old_pos.diff(&self.position);
        let items = item.at(delta, self.time_span);
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
    pub started: bool
}

impl Interpreter for BoinxInterpreter {
    fn execute_next(
        &mut self,
        ctx: &mut EvaluationContext,
    ) -> (Option<ConcreteEvent>, Option<SyncTime>) {
        if !self.started {
            self.execution_lines = self.prog.start(
                ctx.clock.micros(), 
                TimeSpan::Beats(ctx.frame_len),
                ctx
            );
            self.started = true;
        }
        let mut new_lines = Vec::new();
        let mut event = None;
        let mut wait = NEVER;
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

impl From<BoinxProg> for BoinxInterpreter {
    fn from(prog: BoinxProg) -> Self {
        BoinxInterpreter {
            prog,
            execution_lines: Vec::new(),
            started: false,
        }
    }
}

pub struct BoinxInterpreterFactory;

impl InterpreterFactory for BoinxInterpreterFactory {
    fn name(&self) -> &str {
        "boinx"
    }

    fn make_instance(&self, script: &Script) -> Result<Box<dyn Interpreter>, String> {
        match parse_boinx(script.content()) {
            Ok(prog) => {
                Ok(Box::new(BoinxInterpreter::from(prog)))
            }
            Err(e) => Err(e.to_string())
        }
    }
}
