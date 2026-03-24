use std::{cmp, collections::{HashMap, VecDeque}, mem};

use sova_core::{
    clock::{NEVER, SyncTime, TimeSpan}, compiler::CompilationState, scene::script::Script, vm::{
        EvaluationContext, Language, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}, language::{LanguageDocumentation, LanguageSyntax}, variable::VariableValue
    }
};

mod ast;
mod parser;
mod position;

mod doc;
use doc::make_documentation;

use ast::*;
pub use position::*;

pub use parser::parse_boinx;

/// Represents a single Line of execution in Boinx, with a starting date, and a timespan.
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
    ) -> Option<ConcreteEvent> {
        if let BoinxItem::Previous = item {
            if let Some(prev) = self.previous.clone() {
                return self.execute_item(ctx, &prev, dur, device, channel);
            }
            return None;
        };
        self.previous = Some(item.clone());

        let dur = dur.as_micros(ctx.clock, ctx.frame_len);

        let addr = channel.clone().as_str(ctx);

        match item {
            BoinxItem::Note(n) => {
                Some(ConcreteEvent::Generic(VariableValue::from(*n), dur, addr, device))
            }
            BoinxItem::Number(f) => {
                Some(ConcreteEvent::Generic(VariableValue::from(*f), dur, addr, device))
            }
            BoinxItem::ArgMap(map) => {
                let map : HashMap<String, VariableValue> = 
                    map.iter().filter_map(|(key, value)| {
                        if !value.is_primitive() {
                            None
                        } else {
                            Some((key.clone(), VariableValue::from(value.clone())))
                        }
                    }).collect();
                Some(ConcreteEvent::Generic(map.into(), dur, addr, device))
            }
            BoinxItem::Str(s) => {
                Some(ConcreteEvent::Generic(s.clone().into(), dur, addr, device))
            }
            _ => None,
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
            let items = dev_item.untimed_at(pos);
            items
                .into_iter()
                .map(|i| match i {
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
            vec![String::new().into()]
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

    /// Updates the position of the line, and refresh the buffer of events with newly triggered ones.
    pub fn update(&mut self, ctx: &mut EvaluationContext) -> Vec<BoinxLine> {
        let date = ctx.logic_date;
        if !self.ready(date) {
            return Vec::new();
        }
        let len = self.time_span.as_beats(ctx.clock, ctx.frame_len);
        let mut sub_ctx = ctx.with_len(len);
        let item = if self.has_vars {
            self.output.compo.yield_compiled(&mut sub_ctx)
        } else {
            self.output.compo.item.evaluate(&mut sub_ctx)
        };
        let rel_date = date.saturating_sub(self.start_date);
        let (devices, channels) = self.get_targets(&mut sub_ctx, rel_date);
        let (pos, next_wait) = item.position(&mut sub_ctx, rel_date);
        self.next_date = self.next_date.saturating_add(next_wait);
        if self.next_date == NEVER {
            self.finished = true;
        }
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
                    self.execute_for_each_target(ctx, item, dur, &devices, &channels);
                }
            }
        }
        new_lines
    }

    fn execute_for_each_target(
        &mut self,
        ctx: &mut EvaluationContext,
        item: BoinxItem, 
        dur: TimeSpan,
        devices: &[usize],
        channels: &[VariableValue]
    ) {
        for device in devices.iter() {
            for channel in channels.iter() {
                if let Some(ev) = self.execute_item(ctx, &item, dur, *device, channel) {
                    self.out_buffer.push_back(ev);
                }
            }
        }
    }

    /// Pop the next event that should be executed
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

/// Interpreter for a Boinx program.
pub struct BoinxInterpreter {
    prog: BoinxProg,
    execution_lines: Vec<BoinxLine>,
    started: bool,
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
        self.execution_lines.retain(|line| !line.finished);
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

/// Factory to generate BoinxInterpreters from Boinx code.
pub struct BoinxInterpreterFactory;

impl Language for BoinxInterpreterFactory {
    fn name(&self) -> &str {
        "boinx"
    }

    fn version(&self) -> (usize, usize, usize) {
        (1,0,0)
    }

    fn documentation(&self) -> LanguageDocumentation {
        make_documentation()
    }

    fn syntax(&self) -> Option<LanguageSyntax> {
        use sova_core::vm::language::{SyntaxRule, TokenCategory::*};
        Some(LanguageSyntax {
            rules: vec![
                SyntaxRule::new(Comment, r"//[^\n]*"),
                SyntaxRule::new(String, r#""[^"]*"|'[^']*'"#),
                SyntaxRule::new(Special, r"\d+u\b|\d+(\.\d+)?''|\d+(\.\d+)?'"),
                SyntaxRule::new(Special, r"\b[A-G][#b]*\d*\b"),
                SyntaxRule::new(Number, r"\d+\.\d+|\d+"),
                SyntaxRule::new(Variable, r"\$(?:l_|f_)?[a-zA-Z_]\w*|_[a-zA-Z_]\w*"),
                SyntaxRule::new(Operator, r"[|°~!#]|<<|>>|\^"),
                SyntaxRule::new(Operator, r"[+\-*/]"),
                SyntaxRule::new(Operator, r"[<>]=?|==|!="),
                SyntaxRule::new(Keyword, r"[?:]|="),
                SyntaxRule::new(Keyword, r"\b_\b"),
                SyntaxRule::new(Punctuation, r"\."),
                SyntaxRule::new(Symbol, r"\b[a-zA-Z_]\w*\s*:"),
                SyntaxRule::new(Punctuation, r"[{}\[\]()<>@,]"),
            ],
        })
    }
}

impl InterpreterFactory for BoinxInterpreterFactory {

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

#[cfg(test)]
mod tests {
    use super::*;
    use sova_core::vm::language::TokenCategory;

    fn categories_for(text: &str) -> Vec<(String, TokenCategory)> {
        let factory = BoinxInterpreterFactory;
        let syntax = factory.syntax().expect("syntax() returned None");
        let mut parts = Vec::new();
        let mut cats = Vec::new();
        let mut names = Vec::new();
        for (i, rule) in syntax.rules.iter().enumerate() {
            let name = format!("g{i}");
            parts.push(format!("(?P<{name}>{})", rule.pattern));
            names.push(name);
            cats.push(rule.category);
        }
        let regex = regex::Regex::new(&parts.join("|")).expect("regex failed to compile");
        let mut result = Vec::new();
        for caps in regex.captures_iter(text) {
            for (i, cat) in cats.iter().enumerate() {
                if let Some(m) = caps.name(&names[i]) {
                    result.push((text[m.start()..m.end()].to_owned(), *cat));
                    break;
                }
            }
        }
        result
    }

    #[test]
    fn syntax_regex_compiles() {
        let _ = categories_for("");
    }

    #[test]
    fn syntax_highlights_sample() {
        use TokenCategory::*;
        let tokens = categories_for(
            "// a boinx line\nC4 | _ ? $vol = 90 \"kick\" 0.5' {1 2 3} sound: foo"
        );
        let has = |cat: TokenCategory| tokens.iter().any(|(_, c)| *c == cat);
        assert!(has(Comment), "missing Comment");
        assert!(has(Special), "missing Special");
        assert!(has(Keyword), "missing Keyword");
        assert!(has(Variable), "missing Variable");
        assert!(has(Number), "missing Number");
        assert!(has(String), "missing String");
        assert!(has(Operator), "missing Operator");
        assert!(has(Symbol), "missing Symbol");
        assert!(has(Punctuation), "missing Punctuation");
    }
}
