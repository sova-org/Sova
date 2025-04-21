use crate::App;
use crate::{
    components::Component,
    components::logs::LogLevel,
    app::EditorKeymapMode,
};
use color_eyre::Result as EyreResult;
use bubocorelib::server::client::ClientMessage;
use bubocorelib::schedule::ActionTiming;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Rect, Modifier},
    style::{Color, Style},

    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, BorderType},
};
use std::{cmp::min, fmt};
use tui_textarea::{TextArea, Input, Key, CursorMove, Scrolling};

// --- Vim Mode Definitions ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Operator(char),
}

impl VimMode {
    // Helper to get a title string for the block
    fn title_string(&self) -> String {
        match self {
            Self::Normal => "NORMAL".to_string(),
            Self::Insert => "INSERT".to_string(),
            Self::Visual => "VISUAL".to_string(),
            Self::Operator(c) => format!("OPERATOR({})", c),
        }
    }

    // Helper to get cursor style (copied from example)
    fn cursor_style(&self) -> Style {
        let color = match self {
            Self::Normal => Color::Reset,
            Self::Insert => Color::LightBlue,
            Self::Visual => Color::LightYellow,
            Self::Operator(_) => Color::LightGreen,
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
    Nop, // No operation / state change
    Mode(VimMode), // Switch to a new mode
    Pending(Input), // Waiting for the next key (e.g., after 'g')
    // Quit is handled by the main editor Esc logic now
}

// State of Vim emulation
#[derive(Debug, Clone)]
pub struct VimState {
    pub mode: VimMode,
    pending: Input, // For multi-key sequences like 'gg'
    replace_pending: bool, // Flag for 'r' command
}

impl VimState {
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            pending: Input::default(),
            replace_pending: false, // Initialize flag
        }
    }

    // Helper to update state with pending input
    fn set_pending(&mut self, pending: Input) {
        self.pending = pending;
        self.replace_pending = false; // Clear replace flag if setting other pending input
    }

    // Helper to reset pending input
    fn clear_pending(&mut self) {
        self.pending = Input::default();
        self.replace_pending = false; // Also clear replace flag
    }

    // Helper to set Vim mode
    fn set_mode(&mut self, mode: VimMode) {
        self.mode = mode;
        self.pending = Input::default();
        self.replace_pending = false; // Clear flags on mode change
    }

     // Helper to enter replace pending state
     fn set_replace_pending(&mut self) {
         self.pending = Input::default(); // Clear other pending
         self.replace_pending = true;
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
                .title(" Search Query (Esc: Cancel, Enter: Find, ^N/↓: Next, ^P/↑: Prev) ")
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
    fn handle_vim_input(
        &mut self,
        app: &mut App,
        input: Input,
    ) -> bool {
        let textarea = &mut app.editor.textarea;
        let vim_state = &mut app.editor.vim_state;

        // --- Handle Replace Pending State FIRST ---
        if vim_state.replace_pending {
            vim_state.replace_pending = false; // Consume the pending state
            match input {
                Input { key: Key::Char(c), .. } => {
                    textarea.delete_next_char(); // Delete char under cursor
                    textarea.insert_char(c);     // Insert the new char
                    // insert_char moves cursor forward, move back to stay on the replaced char
                    textarea.move_cursor(CursorMove::Back);
                    // Stay in Normal mode
                    return self.update_vim_state(vim_state, VimTransition::Mode(VimMode::Normal), textarea);
                }
                 Input { key: Key::Esc, .. } => {
                     // Cancel replace, do nothing else
                     return self.update_vim_state(vim_state, VimTransition::Mode(VimMode::Normal), textarea);
                 }
                _ => {
                    // Invalid key after 'r', just cancel and go back to normal
                     return self.update_vim_state(vim_state, VimTransition::Mode(VimMode::Normal), textarea);
                }
            }
        }
        // --- End Replace Pending Handling ---

        let current_mode = vim_state.mode;
        let pending_input = vim_state.pending.clone();

        let transition = match current_mode {
            VimMode::Normal | VimMode::Visual | VimMode::Operator(_) => {
                let mut op_applied_transition = VimTransition::Nop;

                match input {
                    // --- Existing Movements ---
                    Input { key: Key::Char('h'), .. } => { textarea.move_cursor(CursorMove::Back); }
                    Input { key: Key::Char('j'), .. } => { textarea.move_cursor(CursorMove::Down); }
                    Input { key: Key::Char('k'), .. } => { textarea.move_cursor(CursorMove::Up); }
                    Input { key: Key::Char('l'), .. } => { textarea.move_cursor(CursorMove::Forward); }
                    Input { key: Key::Char('w'), .. } => { textarea.move_cursor(CursorMove::WordForward); }
                    Input { key: Key::Char('e'), ctrl: false, .. } => {
                        textarea.move_cursor(CursorMove::WordEnd);
                        if matches!(current_mode, VimMode::Operator(_)) {
                            textarea.move_cursor(CursorMove::Forward);
                        }
                    }
                    Input { key: Key::Char('b'), ctrl: false, .. } => { textarea.move_cursor(CursorMove::WordBack); }
                    Input { key: Key::Char('^'), .. } => { textarea.move_cursor(CursorMove::Head); } // To first non-whitespace
                    Input { key: Key::Char('$'), .. } => { textarea.move_cursor(CursorMove::End); } // To end of line

                    // --- NEW '0' Movement ---
                    Input { key: Key::Char('0'), .. } => {
                        let (row, _) = textarea.cursor(); // Get current row
                        textarea.move_cursor(CursorMove::Jump(row as u16, 0)); // Jump to column 0, casting row
                    }

                    // --- NEW Arrow Key Movements ---
                    Input { key: Key::Left, .. } => { textarea.move_cursor(CursorMove::Back); }
                    Input { key: Key::Right, .. } => { textarea.move_cursor(CursorMove::Forward); }
                    Input { key: Key::Up, .. } => { textarea.move_cursor(CursorMove::Up); }
                    Input { key: Key::Down, .. } => { textarea.move_cursor(CursorMove::Down); }

                    // --- Existing Edits ---
                    Input { key: Key::Char('D'), .. } => {
                        textarea.delete_line_by_end();
                        op_applied_transition = VimTransition::Mode(VimMode::Normal);
                    }
                     Input { key: Key::Char('C'), .. } => { textarea.delete_line_by_end(); textarea.cancel_selection(); op_applied_transition = VimTransition::Mode(VimMode::Insert); }
                     Input { key: Key::Char('p'), .. } => { textarea.paste(); op_applied_transition = VimTransition::Mode(VimMode::Normal); }
                     Input { key: Key::Char('u'), ctrl: false, .. } => { textarea.undo(); op_applied_transition = VimTransition::Mode(VimMode::Normal); }
                     Input { key: Key::Char('r'), ctrl: true, .. } => { textarea.redo(); op_applied_transition = VimTransition::Mode(VimMode::Normal); }
                     Input { key: Key::Char('x'), .. } => {
                        let (row, col) = textarea.cursor();
                        let lines = textarea.lines();
                        let num_lines = lines.len();

                        // Determine if the cursor is exactly on the last character of the buffer
                        let is_on_last_char = if num_lines > 0 {
                            let last_line_idx = num_lines - 1;
                            if row == last_line_idx {
                                let last_line_len = lines.get(last_line_idx).map_or(0, |s| s.chars().count());
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

                        op_applied_transition = VimTransition::Mode(VimMode::Normal);
                     }

                    // --- Mode Changes ---
                    Input { key: Key::Char('i'), .. } => { textarea.cancel_selection(); op_applied_transition = VimTransition::Mode(VimMode::Insert); }
                     Input { key: Key::Char('a'), .. } => { textarea.cancel_selection(); textarea.move_cursor(CursorMove::Forward); op_applied_transition = VimTransition::Mode(VimMode::Insert); }
                     Input { key: Key::Char('A'), .. } => { textarea.cancel_selection(); textarea.move_cursor(CursorMove::End); op_applied_transition = VimTransition::Mode(VimMode::Insert); }
                     Input { key: Key::Char('o'), .. } => { textarea.move_cursor(CursorMove::End); textarea.insert_newline(); op_applied_transition = VimTransition::Mode(VimMode::Insert); }
                     Input { key: Key::Char('O'), .. } => { textarea.move_cursor(CursorMove::Head); textarea.insert_newline(); textarea.move_cursor(CursorMove::Up); op_applied_transition = VimTransition::Mode(VimMode::Insert); }
                     Input { key: Key::Char('I'), .. } => { textarea.cancel_selection(); textarea.move_cursor(CursorMove::Head); op_applied_transition = VimTransition::Mode(VimMode::Insert); }

                    // --- Scrolling ---
                    Input { key: Key::Char('e'), ctrl: true, .. } => { textarea.scroll((1, 0)); }
                     Input { key: Key::Char('y'), ctrl: true, .. } => { textarea.scroll((-1, 0)); }
                     Input { key: Key::Char('d'), ctrl: true, .. } => { textarea.scroll(Scrolling::HalfPageDown); }
                     Input { key: Key::Char('u'), ctrl: true, .. } => { textarea.scroll(Scrolling::HalfPageUp); }
                     Input { key: Key::Char('f'), ctrl: true, .. } => { textarea.scroll(Scrolling::PageDown); }
                     Input { key: Key::Char('b'), ctrl: true, .. } => { textarea.scroll(Scrolling::PageUp); }

                    // --- Visual Mode Transitions ---
                    Input { key: Key::Char('v'), ctrl: false, .. } if current_mode == VimMode::Normal => { textarea.start_selection(); op_applied_transition = VimTransition::Mode(VimMode::Visual); }
                    Input { key: Key::Char('V'), ctrl: false, .. } if current_mode == VimMode::Normal => { textarea.move_cursor(CursorMove::Head); textarea.start_selection(); textarea.move_cursor(CursorMove::End); op_applied_transition = VimTransition::Mode(VimMode::Visual); }

                    // --- Esc Handling ---
                    Input { key: Key::Esc, .. } if current_mode == VimMode::Normal => { op_applied_transition = VimTransition::Nop; }
                    Input { key: Key::Esc, .. } | Input { key: Key::Char('v'), ctrl: false, .. }
                       if matches!(current_mode, VimMode::Visual | VimMode::Operator(_)) =>
                    { textarea.cancel_selection(); op_applied_transition = VimTransition::Mode(VimMode::Normal); }

                    // --- Pending sequences (gg, operators) ---
                    Input { key: Key::Char('g'), ctrl: false, .. } if matches!(pending_input, Input { key: Key::Char('g'), .. }) => { textarea.move_cursor(CursorMove::Top); }
                    Input { key: Key::Char('G'), ctrl: false, .. } => { textarea.move_cursor(CursorMove::Bottom); }
                    Input { key: Key::Char(c), ctrl: false, .. } if current_mode == VimMode::Operator(c) => { /* Handle yy, dd, cc */ textarea.move_cursor(CursorMove::Head); textarea.start_selection(); let cursor = textarea.cursor(); textarea.move_cursor(CursorMove::Down); if cursor.0 == textarea.cursor().0 { textarea.move_cursor(CursorMove::End); } else { textarea.move_cursor(CursorMove::Up); textarea.move_cursor(CursorMove::End); } }
                    Input { key: Key::Char(op @ ('y' | 'd' | 'c')), ctrl: false, .. } if current_mode == VimMode::Normal => {
                        textarea.start_selection();
                        // Use update_vim_state to set mode and clear pending flags correctly
                        return self.update_vim_state(vim_state, VimTransition::Mode(VimMode::Operator(op)), textarea);
                    }
                    Input { key: Key::Char('y'), ctrl: false, .. } if current_mode == VimMode::Visual => { textarea.copy(); op_applied_transition = VimTransition::Mode(VimMode::Normal); }
                    Input { key: Key::Char('d'), ctrl: false, .. } if current_mode == VimMode::Visual => { textarea.cut(); op_applied_transition = VimTransition::Mode(VimMode::Normal); }
                    Input { key: Key::Char('c'), ctrl: false, .. } if current_mode == VimMode::Visual => { textarea.cut(); op_applied_transition = VimTransition::Mode(VimMode::Insert); }

                    // --- NEW 'r' command ---
                    Input { key: Key::Char('r'), ctrl: false, .. } if current_mode == VimMode::Normal => {
                        vim_state.set_replace_pending();
                        op_applied_transition = VimTransition::Nop; // Stay in normal mode, but waiting
                    }

                    // --- NEW 'J' command ---
                    Input { key: Key::Char('J'), ctrl: false, .. } if current_mode == VimMode::Normal => {
                         let (row, _) = textarea.cursor();
                         if row < textarea.lines().len() - 1 { // Check if not the last line
                             textarea.move_cursor(CursorMove::End);
                             textarea.insert_char(' ');
                             // delete_next_char should remove the newline if cursor is at the end
                             textarea.delete_next_char();
                             // We might want to trim leading whitespace from the joined line later
                         }
                         op_applied_transition = VimTransition::Nop; // Stay in Normal mode
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
                             op_applied_transition = VimTransition::Nop;
                        }
                    }
                }
                 // Apply pending operator logic
                 if op_applied_transition != VimTransition::Nop { op_applied_transition }
                 else { match current_mode { VimMode::Operator('y') => {textarea.copy(); VimTransition::Mode(VimMode::Normal) } VimMode::Operator('d') => {textarea.cut(); VimTransition::Mode(VimMode::Normal) } VimMode::Operator('c') => {textarea.cut(); VimTransition::Mode(VimMode::Insert) } _ => VimTransition::Nop, } }
            }
            VimMode::Insert => {
                  match input {
                      Input { key: Key::Esc, .. } | Input { key: Key::Char('c'), ctrl: true, .. } => { textarea.move_cursor(CursorMove::Back); VimTransition::Mode(VimMode::Normal) }
                      _ => { textarea.input(input); VimTransition::Mode(VimMode::Insert) }
                  }
            }
        };

        self.update_vim_state(vim_state, transition, textarea)
    }

    // Helper to update Vim state and textarea style based on transition
    fn update_vim_state(&self, vim_state: &mut VimState, transition: VimTransition, textarea: &mut TextArea) -> bool {
        let old_mode = vim_state.mode;
        match transition {
            VimTransition::Mode(new_mode) => {
                vim_state.set_mode(new_mode);
                if old_mode != new_mode {
                    textarea.set_cursor_style(new_mode.cursor_style());
                }
                true // Consumed
            }
            VimTransition::Pending(pending_input) => {
                 vim_state.set_pending(pending_input);
                 true // Consumed (waiting for next)
            }
            VimTransition::Nop => {
                 vim_state.clear_pending();
                 true // Consumed (action performed, no mode change)
            }
        }
    }

    // --- Normal (Emacs-like) Input Handler ---
    fn handle_normal_input(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> bool { // Returns true if input was consumed
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
            KeyCode::Backspace => { // Or Ctrl+H? Need to decide
                textarea.delete_char();
                return true;
            }
            KeyCode::Delete => {
                textarea.delete_next_char();
                 return true;
            }
             // Cut/Copy/Paste (Example using Alt keys, adjust as needed)
            KeyCode::Char('w') if key_event.modifiers == KeyModifiers::ALT => { // Alt+W for copy (like kill-ring-save)
                // Need selection first, TBD how to handle Emacs selection
                // textarea.copy();
                 app.set_status_message("Copy (Alt+W) - requires selection (TBD)".to_string());
                 return true;
            }
            KeyCode::Char('k') if key_event.modifiers == KeyModifiers::CONTROL => { // Ctrl+K for kill-line
                textarea.delete_line_by_end();
                // TODO: Add to a conceptual kill ring?
                app.set_status_message("Kill line (Ctrl+K)".to_string());
                return true;
            }
            KeyCode::Char('y') if key_event.modifiers == KeyModifiers::CONTROL => { // Ctrl+Y for yank (paste)
                textarea.paste();
                app.set_status_message("Yank (Ctrl+Y)".to_string());
                return true;
            }

            // Undo/Redo (Simple)
             KeyCode::Char('/') if key_event.modifiers == KeyModifiers::CONTROL => { // Ctrl+/ for undo
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
}


impl Component for EditorComponent {

    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {

        // --- Priority Handling (Search, Global Actions) ---

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
                    tui_textarea::TextArea::set_search_pattern(main_textarea, "").expect("Empty pattern should be valid");
                    search_state.query_textarea.move_cursor(CursorMove::End);
                    search_state.query_textarea.delete_line_by_head();
                    app.set_status_message("Search cancelled.".to_string());
                    return Ok(true);
                }
                Input { key: Key::Enter, .. } => {
                    if !tui_textarea::TextArea::search_forward(main_textarea, true) {
                        search_state.error_message = Some("Pattern not found".to_string());
                    } else {
                        search_state.error_message = None;
                    }
                    search_state.is_active = false;
                    search_state.query_textarea.move_cursor(CursorMove::End);
                    search_state.query_textarea.delete_line_by_head();
                    app.set_status_message("Search closed.".to_string());
                    return Ok(true);
                }
                 Input { key: Key::Char('n'), ctrl: true, .. } | Input { key: Key::Down, .. } => {
                     if !tui_textarea::TextArea::search_forward(main_textarea, false) {
                         search_state.error_message = Some("Pattern not found".to_string());
                     } else {
                         search_state.error_message = None;
                     }
                     return Ok(true);
                 }
                 Input { key: Key::Char('p'), ctrl: true, .. } | Input { key: Key::Up, .. } => {
                     if !tui_textarea::TextArea::search_back(main_textarea, false) {
                         search_state.error_message = Some("Pattern not found".to_string());
                     } else {
                         search_state.error_message = None;
                     }
                     return Ok(true);
                 }
                 input => {
                     // Prevent Enter/Ctrl+M from adding newline in search box
                     if matches!(input, Input { key: Key::Enter, .. } | Input { key: Key::Char('m'), ctrl: true, ..}) {
                          return Ok(true);
                     }
                     let modified = search_state.query_textarea.input(input);
                     if modified {
                         let query = search_state.query_textarea.lines().get(0).map_or("", |s| s.as_str());
                         match tui_textarea::TextArea::set_search_pattern(main_textarea, query) {
                             Ok(_) => search_state.error_message = None,
                             Err(e) => search_state.error_message = Some(e.to_string()),
                         }
                     }
                     return Ok(true);
                 }
             }
            // End Search Input Handling
        } // End Search Mode block

        // 2. Handle Editor Exit (Esc) - ONLY if not searching
        if key_event.code == KeyCode::Esc && app.settings.editor_keymap_mode == EditorKeymapMode::Normal {
            // In Vim mode, Esc is handled by the Vim input handler to switch modes.
            // It should only exit the editor if pressed while already in Vim Normal mode.
            // This will be handled inside handle_vim_input later if needed.
            app.send_client_message(ClientMessage::StoppedEditingFrame(
                app.editor.active_line.line_index,
                app.editor.active_line.frame_index
            ));
            app.editor.compilation_error = None;
            app.events.sender.send(crate::event::Event::App(crate::event::AppEvent::SwitchToGrid))?;
            app.set_status_message("Exited editor (Esc).".to_string());
            return Ok(true);
        }
        // Note: Vim Esc handling is inside handle_vim_input

        // 3. Handle Global Editor Actions (Ctrl+S, Ctrl+G, Ctrl+E, Ctrl+Arrows)
        // These should work regardless of Normal/Vim mode (unless Vim mode rebinds them, which we avoid here)
        if key_event.modifiers == KeyModifiers::CONTROL {
            match key_event.code {
                // --- Send Script ---
                KeyCode::Char('s') => {
                    app.add_log(LogLevel::Debug, "Ctrl+S detected, attempting to send script...".to_string());
                    app.send_client_message(ClientMessage::SetScript(
                        app.editor.active_line.line_index,
                        app.editor.active_line.frame_index,
                        app.editor.textarea.lines().join("\n"),
                        ActionTiming::Immediate
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
                    // Reset Vim mode to Normal if activating search from Vim mode? Optional.
                    // if app.settings.editor_keymap_mode == EditorKeymapMode::Vim {
                    //    self.set_vim_mode(app, VimMode::Normal);
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
                                     ClientMessage::DisableFrames(line_idx, vec![frame_idx], ActionTiming::Immediate)
                                 } else {
                                     ClientMessage::EnableFrames(line_idx, vec![frame_idx], ActionTiming::Immediate)
                                 };
                                 app.send_client_message(message);
                                 app.set_status_message(format!(
                                     "Toggled Frame {}/{} to {}",
                                     line_idx, frame_idx, if !current_enabled_status { "Enabled" } else { "Disabled" }
                                 ));
                             } else { app.set_status_message("Cannot toggle: Invalid frame index.".to_string()); }
                         } else { app.set_status_message("Cannot toggle: Invalid line index.".to_string()); }
                     } else { app.set_status_message("Cannot toggle: scene not loaded.".to_string()); }
                     return Ok(true);
                 }
                 // --- Navigate Script ---
                 KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                      if let Some(scene) = &app.editor.scene {
                          let current_line_idx = app.editor.active_line.line_index;
                          let current_frame_idx = app.editor.active_line.frame_index;
                          let num_lines = scene.lines.len();
                          if num_lines == 0 { app.set_status_message("No lines to navigate.".to_string()); return Ok(true); }
                          match key_event.code {
                              KeyCode::Up => {
                                  if current_frame_idx == 0 { app.set_status_message("Already at first frame.".to_string()); return Ok(true); }
                                  let target_line_idx = current_line_idx; let target_frame_idx = current_frame_idx - 1;
                                  app.editor.compilation_error = None;
                                  app.send_client_message(ClientMessage::GetScript(target_line_idx, target_frame_idx));
                                  app.set_status_message(format!("Requested script Line {}, Frame {}", target_line_idx, target_frame_idx));
                              }
                              KeyCode::Down => {
                                  if let Some(line) = scene.lines.get(current_line_idx) {
                                      if current_frame_idx + 1 >= line.frames.len() { app.set_status_message("Already at last frame.".to_string()); return Ok(true); }
                                      let target_line_idx = current_line_idx; let target_frame_idx = current_frame_idx + 1;
                                      app.editor.compilation_error = None;
                                      app.send_client_message(ClientMessage::GetScript(target_line_idx, target_frame_idx));
                                      app.set_status_message(format!("Requested script Line {}, Frame {}", target_line_idx, target_frame_idx));
                                  } else { return Ok(true); }
                              }
                              KeyCode::Left => {
                                  if current_line_idx == 0 { app.set_status_message("Already at first line.".to_string()); return Ok(true); }
                                  let target_line_idx = current_line_idx - 1;
                                  let target_line_len = scene.lines[target_line_idx].frames.len();
                                  if target_line_len == 0 { app.set_status_message(format!("Line {} is empty.", target_line_idx)); return Ok(true); }
                                  let target_frame_idx = min(current_frame_idx, target_line_len - 1);
                                  app.editor.compilation_error = None;
                                  app.send_client_message(ClientMessage::GetScript(target_line_idx, target_frame_idx));
                                  app.set_status_message(format!("Requested script Line {}, Frame {}", target_line_idx, target_frame_idx));
                              }
                              KeyCode::Right => {
                                  if current_line_idx + 1 >= num_lines { app.set_status_message("Already at last line.".to_string()); return Ok(true); }
                                  let target_line_idx = current_line_idx + 1;
                                  let target_line_len = scene.lines[target_line_idx].frames.len();
                                  if target_line_len == 0 { app.set_status_message(format!("Line {} is empty.", target_line_idx)); return Ok(true); }
                                  let target_frame_idx = min(current_frame_idx, target_line_len - 1);
                                  app.editor.compilation_error = None;
                                  app.send_client_message(ClientMessage::GetScript(target_line_idx, target_frame_idx));
                                  app.set_status_message(format!("Requested script Line {}, Frame {}", target_line_idx, target_frame_idx));
                              }
                              _ => unreachable!(),
                          }
                          return Ok(true);
                      } else { app.set_status_message("scene not loaded, cannot navigate.".to_string()); return Ok(true); }
                  } // End Ctrl + Arrow case
                  // --- Fallthrough for other Ctrl keys ---
                  _ => {}
              }
          } // End Ctrl modifier check

          // --- Mode-Specific Input Handling ---
          let handled = match app.settings.editor_keymap_mode {
              EditorKeymapMode::Vim => {
                  let input: Input = key_event.into();
                  // Delegate ALL keys (including Esc) to Vim handler when Vim mode is active
                  self.handle_vim_input(app, input)
              }
              EditorKeymapMode::Normal => {
                  // Delegate to Normal (Emacs-like) handler
                  self.handle_normal_input(app, key_event)
              }
          };

          Ok(handled)
    }


    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let line_idx = app.editor.active_line.line_index;
        let frame_idx = app.editor.active_line.frame_index;

        // Get frame status and length, with default values if not found
        let (status_str, length_str, is_enabled) = 
            if let Some(scene) = &app.editor.scene {
                if let Some(line) = scene.lines.get(line_idx) {
                    if frame_idx < line.frames.len() {
                        let enabled = line.is_frame_enabled(frame_idx);
                        let length = line.frames[frame_idx];
                        ( if enabled { "Enabled" } else { "Disabled" },
                          format!("Len: {:.2}", length),
                          enabled
                        )
                    } else {
                        ("Invalid Frame", "Len: N/A".to_string(), true) // Default to enabled appearance if invalid
                    }
                } else {
                    ("Invalid Line", "Len: N/A".to_string(), true) // Default to enabled appearance if invalid
                }
            } else {
                ("No scene", "Len: N/A".to_string(), true) // Default to enabled appearance if no scene
            };

        // Determine border color based on frame status
        let border_color = if is_enabled { Color::White } else { Color::DarkGray };

        // --- Adjust Title based on Vim Mode ---
        let vim_mode_indicator = if app.settings.editor_keymap_mode == EditorKeymapMode::Vim {
            format!(" [{}]", app.editor.vim_state.mode.title_string()) // Use helper
        } else {
            String::new() // No indicator for Normal mode
        };

        let editor_block = Block::default()
            .title(format!(
                " Editor (Line: {}, Frame: {} | {} | {}){} ", // Add vim_mode_indicator here
                line_idx,
                frame_idx,
                status_str, // Show enabled/disabled status
                length_str,  // Show length
                vim_mode_indicator
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(border_color));

        frame.render_widget(editor_block.clone(), area);
        let inner_editor_area = editor_block.inner(area);

        // Layout Definition 
        let editor_text_area: Rect; 
        let help_area: Rect;

        let search_active = app.editor.search_state.is_active;
        let compilation_error_present = app.editor.compilation_error.is_some();

        // Define constraints based on active panels
        let mut constraints = vec![Constraint::Min(0)]; // Editor content always present
        if search_active {
            constraints.push(Constraint::Length(3)); // Search box takes priority
        } else if compilation_error_present {
            constraints.push(Constraint::Length(5)); // Error panel if search not active
        }
        constraints.push(Constraint::Length(1)); // Help text always present

        let editor_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner_editor_area);

        // Assign areas based on layout
        editor_text_area = editor_chunks[0];
        let mut current_index = 1;
        if search_active || compilation_error_present { 
            let panel_area = editor_chunks[current_index];
            current_index += 1;

            if search_active {
                // --- Render Search Box --- 
                let search_state = &app.editor.search_state;
                let mut query_textarea = search_state.query_textarea.clone(); 
                if let Some(err_msg) = &search_state.error_message {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(format!(
                            " Search Query (Error: {}) (Esc: Cancel, Enter: Find, ^N/↓: Next, ^P/↑: Prev) ",
                            err_msg
                        ))
                        .style(Style::default().fg(Color::Red));
                    query_textarea.set_block(block);
                } // No need for else, default block is set in SearchState::new

                frame.render_widget(&query_textarea, panel_area); 
            } else {
                // --- Render Compilation Error Panel (only if search is not active) ---
                if let Some(error_msg) = &app.editor.compilation_error {
                    // Calculate line and column from character index
                    let mut error_line_num = 0;
                    let mut error_col_num = 0;
                    let mut char_idx_count = 0;
                    let editor_lines = app.editor.textarea.lines();

                    for (i, line) in editor_lines.iter().enumerate() {
                        let line_char_count = line.chars().count();
                        // Check if the 'from' index falls within this line (char indices)
                        if error_msg.from >= char_idx_count && error_msg.from < char_idx_count + line_char_count {
                            error_line_num = i;
                            error_col_num = error_msg.from - char_idx_count;
                            break;
                        }
                        // Add line length + 1 (for newline char) to cumulative count
                        char_idx_count += line_char_count + 1;
                        // If error index is exactly after the last char + newline, it's start of next line
                        if error_msg.from == char_idx_count {
                            error_line_num = i + 1;
                            error_col_num = 0;
                            break;
                        }
                    }

                    let error_block = Block::default()
                        .title(format!(
                            " Compilation Error ({}: Line {}, Col {}) ",
                            error_msg.lang, error_line_num + 1, error_col_num + 1
                        ))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Plain)
                        .style(Style::default().fg(Color::Red));
                    let error_paragraph = Paragraph::new(error_msg.info.as_str())
                        .wrap(ratatui::widgets::Wrap { trim: true })
                        .block(error_block.clone());
                    frame.render_widget(error_paragraph, panel_area);
                    frame.render_widget(error_block, panel_area); // Render border over content
                }
            }
        }
        help_area = editor_chunks[current_index];
 
        let mut text_area = app.editor.textarea.clone();
        text_area.set_line_number_style(Style::default().fg(Color::DarkGray));

        // --- Render Main Editor --- 
        frame.render_widget(&text_area, editor_text_area);
 
        // Indication des touches
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);

        // --- Render Help Text --- 
        let help_line = if search_active {
            Line::from(vec![
                Span::styled(" Esc ", key_style), Span::styled("Cancel | ", help_style),
                Span::styled(" Enter ", key_style), Span::styled("Find First & Close | ", help_style),
                Span::styled(" ^N/↓ ", key_style), Span::styled("Next Match | ", help_style),
                Span::styled(" ^P/↑ ", key_style), Span::styled("Prev Match", help_style),
            ])
        } else {
            // Simplified help text - only show app-level bindings
            Line::from(vec![
                Span::styled("Ctrl+S", key_style), Span::styled(": Send | ", help_style),
                Span::styled("Ctrl+E", key_style), Span::styled(": Toggle | ", help_style),
                Span::styled("Ctrl+G", key_style), Span::styled(": Search | ", help_style),
                Span::styled("Ctrl+←↑↓→", key_style), Span::styled(": Navigate Script", help_style),
            ])
        };

        let help = Paragraph::new(help_line)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
