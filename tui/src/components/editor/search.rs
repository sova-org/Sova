use crate::app::App;
use color_eyre::Result as EyreResult;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    prelude::{Color, Rect, Style},
    widgets::{Block, Borders},
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[derive(Clone)]
/// Represents the state of the search functionality in the editor.
///
/// This struct manages the search panel's visibility, input handling, and error states.
/// It provides a user interface for entering search queries and displays any search-related
/// errors or feedback.
///
/// # Fields
///
/// * `is_active` - Controls whether the search panel is currently visible and accepting input.
///   When `true`, the search panel is shown and keyboard input is captured for search queries.
///   When `false`, the search panel is hidden and input is passed to the main editor.
///
/// * `query_textarea` - A text input area specifically for entering search queries.
///   This is wrapped in a bordered block with instructions for search navigation.
///   The textarea is configured to prevent multi-line input and provides visual feedback
///   for the current search query.
///
/// * `error_message` - An optional string containing any error messages related to the search.
///   This could include invalid search patterns, no matches found, or other search-related
///   issues. When `None`, no error is being displayed.
pub struct SearchState {
    pub is_active: bool,
    pub query_textarea: TextArea<'static>,
    pub error_message: Option<String>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
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

/// Handles key events when the search panel is potentially active.
///
/// If the search panel is active (`app.editor.search_state.is_active`), this function
/// processes keys relevant to search input (typing, Enter, Esc, navigation).
/// If the panel is not active, it immediately returns `Ok(false)`.
///
/// # Arguments
///
/// * `app` - Mutable reference to the main application state.
/// * `key_event` - The `KeyEvent` to process.
///
/// # Returns
///
/// * `Ok(true)` if the key event was consumed by the search handler.
/// * `Ok(false)` if the search panel was not active.
/// * `Err` if an underlying error occurs (although currently this function always returns `Ok`).
pub fn handle_search_input(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
    // Return Ok(true) if the input was consumed by the search handler
    if !app.editor.search_state.is_active {
        return Ok(false); // Not in search mode, didn't consume
    }

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
            Ok(true)
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
            Ok(true)
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
            Ok(true)
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
            Ok(true)
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
                    .first()
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
            Ok(true)
        }
    }
}

/// Renders the search input panel within the specified area.
///
/// The panel includes the query `TextArea` and displays any current search error message
/// in the title bar.
///
/// # Arguments
///
/// * `app` - Immutable reference to the main application state.
/// * `frame` - The mutable frame to render onto.
/// * `area` - The `Rect` defining the area allocated for the search panel.
pub fn render_search_panel(app: &App, frame: &mut Frame, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return; // Nothing to render in zero area
    }
    let search_state = &app.editor.search_state;
    let mut query_textarea = search_state.query_textarea.clone();
    let search_block_title = if let Some(err_msg) = &search_state.error_message {
        format!(
            " Search (Error: {}) (Esc:Cancel Enter:Find ^N/↓:Next ^P/↑:Prev) ",
            err_msg
        )
    } else {
        " Search Query (Esc: Cancel, Enter: Find, ^N/↓: Next, ^P/↑: Prev) ".to_string()
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

    frame.render_widget(&query_textarea, area);
}
