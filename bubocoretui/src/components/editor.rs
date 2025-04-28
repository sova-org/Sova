use crate::app::App;
use crate::{app::EditorKeymapMode, components::Component, components::logs::LogLevel};
use arboard::Clipboard;
use bubocorelib::schedule::ActionTiming;
use bubocorelib::server::client::ClientMessage;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Modifier, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};
use std::{cmp::min, fmt};
use tui_textarea::{CursorMove, Input, Key, Scrolling, TextArea, SyntaxHighlighter};
use unicode_width::UnicodeWidthStr; // Needed for calculating display width

// --- Vim Mode Definitions ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Operator(char),
    Command,        // New mode for entering commands like :1
    SearchForward,  // New mode for typing forward search query
    SearchBackward, // New mode for typing backward search query
}

impl VimMode {
    // Helper to get a title string for the block
    fn title_string(&self) -> String {
        match self {
            Self::Normal => "NORMAL".to_string(),
            Self::Insert => "INSERT".to_string(),
            Self::Visual => "VISUAL".to_string(),
            Self::Operator(c) => format!("OPERATOR({})", c),
            Self::Command => "COMMAND".to_string(), // Title for Command mode
            Self::SearchForward => "SEARCH".to_string(), // Title for Search modes
            Self::SearchBackward => "SEARCH".to_string(),
        }
    }

    // Helper to get cursor style (copied from example)
    fn cursor_style(&self) -> Style {
        let color = match self {
            Self::Normal => Color::Reset,
            Self::Insert => Color::LightBlue,
            Self::Visual => Color::LightYellow,
            Self::Operator(_) => Color::LightGreen,
            Self::Command => Color::Yellow, // Cursor style for Command mode
            Self::SearchForward => Color::LightMagenta, // Cursor style for Search modes
            Self::SearchBackward => Color::LightMagenta,
        };
        Style::default().fg(color).add_modifier(Modifier::REVERSED)
    }
}

impl fmt::Display for VimMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.title_string())
    }
}

// How the Vim emulation state transitions
#[derive(Debug, Clone, PartialEq, Eq)] // Removed Copy
enum VimTransition {
    Nop(Option<String>), // No operation / state change (optional status message)
    Mode(VimMode, Option<String>), // Switch to a new mode (optional status message)
    Pending(Input),      // Waiting for the next key (e.g., after 'g')
                         // Quit is handled by the main editor Esc logic now
}

// State of Vim emulation
#[derive(Debug, Clone)]
pub struct VimState {
    pub mode: VimMode,
    pending: Input,             // For multi-key sequences like 'gg'
    replace_pending: bool,      // Flag for 'r' command
    pub command_buffer: String, // Buffer for command mode input
}

impl VimState {
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            pending: Input::default(),
            replace_pending: false,
            command_buffer: String::new(), // Initialize command buffer
        }
    }

    // Helper to update state with pending input
    fn set_pending(&mut self, pending: Input) {
        self.pending = pending;
        self.replace_pending = false; // Clear replace flag if setting other pending input
        self.command_buffer.clear(); // Clear command buffer
    }

    // Helper to reset pending input
    fn clear_pending(&mut self) {
        self.pending = Input::default();
        self.replace_pending = false; // Also clear replace flag
        // Keep command buffer as is, only clear on mode change or explicit command actions
    }

    // Helper to set Vim mode
    fn set_mode(&mut self, mode: VimMode) {
        self.mode = mode;
        self.pending = Input::default();
        self.replace_pending = false; // Clear flags on mode change
        // Don't clear buffer when entering command or search modes
        if !matches!(
            mode,
            VimMode::Command | VimMode::SearchForward | VimMode::SearchBackward
        ) {
            self.command_buffer.clear();
        }
    }

    // Helper to enter replace pending state
    fn set_replace_pending(&mut self) {
        self.pending = Input::default(); // Clear other pending
        self.replace_pending = true;
        self.command_buffer.clear(); // Clear command buffer
        // Mode remains Normal
    }
}

// --- End Vim Mode Definitions ---

// Define the state for the search functionality
#[derive(Clone)]
pub struct SearchState {
    pub is_active: bool,
    pub query_textarea: TextArea<'static>,
    pub error_message: Option<String>,
}

impl SearchState {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search Query (Esc: Cancel, Enter: Find, ^N/↓: Next, ^P/↑: Prev) "),
        );
        // Ensure it doesn't allow multi-line input by default
        // Note: tui-textarea doesn't have a strict single-line mode, but
        // we prevent Enter from inserting newlines in the handler.
        Self {
            is_active: false,
            query_textarea: textarea,
            error_message: None,
        }
    }
}

pub struct EditorComponent;

impl EditorComponent {
    pub fn new() -> Self {
        Self {}
    }

    // --- Vim Input Handler ---
    fn handle_vim_input(&mut self, app: &mut App, input: Input) -> bool {
        let textarea = &mut app.editor.textarea;
        let vim_state = &mut app.editor.vim_state;
        let current_mode = vim_state.mode;

        let is_esc_in_normal_mode =
            matches!(input, Input { key: Key::Esc, .. }) && current_mode == VimMode::Normal;
        if is_esc_in_normal_mode {
            return false;
        }

        // --- Handle Replace Pending State FIRST ---
        if vim_state.replace_pending {
            vim_state.replace_pending = false; // Consume the pending state
            match input {
                Input {
                    key: Key::Char(c), ..
                } => {
                    textarea.delete_next_char(); // Delete char under cursor
                    textarea.insert_char(c); // Insert the new char
                    // insert_char moves cursor forward, move back to stay on the replaced char
                    textarea.move_cursor(CursorMove::Back);
                    // Stay in Normal mode
                    let (consumed, _) = self.update_vim_state(
                        vim_state,
                        VimTransition::Mode(VimMode::Normal, None),
                        textarea,
                    );
                    return consumed;
                }
                Input { key: Key::Esc, .. } => {
                    // Cancel replace, do nothing else
                    let (consumed, _) = self.update_vim_state(
                        vim_state,
                        VimTransition::Mode(VimMode::Normal, None),
                        textarea,
                    );
                    return consumed;
                }
                _ => {
                    // Invalid key after 'r', just cancel and go back to normal
                    let (consumed, _) = self.update_vim_state(
                        vim_state,
                        VimTransition::Mode(VimMode::Normal, None),
                        textarea,
                    );
                    return consumed;
                }
            }
        }
        // --- End Replace Pending Handling ---

        let pending_input = vim_state.pending.clone();

        let transition = match current_mode {
            VimMode::Normal | VimMode::Visual | VimMode::Operator(_) => {
                let mut op_applied_transition = VimTransition::Nop(None);

                match input {
                    // --- Existing Movements ---
                    Input {
                        key: Key::Char('h'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Back);
                    }
                    Input {
                        key: Key::Char('j'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Down);
                    }
                    Input {
                        key: Key::Char('k'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Up);
                    }
                    Input {
                        key: Key::Char('l'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Forward);
                    }
                    Input {
                        key: Key::Char('w'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordForward);
                    }
                    Input {
                        key: Key::Char('e'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordEnd);
                        if matches!(current_mode, VimMode::Operator(_)) {
                            textarea.move_cursor(CursorMove::Forward);
                        }
                    }
                    Input {
                        key: Key::Char('b'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordBack);
                    }
                    Input {
                        key: Key::Char('^'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Head);
                    } // To first non-whitespace
                    Input {
                        key: Key::Char('$'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::End);
                    } // To end of line

                    // --- NEW '0' Movement ---
                    Input {
                        key: Key::Char('0'),
                        ..
                    } => {
                        let (row, _) = textarea.cursor(); // Get current row
                        textarea.move_cursor(CursorMove::Jump(row as u16, 0)); // Jump to column 0, casting row
                    }

                    // --- NEW Arrow Key Movements ---
                    Input { key: Key::Left, .. } => {
                        textarea.move_cursor(CursorMove::Back);
                    }
                    Input {
                        key: Key::Right, ..
                    } => {
                        textarea.move_cursor(CursorMove::Forward);
                    }
                    Input { key: Key::Up, .. } => {
                        textarea.move_cursor(CursorMove::Up);
                    }
                    Input { key: Key::Down, .. } => {
                        textarea.move_cursor(CursorMove::Down);
                    }

                    // --- Existing Edits ---
                    Input {
                        key: Key::Char('D'),
                        ..
                    } => {
                        textarea.delete_line_by_end();
                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }
                    Input {
                        key: Key::Char('C'),
                        ..
                    } => {
                        textarea.delete_line_by_end();
                        textarea.cancel_selection();
                        op_applied_transition = VimTransition::Mode(VimMode::Insert, None);
                    }
                    Input {
                        key: Key::Char('p'),
                        ..
                    } => {
                        let mut status_message: Option<String> = None;
                        match Clipboard::new() {
                            Ok(mut clipboard) => {
                                match clipboard.get_text() {
                                    Ok(text) => {
                                        if text.ends_with('\n') {
                                            // Line-wise paste: paste below current line
                                            textarea.move_cursor(CursorMove::End);
                                            textarea.insert_newline();
                                            textarea.paste();
                                            // Cursor usually ends up at the start of the *next* line after paste inserts its own newline
                                            // Move up to the beginning of the pasted content.
                                            textarea.move_cursor(CursorMove::Up);
                                            textarea.move_cursor(CursorMove::Head);
                                        } else {
                                            // Character-wise paste: paste after cursor
                                            textarea.move_cursor(CursorMove::Forward);
                                            textarea.paste();
                                            // Move cursor back to end of pasted text (Vim behavior)
                                            // textarea.move_cursor(CursorMove::Back); // Optional: depending on exact desired cursor pos
                                        }
                                    }
                                    Err(err) => {
                                        status_message = Some(format!("Clipboard error: {}", err));
                                        // Fallback? Or do nothing?
                                    }
                                }
                            }
                            Err(err) => {
                                status_message = Some(format!("Clipboard context error: {}", err));
                                // Fallback? Or do nothing?
                            }
                        }
                        if let Some(msg) = status_message {
                            op_applied_transition = VimTransition::Nop(Some(msg));
                        }
                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }
                    Input {
                        key: Key::Char('u'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.undo();
                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }
                    Input {
                        key: Key::Char('r'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.redo();
                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }
                    Input {
                        key: Key::Char('x'),
                        ..
                    } => {
                        let (row, col) = textarea.cursor();
                        let lines = textarea.lines();
                        let num_lines = lines.len();

                        // Determine if the cursor is exactly on the last character of the buffer
                        let is_on_last_char = if num_lines > 0 {
                            let last_line_idx = num_lines - 1;
                            if row == last_line_idx {
                                let last_line_len =
                                    lines.get(last_line_idx).map_or(0, |s| s.chars().count());
                                // Check if cursor column is the index of the last character (0-based)
                                col == last_line_len.saturating_sub(1)
                            } else {
                                false // Not on the last line
                            }
                        } else {
                            false // Empty buffer
                        };

                        // Assuming delete_next_char deletes the char AT the cursor position
                        let deleted = textarea.delete_next_char();

                        if deleted && is_on_last_char {
                            // If we deleted the exact last character, move cursor back
                            textarea.move_cursor(CursorMove::Back);
                        }
                        // Otherwise, the cursor stays put, which is the desired behavior.

                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }

                    // --- Mode Changes ---
                    Input {
                        key: Key::Char('i'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        op_applied_transition = VimTransition::Mode(VimMode::Insert, None);
                    }
                    Input {
                        key: Key::Char('a'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::Forward);
                        op_applied_transition = VimTransition::Mode(VimMode::Insert, None);
                    }
                    Input {
                        key: Key::Char('A'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::End);
                        op_applied_transition = VimTransition::Mode(VimMode::Insert, None);
                    }
                    Input {
                        key: Key::Char('o'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::End);
                        textarea.insert_newline();
                        op_applied_transition = VimTransition::Mode(VimMode::Insert, None);
                    }
                    Input {
                        key: Key::Char('O'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Head);
                        textarea.insert_newline();
                        textarea.move_cursor(CursorMove::Up);
                        op_applied_transition = VimTransition::Mode(VimMode::Insert, None);
                    }
                    Input {
                        key: Key::Char('I'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::Head);
                        op_applied_transition = VimTransition::Mode(VimMode::Insert, None);
                    }

                    // --- Scrolling ---
                    Input {
                        key: Key::Char('e'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll((1, 0));
                    }
                    Input {
                        key: Key::Char('y'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll((-1, 0));
                    }
                    Input {
                        key: Key::Char('d'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll(Scrolling::HalfPageDown);
                    }
                    Input {
                        key: Key::Char('u'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll(Scrolling::HalfPageUp);
                    }
                    Input {
                        key: Key::Char('f'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll(Scrolling::PageDown);
                    }
                    Input {
                        key: Key::Char('b'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.scroll(Scrolling::PageUp);
                    }

                    // --- Visual Mode Transitions ---
                    Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if current_mode == VimMode::Normal => {
                        textarea.start_selection();
                        op_applied_transition = VimTransition::Mode(VimMode::Visual, None);
                    }
                    Input {
                        key: Key::Char('V'),
                        ctrl: false,
                        ..
                    } if current_mode == VimMode::Normal => {
                        textarea.move_cursor(CursorMove::Head);
                        textarea.start_selection();
                        textarea.move_cursor(CursorMove::End);
                        op_applied_transition = VimTransition::Mode(VimMode::Visual, None);
                    }

                    // --- Esc Handling ---
                    Input { key: Key::Esc, .. } if current_mode == VimMode::Normal => {
                        op_applied_transition = VimTransition::Nop(None);
                    }
                    Input { key: Key::Esc, .. }
                    | Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if matches!(current_mode, VimMode::Visual | VimMode::Operator(_)) => {
                        textarea.cancel_selection();
                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }

                    // --- Pending sequences (gg, operators) ---
                    Input {
                        key: Key::Char('g'),
                        ctrl: false,
                        ..
                    } if matches!(
                        pending_input,
                        Input {
                            key: Key::Char('g'),
                            ..
                        }
                    ) =>
                    {
                        textarea.move_cursor(CursorMove::Top);
                    }
                    Input {
                        key: Key::Char('G'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Bottom);
                    }
                    Input {
                        key: Key::Char(c),
                        ctrl: false,
                        ..
                    } if current_mode == VimMode::Operator(c) => {
                        /* Handle yy, dd, cc */
                        let (start_row, _) = textarea.cursor();
                        textarea.move_cursor(CursorMove::Head); // Go to start of current line
                        textarea.start_selection(); // Start selection
                        textarea.move_cursor(CursorMove::Down); // Move to next line
                        let (end_row, _) = textarea.cursor();
                        if start_row == end_row {
                            // If cursor didn't move down (last line)
                            textarea.move_cursor(CursorMove::End); // Select to end of the last line
                        } else {
                            textarea.move_cursor(CursorMove::Head); // Select to start of the next line (includes newline of current)
                        }
                        // The actual copy/cut happens in the operator logic below
                    }
                    Input {
                        key: Key::Char(op @ ('y' | 'd' | 'c')),
                        ctrl: false,
                        ..
                    } if current_mode == VimMode::Normal => {
                        textarea.start_selection();
                        // Use update_vim_state to set mode and clear pending flags correctly
                        let (consumed, _) = self.update_vim_state(
                            vim_state,
                            VimTransition::Mode(VimMode::Operator(op), None),
                            textarea,
                        );
                        return consumed;
                    }
                    Input {
                        key: Key::Char('y'),
                        ctrl: false,
                        ..
                    } if current_mode == VimMode::Visual => {
                        textarea.copy();
                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }
                    Input {
                        key: Key::Char('d'),
                        ctrl: false,
                        ..
                    } if current_mode == VimMode::Visual => {
                        textarea.cut();
                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }
                    Input {
                        key: Key::Char('c'),
                        ctrl: false,
                        ..
                    } if current_mode == VimMode::Visual => {
                        textarea.cut();
                        op_applied_transition = VimTransition::Mode(VimMode::Insert, None);
                    }

                    // --- NEW 'r' command ---
                    Input {
                        key: Key::Char('r'),
                        ctrl: false,
                        ..
                    } if current_mode == VimMode::Normal => {
                        vim_state.set_replace_pending();
                        op_applied_transition = VimTransition::Nop(None); // Stay in normal mode, but waiting
                    }

                    // --- NEW 'J' command ---
                    Input {
                        key: Key::Char('J'),
                        ctrl: false,
                        ..
                    } if current_mode == VimMode::Normal => {
                        let (row, _) = textarea.cursor();
                        if row < textarea.lines().len() - 1 {
                            // Check if not the last line
                            textarea.move_cursor(CursorMove::End);
                            textarea.insert_char(' ');
                            // delete_next_char should remove the newline if cursor is at the end
                            textarea.delete_next_char();
                            // We might want to trim leading whitespace from the joined line later
                        }
                        vim_state.clear_pending(); // Explicitly clear pending state here
                        op_applied_transition = VimTransition::Nop(None); // Stay in Normal mode
                    }

                    // --- NEW ':' command mode trigger ---
                    Input {
                        key: Key::Char(':'),
                        ..
                    } if current_mode == VimMode::Normal => {
                        op_applied_transition = VimTransition::Mode(VimMode::Command, None);
                    }

                    // --- NEW '/' and '?' search triggers ---
                    Input {
                        key: Key::Char('/'),
                        ..
                    } if current_mode == VimMode::Normal => {
                        op_applied_transition = VimTransition::Mode(VimMode::SearchForward, None);
                    }
                    Input {
                        key: Key::Char('?'),
                        ..
                    } if current_mode == VimMode::Normal => {
                        op_applied_transition = VimTransition::Mode(VimMode::SearchBackward, None);
                    }

                    // --- NEW 'n' and 'N' search repeat ---
                    Input {
                        key: Key::Char('n'),
                        ..
                    } if current_mode == VimMode::Normal => {
                        if !textarea.search_forward(false) {
                            // TODO: Status message "Pattern not found"?
                        }
                        op_applied_transition = VimTransition::Nop(None); // Stay in normal
                    }
                    Input {
                        key: Key::Char('N'),
                        ..
                    } if current_mode == VimMode::Normal => {
                        if !textarea.search_back(false) {
                            // TODO: Status message "Pattern not found"?
                        }
                        op_applied_transition = VimTransition::Nop(None); // Stay in normal
                    }

                    // --- Fallback for Pending ---
                    pending => {
                        // If it wasn't 'g' or 'r', set it as pending (e.g., for future multi-key commands)
                        // Don't overwrite replace_pending if it's active
                        if !vim_state.replace_pending {
                            op_applied_transition = VimTransition::Pending(pending);
                        } else {
                            // Invalid key during replace pending already handled above,
                            // but defensively return Nop here if somehow reached.
                            op_applied_transition = VimTransition::Nop(None);
                        }
                    }
                }
                // Apply pending operator logic
                if op_applied_transition != VimTransition::Nop(None) {
                    op_applied_transition
                } else {
                    match current_mode {
                        VimMode::Operator('y') => {
                            textarea.copy();
                            VimTransition::Mode(VimMode::Normal, None)
                        }
                        VimMode::Operator('d') => {
                            textarea.cut();
                            VimTransition::Mode(VimMode::Normal, None)
                        }
                        VimMode::Operator('c') => {
                            textarea.cut();
                            VimTransition::Mode(VimMode::Insert, None)
                        }
                        _ => VimTransition::Nop(None),
                    }
                }
            }
            VimMode::Insert => match input {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Back);
                    VimTransition::Mode(VimMode::Normal, None)
                }
                _ => {
                    textarea.input(input);
                    VimTransition::Mode(VimMode::Insert, None)
                }
            },
            VimMode::Command => {
                match input {
                    Input { key: Key::Esc, .. } => {
                        // Cancel command, return to Normal mode
                        VimTransition::Mode(VimMode::Normal, None)
                    }
                    Input {
                        key: Key::Enter, ..
                    } => {
                        // Execute command
                        let command = vim_state.command_buffer.trim();
                        if let Ok(line_num) = command.parse::<u16>() {
                            if line_num > 0 && (line_num as usize) <= textarea.lines().len() {
                                // Valid line number (1-based)
                                textarea.move_cursor(CursorMove::Jump(line_num - 1, 0));
                            } else {
                                // Invalid line number (out of bounds)
                                // TODO: Add status message feedback?
                            }
                        } else {
                            // Failed to parse as number
                            // TODO: Add status message feedback for unknown command?
                        }
                        // Always return to Normal mode after Enter
                        VimTransition::Mode(VimMode::Normal, None)
                    }
                    Input {
                        key: Key::Backspace,
                        ..
                    } => {
                        vim_state.command_buffer.pop();
                        // Stay in Command mode
                        VimTransition::Mode(VimMode::Command, None)
                    }
                    Input {
                        key: Key::Char(c), ..
                    } => {
                        vim_state.command_buffer.push(c);
                        // Stay in Command mode
                        VimTransition::Mode(VimMode::Command, None)
                    }
                    _ => {
                        // Ignore other keys (like Ctrl combinations, arrows etc.)
                        // Stay in Command mode
                        VimTransition::Mode(VimMode::Command, None)
                    }
                }
            }
            VimMode::SearchForward | VimMode::SearchBackward => {
                let is_forward = current_mode == VimMode::SearchForward;
                match input {
                    Input { key: Key::Esc, .. } => {
                        // Cancel search, clear buffer and pattern, return to Normal
                        vim_state.command_buffer.clear();
                        textarea.set_search_pattern("").ok(); // Ignore error if regex was invalid
                        VimTransition::Mode(VimMode::Normal, None)
                    }
                    Input {
                        key: Key::Enter, ..
                    } => {
                        // Execute search
                        let query = &vim_state.command_buffer;
                        match textarea.set_search_pattern(query) {
                            Ok(_) => {
                                let found = if is_forward {
                                    textarea.search_forward(true)
                                } else {
                                    textarea.search_back(true)
                                };
                                if !found {
                                    // TODO: Status message "Pattern not found"?
                                }
                            }
                            Err(_e) => {
                                // TODO: Status message for invalid regex?
                                // textarea.set_search_pattern("").ok(); // Clear pattern on error?
                            }
                        }
                        // Return to Normal mode after Enter, keeping pattern active
                        VimTransition::Mode(VimMode::Normal, None)
                    }
                    Input {
                        key: Key::Backspace,
                        ..
                    } => {
                        vim_state.command_buffer.pop();
                        // Stay in Search mode
                        VimTransition::Mode(current_mode, None) // Stay in SearchForward or SearchBackward
                    }
                    Input {
                        key: Key::Char(c), ..
                    } => {
                        vim_state.command_buffer.push(c);
                        // Stay in Search mode
                        VimTransition::Mode(current_mode, None)
                    }
                    _ => {
                        // Ignore other keys
                        // Stay in Search mode
                        VimTransition::Mode(current_mode, None)
                    }
                }
            }
        };

        // Update state and handle potential status message
        let (consumed, status_msg_opt) = self.update_vim_state(vim_state, transition, textarea);
        if let Some(msg) = status_msg_opt {
            app.set_status_message(msg);
        }
        consumed
    }

    // Helper to update Vim state and textarea style based on transition
    fn update_vim_state(
        &self,
        vim_state: &mut VimState,
        transition: VimTransition,
        textarea: &mut TextArea,
    ) -> (bool, Option<String>) {
        let old_mode = vim_state.mode;
        let mut status_msg = None;

        match transition {
            VimTransition::Mode(new_mode, msg_opt) => {
                vim_state.set_mode(new_mode);
                if old_mode != new_mode {
                    textarea.set_cursor_style(new_mode.cursor_style());
                }
                status_msg = msg_opt;
                (true, status_msg) // Consumed
            }
            VimTransition::Pending(pending_input) => {
                vim_state.set_pending(pending_input);
                (true, None) // Consumed (waiting for next)
            }
            VimTransition::Nop(msg_opt) => {
                status_msg = msg_opt;
                (true, status_msg) // Consumed (action performed, no mode change)
            }
        }
    }

    // --- Normal (Emacs-like) Input Handler ---
    fn handle_normal_input(&mut self, app: &mut App, key_event: KeyEvent) -> bool {
        // Returns true if input was consumed
        let textarea = &mut app.editor.textarea;
        match key_event.code {
            // Basic Emacs-like navigation
            KeyCode::Char('f') if key_event.modifiers == KeyModifiers::CONTROL => {
                textarea.move_cursor(CursorMove::Forward);
                return true;
            }
            KeyCode::Char('b') if key_event.modifiers == KeyModifiers::CONTROL => {
                textarea.move_cursor(CursorMove::Back);
                return true;
            }
            KeyCode::Char('n') if key_event.modifiers == KeyModifiers::CONTROL => {
                textarea.move_cursor(CursorMove::Down);
                return true;
            }
            KeyCode::Char('p') if key_event.modifiers == KeyModifiers::CONTROL => {
                textarea.move_cursor(CursorMove::Up);
                return true;
            }
            KeyCode::Char('a') if key_event.modifiers == KeyModifiers::CONTROL => {
                textarea.move_cursor(CursorMove::Head);
                return true;
            }
            KeyCode::Char('e') if key_event.modifiers == KeyModifiers::CONTROL => {
                textarea.move_cursor(CursorMove::End);
                return true;
            }
            // Deletion
            KeyCode::Char('d') if key_event.modifiers == KeyModifiers::CONTROL => {
                textarea.delete_next_char();
                return true;
            }
            KeyCode::Backspace => {
                // Or Ctrl+H? Need to decide
                textarea.delete_char();
                return true;
            }
            KeyCode::Delete => {
                textarea.delete_next_char();
                return true;
            }
            // Cut/Copy/Paste (Example using Alt keys, adjust as needed)
            KeyCode::Char('w') if key_event.modifiers == KeyModifiers::ALT => {
                // Alt+W for copy (like kill-ring-save)
                // Need selection first, TBD how to handle Emacs selection
                // textarea.copy();
                app.set_status_message("Copy (Alt+W) - requires selection (TBD)".to_string());
                return true;
            }
            KeyCode::Char('k') if key_event.modifiers == KeyModifiers::CONTROL => {
                // Ctrl+K for kill-line
                textarea.delete_line_by_end();
                // TODO: Add to a conceptual kill ring?
                app.set_status_message("Kill line (Ctrl+K)".to_string());
                return true;
            }
            KeyCode::Char('y') if key_event.modifiers == KeyModifiers::CONTROL => {
                // Ctrl+Y for yank (paste)
                textarea.paste();
                app.set_status_message("Yank (Ctrl+Y)".to_string());
                return true;
            }

            // Undo/Redo (Simple)
            KeyCode::Char('/') if key_event.modifiers == KeyModifiers::CONTROL => {
                // Ctrl+/ for undo
                textarea.undo();
                return true;
            }
            // Redo often doesn't have a standard simple Emacs binding, maybe skip or use Alt?

            // Let other keys fall through to default handling
            _ => {
                // Use tui_textarea's default input handling for typing, arrows, etc.
                // if they weren't specifically handled above.
                textarea.input(key_event)
            }
        }
    }

    fn render_single_line_view(
        &self,
        app: &App,
        frame: &mut Frame,
        area: Rect,
        line_idx: usize,
        current_edit_frame_idx: usize,
        playhead_pos_opt: Option<usize>,
    ) {
        let line_view_block = Block::default()
            .title(" Line ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let inner_area = line_view_block.inner(area);
        frame.render_widget(line_view_block, area);

        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        if let Some(scene) = app.editor.scene.as_ref() {
            if let Some(line) = scene.lines.get(line_idx) {
                if line.frames.is_empty() {
                    frame.render_widget(
                        Paragraph::new("Line is empty")
                            .centered()
                            .style(Style::default().fg(Color::DarkGray)),
                        inner_area,
                    );
                    return;
                }

                let items: Vec<ListItem> = line
                    .frames
                    .iter()
                    .enumerate()
                    .map(|(i, _frame_val)| {
                        let is_enabled = line.is_frame_enabled(i);
                        let is_playhead = playhead_pos_opt == Some(i);
                        let is_start = line.start_frame == Some(i);
                        let is_end = line.end_frame == Some(i);
                        let is_current_edit = i == current_edit_frame_idx;

                        // Fixed elements width calculation
                        let playhead_width = 1;
                        let marker_width = 1;
                        let block_width = 1; // UnicodeWidthStr::width("█") is 1
                        let index_width = 3; // " {:<2}" -> " 1", " 10", "100" might need adjustment
                        let fixed_spacers_width = 3; // Between playhead/marker, marker/name, block/index
                        let total_fixed_width = playhead_width
                            + marker_width
                            + block_width
                            + index_width
                            + fixed_spacers_width;
                        let max_name_width =
                            (inner_area.width as usize).saturating_sub(total_fixed_width);

                        // Fetch and truncate name
                        let frame_name = line.frame_names.get(i).cloned().flatten();
                        let name_str = frame_name.unwrap_or_default();
                        let truncated_name: String = if name_str.width() > max_name_width {
                            name_str
                                .chars()
                                .take(max_name_width.saturating_sub(1))
                                .collect::<String>()
                                + "…"
                        } else {
                            name_str
                        };
                        let name_span = Span::raw(format!(
                            "{:<width$}",
                            truncated_name,
                            width = max_name_width
                        ));

                        // Build Spans
                        let playhead_span = Span::raw(if is_playhead { "▶" } else { " " });
                        let marker_span = Span::raw(if is_start {
                            "b"
                        } else if is_end {
                            "e"
                        } else {
                            " "
                        });
                        let frame_block_char = "█";
                        let frame_block_span = Span::styled(
                            frame_block_char,
                            Style::default().fg(if is_enabled { Color::Green } else { Color::Red }),
                        );
                        let index_span = Span::raw(format!(" {:<2}", i));

                        // Build Style
                        let mut item_style = Style::default();
                        if is_current_edit {
                            item_style =
                                item_style.add_modifier(Modifier::REVERSED).fg(Color::White);
                        } else {
                            item_style = item_style.fg(Color::Gray);
                        }

                        ListItem::new(Line::from(vec![
                            playhead_span,
                            marker_span,
                            Span::raw(" "), // Spacer 1
                            name_span,      // Truncated Name
                            Span::raw(" "), // Spacer 2
                            frame_block_span,
                            Span::raw(" "), // Spacer 3
                            index_span,
                        ]))
                        .style(item_style)
                    })
                    .collect();

                let list = List::new(items);

                frame.render_widget(list, inner_area);
            } else {
                frame.render_widget(
                    Paragraph::new("Invalid Line")
                        .centered()
                        .style(Style::default().fg(Color::Red)),
                    inner_area,
                );
            }
        } else {
            frame.render_widget(
                Paragraph::new("No Scene")
                    .centered()
                    .style(Style::default().fg(Color::DarkGray)),
                inner_area,
            );
        }
    }
}

impl Component for EditorComponent {
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        // --- Priority Handling (Language Popup, Search, Global Actions) ---

        // 0. Handle Language Popup First (if active)
        if app.editor.is_lang_popup_active {
            let num_langs = app.editor.available_languages.len();
            if num_langs == 0 {
                // Should not happen if initialized correctly
                app.editor.is_lang_popup_active = false;
                app.set_status_message("No languages available to select.".to_string());
                return Ok(true);
            }

            match key_event.code {
                KeyCode::Esc => {
                    app.editor.is_lang_popup_active = false;
                    app.set_status_message("Language selection cancelled.".to_string());
                    return Ok(true);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.editor.selected_lang_index =
                        app.editor.selected_lang_index.saturating_sub(1);
                    return Ok(true);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.editor.selected_lang_index =
                        (app.editor.selected_lang_index + 1).min(num_langs - 1);
                    return Ok(true);
                }
                KeyCode::Enter => {
                    let lang_to_set: Option<String> = app
                        .editor
                        .available_languages
                        .get(app.editor.selected_lang_index)
                        .cloned(); // Clone the string here

                    if let Some(selected_lang) = lang_to_set {
                        let line_idx = app.editor.active_line.line_index;
                        let frame_idx = app.editor.active_line.frame_index;
                        app.send_client_message(ClientMessage::SetScriptLanguage(
                            line_idx,
                            frame_idx,
                            selected_lang.clone(),
                            ActionTiming::Immediate, // Clone again for the message
                        ));
                        app.set_status_message(format!(
                            "Set language for Frame {}/{} to {}",
                            line_idx, frame_idx, selected_lang
                        ));
                    } else {
                        app.set_status_message("Error selecting language.".to_string());
                    }
                    app.editor.is_lang_popup_active = false;
                    return Ok(true);
                }
                _ => {
                    return Ok(true);
                } // Consume other keys while popup is active
            }
        }

        // 1. Handle Search Mode First
        if app.editor.search_state.is_active {
            let search_state = &mut app.editor.search_state;
            let main_textarea = &mut app.editor.textarea;
            let input = key_event.into(); // Convert once

            // --- Search Input Handling ---
            match input {
                Input { key: Key::Esc, .. } => {
                    search_state.is_active = false;
                    search_state.error_message = None;
                    tui_textarea::TextArea::set_search_pattern(main_textarea, "")
                        .expect("Empty pattern should be valid");
                    search_state.query_textarea.move_cursor(CursorMove::End);
                    // Clear the search text area content
                    if !search_state.query_textarea.is_empty() {
                        search_state.query_textarea.move_cursor(CursorMove::Head); // Go to start
                        search_state.query_textarea.delete_line_by_end(); // Delete everything
                    }
                    app.set_status_message("Search cancelled.".to_string());
                    return Ok(true);
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    if !tui_textarea::TextArea::search_forward(main_textarea, true) {
                        search_state.error_message = Some("Pattern not found".to_string());
                    } else {
                        search_state.error_message = None;
                    }
                    search_state.is_active = false; // Deactivate search after Enter
                    // Clear the search text area content
                    if !search_state.query_textarea.is_empty() {
                        search_state.query_textarea.move_cursor(CursorMove::Head);
                        search_state.query_textarea.delete_line_by_end();
                    }
                    tui_textarea::TextArea::set_search_pattern(main_textarea, "").ok(); // Clear highlight
                    app.set_status_message("Search closed.".to_string());
                    return Ok(true);
                }
                Input {
                    key: Key::Char('n'),
                    ctrl: true,
                    ..
                }
                | Input { key: Key::Down, .. } => {
                    if !tui_textarea::TextArea::search_forward(main_textarea, false) {
                        search_state.error_message = Some("Pattern not found".to_string());
                    } else {
                        search_state.error_message = None;
                    }
                    return Ok(true);
                }
                Input {
                    key: Key::Char('p'),
                    ctrl: true,
                    ..
                }
                | Input { key: Key::Up, .. } => {
                    if !tui_textarea::TextArea::search_back(main_textarea, false) {
                        search_state.error_message = Some("Pattern not found".to_string());
                    } else {
                        search_state.error_message = None;
                    }
                    return Ok(true);
                }
                input => {
                    // Prevent Enter/Ctrl+M from adding newline in search box
                    if matches!(
                        input,
                        Input {
                            key: Key::Enter,
                            ..
                        } | Input {
                            key: Key::Char('m'),
                            ctrl: true,
                            ..
                        }
                    ) {
                        return Ok(true);
                    }
                    let modified = search_state.query_textarea.input(input);
                    if modified {
                        // Handle empty query correctly - should clear pattern
                        let query = search_state
                            .query_textarea
                            .lines()
                            .get(0)
                            .map_or("", |s| s.as_str());
                        match tui_textarea::TextArea::set_search_pattern(main_textarea, query) {
                            Ok(_) => {
                                search_state.error_message = None;
                                // Try to find first match immediately if pattern is valid and not empty
                                if !query.is_empty() {
                                    tui_textarea::TextArea::search_forward(main_textarea, true);
                                }
                            }
                            Err(e) => search_state.error_message = Some(e.to_string()),
                        }
                    }
                    return Ok(true);
                }
            }
            // End Search Input Handling
        } // End Search Mode block

        // 2. Handle Editor Exit (Esc) - ONLY if not searching
        if key_event.code == KeyCode::Esc {
            match app.settings.editor_keymap_mode {
                EditorKeymapMode::Normal => {
                    // Normal mode: Esc always exits editor
                    app.send_client_message(ClientMessage::StoppedEditingFrame(
                        app.editor.active_line.line_index,
                        app.editor.active_line.frame_index,
                    ));
                    app.editor.compilation_error = None;
                    app.events.sender.send(crate::event::Event::App(
                        crate::event::AppEvent::SwitchToGrid,
                    ))?;
                    app.set_status_message("Exited editor (Esc).".to_string());
                    return Ok(true);
                }
                EditorKeymapMode::Vim => {
                    // Vim mode: Esc is handled by handle_vim_input.
                    // Let it fall through to the mode-specific handler below.
                    // handle_vim_input will return false if Esc was pressed in Normal mode,
                    // signaling that it didn't consume the event for mode switching.
                    // We'll handle the exit *after* the mode-specific handlers are called.
                }
            }
        }

        // 3. Handle Global Editor Actions (Ctrl+S, Ctrl+G, Ctrl+E, Ctrl+Arrows)
        // These should work regardless of Normal/Vim mode (unless Vim mode rebinds them, which we avoid here)
        if key_event.modifiers == KeyModifiers::CONTROL {
            match key_event.code {
                // --- Send Script ---
                KeyCode::Char('s') => {
                    app.add_log(
                        LogLevel::Debug,
                        "Ctrl+S detected, attempting to send script...".to_string(),
                    );
                    app.send_client_message(ClientMessage::SetScript(
                        app.editor.active_line.line_index,
                        app.editor.active_line.frame_index,
                        app.editor.textarea.lines().join("\n"),
                        ActionTiming::Immediate,
                    ));
                    app.editor.compilation_error = None;
                    app.set_status_message("Sent script content (Ctrl+S).".to_string());
                    app.flash_screen();
                    return Ok(true);
                }
                // --- Activate Search ---
                KeyCode::Char('g') => {
                    app.editor.search_state.is_active = true;
                    app.editor.search_state.error_message = None;
                    // Clear previous search query visually
                    if !app.editor.search_state.query_textarea.is_empty() {
                        app.editor
                            .search_state
                            .query_textarea
                            .move_cursor(CursorMove::Head);
                        app.editor.search_state.query_textarea.delete_line_by_end();
                    }
                    // Reset Vim mode to Normal if activating search from Vim mode? Optional.
                    // if app.settings.editor_keymap_mode == EditorKeymapMode::Vim {
                    //    // Need mutable self here, cannot call set_vim_mode directly
                    //    // Maybe reset vim_state directly?
                    //    app.editor.vim_state.set_mode(VimMode::Normal);
                    //    app.editor.textarea.set_cursor_style(VimMode::Normal.cursor_style());
                    // }
                    app.set_status_message("Search activated. Type query...".to_string());
                    return Ok(true);
                }
                // --- Toggle Frame Enabled ---
                KeyCode::Char('e') => {
                    if let Some(scene) = &app.editor.scene {
                        let line_idx = app.editor.active_line.line_index;
                        let frame_idx = app.editor.active_line.frame_index;

                        if let Some(line) = scene.lines.get(line_idx) {
                            if frame_idx < line.frames.len() {
                                let current_enabled_status = line.is_frame_enabled(frame_idx);
                                let message = if current_enabled_status {
                                    ClientMessage::DisableFrames(
                                        line_idx,
                                        vec![frame_idx],
                                        ActionTiming::Immediate,
                                    )
                                } else {
                                    ClientMessage::EnableFrames(
                                        line_idx,
                                        vec![frame_idx],
                                        ActionTiming::Immediate,
                                    )
                                };
                                app.send_client_message(message);
                                app.set_status_message(format!(
                                    "Toggled Frame {}/{} to {}",
                                    line_idx,
                                    frame_idx,
                                    if !current_enabled_status {
                                        "Enabled"
                                    } else {
                                        "Disabled"
                                    }
                                ));
                            } else {
                                app.set_status_message(
                                    "Cannot toggle: Invalid frame index.".to_string(),
                                );
                            }
                        } else {
                            app.set_status_message(
                                "Cannot toggle: Invalid line index.".to_string(),
                            );
                        }
                    } else {
                        app.set_status_message("Cannot toggle: scene not loaded.".to_string());
                    }
                    return Ok(true);
                }
                // --- Navigate Script ---
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    if let Some(scene) = &app.editor.scene {
                        let current_line_idx = app.editor.active_line.line_index;
                        let current_frame_idx = app.editor.active_line.frame_index;
                        let num_lines = scene.lines.len();
                        if num_lines == 0 {
                            app.set_status_message("No lines to navigate.".to_string());
                            return Ok(true);
                        }
                        let (target_line_idx, target_frame_idx) = match key_event.code {
                            KeyCode::Up => {
                                if current_frame_idx == 0 {
                                    app.set_status_message("Already at first frame.".to_string());
                                    return Ok(true);
                                }
                                (current_line_idx, current_frame_idx - 1)
                            }
                            KeyCode::Down => {
                                if let Some(line) = scene.lines.get(current_line_idx) {
                                    if current_frame_idx + 1 >= line.frames.len() {
                                        app.set_status_message(
                                            "Already at last frame.".to_string(),
                                        );
                                        return Ok(true);
                                    }
                                    (current_line_idx, current_frame_idx + 1)
                                } else {
                                    return Ok(true);
                                } // Should not happen if line exists
                            }
                            KeyCode::Left => {
                                if current_line_idx == 0 {
                                    app.set_status_message("Already at first line.".to_string());
                                    return Ok(true);
                                }
                                let target_line = current_line_idx - 1;
                                let target_line_len =
                                    scene.lines.get(target_line).map_or(0, |l| l.frames.len());
                                if target_line_len == 0 {
                                    app.set_status_message(format!(
                                        "Line {} is empty.",
                                        target_line
                                    ));
                                    return Ok(true);
                                }
                                (target_line, min(current_frame_idx, target_line_len - 1))
                            }
                            KeyCode::Right => {
                                if current_line_idx + 1 >= num_lines {
                                    app.set_status_message("Already at last line.".to_string());
                                    return Ok(true);
                                }
                                let target_line = current_line_idx + 1;
                                let target_line_len =
                                    scene.lines.get(target_line).map_or(0, |l| l.frames.len());
                                if target_line_len == 0 {
                                    app.set_status_message(format!(
                                        "Line {} is empty.",
                                        target_line
                                    ));
                                    return Ok(true);
                                }
                                (target_line, min(current_frame_idx, target_line_len - 1))
                            }
                            _ => unreachable!(),
                        };
                        // Stop editing current frame before requesting new one
                        app.send_client_message(ClientMessage::StoppedEditingFrame(
                            app.editor.active_line.line_index,
                            app.editor.active_line.frame_index,
                        ));
                        app.editor.compilation_error = None;
                        // Request new script
                        app.send_client_message(ClientMessage::GetScript(
                            target_line_idx,
                            target_frame_idx,
                        ));
                        // Immediately request *starting* editing the new frame
                        app.send_client_message(ClientMessage::StartedEditingFrame(
                            target_line_idx,
                            target_frame_idx,
                        ));
                        app.set_status_message(format!(
                            "Requested script Line {}, Frame {}",
                            target_line_idx, target_frame_idx
                        ));
                        return Ok(true);
                    } else {
                        app.set_status_message("scene not loaded, cannot navigate.".to_string());
                        return Ok(true);
                    }
                }
                KeyCode::Char('l') => {
                    let current_lang_opt = app
                        .editor
                        .scene
                        .as_ref()
                        .and_then(|s| s.lines.get(app.editor.active_line.line_index))
                        .and_then(|l| {
                            l.scripts
                                .iter()
                                .find(|scr| scr.index == app.editor.active_line.frame_index)
                        })
                        .map(|scr| scr.lang.clone());

                    if let Some(current_lang) = current_lang_opt {
                        if let Some(index) = app
                            .editor
                            .available_languages
                            .iter()
                            .position(|l| l == &current_lang)
                        {
                            app.editor.selected_lang_index = index;
                        }
                    } // else keep default index 0

                    app.editor.is_lang_popup_active = true;
                    app.set_status_message("Select language (↑/↓/Enter/Esc)".to_string());
                    return Ok(true);
                }
                _ => {}
            }
        } // End Ctrl modifier check

        // --- Mode-Specific Input Handling ---
        let consumed_in_mode;
        match app.settings.editor_keymap_mode {
            EditorKeymapMode::Vim => {
                let input: Input = key_event.into();
                consumed_in_mode = self.handle_vim_input(app, input);
                // If Esc was pressed AND vim handler didn't consume it (meaning it was in Normal mode)
                if key_event.code == KeyCode::Esc
                    && !consumed_in_mode
                    && app.editor.vim_state.mode == VimMode::Normal
                {
                    // Then exit the editor
                    app.send_client_message(ClientMessage::StoppedEditingFrame(
                        app.editor.active_line.line_index,
                        app.editor.active_line.frame_index,
                    ));
                    app.editor.compilation_error = None;
                    app.events.sender.send(crate::event::Event::App(
                        crate::event::AppEvent::SwitchToGrid,
                    ))?;
                    app.set_status_message("Exited editor (Esc in Normal Mode).".to_string());
                    return Ok(true); // Exit handled
                }
            }
            EditorKeymapMode::Normal => {
                consumed_in_mode = self.handle_normal_input(app, key_event);
                // In Normal mode, Esc exit is handled earlier (before mode-specific block)
            }
        };

        // If the mode-specific handler consumed the input, we are done.
        Ok(consumed_in_mode)
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let line_idx = app.editor.active_line.line_index;
        let frame_idx = app.editor.active_line.frame_index;

        let scene_opt = app.editor.scene.as_ref();
        let line_opt = scene_opt.and_then(|s| s.lines.get(line_idx));
        let frame_name_opt = line_opt.and_then(|l| l.frame_names.get(frame_idx).cloned().flatten());
        let playhead_pos_opt = app
            .server
            .current_frame_positions
            .as_ref()
            .and_then(|p| p.get(line_idx))
            .map(|&v| v);

        let (status_str, length_str, is_enabled) = if let Some(line) = line_opt {
            if frame_idx < line.frames.len() {
                let enabled = line.is_frame_enabled(frame_idx);
                let length = line.frames[frame_idx];
                (
                    if enabled { "Enabled" } else { "Disabled" },
                    format!("Len: {:.2}", length),
                    enabled,
                )
            } else {
                ("Invalid Frame", "Len: N/A".to_string(), true)
            }
        } else {
            ("Invalid Line/Scene", "Len: N/A".to_string(), true)
        };

        let border_color = if is_enabled {
            Color::White
        } else {
            Color::DarkGray
        };

        let script_lang_indicator = scene_opt
            .and_then(|s| s.lines.get(line_idx))
            .and_then(|l| l.scripts.iter().find(|scr| scr.index == frame_idx))
            .map(|scr| format!(" | Lang: {}", scr.lang))
            .unwrap_or_else(|| " | Lang: N/A".to_string());

        let vim_mode_indicator = if app.settings.editor_keymap_mode == EditorKeymapMode::Vim {
            format!(" [{}]", app.editor.vim_state.mode.title_string())
        } else {
            String::new()
        };
        let frame_name_indicator =
            frame_name_opt.map_or(String::new(), |name| format!(" ({})", name));

        let editor_block = Block::default()
            .title(format!(
                " Editor (L: {}, F: {}{}{} | {} | {}{}) ",
                line_idx,
                frame_idx,
                frame_name_indicator,
                vim_mode_indicator,
                status_str,
                length_str,
                script_lang_indicator
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(border_color));

        frame.render_widget(editor_block.clone(), area);
        let inner_area = editor_block.inner(area);

        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        let line_view_width = 18; // Increased width
        let actual_line_view_width = min(line_view_width, inner_area.width);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(actual_line_view_width),
            ])
            .split(inner_area);

        let main_editor_area = horizontal_chunks[0];
        let line_view_area = horizontal_chunks[1];

        if main_editor_area.width > 0 && main_editor_area.height > 0 {
            let editor_text_area: Rect;
            let help_area: Rect;
            let mut bottom_panel_area: Option<Rect> = None;

            let search_active = app.editor.search_state.is_active;
            let compilation_error_present = app.editor.compilation_error.is_some();
            let command_mode_active = app.settings.editor_keymap_mode == EditorKeymapMode::Vim
                && app.editor.vim_state.mode == VimMode::Command;
            let search_input_mode_active = app.settings.editor_keymap_mode == EditorKeymapMode::Vim
                && matches!(
                    app.editor.vim_state.mode,
                    VimMode::SearchForward | VimMode::SearchBackward
                );

            let mut constraints = vec![Constraint::Min(0)];
            let mut bottom_panel_height = 0;
            let mut command_line_height = 0;

            // Determine heights, prioritizing Search/Error
            if search_active {
                bottom_panel_height = 3;
            } else if compilation_error_present {
                bottom_panel_height = 5;
            }
            bottom_panel_height = min(
                bottom_panel_height,
                main_editor_area.height.saturating_sub(1),
            );

            // Command line only if no search/error and space permits
            if !search_active
                && !compilation_error_present
                && (command_mode_active || search_input_mode_active)
            {
                command_line_height = min(
                    1,
                    main_editor_area
                        .height
                        .saturating_sub(bottom_panel_height + 1),
                ); // Needs 1 for command, 1 for help
            }

            // Push constraints
            if bottom_panel_height > 0 {
                constraints.push(Constraint::Length(bottom_panel_height));
            }
            if command_line_height > 0 {
                constraints.push(Constraint::Length(command_line_height));
            }
            let help_height = if main_editor_area.height > bottom_panel_height + command_line_height
            {
                1
            } else {
                0
            };
            if help_height > 0 {
                constraints.push(Constraint::Length(help_height));
            }

            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(main_editor_area);

            editor_text_area = vertical_chunks[0];
            let mut current_index = 1;
            if bottom_panel_height > 0 {
                bottom_panel_area = Some(vertical_chunks[current_index]);
                current_index += 1;
            }
            let mut command_line_area: Option<Rect> = None;
            if command_line_height > 0 {
                command_line_area = Some(vertical_chunks[current_index]);
                current_index += 1;
            }
            if current_index < vertical_chunks.len() {
                help_area = vertical_chunks[current_index];
            } else {
                help_area = Rect::new(
                    main_editor_area.x,
                    main_editor_area.y + main_editor_area.height,
                    main_editor_area.width,
                    0,
                );
            }

            if let Some(panel_area) = bottom_panel_area {
                if panel_area.width > 0 && panel_area.height > 0 {
                    if search_active {
                        let search_state = &app.editor.search_state;
                        let mut query_textarea = search_state.query_textarea.clone();
                        let search_block_title = if let Some(err_msg) = &search_state.error_message
                        {
                            format!(
                                " Search (Error: {}) (Esc:Cancel Enter:Find ^N/↓:Next ^P/↑:Prev) ",
                                err_msg
                            )
                        } else {
                            " Search Query (Esc: Cancel, Enter: Find, ^N/↓: Next, ^P/↑: Prev) "
                                .to_string()
                        };
                        let search_block_style = if search_state.error_message.is_some() {
                            Style::default().fg(Color::Red)
                        } else {
                            Style::default().fg(Color::Yellow)
                        };

                        query_textarea.set_block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(search_block_title)
                                .style(search_block_style),
                        );

                        frame.render_widget(&query_textarea, panel_area);
                    } else if let Some(error_msg) = &app.editor.compilation_error {
                        let mut error_line_num = 0;
                        let mut error_col_num = 0;
                        let mut char_idx_count = 0;
                        let editor_lines = app.editor.textarea.lines();

                        for (i, line) in editor_lines.iter().enumerate() {
                            let line_len = line.chars().count();
                            let line_end_idx = char_idx_count + line_len;

                            if error_msg.from >= char_idx_count && error_msg.from < line_end_idx {
                                error_line_num = i;
                                error_col_num = error_msg.from - char_idx_count;
                                break;
                            }
                            let newline_offset = if i < editor_lines.len() - 1 { 1 } else { 0 };
                            char_idx_count = line_end_idx + newline_offset;

                            if error_msg.from == char_idx_count && i + 1 < editor_lines.len() {
                                error_line_num = i + 1;
                                error_col_num = 0;
                                break;
                            }
                        }

                        let error_block = Block::default()
                            .title(format!(
                                " Compilation Error ({}: Line {}, Col {}) ",
                                error_msg.lang,
                                error_line_num + 1,
                                error_col_num + 1
                            ))
                            .borders(Borders::ALL)
                            .border_type(BorderType::Plain)
                            .style(Style::default().fg(Color::Red));
                        let error_paragraph = Paragraph::new(error_msg.info.as_str())
                            .wrap(ratatui::widgets::Wrap { trim: true })
                            .block(error_block.clone());
                        frame.render_widget(error_paragraph, panel_area);
                    }
                }
            }

            // --- Render Command Line (if active) ---
            if let Some(cmd_area) = command_line_area {
                if cmd_area.width > 0 && cmd_area.height > 0 {
                    let buffer_text = &app.editor.vim_state.command_buffer;
                    let (prefix, style) = match app.editor.vim_state.mode {
                        VimMode::Command => (":", Style::default().fg(Color::Yellow)),
                        VimMode::SearchForward => ("/", Style::default().fg(Color::LightMagenta)),
                        VimMode::SearchBackward => ("?", Style::default().fg(Color::LightMagenta)),
                        _ => ("", Style::default()), // Should not be reached if command_line_area is Some
                    };

                    if !prefix.is_empty() {
                        let display_text = format!("{}{}", prefix, buffer_text);
                        let paragraph = Paragraph::new(display_text).style(style);
                        frame.render_widget(paragraph, cmd_area);
                    }
                }
            }
            // --- End Render Command Line ---

            if editor_text_area.width > 0 && editor_text_area.height > 0 {
                let mut text_area = app.editor.textarea.clone();
                text_area.set_line_number_style(Style::default().fg(Color::DarkGray));

                // --- Syntax Highlighting Configuration ---
                if let Some(highlighter) = app.editor.syntax_highlighter.as_ref() { // Assuming highlighter is Option<Arc<SyntaxHighlighter>>
                    // 1. Determine the language of the current frame
                    let current_lang_opt: Option<String> = scene_opt
                        .and_then(|s| s.lines.get(line_idx))
                        .and_then(|l| l.scripts.iter().find(|scr| scr.index == frame_idx))
                        .map(|scr| scr.lang.clone());

                    // 2. Look up the syntect syntax name from the map
                    let syntax_name_opt: Option<String> = current_lang_opt
                        .and_then(|lang| app.editor.syntax_name_map.get(&lang).cloned());

                    // 3. Configure the TextArea
                    text_area.set_syntax_highlighter((**highlighter).clone()); 
                    text_area.set_syntax(syntax_name_opt);
                    // TODO: Make theme configurable?
                    text_area.set_theme(Some("base16-ocean.dark".to_string()));
                } else {
                    // Fallback if highlighter isn't loaded
                    text_area.set_syntax(None);
                }
                // --- End Syntax Highlighting Configuration ---

                frame.render_widget(&text_area, editor_text_area); // Render the configured text_area
            }

            if help_area.width > 0 && help_area.height > 0 {
                let help_style = Style::default().fg(Color::DarkGray);
                let key_style = Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD);

                let help_line = if search_active {
                    Line::from(vec![
                        Span::styled(" Esc ", key_style),
                        Span::styled("Cancel | ", help_style),
                        Span::styled(" Enter ", key_style),
                        Span::styled("Find & Close | ", help_style),
                        Span::styled(" ^N/↓ ", key_style),
                        Span::styled("Next | ", help_style),
                        Span::styled(" ^P/↑ ", key_style),
                        Span::styled("Prev", help_style),
                    ])
                } else {
                    // Base help line
                    let mut help_spans = vec![
                        Span::styled("Ctrl+S", key_style),
                        Span::styled(": Send | ", help_style),
                        Span::styled("Ctrl+E", key_style),
                        Span::styled(": Toggle | ", help_style),
                        Span::styled("Ctrl+G", key_style),
                        Span::styled(": Search | ", help_style),
                        Span::styled("Ctrl+←↑↓→", key_style),
                        Span::styled(": Navigate | ", help_style),
                    ];
                    // Add Vim specific help if applicable
                    // if app.settings.editor_keymap_mode == EditorKeymapMode::Vim {
                    //     help_spans.push(Span::styled(" :<num> ", key_style));
                    //     help_spans.push(Span::styled(": Go Line | ", help_style));
                    //     help_spans.push(Span::styled(" /query ", key_style));
                    //     help_spans.push(Span::styled(": Search Fwd | ", help_style));
                    //     help_spans.push(Span::styled(" ?query ", key_style));
                    //     help_spans.push(Span::styled(": Search Bwd | ", help_style));
                    //     help_spans.push(Span::styled(" n/N ", key_style));
                    //     help_spans.push(Span::styled(": Repeat Search | ", help_style));
                    // }
                    help_spans.push(Span::styled("Esc", key_style));
                    help_spans.push(Span::styled(": Exit", help_style));

                    Line::from(help_spans)
                };

                let help = Paragraph::new(help_line).alignment(ratatui::layout::Alignment::Center);
                frame.render_widget(help, help_area);
            }
        } else {
            frame.render_widget(
                Paragraph::new("Editor Area Too Small")
                    .centered()
                    .style(Style::default().fg(Color::Red)),
                main_editor_area,
            );
        }

        if line_view_area.width > 0 && line_view_area.height > 0 {
            self.render_single_line_view(
                app,
                frame,
                line_view_area,
                line_idx,
                frame_idx,
                playhead_pos_opt,
            );
        } else {
            if inner_area.width > 0 && inner_area.height > 0 {
                let indicator_area = Rect {
                    x: inner_area.right() - 1,
                    y: inner_area.top(),
                    width: 1,
                    height: 1,
                };
                frame.render_widget(
                    Span::styled("…", Style::default().fg(Color::White)),
                    indicator_area,
                );
            }
        }

        // --- Render Language Selection Popup (if active) ---
        if app.editor.is_lang_popup_active {
            use ratatui::widgets::Clear;
            use ratatui::widgets::ListState;

            let popup_width = 30;
            let popup_height = min(app.editor.available_languages.len() + 2, 10) as u16; // +2 for borders, max 10 items high

            // Use the fixed-size centering function
            let popup_area = centered_rect_fixed(popup_width, popup_height, area);

            frame.render_widget(Clear, popup_area); // Clear background

            let items: Vec<ListItem> = app
                .editor
                .available_languages
                .iter()
                .map(|lang| ListItem::new(lang.as_str()))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Select Language (↑/↓/Enter/Esc)")
                        .borders(Borders::ALL),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::REVERSED)
                        .fg(Color::Yellow),
                )
                .highlight_symbol("> ");

            let mut list_state = ListState::default();
            list_state.select(Some(app.editor.selected_lang_index));

            frame.render_stateful_widget(list, popup_area, &mut list_state);
        }
        // --- End Language Selection Popup ---
    }
}

/// Helper function to create a centered rectangle with fixed width/height.
fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let vertical_margin = r.height.saturating_sub(height) / 2;
    let horizontal_margin = r.width.saturating_sub(width) / 2;

    let popup_layout = Layout::vertical([
        Constraint::Length(vertical_margin),
        Constraint::Length(height),
        Constraint::Length(vertical_margin),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Length(horizontal_margin),
        Constraint::Length(width),
        Constraint::Length(horizontal_margin),
    ])
    .split(popup_layout[1])[1]
}
