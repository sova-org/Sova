use crate::app::App;
use arboard::Clipboard;
use tui_textarea::{CursorMove, TextArea};

use super::{Command, Mode, Motion, MotionExecutor, Operator, TextRange, YankType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorResult {
    Success(Mode),
    Error(String),
}

pub fn execute_command(app: &mut App, command: Command) -> OperatorResult {
    let textarea = &mut app.editor.textarea;

    if let Some(operator) = command.operator {
        execute_operator(app, operator, command)
    } else {
        execute_motion_only(textarea, command)
    }
}

fn execute_operator(app: &mut App, operator: Operator, command: Command) -> OperatorResult {
    let textarea = &mut app.editor.textarea;
    let range = resolve_range(textarea, command.clone());

    match operator {
        Operator::Delete => {
            app.editor.vim_state.yank_register.yank_type =
                operator.yank_type_for_motion(command.motion);

            if command.is_linewise() {
                execute_linewise_delete(textarea, range);
            } else {
                execute_characterwise_delete(textarea, range);
            }

            OperatorResult::Success(Mode::Normal)
        }

        Operator::Change => {
            app.editor.vim_state.yank_register.yank_type =
                operator.yank_type_for_motion(command.motion);

            if command.is_linewise() {
                execute_linewise_delete(textarea, range);
            } else {
                execute_characterwise_delete(textarea, range);
            }

            OperatorResult::Success(Mode::Insert)
        }

        Operator::Yank => {
            app.editor.vim_state.yank_register.yank_type =
                operator.yank_type_for_motion(command.motion);

            if command.is_linewise() {
                execute_linewise_yank(textarea, range);
            } else {
                execute_characterwise_yank(textarea, range);
            }

            OperatorResult::Success(Mode::Normal)
        }
    }
}

fn execute_motion_only(textarea: &mut TextArea, command: Command) -> OperatorResult {
    textarea.execute_motion(command.motion, command.effective_count());
    OperatorResult::Success(Mode::Normal)
}

fn resolve_range(textarea: &mut TextArea, command: Command) -> TextRange {
    match textarea.find_text_object(command.motion) {
        Some(range) => {
            // Position cursor at range start for text objects
            textarea.move_cursor(CursorMove::Jump(
                range.start_row as u16,
                range.start_col as u16,
            ));
            range
        }
        None => {
            // Handle special case for linewise operations (dd, yy, cc)
            if matches!(command.motion, Motion::Line(_)) {
                let current_row = textarea.cursor().0;
                let count = command.effective_count() as usize;
                TextRange::new(current_row, 0, current_row + count - 1, 0)
            } else {
                // Execute motion and calculate range
                let start = textarea.cursor();
                textarea.execute_motion(command.motion, command.effective_count());
                let end = textarea.cursor();

                TextRange::new(start.0, start.1, end.0, end.1)
            }
        }
    }
}

fn execute_characterwise_delete(textarea: &mut TextArea, range: TextRange) {
    select_range(textarea, range);
    textarea.cut();
    simple_sync_to_system_clipboard(textarea);
}

fn execute_linewise_delete(textarea: &mut TextArea, range: TextRange) {
    // For linewise operations, select entire lines including newlines
    textarea.move_cursor(CursorMove::Jump(range.start_row as u16, 0));
    textarea.start_selection();

    // Include the newline by going to start of next line, or end of file
    if range.end_row < textarea.lines().len() - 1 {
        textarea.move_cursor(CursorMove::Jump(range.end_row as u16 + 1, 0));
    } else {
        // Last line - select to end including potential final newline
        textarea.move_cursor(CursorMove::Jump(range.end_row as u16, 0));
        textarea.move_cursor(CursorMove::End);
        // Try to include newline if there is one
        let lines = textarea.lines();
        if range.end_row < lines.len() {
            textarea.move_cursor(CursorMove::Forward);
        }
    }

    textarea.cut();
    simple_sync_to_system_clipboard(textarea);
}

fn execute_characterwise_yank(textarea: &mut TextArea, range: TextRange) {
    let original_cursor = textarea.cursor();
    select_range(textarea, range);
    textarea.copy();
    simple_sync_to_system_clipboard(textarea);
    textarea.cancel_selection();
    textarea.move_cursor(CursorMove::Jump(
        original_cursor.0 as u16,
        original_cursor.1 as u16,
    ));
}

fn execute_linewise_yank(textarea: &mut TextArea, range: TextRange) {
    let original_cursor = textarea.cursor();

    // Select entire lines including newlines
    textarea.move_cursor(CursorMove::Jump(range.start_row as u16, 0));
    textarea.start_selection();

    if range.end_row < textarea.lines().len() - 1 {
        textarea.move_cursor(CursorMove::Jump(range.end_row as u16 + 1, 0));
    } else {
        textarea.move_cursor(CursorMove::Jump(range.end_row as u16, 0));
        textarea.move_cursor(CursorMove::End);
        let lines = textarea.lines();
        if range.end_row < lines.len() {
            textarea.move_cursor(CursorMove::Forward);
        }
    }

    textarea.copy();
    simple_sync_to_system_clipboard(textarea);
    textarea.cancel_selection();
    textarea.move_cursor(CursorMove::Jump(
        original_cursor.0 as u16,
        original_cursor.1 as u16,
    ));
}

fn select_range(textarea: &mut TextArea, range: TextRange) {
    textarea.move_cursor(CursorMove::Jump(
        range.start_row as u16,
        range.start_col as u16,
    ));
    textarea.start_selection();
    textarea.move_cursor(CursorMove::Jump(range.end_row as u16, range.end_col as u16));
}

fn simple_sync_to_system_clipboard(textarea: &TextArea) {
    if let Ok(mut clipboard) = Clipboard::new() {
        let text = textarea.yank_text();
        if !text.is_empty() {
            let _ = clipboard.set_text(text);
        }
    }
}

pub fn paste_before(app: &mut App) -> OperatorResult {
    match paste_text(
        &mut app.editor.textarea,
        true,
        app.editor.vim_state.yank_register.yank_type,
    ) {
        Ok(_) => OperatorResult::Success(Mode::Normal),
        Err(e) => OperatorResult::Error(e),
    }
}

pub fn paste_after(app: &mut App) -> OperatorResult {
    match paste_text(
        &mut app.editor.textarea,
        false,
        app.editor.vim_state.yank_register.yank_type,
    ) {
        Ok(_) => OperatorResult::Success(Mode::Normal),
        Err(e) => OperatorResult::Error(e),
    }
}

fn paste_text(textarea: &mut TextArea, before: bool, yank_type: YankType) -> Result<(), String> {
    match yank_type {
        YankType::Linewise => {
            if before {
                // Paste above current line
                textarea.move_cursor(CursorMove::Head);
                textarea.paste();
                textarea.insert_newline();
                textarea.move_cursor(CursorMove::Up);
                textarea.move_cursor(CursorMove::Head);
            } else {
                // Paste below current line
                textarea.move_cursor(CursorMove::End);
                textarea.insert_newline();
                textarea.paste();
                textarea.move_cursor(CursorMove::Up);
                textarea.move_cursor(CursorMove::Head);
            }
        }
        YankType::Characterwise => {
            if !before {
                // Paste after cursor
                textarea.move_cursor(CursorMove::Forward);
            }
            // Paste before/at cursor
            textarea.paste();
        }
    }

    Ok(())
}
