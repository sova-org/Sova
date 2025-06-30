use crate::app::App;
use tui_textarea::{CursorMove, Input, Key};

use super::super::Mode;

pub fn handle_insert_mode(app: &mut App, input: Input) -> bool {
    match input {
        Input { key: Key::Esc, .. }
        | Input {
            key: Key::Char('c'),
            ctrl: true,
            ..
        } => {
            // Handle repeat before exiting insert mode
            if let Some(repeat_info) = app.editor.vim_state.insert_repeat.take() {
                perform_insert_repeat(app, repeat_info);
            }

            app.editor.textarea.move_cursor(CursorMove::Back);
            app.editor
                .vim_state
                .set_mode(Mode::Normal, &mut app.editor.textarea);
        }
        _ => {
            app.editor.textarea.input(input);
        }
    }

    true
}

fn perform_insert_repeat(app: &mut App, repeat_info: super::super::InsertRepeat) {
    let current_cursor = app.editor.textarea.cursor();
    let start_cursor = repeat_info.start_cursor;

    // Calculate what was inserted
    let inserted_text = extract_inserted_text(&app.editor.textarea, start_cursor, current_cursor);

    // Repeat the inserted text (count - 1) times
    for _ in 1..repeat_info.count {
        app.editor.textarea.insert_str(&inserted_text);
    }
}

fn extract_inserted_text(
    textarea: &tui_textarea::TextArea,
    start: (usize, usize),
    end: (usize, usize),
) -> String {
    let lines = textarea.lines();

    if start.0 == end.0 {
        // Same line - extract text between start and current position
        if let Some(line) = lines.get(end.0) {
            let chars: Vec<char> = line.chars().collect();
            let start_col = start.1.min(chars.len());
            let end_col = end.1.min(chars.len());
            if end_col > start_col {
                chars[start_col..end_col].iter().collect()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else if end.0 > start.0 {
        // Multi-line insertion
        let mut result = String::new();

        // First line: from start_col to end of line
        if let Some(first_line) = lines.get(start.0) {
            let chars: Vec<char> = first_line.chars().collect();
            if start.1 < chars.len() {
                result.push_str(&chars[start.1..].iter().collect::<String>());
            }
            result.push('\n');
        }

        // Middle lines: complete lines
        for row in (start.0 + 1)..end.0 {
            if let Some(line) = lines.get(row) {
                result.push_str(line);
                result.push('\n');
            }
        }

        // Last line: from start to end_col
        if let Some(last_line) = lines.get(end.0) {
            let chars: Vec<char> = last_line.chars().collect();
            let end_col = end.1.min(chars.len());
            if end_col > 0 {
                result.push_str(&chars[0..end_col].iter().collect::<String>());
            }
        }

        result
    } else {
        // End cursor is before start cursor - shouldn't happen normally
        String::new()
    }
}
