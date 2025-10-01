use crate::app::App;
use tui_textarea::{Input, Key};

use super::super::Mode;

pub fn handle_search_mode(app: &mut App, input: Input) -> bool {
    let is_forward = matches!(app.editor.vim_state.mode, Mode::Search { forward: true });

    match input {
        Input { key: Key::Esc, .. } => {
            app.editor.vim_state.command_buffer.clear();
            let _ = app.editor.textarea.set_search_pattern("");
            app.editor
                .vim_state
                .set_mode(Mode::Normal, &mut app.editor.textarea);
        }

        Input {
            key: Key::Enter, ..
        } => {
            let query = &app.editor.vim_state.command_buffer;
            let mut status_msg = None;

            match app.editor.textarea.set_search_pattern(query) {
                Ok(_) => {
                    let found = if is_forward {
                        app.editor.textarea.search_forward(true)
                    } else {
                        app.editor.textarea.search_back(true)
                    };

                    if !found {
                        status_msg = Some(format!("Pattern not found: {}", query));
                    } else {
                        app.editor.vim_state.last_search = Some(query.clone());
                    }
                }
                Err(e) => {
                    status_msg = Some(format!("Invalid regex: {}", e));
                    let _ = app.editor.textarea.set_search_pattern("");
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
