use crate::app::App;
use tui_textarea::{CursorMove, Input, Key};

use super::super::Mode;

pub fn handle_command_mode(app: &mut App, input: Input) -> bool {
    match input {
        Input { key: Key::Esc, .. } => {
            app.editor
                .vim_state
                .set_mode(Mode::Normal, &mut app.editor.textarea);
        }

        Input {
            key: Key::Enter, ..
        } => {
            let command = app.editor.vim_state.command_buffer.trim();
            let mut status_msg = None;

            if command == "help" || command == "?" {
                app.editor.is_help_popup_active = true;
                status_msg = Some("Help opened (:help or :?)".to_string());
            } else if let Ok(line_num) = command.parse::<u16>() {
                if line_num > 0 && (line_num as usize) <= app.editor.textarea.lines().len() {
                    app.editor
                        .textarea
                        .move_cursor(CursorMove::Jump(line_num - 1, 0));
                } else {
                    status_msg = Some(format!("Invalid line number: {}", line_num));
                }
            } else {
                match command {
                    "q" | "quit" => {
                        status_msg = Some("Quit command not implemented".to_string());
                    }
                    "w" | "write" => {
                        status_msg = Some("Write command not implemented".to_string());
                    }
                    _ => {
                        status_msg = Some(format!("Not an editor command: {}", command));
                    }
                }
            }

            if let Some(msg) = status_msg {
                app.set_status_message(msg);
            }

            app.editor
                .vim_state
                .set_mode(Mode::Normal, &mut app.editor.textarea);
        }

        Input {
            key: Key::Backspace,
            ..
        } => {
            app.editor.vim_state.command_buffer.pop();
        }

        Input {
            key: Key::Char(c), ..
        } => {
            app.editor.vim_state.command_buffer.push(c);
        }

        _ => {}
    }

    true
}
