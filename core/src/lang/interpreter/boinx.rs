use std::collections::VecDeque;

use crate::{clock::{SyncTime, TimeSpan}, lang::{evaluation_context::EvaluationContext, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}}, scene::script::Script};

mod boinx_ast;

use boinx_ast::*;

pub struct BoinxLine {
    pub start_date: SyncTime,
    pub time_span: TimeSpan,
    pub next_date: SyncTime,
    pub output: BoinxOutput,
    pub finished: bool,
    out_buffer: VecDeque<ConcreteEvent>
}

impl BoinxLine {

    pub fn execute_item(
        &mut self,
        ctx: &mut EvaluationContext,
        item: BoinxItem,
        dur: TimeSpan
    ) -> Option<ConcreteEvent> {
        todo!()
    }

    pub fn update(
        &mut self,
        ctx: &mut EvaluationContext
    ) -> Vec<BoinxLine> {
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
                
                continue;
            };
            let Some(event) = self.execute_item(ctx, item, dur) else {
                continue;
            };
            self.out_buffer.push_back(event);
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
        ctx : &mut EvaluationContext
    ) -> (Option<ConcreteEvent>, Option<SyncTime>) { 
        for line in self.execution_lines.iter() {
            
        };
        (None, None)
    }

    fn has_terminated(&self) -> bool {
        !self.execution_lines.is_empty()
    }

    fn stop(&mut self) {
        self.execution_lines.clear();
    }

}

pub struct BoinxInterpreterFactory {

}

impl InterpreterFactory for BoinxInterpreterFactory {
    
    fn name(&self) -> &str {
        "boinx"
    }

    fn make_instance(&self, script : &Script) -> Box<dyn Interpreter> {
        todo!()
    }

}
