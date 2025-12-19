use std::{cmp, collections::{HashMap, VecDeque}, mem};

use crate::{
    clock::{NEVER, SyncTime, TimeSpan},
    compiler::CompilationState,
    vm::{
        EvaluationContext,
        event::ConcreteEvent,
        interpreter::{Interpreter, InterpreterFactory},
        variable::VariableValue,
    },
    protocol::osc::OSCMessage,
    scene::script::Script,
};

mod ast;
mod parser;
mod position;

use ast::*;
pub use position::*;

pub use parser::parse_boinx;

pub struct BoinxLine {
    pub start_date: SyncTime,
    pub time_span: TimeSpan,
    pub output: BoinxOutput,
    pub finished: bool,
    pub position: BoinxPosition,
    pub has_vars: bool,
    next_date: SyncTime,
    out_buffer: VecDeque<ConcreteEvent>,
    previous: Option<BoinxItem>,
}

impl BoinxLine {
    pub fn new(start_date: SyncTime, time_span: TimeSpan, mut output: BoinxOutput) -> Self {
        let has_vars = output.compo.has_vars();
        if !has_vars {
            output.compo = output.compo.flatten().into();
        }
        BoinxLine {
            start_date,
            time_span,
            output,
            has_vars,
            finished: false,
            position: BoinxPosition::Undefined,
            next_date: start_date,
            out_buffer: VecDeque::new(),
            previous: None,
        }
    }

    pub fn execute_item(
        &mut self,
        ctx: &mut EvaluationContext,
        item: &BoinxItem,
        dur: TimeSpan,
        device: usize,
        channel: &VariableValue,
    ) -> Vec<ConcreteEvent> {
        if let BoinxItem::Previous = item {
            if let Some(prev) = self.previous.clone() {
                return self.execute_item(ctx, &prev, dur, device, channel);
            }
            return Vec::new();
        };
        self.previous = Some(item.clone());

        let dur = dur.as_micros(ctx.clock, ctx.frame_len);

        match item {
            BoinxItem::Note(n) => {
                let channel = channel.as_integer(ctx.clock, ctx.frame_len) as u64;
                vec![ConcreteEvent::MidiNote(*n as u64, 90, channel, dur, device)]
            }
            BoinxItem::ArgMap(map) => {
                let mut map : HashMap<String, VariableValue> = 
                    map.iter().filter_map(|(key, value)| {
                        if *value == BoinxItem::Mute {
                            None
                        } else {
                            Some((key.clone(), VariableValue::from(value.clone())))
                        }
                    }).collect();
                if !map.contains_key("sustain") {
                    let dur_s = (dur as f64) / 1_000_000.0;
                    map.insert("sustain".to_owned(), VariableValue::from(dur_s));
                }
                if channel.is_str() {
                    let mut args = Vec::new();
                    for (key, value) in map.iter() {
                        args.push(VariableValue::Str(key.clone()));
                        args.push(VariableValue::from(value.clone()));
                    }
                    let addr = channel.as_str(ctx.clock, ctx.frame_len);
                    vec![ConcreteEvent::Osc {
                        message: OSCMessage::new(addr, args),
                        device_id: device,
                    }]
                } else {
                    vec![ConcreteEvent::Dirt {
                        args: map,
                        device_id: device,
                    }]
                }
            }
            _ => Vec::new(),
        }
    }

    pub fn get_targets(
        &self,
        ctx: &mut EvaluationContext,
        date: SyncTime,
    ) -> (Vec<usize>, Vec<VariableValue>) {
        let devices = if let Some(dev_item) = &self.output.device {
            let dev_item = dev_item.evaluate(ctx);
            let (pos, _) = dev_item.position(ctx, date);
            let items = dev_item.at(ctx, pos);
            items
                .into_iter()
                .map(|(i, _)| match i {
                    BoinxItem::Note(n) => n as usize,
                    BoinxItem::Str(s) => ctx.device_map.get_slot_for_name(&s).unwrap_or(1),
                    _ => 1,
                })
                .collect()
        } else {
            vec![1]
        };
        let channels = if let Some(chan_item) = &self.output.channel {
            let chan_item = chan_item.evaluate(ctx);
            let (pos, _) = chan_item.position(ctx, date);
            let items = chan_item.at(ctx, pos);
            items
                .into_iter()
                .map(|(i, _)| VariableValue::from(i))
                .collect()
        } else {
            vec![0.into()]
        };
        (devices, channels)
    }

    pub fn start_subprog(
        &self,
        prog: BoinxProg,
        ctx: &mut EvaluationContext,
        len: TimeSpan,
        at: SyncTime,
    ) -> Vec<BoinxLine> {
        let mut prog_lines = prog.start(at, len, ctx);
        for line in prog_lines.iter_mut() {
            if line.output.device.is_none() {
                line.output.device = self.output.device.clone();
            }
            if line.output.channel.is_none() {
                line.output.channel = self.output.channel.clone();
            }
        }
        prog_lines
    }

    pub fn update(&mut self, ctx: &mut EvaluationContext) -> Vec<BoinxLine> {
        let date = ctx.logic_date;
        if !self.ready(date) {
            return Vec::new();
        }
        let item = if self.has_vars {
            self.output.compo.yield_compiled(ctx)
        } else {
            self.output.compo.item.evaluate(ctx)
        };
        let len = item
            .duration()
            .unwrap_or(self.time_span)
            .as_beats(&ctx.clock, ctx.frame_len);
        let mut sub_ctx = ctx.with_len(len);
        let (devices, channels) = self.get_targets(&mut sub_ctx, date);
        let (pos, next_wait) = item.position(&mut sub_ctx, date.saturating_sub(self.start_date));
        self.next_date = self.next_date.saturating_add(next_wait);
        let old_pos = mem::replace(&mut self.position, pos);
        let delta = old_pos.diff(&self.position);
        let items = item.at(&mut sub_ctx, delta);
        let mut new_lines = Vec::new();
        for (item, dur) in items {
            match item {
                BoinxItem::SubProg(prog) => {
                    let mut prog_lines = self.start_subprog(*prog, ctx, dur, self.next_date);
                    new_lines.append(&mut prog_lines);
                }
                BoinxItem::External(prog) => {
                    self.out_buffer.push_back(ConcreteEvent::StartProgram(prog));
                }
                BoinxItem::Stop => {
                    self.finished = true;
                }
                item => {
                    for device in devices.iter() {
                        for channel in channels.iter() {
                            let vec = self.execute_item(ctx, &item, dur, *device, channel);
                            self.out_buffer.append(&mut vec.into());
                        }
                    }
                }
            }
        }
        new_lines
    }

    pub fn get_event(&mut self) -> Option<ConcreteEvent> {
        self.out_buffer.pop_front()
    }

    pub fn ready(&self, date: SyncTime) -> bool {
        self.next_date <= date
    }

    pub fn remaining_before_ready(&self, date: SyncTime) -> SyncTime {
        self.next_date.saturating_sub(date)
    }
}

pub struct BoinxInterpreter {
    pub prog: BoinxProg,
    pub execution_lines: Vec<BoinxLine>,
    pub started: bool,
}

impl Interpreter for BoinxInterpreter {
    fn execute_next(&mut self, ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        let date = ctx.logic_date;
        if !self.started {
            self.execution_lines = self.prog.start(date, TimeSpan::Beats(ctx.frame_len), ctx);
            self.started = true;
        }
        let mut new_lines = Vec::new();
        let mut event = None;
        let mut wait = NEVER;
        for line in self.execution_lines.iter_mut() {
            let rem = line.remaining_before_ready(date);
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
        if let Some(prog_var) = script.compilation_state().cache() {
            let prog = BoinxProg::from(prog_var.clone());
            return Ok(Box::new(BoinxInterpreter::from(prog)));
        }
        match parse_boinx(script.content()) {
            Ok(prog) => Ok(Box::new(BoinxInterpreter::from(prog))),
            Err(e) => Err(e.to_string()),
        }
    }

    fn check(&self, script: &Script) -> CompilationState {
        match parse_boinx(script.content()) {
            Ok(prog) => CompilationState::Parsed(Some(VariableValue::from(prog))),
            Err(e) => CompilationState::Error(e),
        }
    }
}
