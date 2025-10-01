use crate::app::App;
use tui_textarea::{CursorMove, Input, Key};

use super::super::{Mode, ParseResult, execute_command, paste_after, paste_before};

pub fn handle_normal_mode(app: &mut App, input: Input) -> bool {
    // Check if we're waiting for a replace character
    if let Some(count) = app.editor.vim_state.replace_pending {
        return handle_replace_char(app, input, count);
    }

    match input {
        // Mode transitions
        Input {
            key: Key::Char('i'),
            ..
        } => {
            app.editor.textarea.cancel_selection();
            let count = get_pending_count(&mut app.editor.vim_state.parser);
            app.editor
                .vim_state
                .start_insert_mode(count, &mut app.editor.textarea);
        }
        Input {
            key: Key::Char('a'),
            ..
        } => {
            app.editor.textarea.cancel_selection();
            app.editor.textarea.move_cursor(CursorMove::Forward);
            let count = get_pending_count(&mut app.editor.vim_state.parser);
            app.editor
                .vim_state
                .start_insert_mode(count, &mut app.editor.textarea);
        }
        Input {
            key: Key::Char('A'),
            ..
        } => {
            app.editor.textarea.cancel_selection();
            app.editor.textarea.move_cursor(CursorMove::End);
            let count = get_pending_count(&mut app.editor.vim_state.parser);
            app.editor
                .vim_state
                .start_insert_mode(count, &mut app.editor.textarea);
        }
        Input {
            key: Key::Char('I'),
            ..
        } => {
            app.editor.textarea.cancel_selection();
            app.editor.textarea.move_cursor(CursorMove::Head);
            let count = get_pending_count(&mut app.editor.vim_state.parser);
            app.editor
                .vim_state
                .start_insert_mode(count, &mut app.editor.textarea);
        }
        Input {
            key: Key::Char('o'),
            ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::End);
            app.editor.textarea.insert_newline();
            let count = get_pending_count(&mut app.editor.vim_state.parser);
            app.editor
                .vim_state
                .start_insert_mode(count, &mut app.editor.textarea);
        }
        Input {
            key: Key::Char('O'),
            ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::Head);
            app.editor.textarea.insert_newline();
            app.editor.textarea.move_cursor(CursorMove::Up);
            let count = get_pending_count(&mut app.editor.vim_state.parser);
            app.editor
                .vim_state
                .start_insert_mode(count, &mut app.editor.textarea);
        }

        // Visual mode
        Input {
            key: Key::Char('v'),
            ctrl: false,
            ..
        } => {
            app.editor.textarea.start_selection();
            app.editor
                .vim_state
                .set_mode(Mode::Visual { line_wise: false }, &mut app.editor.textarea);
        }
        Input {
            key: Key::Char('V'),
            ctrl: false,
            ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::Head);
            app.editor.textarea.start_selection();
            app.editor.textarea.move_cursor(CursorMove::End);
            app.editor
                .vim_state
                .set_mode(Mode::Visual { line_wise: true }, &mut app.editor.textarea);
        }

        // Command mode
        Input {
            key: Key::Char(':'),
            ..
        } => {
            app.editor
                .vim_state
                .set_mode(Mode::Command, &mut app.editor.textarea);
        }

        // Search mode
        Input {
            key: Key::Char('/'),
            ..
        } => {
            app.editor
                .vim_state
                .set_mode(Mode::Search { forward: true }, &mut app.editor.textarea);
        }
        Input {
            key: Key::Char('?'),
            ..
        } => {
            app.editor
                .vim_state
                .set_mode(Mode::Search { forward: false }, &mut app.editor.textarea);
        }

        // Search navigation
        Input {
            key: Key::Char('n'),
            ..
        } => {
            if !app.editor.textarea.search_forward(false) {
                app.set_status_message("Pattern not found".to_string());
            }
        }
        Input {
            key: Key::Char('N'),
            ..
        } => {
            if !app.editor.textarea.search_back(false) {
                app.set_status_message("Pattern not found".to_string());
            }
        }

        // Paste operations
        Input {
            key: Key::Char('p'),
            ..
        } => match paste_after(app) {
            super::super::OperatorResult::Success(mode) => {
                app.editor
                    .vim_state
                    .set_mode(mode, &mut app.editor.textarea);
            }
            super::super::OperatorResult::Error(msg) => {
                app.set_status_message(msg);
            }
        },
        Input {
            key: Key::Char('P'),
            ..
        } => match paste_before(app) {
            super::super::OperatorResult::Success(mode) => {
                app.editor
                    .vim_state
                    .set_mode(mode, &mut app.editor.textarea);
            }
            super::super::OperatorResult::Error(msg) => {
                app.set_status_message(msg);
            }
        },

        // Undo/Redo
        Input {
            key: Key::Char('u'),
            ctrl: false,
            ..
        } => {
            app.editor.textarea.undo();
        }
        Input {
            key: Key::Char('r'),
            ctrl: true,
            ..
        } => {
            app.editor.textarea.redo();
        }
        Input {
            key: Key::Char('r'),
            ctrl: false,
            ..
        } => {
            // Replace command - wait for next character
            let count = get_pending_count(&mut app.editor.vim_state.parser);
            app.editor.vim_state.replace_pending = Some(count.unwrap_or(1));
        }

        // Line operations
        Input {
            key: Key::Char('x'),
            ..
        } => {
            handle_delete_char(app);
        }
        Input {
            key: Key::Char('J'),
            ..
        } => {
            handle_join_lines(app);
        }

        // Scrolling
        Input {
            key: Key::Char('e'),
            ctrl: true,
            ..
        } => {
            app.editor.textarea.scroll((1, 0));
        }
        Input {
            key: Key::Char('y'),
            ctrl: true,
            ..
        } => {
            app.editor.textarea.scroll((-1, 0));
        }
        Input {
            key: Key::Char('d'),
            ctrl: true,
            ..
        } => {
            app.editor
                .textarea
                .scroll(tui_textarea::Scrolling::HalfPageDown);
        }
        Input {
            key: Key::Char('u'),
            ctrl: true,
            ..
        } => {
            app.editor
                .textarea
                .scroll(tui_textarea::Scrolling::HalfPageUp);
        }
        Input {
            key: Key::Char('f'),
            ctrl: true,
            ..
        } => {
            app.editor
                .textarea
                .scroll(tui_textarea::Scrolling::PageDown);
        }
        Input {
            key: Key::Char('b'),
            ctrl: true,
            ..
        } => {
            app.editor.textarea.scroll(tui_textarea::Scrolling::PageUp);
        }

        // Arrow keys
        Input { key: Key::Left, .. } => {
            app.editor.textarea.move_cursor(CursorMove::Back);
        }
        Input {
            key: Key::Right, ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::Forward);
        }
        Input { key: Key::Up, .. } => {
            app.editor.textarea.move_cursor(CursorMove::Up);
        }
        Input { key: Key::Down, .. } => {
            app.editor.textarea.move_cursor(CursorMove::Down);
        }

        // Grammar-based commands
        Input {
            key: Key::Char(c), ..
        } => {
            let result = app.editor.vim_state.parser.push_key(c);
            match result {
                ParseResult::Complete(command) => {
                    match execute_command(app, command) {
                        super::super::OperatorResult::Success(mode) => {
                            app.editor
                                .vim_state
                                .set_mode(mode, &mut app.editor.textarea);
                        }
                        super::super::OperatorResult::Error(msg) => {
                            app.set_status_message(msg);
                            app.editor
                                .vim_state
                                .set_mode(Mode::Normal, &mut app.editor.textarea);
                        }
                    }
                    app.editor.vim_state.parser.reset();
                }
                ParseResult::Incomplete => {
                    app.editor
                        .vim_state
                        .set_mode(Mode::OperatorPending, &mut app.editor.textarea);
                }
                ParseResult::Invalid => {
                    app.editor.vim_state.parser.reset();
                    app.editor
                        .vim_state
                        .set_mode(Mode::Normal, &mut app.editor.textarea);
                }
            }
        }

        _ => {}
    }

    true
}

pub fn handle_operator_pending(app: &mut App, input: Input) -> bool {
    match input {
        Input { key: Key::Esc, .. } => {
            app.editor.vim_state.parser.reset();
            app.editor
                .vim_state
                .set_mode(Mode::Normal, &mut app.editor.textarea);
        }

        Input {
            key: Key::Char(c), ..
        } => {
            let result = app.editor.vim_state.parser.push_key(c);
            match result {
                ParseResult::Complete(command) => {
                    match execute_command(app, command) {
                        super::super::OperatorResult::Success(mode) => {
                            app.editor
                                .vim_state
                                .set_mode(mode, &mut app.editor.textarea);
                        }
                        super::super::OperatorResult::Error(msg) => {
                            app.set_status_message(msg);
                            app.editor
                                .vim_state
                                .set_mode(Mode::Normal, &mut app.editor.textarea);
                        }
                    }
                    app.editor.vim_state.parser.reset();
                }
                ParseResult::Incomplete => {
                    // Stay in operator pending
                }
                ParseResult::Invalid => {
                    app.editor.vim_state.parser.reset();
                    app.editor
                        .vim_state
                        .set_mode(Mode::Normal, &mut app.editor.textarea);
                }
            }
        }

        _ => {}
    }

    true
}

fn handle_delete_char(app: &mut App) {
    let (row, col) = app.editor.textarea.cursor();
    let lines = app.editor.textarea.lines();

    if let Some(line) = lines.get(row) {
        let chars: Vec<char> = line.chars().collect();
        if col < chars.len() {
            app.editor.textarea.delete_next_char();

            // If we deleted the last character, move cursor back
            if col == chars.len() - 1 {
                app.editor.textarea.move_cursor(CursorMove::Back);
            }
        }
    }
}

fn handle_join_lines(app: &mut App) {
    let (row, _) = app.editor.textarea.cursor();
    let lines = app.editor.textarea.lines();

    if row < lines.len() - 1 {
        app.editor.textarea.move_cursor(CursorMove::End);
        app.editor.textarea.delete_next_char(); // Remove newline
        app.editor.textarea.insert_char(' '); // Add space
    }
}

fn get_pending_count(parser: &mut super::super::CommandParser) -> Option<u32> {
    parser.extract_count()
}

fn handle_replace_char(app: &mut App, input: Input, count: u32) -> bool {
    match input {
        Input { key: Key::Esc, .. } => {
            // Cancel replace operation
            app.editor.vim_state.replace_pending = None;
        }
        Input {
            key: Key::Char(c), ..
        } => {
            // Perform the replace operation
            app.editor.vim_state.replace_pending = None;

            let (row, col) = app.editor.textarea.cursor();
            let lines = app.editor.textarea.lines();

            if let Some(line) = lines.get(row) {
                let chars: Vec<char> = line.chars().collect();
                let available_chars = chars.len().saturating_sub(col);
                let replace_count = (count as usize).min(available_chars);

                if replace_count > 0 {
                    // Delete the characters to be replaced
                    for _ in 0..replace_count {
                        app.editor.textarea.delete_next_char();
                    }

                    // Insert the replacement characters
                    for _ in 0..replace_count {
                        app.editor.textarea.insert_char(c);
                    }

                    // Move cursor back to the original position
                    app.editor
                        .textarea
                        .move_cursor(CursorMove::Jump(row as u16, col as u16));
                }
            }
        }
        _ => {
            // Invalid key for replace - cancel the operation
            app.editor.vim_state.replace_pending = None;
        }
    }

    true
}
