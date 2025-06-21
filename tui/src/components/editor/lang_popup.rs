use crate::app::App;
use color_eyre::Result as EyreResult;
use corelib::{schedule::action_timing::ActionTiming, server::client::ClientMessage};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    prelude::{Color, Constraint, Layout, Modifier, Rect, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState},
};
use std::cmp::min;

/// Handles key events specifically when the language selection popup is active.
///
/// It processes keys for navigation (Up/Down/k/j), confirmation (Enter),
/// cancellation (Esc), and consumes all other keys to prevent them from
/// affecting the underlying editor.
///
/// # Arguments
///
/// * `app` - Mutable reference to the main application state.
/// * `key_event` - The `KeyEvent` to process.
///
/// # Returns
///
/// * `Ok(true)` if the key event was consumed by the popup handler (i.e., the popup
///   was active and the key was relevant to it).
/// * `Ok(false)` if the popup was not active.
/// * `Err` if an error occurs during event handling (e.g., sending a client message).
pub fn handle_lang_popup_input(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
    if !app.editor.is_lang_popup_active {
        return Ok(false);
    }

    let num_langs = app.editor.available_languages.len();
    if num_langs == 0 {
        // Should not happen if initialized correctly
        app.editor.is_lang_popup_active = false;
        app.set_status_message("No languages available to select.".to_string());
        return Ok(true); // Consumed the event (closed the popup)
    }

    match key_event.code {
        KeyCode::Esc => {
            app.editor.is_lang_popup_active = false;
            app.set_status_message("Language selection cancelled.".to_string());
            Ok(true)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.editor.selected_lang_index = app.editor.selected_lang_index.saturating_sub(1);
            Ok(true)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.editor.selected_lang_index =
                (app.editor.selected_lang_index + 1).min(num_langs - 1);
            Ok(true)
        }
        KeyCode::Enter => {
            let lang_to_set: Option<String> = app
                .editor
                .available_languages
                .get(app.editor.selected_lang_index)
                .cloned(); // Clone the string here

            if let Some(selected_lang) = lang_to_set {
                let line_idx = app.editor.active_line.line_index;
                let frame_idx = app.editor.active_line.frame_index;
                app.send_client_message(ClientMessage::SetScriptLanguage(
                    line_idx,
                    frame_idx,
                    selected_lang.clone(),
                    ActionTiming::Immediate, // Clone again for the message
                ));
                app.set_status_message(format!(
                    "Set language for Frame {}/{} to {}",
                    line_idx, frame_idx, selected_lang
                ));
            } else {
                app.set_status_message("Error selecting language.".to_string());
            }
            app.editor.is_lang_popup_active = false;
            Ok(true)
        }
        _ => {
            // Consume other keys while popup is active
            Ok(true)
        }
    }
}

/// Renders the language selection popup list centered within the given area, but only if
/// `app.editor.is_lang_popup_active` is true.
///
/// Clears the area behind the popup before rendering.
///
/// # Arguments
///
/// * `app` - Immutable reference to the main application state.
/// * `frame` - The mutable frame to render onto.
/// * `area` - The `Rect` defining the total area available for the editor, within which
///   the popup will be centered.
pub fn render_lang_popup(app: &App, frame: &mut Frame, area: Rect) {
    if !app.editor.is_lang_popup_active {
        return;
    }

    let popup_width = 30;
    let popup_height = min(app.editor.available_languages.len() + 2, 10) as u16;

    let popup_area = centered_rect_fixed(popup_width, popup_height, area);

    frame.render_widget(Clear, popup_area); // Clear background

    let items: Vec<ListItem> = app
        .editor
        .available_languages
        .iter()
        .map(|lang| ListItem::new(lang.as_str()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Select Language")
                .borders(Borders::ALL)
                .border_type(BorderType::Double),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Yellow),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.editor.selected_lang_index));

    frame.render_stateful_widget(list, popup_area, &mut list_state);
}

/// Helper function to create a centered rectangle with a fixed width and height.
/// Calculates margins to center the inner rectangle within the outer rectangle `r`.
///
/// # Arguments
///
/// * `width` - The desired fixed width of the centered rectangle.
/// * `height` - The desired fixed height of the centered rectangle.
/// * `r` - The outer `Rect` within which to center the new rectangle.
///
/// # Returns
///
/// * A `Rect` centered within `r` with the specified `width` and `height`.
///   If `width` or `height` are larger than `r`, the resulting rect might
///   be smaller due to saturation.
// Moved from editor.rs as it's only used by the popup rendering logic now.
fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let vertical_margin = r.height.saturating_sub(height) / 2;
    let horizontal_margin = r.width.saturating_sub(width) / 2;

    let popup_layout = Layout::vertical([
        Constraint::Length(vertical_margin),
        Constraint::Length(height),
        Constraint::Length(vertical_margin),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Length(horizontal_margin),
        Constraint::Length(width),
        Constraint::Length(horizontal_margin),
    ])
    .split(popup_layout[1])[1]
}
