use std::collections::HashMap;

use crate::clock::{NEVER, SyncTime};
use crate::vm::EvaluationContext;
use crate::vm::event::ConcreteEvent;
use crate::vm::interpreter::Interpreter;

use super::parser::tokenize;
use super::types::{ForthState, Word};
use super::words::builtin_words;

pub struct ForthInterpreter {
    tokens: Vec<String>,
    ip: usize,
    dictionary: HashMap<String, Word>,
    state: ForthState,
    terminated: bool,
    call_stack: Vec<(Vec<String>, usize)>,
}

impl ForthInterpreter {
    pub fn new(source: &str) -> Self {
        Self {
            tokens: tokenize(source),
            ip: 0,
            dictionary: builtin_words(),
            state: ForthState::default(),
            terminated: false,
            call_stack: Vec::new(),
        }
    }

    fn try_parse_number(token: &str) -> Option<f64> {
        if let Ok(n) = token.parse::<f64>() {
            return Some(n);
        }
        if let Some(hex) = token.strip_prefix("0x") {
            if let Ok(n) = i64::from_str_radix(hex, 16) {
                return Some(n as f64);
            }
        }
        if let Some(bin) = token.strip_prefix("0b") {
            if let Ok(n) = i64::from_str_radix(bin, 2) {
                return Some(n as f64);
            }
        }
        None
    }

    fn execute_token(&mut self, token: &str) {
        if let Some(n) = Self::try_parse_number(token) {
            self.state.push(n);
            return;
        }

        if let Some(word) = self.dictionary.get(token).cloned() {
            match word {
                Word::Builtin(builtin) => {
                    builtin.0(&mut self.state);
                }
                Word::UserDefined(body) => {
                    self.call_stack
                        .push((std::mem::take(&mut self.tokens), self.ip));
                    self.tokens = body;
                    self.ip = 0;
                }
            }
        }
    }

    fn handle_colon_definition(&mut self) {
        let name = if self.ip < self.tokens.len() {
            self.tokens[self.ip].clone()
        } else {
            return;
        };
        self.ip += 1;

        let mut body = Vec::new();
        while self.ip < self.tokens.len() {
            let tok = &self.tokens[self.ip];
            self.ip += 1;
            if tok == ";" {
                break;
            }
            body.push(tok.clone());
        }

        self.dictionary.insert(name, Word::UserDefined(body));
    }

    fn handle_if(&mut self) {
        let cond = self.state.pop();
        if cond != 0.0 {
            return;
        }

        let mut depth = 1;
        while self.ip < self.tokens.len() {
            let tok = &self.tokens[self.ip];
            self.ip += 1;
            if tok.eq_ignore_ascii_case("if") {
                depth += 1;
            } else if tok.eq_ignore_ascii_case("then") {
                depth -= 1;
                if depth == 0 {
                    return;
                }
            } else if tok.eq_ignore_ascii_case("else") && depth == 1 {
                return;
            }
        }
    }

    fn handle_else(&mut self) {
        let mut depth = 1;
        while self.ip < self.tokens.len() {
            let tok = &self.tokens[self.ip];
            self.ip += 1;
            if tok.eq_ignore_ascii_case("if") {
                depth += 1;
            } else if tok.eq_ignore_ascii_case("then") {
                depth -= 1;
                if depth == 0 {
                    return;
                }
            }
        }
    }

    fn handle_do(&mut self) {
        let start = self.state.pop();
        let limit = self.state.pop();

        self.state.return_stack.push(self.ip);
        self.state.return_stack.push(start as usize);
        self.state.return_stack.push(limit as usize);
    }

    fn handle_loop(&mut self) {
        if self.state.return_stack.len() < 3 {
            return;
        }

        let limit = self.state.return_stack.pop().unwrap();
        let current = self.state.return_stack.pop().unwrap();
        let return_addr = self.state.return_stack.pop().unwrap();

        let next = current + 1;
        if next < limit {
            self.state.return_stack.push(return_addr);
            self.state.return_stack.push(next);
            self.state.return_stack.push(limit);
            self.ip = return_addr;
        }
    }

    fn handle_begin(&mut self) {
        self.state.return_stack.push(self.ip);
    }

    fn handle_until(&mut self) {
        let cond = self.state.pop();
        if let Some(addr) = self.state.return_stack.pop() {
            if cond == 0.0 {
                self.ip = addr;
                self.state.return_stack.push(addr);
            }
        }
    }

    fn handle_i(&mut self) {
        if self.state.return_stack.len() >= 2 {
            let idx = self.state.return_stack.len() - 2;
            let current = self.state.return_stack[idx];
            self.state.push(current as f64);
        }
    }

    fn step(&mut self) {
        if self.ip >= self.tokens.len() {
            if let Some((tokens, ip)) = self.call_stack.pop() {
                self.tokens = tokens;
                self.ip = ip;
                return;
            }
            self.terminated = true;
            return;
        }

        let token = self.tokens[self.ip].clone();
        self.ip += 1;

        match token.as_str() {
            ":" => self.handle_colon_definition(),
            "if" | "IF" => self.handle_if(),
            "else" | "ELSE" => self.handle_else(),
            "then" | "THEN" => {}
            "do" | "DO" => self.handle_do(),
            "loop" | "LOOP" => self.handle_loop(),
            "begin" | "BEGIN" => self.handle_begin(),
            "until" | "UNTIL" => self.handle_until(),
            "i" | "I" => self.handle_i(),
            _ => self.execute_token(&token),
        }
    }

    pub fn run(&mut self) {
        while !self.terminated {
            self.step();
        }
    }

    pub fn stack(&self) -> &[f64] {
        &self.state.data_stack
    }
}

impl Interpreter for ForthInterpreter {
    fn execute_next(&mut self, _ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        while !self.terminated {
            self.step();
        }
        (None, NEVER)
    }

    fn has_terminated(&self) -> bool {
        self.terminated
    }

    fn stop(&mut self) {
        self.terminated = true;
    }
}
