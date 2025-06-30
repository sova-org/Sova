use tui_textarea::{CursorMove, TextArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    // Character motions
    Left,
    Right,
    Up,
    Down,

    // Word motions
    WordForward,
    WordBackward,
    WordEnd,

    // Line motions
    LineStart,
    LineEnd,
    LineFirst,

    // Document motions
    Top,
    Bottom,
    Line(u32),

    // Text objects
    InnerWord,
    AroundWord,
    InnerQuote(char),
    AroundQuote(char),
    InnerBracket(char),
    AroundBracket(char),
}

impl Motion {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'h' => Some(Self::Left),
            'j' => Some(Self::Down),
            'k' => Some(Self::Up),
            'l' => Some(Self::Right),
            'w' => Some(Self::WordForward),
            'b' => Some(Self::WordBackward),
            'e' => Some(Self::WordEnd),
            '0' => Some(Self::LineStart),
            '$' => Some(Self::LineEnd),
            '^' => Some(Self::LineFirst),
            'G' => Some(Self::Bottom),
            _ => None,
        }
    }

    pub fn from_text_object(prefix: char, target: char) -> Option<Self> {
        let text_obj = match (prefix, target) {
            ('i', 'w') => Self::InnerWord,
            ('a', 'w') => Self::AroundWord,
            ('i', '"') => Self::InnerQuote('"'),
            ('a', '"') => Self::AroundQuote('"'),
            ('i', '\'') => Self::InnerQuote('\''),
            ('a', '\'') => Self::AroundQuote('\''),
            ('i', '{') | ('i', '}') => Self::InnerBracket('{'),
            ('a', '{') | ('a', '}') => Self::AroundBracket('{'),
            ('i', '[') | ('i', ']') => Self::InnerBracket('['),
            ('a', '[') | ('a', ']') => Self::AroundBracket('['),
            ('i', '(') | ('i', ')') => Self::InnerBracket('('),
            ('a', '(') | ('a', ')') => Self::AroundBracket('('),
            _ => return None,
        };
        Some(text_obj)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl TextRange {
    pub fn new(start_row: usize, start_col: usize, end_row: usize, end_col: usize) -> Self {
        Self {
            start_row,
            start_col,
            end_row,
            end_col,
        }
    }
}

pub trait MotionExecutor {
    fn execute_motion(&mut self, motion: Motion, count: u32) -> TextRange;
    fn find_text_object(&self, motion: Motion) -> Option<TextRange>;
}

impl MotionExecutor for TextArea<'_> {
    fn execute_motion(&mut self, motion: Motion, count: u32) -> TextRange {
        let start = self.cursor();

        match motion {
            Motion::Left => {
                for _ in 0..count {
                    self.move_cursor(CursorMove::Back);
                }
            }
            Motion::Right => {
                for _ in 0..count {
                    self.move_cursor(CursorMove::Forward);
                }
            }
            Motion::Up => {
                for _ in 0..count {
                    self.move_cursor(CursorMove::Up);
                }
            }
            Motion::Down => {
                for _ in 0..count {
                    self.move_cursor(CursorMove::Down);
                }
            }
            Motion::WordForward => {
                for _ in 0..count {
                    self.move_cursor(CursorMove::WordForward);
                }
            }
            Motion::WordBackward => {
                for _ in 0..count {
                    self.move_cursor(CursorMove::WordBack);
                }
            }
            Motion::WordEnd => {
                for _ in 0..count {
                    self.move_cursor(CursorMove::WordEnd);
                }
            }
            Motion::LineStart => {
                let (row, _) = self.cursor();
                self.move_cursor(CursorMove::Jump(row as u16, 0));
            }
            Motion::LineEnd => {
                self.move_cursor(CursorMove::End);
            }
            Motion::LineFirst => {
                self.move_cursor(CursorMove::Head);
            }
            Motion::Top => {
                self.move_cursor(CursorMove::Top);
            }
            Motion::Bottom => {
                self.move_cursor(CursorMove::Bottom);
            }
            Motion::Line(line) => {
                if line > 0 && (line as usize) <= self.lines().len() {
                    self.move_cursor(CursorMove::Jump((line - 1) as u16, 0));
                }
            }
            // Text objects handled separately
            _ => {}
        }

        let end = self.cursor();
        TextRange::new(start.0, start.1, end.0, end.1)
    }

    fn find_text_object(&self, motion: Motion) -> Option<TextRange> {
        let (row, col) = self.cursor();
        let lines = self.lines();
        let line = lines.get(row)?;

        match motion {
            Motion::InnerWord => find_word_boundaries(line, col, false)
                .map(|(start, end)| TextRange::new(row, start, row, end)),
            Motion::AroundWord => find_word_boundaries(line, col, true)
                .map(|(start, end)| TextRange::new(row, start, row, end)),
            Motion::InnerQuote(quote) => find_quote_boundaries(line, col, quote, false)
                .map(|(start, end)| TextRange::new(row, start, row, end)),
            Motion::AroundQuote(quote) => find_quote_boundaries(line, col, quote, true)
                .map(|(start, end)| TextRange::new(row, start, row, end)),
            _ => None,
        }
    }
}

fn find_word_boundaries(line: &str, col: usize, around: bool) -> Option<(usize, usize)> {
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let safe_col = col.min(chars.len().saturating_sub(1));

    // Vim word definition: alphanumeric + underscore
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

    // If cursor is on whitespace, find the next word for 'around'
    if safe_col < chars.len() && chars[safe_col].is_whitespace() {
        if around {
            // Find next word
            let mut start = safe_col;
            while start < chars.len() && chars[start].is_whitespace() {
                start += 1;
            }
            if start >= chars.len() {
                return None;
            }
            let mut end = start;
            while end < chars.len() && is_word_char(chars[end]) {
                end += 1;
            }
            // Include trailing whitespace
            while end < chars.len() && chars[end].is_whitespace() {
                end += 1;
            }
            return Some((safe_col, end));
        } else {
            return None; // 'iw' on whitespace does nothing
        }
    }

    // Find word boundaries around current position
    let mut start = safe_col;
    let mut end = safe_col;

    // Find word start
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    // Find word end
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    if around {
        // Include trailing whitespace
        while end < chars.len() && chars[end].is_whitespace() {
            end += 1;
        }
        // If no trailing whitespace, include leading whitespace
        if end == safe_col + 1 || (end < chars.len() && !chars[end - 1].is_whitespace()) {
            while start > 0 && chars[start - 1].is_whitespace() {
                start -= 1;
            }
        }
    }

    if start < end {
        Some((start, end))
    } else {
        None
    }
}

fn find_quote_boundaries(
    line: &str,
    col: usize,
    quote: char,
    around: bool,
) -> Option<(usize, usize)> {
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let safe_col = col.min(chars.len().saturating_sub(1));

    // Check if cursor is on a quote
    if safe_col < chars.len() && chars[safe_col] == quote {
        // If on opening quote, find matching closing quote
        let mut end = None;
        for i in (safe_col + 1)..chars.len() {
            if chars[i] == quote && (i == 0 || chars[i - 1] != '\\') {
                end = Some(i);
                break;
            }
        }
        if let Some(e) = end {
            return if around {
                Some((safe_col, e + 1))
            } else {
                Some((safe_col + 1, e))
            };
        }
    }

    // Find opening quote before cursor (skipping escaped quotes)
    let mut start = None;
    for i in (0..safe_col).rev() {
        if chars[i] == quote && (i == 0 || chars[i - 1] != '\\') {
            start = Some(i);
            break;
        }
    }

    // Find closing quote after cursor (skipping escaped quotes)
    let mut end = None;
    for i in (safe_col + 1)..chars.len() {
        if chars[i] == quote && (i == 0 || chars[i - 1] != '\\') {
            end = Some(i);
            break;
        }
    }

    match (start, end) {
        (Some(s), Some(e)) => {
            if around {
                Some((s, e + 1))
            } else {
                Some((s + 1, e))
            }
        }
        _ => None,
    }
}
