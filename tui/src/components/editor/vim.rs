use crate::app::App;
use arboard::Clipboard;
use ratatui::prelude::*;
use std::fmt;
use tui_textarea::Input;
use tui_textarea::{CursorMove, Key, Scrolling, TextArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YankType {
    Characterwise,
    Linewise,
}

#[derive(Debug, Clone)]
pub struct YankRegister {
    pub yank_type: YankType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents the different modes of Vim-style editing.
///
/// This enum tracks the current editing mode, which determines how key inputs are interpreted
/// and what operations are available to the user. Each mode has distinct behaviors and visual
/// indicators.
///
/// # Variants
///
/// * `Normal` - Default mode for navigation and command execution
/// * `Insert` - Mode for inserting and editing text
/// * `Visual` - Mode for selecting text
/// * `Operator(char)` - Mode for executing an operator command (e.g., 'd' for delete)
/// * `Command` - Mode for entering command-line commands
/// * `SearchForward` - Mode for forward text search
/// * `SearchBackward` - Mode for backward text search
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Operator(char),
    Command,
    SearchForward,
    SearchBackward,
}

impl VimMode {
    /// Returns a string representation of the current Vim mode for display purposes.
    ///
    /// This method provides a human-readable title for each Vim mode that can be used
    /// in UI elements like status bars or mode indicators. The returned string is
    /// typically displayed in uppercase to match Vim's traditional style.
    ///
    /// # Returns
    ///
    /// A `String` containing the mode's display name:
    /// - "NORMAL" for Normal mode
    /// - "INSERT" for Insert mode
    /// - "VISUAL" for Visual mode
    /// - "OPERATOR(x)" for Operator mode (where x is the operator character)
    /// - "COMMAND" for Command mode
    /// - "SEARCH" for both SearchForward and SearchBackward modes
    pub fn title_string(&self) -> String {
        match self {
            Self::Normal => "NORMAL".to_string(),
            Self::Insert => "INSERT".to_string(),
            Self::Visual => "VISUAL".to_string(),
            Self::Operator(c) => format!("OPERATOR({})", c),
            Self::Command => "COMMAND".to_string(),
            Self::SearchForward => "SEARCH".to_string(),
            Self::SearchBackward => "SEARCH".to_string(),
        }
    }

    /// Returns the appropriate cursor style for the current Vim mode.
    ///
    /// This method determines the visual appearance of the cursor based on the current
    /// Vim mode. Each mode has a distinct color scheme to provide visual feedback to
    /// the user about the current editing state. The cursor is always displayed in
    /// reversed video (inverted colors) for better visibility.
    ///
    /// # Returns
    ///
    /// A `Style` object with the following color mappings:
    /// - Normal mode: Reset color (default terminal color)
    /// - Insert mode: Light blue
    /// - Visual mode: Light yellow
    /// - Operator mode: Light green
    /// - Command mode: Yellow
    /// - Search modes (both forward and backward): Light magenta
    pub fn cursor_style(&self) -> Style {
        let color = match self {
            Self::Normal => Color::Reset,
            Self::Insert => Color::LightBlue,
            Self::Visual => Color::LightYellow,
            Self::Operator(_) => Color::LightGreen,
            Self::Command => Color::Yellow,
            Self::SearchForward | Self::SearchBackward => Color::LightMagenta,
        };
        Style::default().fg(color).add_modifier(Modifier::REVERSED)
    }
}

impl fmt::Display for VimMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.title_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents the possible state transitions in Vim emulation.
///
/// This enum defines the different ways the Vim emulation state can change in response
/// to user input. Each variant may include additional data relevant to the transition.
///
/// # Variants
///
/// * `Nop(Option<String>)` - No state change occurred, but may include a status message
///   to display to the user.
/// * `Mode(VimMode, Option<String>)` - Transition to a new Vim mode, optionally with
///   a status message.
/// * `Pending(Input)` - Input was received that requires additional keystrokes to
///   complete the command (e.g., 'g' waiting for another 'g').
/// * `Quit` - Request to exit the editor.
pub enum VimTransition {
    Nop(Option<String>),
    Mode(VimMode, Option<String>),
    Pending(Input),
}

#[derive(Debug, Clone)]
/// Represents the current state of Vim emulation in the editor.
///
/// This struct maintains the essential state needed to emulate Vim's modal editing behavior,
/// including the current mode, pending input for multi-key commands, and command buffer state.
///
/// # Fields
///
/// * `mode` - The current Vim mode (Normal, Insert, Visual, etc.)
/// * `pending` - Input that has been received but requires additional keystrokes to complete
///   a command (e.g., 'g' waiting for another 'g' for 'gg' command)
/// * `replace_pending` - Flag indicating if we're in the middle of a replace operation
///   (e.g., after pressing 'r' waiting for the character to replace with)
/// * `command_buffer` - String buffer used to accumulate command input in command mode
///   (e.g., when typing ':w' or search patterns)
/// * `yank_register` - Internal register for storing yanked text with type information
pub struct VimState {
    pub mode: VimMode,
    pub pending: Input,
    pub replace_pending: bool,
    pub command_buffer: String,
    pub yank_register: YankRegister,
}

impl Default for VimState {
    fn default() -> Self {
        Self::new()
    }
}

impl VimState {
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            pending: Input::default(),
            replace_pending: false,
            command_buffer: String::new(),
            yank_register: YankRegister {
                yank_type: YankType::Characterwise,
            },
        }
    }

    pub fn set_pending(&mut self, pending: Input) {
        self.pending = pending;
        self.replace_pending = false;
        self.command_buffer.clear();
    }

    pub fn clear_pending(&mut self) {
        self.pending = Input::default();
        self.replace_pending = false;
    }

    pub fn set_mode(&mut self, mode: VimMode) {
        self.mode = mode;
        self.pending = Input::default();
        self.replace_pending = false;
        if !matches!(
            mode,
            VimMode::Command | VimMode::SearchForward | VimMode::SearchBackward
        ) {
            self.command_buffer.clear();
        }
    }

    pub fn set_replace_pending(&mut self) {
        self.pending = Input::default();
        self.replace_pending = true;
        self.command_buffer.clear();
    }
}

// --- Vim Input Handler ---

/// Helper to update Vim state and textarea style based on transition
/// Returns (consumed, status_message_option)
fn update_vim_state(
    vim_state: &mut VimState,
    transition: VimTransition,
    textarea: &mut TextArea,
) -> (bool, Option<String>) {
    let old_mode = vim_state.mode;
    let status_msg;

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

/// Handles Vim-style input for the text editor.
///
/// This function processes keyboard input according to Vim's modal editing paradigm,
/// handling different modes (Normal, Insert, Visual, Command, Search) and their
/// respective commands. It manages cursor movement, text editing, mode transitions,
/// and search operations.
///
/// # Arguments
///
/// * `app` - A mutable reference to the application state
/// * `input` - The keyboard input to process
///
/// # Returns
///
/// Returns `true` if the input was consumed by the Vim handler, `false` if it should
/// be handled by the caller (e.g., Esc in Normal mode to exit the editor).
///
/// # Modes
///
/// * Normal - Default mode for navigation and commands
/// * Insert - For text insertion
/// * Visual - For text selection
/// * Command - For entering commands (e.g., :w, :q)
/// * Search - For forward/backward text search
///
/// # Features
///
/// * Cursor movement (h,j,k,l, arrows, word movements)
/// * Text editing (insert, delete, replace, join lines)
/// * Visual selection and operations
/// * Search and replace
/// * Command mode for line numbers and commands
/// * Clipboard operations
/// * Undo/redo
pub(super) fn handle_vim_input(app: &mut App, input: Input) -> bool {
    let textarea = &mut app.editor.textarea;
    let vim_state = &mut app.editor.vim_state;
    let current_mode = vim_state.mode;

    let is_esc_in_normal_mode =
        matches!(input, Input { key: Key::Esc, .. }) && current_mode == VimMode::Normal;
    if is_esc_in_normal_mode {
        return false; // Signal to caller that Esc in Normal should exit
    }

    // --- Handle Replace Pending State FIRST ---
    if vim_state.replace_pending {
        vim_state.replace_pending = false; // Consume the pending state
        match input {
            Input {
                key: Key::Char(c), ..
            } => {
                textarea.delete_next_char();
                textarea.insert_char(c);
                // insert_char moves cursor forward, move back to stay on the replaced char
                textarea.move_cursor(CursorMove::Back);
                // Stay in Normal mode
                return update_vim_state(
                    vim_state,
                    VimTransition::Mode(VimMode::Normal, None),
                    textarea,
                )
                .0; // Return only consumed bool
            }
            Input { key: Key::Esc, .. } => {
                // Cancel replace
                return update_vim_state(
                    vim_state,
                    VimTransition::Mode(VimMode::Normal, None),
                    textarea,
                )
                .0; // Return only consumed bool
            }
            _ => {
                // Invalid key after 'r', just cancel
                return update_vim_state(
                    vim_state,
                    VimTransition::Mode(VimMode::Normal, None),
                    textarea,
                )
                .0; // Return only consumed bool
            }
        }
    }

    let pending_input = vim_state.pending.clone();

    let transition = match current_mode {
        VimMode::Normal | VimMode::Visual | VimMode::Operator(_) => {
            let mut op_applied_transition = VimTransition::Nop(None);

            match input {
                // --- Movements ---
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

                Input {
                    key: Key::Char('0'),
                    ..
                } => {
                    let (row, _) = textarea.cursor();
                    textarea.move_cursor(CursorMove::Jump(row as u16, 0));
                }

                // --- Arrow Key Movements ---
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

                // --- Edits ---
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
                                Ok(_text) => {
                                    match vim_state.yank_register.yank_type {
                                        YankType::Linewise => {
                                            // Linewise paste: paste below current line
                                            textarea.move_cursor(CursorMove::End);
                                            textarea.insert_newline();
                                            textarea.paste();
                                            textarea.move_cursor(CursorMove::Up);
                                            textarea.move_cursor(CursorMove::Head);
                                        }
                                        YankType::Characterwise => {
                                            // Characterwise paste: paste after cursor
                                            textarea.move_cursor(CursorMove::Forward);
                                            textarea.paste();
                                        }
                                    }
                                }
                                Err(err) => {
                                    status_message = Some(format!("Clipboard error: {}", err));
                                }
                            }
                        }
                        Err(err) => {
                            status_message = Some(format!("Clipboard context error: {}", err));
                        }
                    }
                    if let Some(msg) = status_message {
                        op_applied_transition = VimTransition::Nop(Some(msg));
                    } else {
                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }
                }
                Input {
                    key: Key::Char('P'),
                    ..
                } => {
                    let mut status_message: Option<String> = None;
                    match Clipboard::new() {
                        Ok(mut clipboard) => {
                            match clipboard.get_text() {
                                Ok(_text) => {
                                    match vim_state.yank_register.yank_type {
                                        YankType::Linewise => {
                                            // Linewise paste: paste above current line
                                            textarea.move_cursor(CursorMove::Head);
                                            textarea.paste();
                                            textarea.insert_newline();
                                            textarea.move_cursor(CursorMove::Up);
                                            textarea.move_cursor(CursorMove::Head);
                                        }
                                        YankType::Characterwise => {
                                            // Characterwise paste: paste before cursor
                                            textarea.paste();
                                        }
                                    }
                                }
                                Err(err) => {
                                    status_message = Some(format!("Clipboard error: {}", err));
                                }
                            }
                        }
                        Err(err) => {
                            status_message = Some(format!("Clipboard context error: {}", err));
                        }
                    }
                    if let Some(msg) = status_message {
                        op_applied_transition = VimTransition::Nop(Some(msg));
                    } else {
                        op_applied_transition = VimTransition::Mode(VimMode::Normal, None);
                    }
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
                // Esc in Normal handled at the top level
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
                    // Handle yy, dd, cc - these are always linewise
                    let (start_row, _) = textarea.cursor();
                    textarea.move_cursor(CursorMove::Head); // Go to start of current line
                    textarea.start_selection();
                    textarea.move_cursor(CursorMove::Down); // Move to next line
                    let (end_row, _) = textarea.cursor();
                    if start_row == end_row {
                        // If cursor didn't move down (last line)
                        textarea.move_cursor(CursorMove::End); // Select to end of the last line
                    } else {
                        textarea.move_cursor(CursorMove::Head); // Select to start of the next line
                    }
                    // Mark as linewise for yy/dd/cc operations
                    vim_state.yank_register.yank_type = YankType::Linewise;
                    // The actual copy/cut happens in the operator logic below
                }
                Input {
                    key: Key::Char(op @ ('y' | 'd' | 'c')),
                    ctrl: false,
                    ..
                } if current_mode == VimMode::Normal => {
                    textarea.start_selection();
                    // Use update_vim_state to set mode and clear pending flags correctly
                    return update_vim_state(
                        vim_state,
                        VimTransition::Mode(VimMode::Operator(op), None),
                        textarea,
                    )
                    .0; // Return only consumed bool
                }
                Input {
                    key: Key::Char('y'),
                    ctrl: false,
                    ..
                } if current_mode == VimMode::Visual => {
                    vim_state.yank_register.yank_type = YankType::Characterwise;
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

                Input {
                    key: Key::Char('r'),
                    ctrl: false,
                    ..
                } if current_mode == VimMode::Normal => {
                    vim_state.set_replace_pending();
                    op_applied_transition = VimTransition::Nop(None); // Stay in normal mode, but waiting
                }

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
                        // delete_next_char removes the newline if cursor is at the end
                        textarea.delete_next_char();
                        // TODO: Optionally trim leading whitespace from the joined line
                    }
                    vim_state.clear_pending(); // Explicitly clear pending state here
                    op_applied_transition = VimTransition::Nop(None); // Stay in Normal mode
                }

                Input {
                    key: Key::Char(':'),
                    ..
                } if current_mode == VimMode::Normal => {
                    op_applied_transition = VimTransition::Mode(VimMode::Command, None);
                }

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

                Input {
                    key: Key::Char('n'),
                    ..
                } if current_mode == VimMode::Normal => {
                    if !textarea.search_forward(false) {
                        op_applied_transition =
                            VimTransition::Nop(Some("Pattern not found".to_string()));
                    } else {
                        op_applied_transition = VimTransition::Nop(None); // Stay in normal
                    }
                }
                Input {
                    key: Key::Char('N'),
                    ..
                } if current_mode == VimMode::Normal => {
                    if !textarea.search_back(false) {
                        op_applied_transition =
                            VimTransition::Nop(Some("Pattern not found".to_string()));
                    } else {
                        op_applied_transition = VimTransition::Nop(None); // Stay in normal
                    }
                }

                // --- Fallback for Pending ---
                pending => {
                    // If it wasn't 'g' or 'r', set it as pending
                    // Don't overwrite replace_pending if it's active
                    if !vim_state.replace_pending {
                        op_applied_transition = VimTransition::Pending(pending);
                    } else {
                        // Invalid key during replace pending already handled above
                        op_applied_transition = VimTransition::Nop(None);
                    }
                }
            }
            // Apply pending operator logic AFTER movement/action
            if op_applied_transition != VimTransition::Nop(None) {
                // If a mode change or Nop(Some(...)) was already decided, use that
                op_applied_transition
            } else {
                // Otherwise, check if an operator was pending and apply it
                match current_mode {
                    VimMode::Operator('y') => {
                        // Type is already set in the operator handling above
                        textarea.copy();
                        VimTransition::Mode(VimMode::Normal, None)
                    }
                    VimMode::Operator('d') => {
                        vim_state.yank_register.yank_type = YankType::Characterwise;
                        textarea.cut();
                        VimTransition::Mode(VimMode::Normal, None)
                    }
                    VimMode::Operator('c') => {
                        vim_state.yank_register.yank_type = YankType::Characterwise;
                        textarea.cut();
                        VimTransition::Mode(VimMode::Insert, None)
                    }
                    // No operator was pending, or it was handled differently (like 'dd')
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
                VimTransition::Mode(VimMode::Insert, None) // Stay in Insert mode
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
                    let mut status_msg = None;

                    // Check for help commands FIRST
                    if command == "help" || command == "?" {
                        app.editor.is_help_popup_active = true;
                        status_msg = Some("Help opened (:help or :?).".to_string());
                        // Return to normal mode, clear command buffer in set_mode
                        VimTransition::Mode(VimMode::Normal, status_msg)
                    }
                    // If not help, try parsing as line number
                    else if let Ok(line_num) = command.parse::<u16>() {
                        if line_num > 0 && (line_num as usize) <= textarea.lines().len() {
                            // Valid line number (1-based)
                            textarea.move_cursor(CursorMove::Jump(line_num - 1, 0));
                        } else {
                            // Invalid line number (out of bounds)
                            status_msg = Some(format!("Invalid line number: {}", line_num));
                        }
                        // Return to normal mode after Jump or error
                        VimTransition::Mode(VimMode::Normal, status_msg)
                    } else {
                        // Failed to parse as number or other command
                        match command {
                            "q" | "quit" => {
                                /* TODO: Handle Quit command? Maybe VimTransition::Quit? */
                                status_msg = Some("Quit command not implemented yet".to_string())
                            }
                            "w" | "write" => {
                                /* TODO: Handle Write command? */
                                status_msg = Some("Write command not implemented yet".to_string())
                            }
                            _ => status_msg = Some(format!("Not an editor command: {}", command)),
                        }
                        // Always return to Normal mode after Enter for other commands
                        VimTransition::Mode(VimMode::Normal, status_msg)
                    }
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
                    // Ignore other keys
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
                    let mut status_msg = None;
                    match textarea.set_search_pattern(query) {
                        Ok(_) => {
                            let found = if is_forward {
                                textarea.search_forward(true)
                            } else {
                                textarea.search_back(true)
                            };
                            if !found {
                                status_msg = Some(format!("Pattern not found: {}", query));
                            }
                        }
                        Err(e) => {
                            status_msg = Some(format!("Invalid regex: {}", e));
                            textarea.set_search_pattern("").ok(); // Clear pattern on error
                        }
                    }
                    // Return to Normal mode after Enter, keeping pattern active
                    VimTransition::Mode(VimMode::Normal, status_msg)
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
    let (consumed, status_msg_opt) = update_vim_state(vim_state, transition, textarea);
    if let Some(msg) = status_msg_opt {
        app.set_status_message(msg);
    }
    consumed
}
