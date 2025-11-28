use std::{cmp, collections::VecDeque, mem};

use crate::{
    clock::{Clock, NEVER, SyncTime, TimeSpan}, lang::{
        evaluation_context::EvaluationContext,
        event::ConcreteEvent,
        interpreter::{Interpreter, InterpreterFactory},
    }, scene::script::Script
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
            next_date: start_date,
            out_buffer: VecDeque::new(),
            previous: None,
        }
    }

    // pub fn get_target(&self, ctx: &EvaluationContext) -> (u64, usize) {
    //     let chan = match &self.output.channel {
    //         Some(item) => {

    //         }
    //         None => 1
    //     };
    //     todo!()
    // }

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

        let dur = dur.as_micros(ctx.clock, ctx.frame_len);
        
        match item {
            BoinxItem::Stop => {
                self.finished = true;
                return Vec::new();
            }
            BoinxItem::Note(n) => {
                vec![ConcreteEvent::MidiNote(n as u64, 90, 0, dur, 1)]
            }
            BoinxItem::Number(_) => {
                todo!()
            }
            BoinxItem::Str(_) => {
                todo!()
            }
            BoinxItem::ArgMap(_) => {
                todo!()
                //vec![ConcreteEvent::Osc { message: (), device_id: () }]
            }
            BoinxItem::External(prog) => vec![ConcreteEvent::StartProgram(prog)],
            _ => Vec::new(),
        }
    }

    pub fn update(&mut self, ctx: &mut EvaluationContext) -> Vec<BoinxLine> {
        if !self.ready(ctx.clock) {
            return Vec::new();
        }
        let item = self.output.compo.yield_compiled(ctx);
        let date = ctx.clock.micros();
        let len = self.time_span.as_beats(&ctx.clock, ctx.frame_len);
        let (pos, next_wait) = item.position(ctx, len, date.saturating_sub(self.start_date));
        self.next_date = self.next_date.saturating_add(next_wait);
        let old_pos = mem::replace(&mut self.position, pos);
        let delta = old_pos.diff(&self.position);
        let items = item.at(ctx, delta, len);
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

    pub fn ready(&self, clock: &Clock) -> bool {
        self.next_date <= clock.micros()
    }

    pub fn remaining_before_ready(&self, clock: &Clock) -> SyncTime {
        self.next_date.saturating_sub(clock.micros())
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
    ) -> (Option<ConcreteEvent>, SyncTime) {
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
            let rem = line.remaining_before_ready(ctx.clock);
            let mut lines = line.update(ctx);
            new_lines.append(&mut lines);
            if event.is_none() {
                event = line.get_event();
            }
            wait = cmp::min(wait, rem);
        }
        self.execution_lines.retain(|line| line.next_date < NEVER);
        self.execution_lines.append(&mut new_lines);
        let wait = if event.is_some() { 0 } else { wait };
        (event, wait)
    }

    fn has_terminated(&self) -> bool {
        self.started && self.execution_lines.is_empty()
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
