use super::{Command, Motion, Operator};

#[derive(Debug, Clone)]
pub struct CommandParser {
    count_buffer: String,
    operator: Option<Operator>,
    motion_buffer: String,
    state: ParseState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseState {
    Initial,
    Count,
    Operator,
    Motion,
    TextObject,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResult {
    Incomplete,
    Complete(Command),
    Invalid,
}

impl CommandParser {
    pub fn new() -> Self {
        Self {
            count_buffer: String::new(),
            operator: None,
            motion_buffer: String::new(),
            state: ParseState::Initial,
        }
    }

    pub fn reset(&mut self) {
        self.count_buffer.clear();
        self.operator = None;
        self.motion_buffer.clear();
        self.state = ParseState::Initial;
    }

    pub fn extract_count(&mut self) -> Option<u32> {
        let count = if self.count_buffer.is_empty() {
            None
        } else {
            self.count_buffer.parse().ok()
        };
        self.reset();
        count
    }

    pub fn push_key(&mut self, key: char) -> ParseResult {
        match self.state {
            ParseState::Initial => self.handle_initial(key),
            ParseState::Count => self.handle_count(key),
            ParseState::Operator => self.handle_operator(key),
            ParseState::Motion => self.handle_motion(key),
            ParseState::TextObject => self.handle_text_object(key),
            ParseState::Complete => ParseResult::Invalid,
        }
    }

    fn handle_initial(&mut self, key: char) -> ParseResult {
        if key.is_ascii_digit() && key != '0' {
            self.count_buffer.push(key);
            self.state = ParseState::Count;
            ParseResult::Incomplete
        } else if let Some(op) = Operator::from_char(key) {
            self.operator = Some(op);
            self.state = ParseState::Operator;
            ParseResult::Incomplete
        } else if let Some(motion) = Motion::from_char(key) {
            self.state = ParseState::Complete;
            ParseResult::Complete(self.build_command(motion))
        } else if key == 'g' {
            self.motion_buffer.push(key);
            self.state = ParseState::Motion;
            ParseResult::Incomplete
        } else if key == 'i' || key == 'a' {
            self.motion_buffer.push(key);
            self.state = ParseState::TextObject;
            ParseResult::Incomplete
        } else {
            ParseResult::Invalid
        }
    }

    fn handle_count(&mut self, key: char) -> ParseResult {
        if key.is_ascii_digit() {
            self.count_buffer.push(key);
            ParseResult::Incomplete
        } else if let Some(op) = Operator::from_char(key) {
            self.operator = Some(op);
            self.state = ParseState::Operator;
            ParseResult::Incomplete
        } else if let Some(motion) = Motion::from_char(key) {
            self.state = ParseState::Complete;
            ParseResult::Complete(self.build_command(motion))
        } else if key == 'g' {
            self.motion_buffer.push(key);
            self.state = ParseState::Motion;
            ParseResult::Incomplete
        } else if key == 'i' || key == 'a' {
            self.motion_buffer.push(key);
            self.state = ParseState::TextObject;
            ParseResult::Incomplete
        } else {
            ParseResult::Invalid
        }
    }

    fn handle_operator(&mut self, key: char) -> ParseResult {
        if let Some(op) = self.operator {
            if key == op.to_char() {
                // Double operator (dd, yy, cc) - line operation
                let motion = Motion::Line(1);
                self.state = ParseState::Complete;
                ParseResult::Complete(self.build_command(motion))
            } else if let Some(motion) = Motion::from_char(key) {
                self.state = ParseState::Complete;
                ParseResult::Complete(self.build_command(motion))
            } else if key == 'g' {
                self.motion_buffer.push(key);
                self.state = ParseState::Motion;
                ParseResult::Incomplete
            } else if key == 'i' || key == 'a' {
                self.motion_buffer.push(key);
                self.state = ParseState::TextObject;
                ParseResult::Incomplete
            } else {
                ParseResult::Invalid
            }
        } else {
            ParseResult::Invalid
        }
    }

    fn handle_motion(&mut self, key: char) -> ParseResult {
        self.motion_buffer.push(key);

        let motion = match self.motion_buffer.as_str() {
            "gg" => Some(Motion::Top),
            _ => None,
        };

        if let Some(motion) = motion {
            self.state = ParseState::Complete;
            ParseResult::Complete(self.build_command(motion))
        } else if self.motion_buffer.len() >= 2 {
            ParseResult::Invalid
        } else {
            ParseResult::Incomplete
        }
    }

    fn handle_text_object(&mut self, key: char) -> ParseResult {
        self.motion_buffer.push(key);

        if self.motion_buffer.len() == 2 {
            let chars: Vec<char> = self.motion_buffer.chars().collect();
            if let Some(motion) = Motion::from_text_object(chars[0], chars[1]) {
                self.state = ParseState::Complete;
                ParseResult::Complete(self.build_command(motion))
            } else {
                ParseResult::Invalid
            }
        } else {
            ParseResult::Incomplete
        }
    }

    fn build_command(&self, motion: Motion) -> Command {
        let count = if self.count_buffer.is_empty() {
            None
        } else {
            self.count_buffer.parse().ok()
        };

        Command {
            count,
            operator: self.operator,
            motion,
        }
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}
