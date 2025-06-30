use crate::app::App;
use ratatui::prelude::*;
use tui_textarea::{Input, Key, TextArea};

pub mod command;
pub mod modes;
pub mod motion;
pub mod operator;
pub mod parser;

pub use command::*;
pub use motion::*;
pub use operator::*;
pub use parser::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual { line_wise: bool },
    Command,
    Search { forward: bool },
    OperatorPending,
}

impl Mode {
    pub fn title_string(&self) -> String {
        match self {
            Self::Normal => "NORMAL".to_string(),
            Self::Insert => "INSERT".to_string(),
            Self::Visual { line_wise: true } => "V-LINE".to_string(),
            Self::Visual { line_wise: false } => "VISUAL".to_string(),
            Self::Command => "COMMAND".to_string(),
            Self::Search { .. } => "SEARCH".to_string(),
            Self::OperatorPending => "OPERATOR".to_string(),
        }
    }

    pub fn cursor_style(&self) -> Style {
        let color = match self {
            Self::Normal => Color::Reset,
            Self::Insert => Color::LightBlue,
            Self::Visual { .. } => Color::LightYellow,
            Self::OperatorPending => Color::LightGreen,
            Self::Command => Color::Yellow,
            Self::Search { .. } => Color::LightMagenta,
        };
        Style::default().fg(color).add_modifier(Modifier::REVERSED)
    }
}

#[derive(Debug, Clone)]
pub struct VimState {
    pub mode: Mode,
    pub parser: CommandParser,
    pub command_buffer: String,
    pub last_search: Option<String>,
    pub yank_register: YankRegister,
    pub insert_repeat: Option<InsertRepeat>,
    pub replace_pending: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct YankRegister {
    pub yank_type: YankType,
}

#[derive(Debug, Clone)]
pub struct InsertRepeat {
    pub count: u32,
    pub start_cursor: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YankType {
    Characterwise,
    Linewise,
}

impl Default for VimState {
    fn default() -> Self {
        Self::new()
    }
}

impl VimState {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            parser: CommandParser::new(),
            command_buffer: String::new(),
            last_search: None,
            yank_register: YankRegister {
                yank_type: YankType::Characterwise,
            },
            insert_repeat: None,
            replace_pending: None,
        }
    }

    pub fn set_mode(&mut self, mode: Mode, textarea: &mut TextArea) {
        if self.mode != mode {
            self.mode = mode;
            textarea.set_cursor_style(mode.cursor_style());

            if !matches!(mode, Mode::Command | Mode::Search { .. }) {
                self.command_buffer.clear();
            }

            if mode != Mode::OperatorPending {
                self.parser.reset();
            }
        }
    }

    pub fn start_insert_mode(&mut self, count: Option<u32>, textarea: &mut TextArea) {
        let insert_count = count.unwrap_or(1);
        if insert_count > 1 {
            self.insert_repeat = Some(InsertRepeat {
                count: insert_count,
                start_cursor: textarea.cursor(),
            });
        } else {
            self.insert_repeat = None;
        }
        self.set_mode(Mode::Insert, textarea);
    }

    pub fn mode_display_string(&self) -> String {
        if self.replace_pending.is_some() {
            "REPLACE".to_string()
        } else {
            self.mode.title_string()
        }
    }
}

pub fn handle_vim_input(app: &mut App, input: Input) -> bool {
    let mode = app.editor.vim_state.mode;

    if matches!(input, Input { key: Key::Esc, .. }) && mode == Mode::Normal {
        return false; // Signal to exit editor
    }

    match mode {
        Mode::Normal => modes::normal::handle_normal_mode(app, input),
        Mode::Insert => modes::insert::handle_insert_mode(app, input),
        Mode::Visual { .. } => modes::visual::handle_visual_mode(app, input),
        Mode::Command => modes::command::handle_command_mode(app, input),
        Mode::Search { .. } => modes::search::handle_search_mode(app, input),
        Mode::OperatorPending => modes::normal::handle_operator_pending(app, input),
    }
}
