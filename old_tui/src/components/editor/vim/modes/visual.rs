use crate::app::App;
use tui_textarea::{CursorMove, Input, Key};

use super::super::{Mode, YankType};

pub fn handle_visual_mode(app: &mut App, input: Input) -> bool {
    let is_line_wise = matches!(app.editor.vim_state.mode, Mode::Visual { line_wise: true });

    match input {
        Input { key: Key::Esc, .. }
        | Input {
            key: Key::Char('v'),
            ctrl: false,
            ..
        } => {
            app.editor.textarea.cancel_selection();
            app.editor
                .vim_state
                .set_mode(Mode::Normal, &mut app.editor.textarea);
        }

        // Operations
        Input {
            key: Key::Char('d'),
            ctrl: false,
            ..
        }
        | Input {
            key: Key::Char('x'),
            ctrl: false,
            ..
        } => {
            app.editor.vim_state.yank_register.yank_type = if is_line_wise {
                YankType::Linewise
            } else {
                YankType::Characterwise
            };
            app.editor.textarea.cut();
            copy_to_system_clipboard(&app.editor.textarea);
            app.editor
                .vim_state
                .set_mode(Mode::Normal, &mut app.editor.textarea);
        }

        Input {
            key: Key::Char('c'),
            ctrl: false,
            ..
        } => {
            app.editor.vim_state.yank_register.yank_type = if is_line_wise {
                YankType::Linewise
            } else {
                YankType::Characterwise
            };
            app.editor.textarea.cut();
            copy_to_system_clipboard(&app.editor.textarea);
            app.editor
                .vim_state
                .set_mode(Mode::Insert, &mut app.editor.textarea);
        }

        Input {
            key: Key::Char('y'),
            ctrl: false,
            ..
        } => {
            app.editor.vim_state.yank_register.yank_type = if is_line_wise {
                YankType::Linewise
            } else {
                YankType::Characterwise
            };
            app.editor.textarea.copy();
            copy_to_system_clipboard(&app.editor.textarea);
            app.editor.textarea.cancel_selection();
            app.editor
                .vim_state
                .set_mode(Mode::Normal, &mut app.editor.textarea);
        }

        // Movement in visual mode
        Input {
            key: Key::Char('h'),
            ..
        }
        | Input { key: Key::Left, .. } => {
            app.editor.textarea.move_cursor(CursorMove::Back);
        }
        Input {
            key: Key::Char('j'),
            ..
        }
        | Input { key: Key::Down, .. } => {
            app.editor.textarea.move_cursor(CursorMove::Down);
        }
        Input {
            key: Key::Char('k'),
            ..
        }
        | Input { key: Key::Up, .. } => {
            app.editor.textarea.move_cursor(CursorMove::Up);
        }
        Input {
            key: Key::Char('l'),
            ..
        }
        | Input {
            key: Key::Right, ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::Forward);
        }

        Input {
            key: Key::Char('w'),
            ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::WordForward);
        }
        Input {
            key: Key::Char('b'),
            ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::WordBack);
        }
        Input {
            key: Key::Char('e'),
            ctrl: false,
            ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::WordEnd);
        }

        Input {
            key: Key::Char('0'),
            ..
        } => {
            let (row, _) = app.editor.textarea.cursor();
            app.editor
                .textarea
                .move_cursor(CursorMove::Jump(row as u16, 0));
        }
        Input {
            key: Key::Char('$'),
            ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::End);
        }
        Input {
            key: Key::Char('^'),
            ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::Head);
        }

        Input {
            key: Key::Char('g'),
            ..
        } => {
            // Handle gg in visual mode - need to track pending g
            // For simplicity, just go to top
            app.editor.textarea.move_cursor(CursorMove::Top);
        }
        Input {
            key: Key::Char('G'),
            ..
        } => {
            app.editor.textarea.move_cursor(CursorMove::Bottom);
        }

        _ => {}
    }

    true
}

fn copy_to_system_clipboard(textarea: &tui_textarea::TextArea) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let text = textarea.yank_text();
        if !text.is_empty() {
            let _ = clipboard.set_text(text);
        }
    }
}
