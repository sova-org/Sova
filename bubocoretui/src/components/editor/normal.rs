use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::CursorMove;

/// Handles input events in normal mode, providing Emacs-like keybindings.
///
/// This function processes keyboard input when the editor is in normal mode,
/// implementing a set of Emacs-inspired keybindings for text navigation and editing.
/// It handles cursor movement, text deletion, cut/copy/paste operations, and undo.
///
/// # Arguments
///
/// * `app` - Mutable reference to the main application state
/// * `key_event` - The keyboard event to process
///
/// # Returns
///
/// * `true` if the key event was handled by this function
/// * `false` if the key event was passed to the default textarea handler
///
/// # Keybindings
///
/// ## Navigation
/// * `Ctrl+F` - Move cursor forward
/// * `Ctrl+B` - Move cursor backward
/// * `Ctrl+N` - Move cursor down
/// * `Ctrl+P` - Move cursor up
/// * `Ctrl+A` - Move to beginning of line
/// * `Ctrl+E` - Move to end of line
///
/// ## Deletion
/// * `Ctrl+D` - Delete character forward
/// * `Backspace` - Delete character backward
/// * `Delete` - Delete character forward
///
/// ## Cut/Copy/Paste
/// * `Alt+W` - Copy (kill-ring-save) - TODO: Implement selection
/// * `Ctrl+K` - Kill line (delete to end of line)
/// * `Ctrl+Y` - Yank (paste)
///
/// ## Undo
/// * `Ctrl+/` - Undo last operation
///
/// # Notes
///
/// * Some features like kill ring and selection handling are marked as TODO
/// * Redo functionality is not currently implemented
/// * Unhandled keys are passed to the default textarea input handler
pub(super) fn handle_normal_input(app: &mut App, key_event: KeyEvent) -> bool {
    let textarea = &mut app.editor.textarea;
    match key_event.code {
        // Basic Emacs-like navigation
        KeyCode::Char('f') if key_event.modifiers == KeyModifiers::CONTROL => {
            textarea.move_cursor(CursorMove::Forward);
            true
        }
        KeyCode::Char('b') if key_event.modifiers == KeyModifiers::CONTROL => {
            textarea.move_cursor(CursorMove::Back);
            true
        }
        KeyCode::Char('n') if key_event.modifiers == KeyModifiers::CONTROL => {
            textarea.move_cursor(CursorMove::Down);
            true
        }
        KeyCode::Char('p') if key_event.modifiers == KeyModifiers::CONTROL => {
            textarea.move_cursor(CursorMove::Up);
            true
        }
        KeyCode::Char('a') if key_event.modifiers == KeyModifiers::CONTROL => {
            textarea.move_cursor(CursorMove::Head); // Move to beginning of line
            true
        }
        KeyCode::Char('e') if key_event.modifiers == KeyModifiers::CONTROL => {
            textarea.move_cursor(CursorMove::End); // Move to end of line
            true
        }
        // Deletion
        KeyCode::Char('d') if key_event.modifiers == KeyModifiers::CONTROL => {
            textarea.delete_next_char(); // Delete character forward
            true
        }
        KeyCode::Backspace => {
            textarea.delete_char(); // Delete character backward
            true
        }
        KeyCode::Delete => {
            textarea.delete_next_char(); // Delete character forward
            true
        }
        // Cut/Copy/Paste (Emacs-style)
        KeyCode::Char('w') if key_event.modifiers == KeyModifiers::ALT => {
            // Alt+W for copy (like kill-ring-save)
            // TODO: Implement selection handling for Emacs-style copy (Alt+W)
            app.set_status_message("Copy (Alt+W) - requires selection (TBD)".to_string());
            true
        }
        KeyCode::Char('k') if key_event.modifiers == KeyModifiers::CONTROL => {
            // Ctrl+K for kill-line (delete from cursor to end of line)
            textarea.delete_line_by_end();
            // TODO: Add killed text to a conceptual kill ring?
            app.set_status_message("Kill line (Ctrl+K)".to_string());
            true
        }
        KeyCode::Char('y') if key_event.modifiers == KeyModifiers::CONTROL => {
            // Ctrl+Y for yank (paste)
            textarea.paste();
            // TODO: Implement kill ring for yank functionality
            app.set_status_message("Yank (Ctrl+Y)".to_string());
            true
        }
        // Undo
        KeyCode::Char('/') if key_event.modifiers == KeyModifiers::CONTROL => {
            // Ctrl+/ for undo
            textarea.undo();
            true
        }
        // Note: Redo (e.g., Ctrl+Shift+Z or Alt+/) is not standardly bound here.
        // Fallback to default textarea input handling
        _ => {
            textarea.input(key_event)
        }
    }
} 