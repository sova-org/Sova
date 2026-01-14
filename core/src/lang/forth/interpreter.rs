use std::collections::HashMap;

use crate::clock::{SyncTime, NEVER};
use crate::vm::event::ConcreteEvent;
use crate::vm::interpreter::Interpreter;
use crate::vm::EvaluationContext;

use super::parser::tokenize;
use super::types::{ForthAction, ForthState, Word};
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
        // Try float first (handles both integers and decimals like 0.5)
        if let Ok(n) = token.parse::<f64>() {
            return Some(n);
        }
        // Try hex (0x prefix)
        if let Some(hex) = token.strip_prefix("0x") {
            if let Ok(n) = i64::from_str_radix(hex, 16) {
                return Some(n as f64);
            }
        }
        // Try binary (0b prefix)
        if let Some(bin) = token.strip_prefix("0b") {
            if let Ok(n) = i64::from_str_radix(bin, 2) {
                return Some(n as f64);
            }
        }
        None
    }

    fn execute_token(&mut self, token: &str, _ctx: &mut EvaluationContext) -> Option<ForthAction> {
        // Check for number literal
        if let Some(n) = Self::try_parse_number(token) {
            self.state.push(n);
            return None;
        }

        // Check dictionary
        if let Some(word) = self.dictionary.get(token).cloned() {
            match word {
                Word::Builtin(builtin) => {
                    return builtin.0(&mut self.state);
                }
                Word::UserDefined(body) => {
                    // Save current position and switch to word body
                    self.call_stack.push((
                        std::mem::take(&mut self.tokens),
                        self.ip,
                    ));
                    self.tokens = body;
                    self.ip = 0;
                    return None;
                }
            }
        }

        // Unknown word - ignore for now
        None
    }

    fn handle_colon_definition(&mut self) {
        // : name ... ;
        // Collect tokens until ;
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
        // IF ... ELSE ... THEN  or  IF ... THEN
        let cond = self.state.pop();
        if cond != 0.0 {
            // True: execute until ELSE or THEN
            // We'll handle ELSE when we encounter it
            return;
        }

        // False: skip to ELSE or THEN
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
        // Skip to THEN (we were in the true branch)
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
        // DO ... LOOP  (limit start DO ... LOOP)
        let start = self.state.pop();
        let limit = self.state.pop();

        // Push loop params to return stack as: limit, current, return_addr
        self.state.return_stack.push(self.ip);
        self.state.return_stack.push(start as usize);
        self.state.return_stack.push(limit as usize);
    }

    fn handle_loop(&mut self) -> bool {
        // Increment and check
        if self.state.return_stack.len() < 3 {
            return false;
        }

        let limit = self.state.return_stack.pop().unwrap();
        let current = self.state.return_stack.pop().unwrap();
        let return_addr = self.state.return_stack.pop().unwrap();

        let next = current + 1;
        if next < limit {
            // Continue loop
            self.state.return_stack.push(return_addr);
            self.state.return_stack.push(next);
            self.state.return_stack.push(limit);
            self.ip = return_addr;
            true
        } else {
            // Exit loop
            false
        }
    }

    fn handle_begin(&mut self) {
        // Mark the beginning of the loop
        self.state.return_stack.push(self.ip);
    }

    fn handle_until(&mut self) {
        // Loop back if false
        let cond = self.state.pop();
        if let Some(addr) = self.state.return_stack.pop() {
            if cond == 0.0 {
                self.ip = addr;
                self.state.return_stack.push(addr);
            }
        }
    }

    fn handle_i(&mut self) {
        // Get current loop index
        if self.state.return_stack.len() >= 2 {
            let idx = self.state.return_stack.len() - 2;
            let current = self.state.return_stack[idx];
            self.state.push(current as f64);
        }
    }
}

impl Interpreter for ForthInterpreter {
    fn execute_next(&mut self, ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        // If we have buffered events, emit one
        if let Some(event) = self.state.event_buffer.pop_front() {
            let wait = self.state.wait_time;
            self.state.wait_time = 0;
            return (Some(event), wait);
        }

        // Execute tokens until we produce an event or terminate
        while self.ip < self.tokens.len() {
            let token = self.tokens[self.ip].clone();
            self.ip += 1;

            // Handle control structures
            match token.as_str() {
                ":" => {
                    self.handle_colon_definition();
                    continue;
                }
                "if" | "IF" => {
                    self.handle_if();
                    continue;
                }
                "else" | "ELSE" => {
                    self.handle_else();
                    continue;
                }
                "then" | "THEN" => {
                    continue;
                }
                "do" | "DO" => {
                    self.handle_do();
                    continue;
                }
                "loop" | "LOOP" => {
                    self.handle_loop();
                    continue;
                }
                "begin" | "BEGIN" => {
                    self.handle_begin();
                    continue;
                }
                "until" | "UNTIL" => {
                    self.handle_until();
                    continue;
                }
                "i" | "I" => {
                    self.handle_i();
                    continue;
                }
                _ => {}
            }

            // Execute regular token
            if let Some(action) = self.execute_token(&token, ctx) {
                match action {
                    ForthAction::Emit(event) => {
                        let wait = self.state.wait_time;
                        self.state.wait_time = 0;
                        return (Some(event), wait);
                    }
                    ForthAction::Wait(micros) => {
                        return (None, micros);
                    }
                }
            }
        }

        // Check if we need to return from a word call
        if let Some((tokens, ip)) = self.call_stack.pop() {
            self.tokens = tokens;
            self.ip = ip;
            return self.execute_next(ctx);
        }

        self.terminated = true;
        (None, NEVER)
    }

    fn has_terminated(&self) -> bool {
        self.terminated
    }

    fn stop(&mut self) {
        self.terminated = true;
    }
}
