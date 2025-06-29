use crate::app::App;
use crate::disk;
use crate::{components::Component, components::logs::LogLevel};
use crate::utils::styles::CommonStyles;
use color_eyre::Result as EyreResult;
use corelib::schedule::action_timing::ActionTiming;
use corelib::server::client::ClientMessage;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Modifier, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::cmp::min;
use tui_textarea::{CursorMove, Input};

pub mod help;
pub mod lang_popup;
pub mod line_view;
pub mod normal;
pub mod search;
pub mod vim;

/// The main editor component that handles text editing functionality.
///
/// This component manages the core text editing features including:
/// - Text input and manipulation
/// - Cursor movement and text selection
/// - Mode-specific input handling (Normal/Vim modes)
/// - Search functionality
/// - Language-specific features
/// - Editor state management
///
/// The editor supports multiple keymap modes (Normal and Vim) and provides
/// a unified interface for text editing operations while maintaining
/// mode-specific behaviors and commands.
pub struct EditorComponent;

impl Default for EditorComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for EditorComponent {
    /// Handles keyboard input events for the editor component.
    ///
    /// This function processes keyboard events in the following order:
    /// 1. Help Popup input (if active)
    /// 2. Language popup input (if active)
    /// 3. Search mode input
    /// 4. Editor exit (Esc key)
    /// 5. Global editor actions (Ctrl+key combinations)
    /// 6. Mode-specific input handling (Normal/Vim modes)
    /// 7. Open Help Popup '?'
    ///
    /// # Arguments
    /// * `app` - The application state
    /// * `key_event` - The keyboard event to process
    ///
    /// # Returns
    /// * `EyreResult<bool>` - Ok(true) if the event was handled, Ok(false) if not
    ///
    /// # Global Actions
    /// * Ctrl+S - Send current script to server
    /// * Ctrl+G - Activate search mode
    /// * Ctrl+E - Toggle frame enabled status
    /// * Ctrl+Arrows - Navigate between frames/lines
    /// * Ctrl+L - Open language selection popup
    ///
    /// # Mode-Specific Behavior
    /// * Normal Mode: Esc exits editor
    /// * Vim Mode: Esc handled by vim input handler, exits editor only in Normal mode
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        // 0. Handle Help Popup First (if active)
        if help::handle_help_popup_input(app, key_event)? {
            return Ok(true);
        }

        // 1. Handle Language Popup First (if active)
        if lang_popup::handle_lang_popup_input(app, key_event)? {
            return Ok(true);
        }

        // 2. Handle Search Mode First
        if search::handle_search_input(app, key_event)? {
            return Ok(true);
        }

        // 3. Handle Editor Exit (Esc) - ONLY if not searching
        if key_event.code == KeyCode::Esc {
            match app.client_config.editing_mode {
                disk::EditingMode::Normal => {
                    // Normal mode: Esc always exits editor
                    app.send_client_message(ClientMessage::StoppedEditingFrame(
                        app.editor.active_line.line_index,
                        app.editor.active_line.frame_index,
                    ));
                    app.editor.compilation_error = None;
                    app.events.sender.send(crate::event::Event::App(
                        crate::event::AppEvent::SwitchToGrid,
                    ))?;
                    app.set_status_message("Exited editor (Esc).".to_string());
                    return Ok(true);
                }
                disk::EditingMode::Vim => {
                    // Vim mode: Esc is handled by handle_vim_input.
                    // Let it fall through to the mode-specific handler below.
                    // handle_vim_input will return false if Esc was pressed in Normal mode,
                    // signaling that it didn't consume the event for mode switching.
                    // We'll handle the exit *after* the mode-specific handlers are called.
                }
            }
        }

        // 4. Handle Global Editor Actions (Ctrl+S, Ctrl+G, Ctrl+E, Ctrl+Arrows)
        // These should work regardless of Normal/Vim mode (unless Vim mode rebinds them, which we avoid here)
        if key_event.modifiers == KeyModifiers::CONTROL {
            match key_event.code {
                // --- Send Script ---
                KeyCode::Char('s') => {
                    app.add_log(
                        LogLevel::Debug,
                        "Ctrl+S detected, attempting to send script...".to_string(),
                    );
                    app.send_client_message(ClientMessage::SetScript(
                        app.editor.active_line.line_index,
                        app.editor.active_line.frame_index,
                        app.editor.textarea.lines().join("\n"),
                        ActionTiming::Immediate,
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
                    // Clear previous search query visually
                    if !app.editor.search_state.query_textarea.is_empty() {
                        app.editor
                            .search_state
                            .query_textarea
                            .move_cursor(CursorMove::Head);
                        app.editor.search_state.query_textarea.delete_line_by_end();
                    }
                    // Reset Vim mode to Normal if activating search from Vim mode? Optional.
                    // if app.settings.editor_keymap_mode == EditorKeymapMode::Vim {
                    //    // Need mutable self here, cannot call set_vim_mode directly
                    //    // Maybe reset vim_state directly?
                    //    app.editor.vim_state.set_mode(VimMode::Normal);
                    //    app.editor.textarea.set_cursor_style(VimMode::Normal.cursor_style());
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
                                    ClientMessage::DisableFrames(
                                        line_idx,
                                        vec![frame_idx],
                                        ActionTiming::Immediate,
                                    )
                                } else {
                                    ClientMessage::EnableFrames(
                                        line_idx,
                                        vec![frame_idx],
                                        ActionTiming::Immediate,
                                    )
                                };
                                app.send_client_message(message);
                                app.set_status_message(format!(
                                    "Toggled Frame {}/{} to {}",
                                    line_idx,
                                    frame_idx,
                                    if !current_enabled_status {
                                        "Enabled"
                                    } else {
                                        "Disabled"
                                    }
                                ));
                            } else {
                                app.set_status_message(
                                    "Cannot toggle: Invalid frame index.".to_string(),
                                );
                            }
                        } else {
                            app.set_status_message(
                                "Cannot toggle: Invalid line index.".to_string(),
                            );
                        }
                    } else {
                        app.set_status_message("Cannot toggle: scene not loaded.".to_string());
                    }
                    return Ok(true);
                }
                // --- Navigate Script ---
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    if let Some(scene) = &app.editor.scene {
                        let current_line_idx = app.editor.active_line.line_index;
                        let current_frame_idx = app.editor.active_line.frame_index;
                        let num_lines = scene.lines.len();
                        if num_lines == 0 {
                            app.set_status_message("No lines to navigate.".to_string());
                            return Ok(true);
                        }
                        let (target_line_idx, target_frame_idx) = match key_event.code {
                            KeyCode::Up => {
                                if current_frame_idx == 0 {
                                    app.set_status_message("Already at first frame.".to_string());
                                    return Ok(true);
                                }
                                (current_line_idx, current_frame_idx - 1)
                            }
                            KeyCode::Down => {
                                if let Some(line) = scene.lines.get(current_line_idx) {
                                    if current_frame_idx + 1 >= line.frames.len() {
                                        app.set_status_message(
                                            "Already at last frame.".to_string(),
                                        );
                                        return Ok(true);
                                    }
                                    (current_line_idx, current_frame_idx + 1)
                                } else {
                                    return Ok(true);
                                } // Should not happen if line exists
                            }
                            KeyCode::Left => {
                                if current_line_idx == 0 {
                                    app.set_status_message("Already at first line.".to_string());
                                    return Ok(true);
                                }
                                let target_line = current_line_idx - 1;
                                let target_line_len =
                                    scene.lines.get(target_line).map_or(0, |l| l.frames.len());
                                if target_line_len == 0 {
                                    app.set_status_message(format!(
                                        "Line {} is empty.",
                                        target_line
                                    ));
                                    return Ok(true);
                                }
                                (target_line, min(current_frame_idx, target_line_len - 1))
                            }
                            KeyCode::Right => {
                                if current_line_idx + 1 >= num_lines {
                                    app.set_status_message("Already at last line.".to_string());
                                    return Ok(true);
                                }
                                let target_line = current_line_idx + 1;
                                let target_line_len =
                                    scene.lines.get(target_line).map_or(0, |l| l.frames.len());
                                if target_line_len == 0 {
                                    app.set_status_message(format!(
                                        "Line {} is empty.",
                                        target_line
                                    ));
                                    return Ok(true);
                                }
                                (target_line, min(current_frame_idx, target_line_len - 1))
                            }
                            _ => unreachable!(),
                        };
                        // Stop editing current frame before requesting new one
                        app.send_client_message(ClientMessage::StoppedEditingFrame(
                            app.editor.active_line.line_index,
                            app.editor.active_line.frame_index,
                        ));
                        app.editor.compilation_error = None;
                        // Request new script
                        app.send_client_message(ClientMessage::GetScript(
                            target_line_idx,
                            target_frame_idx,
                        ));
                        // Immediately request *starting* editing the new frame
                        app.send_client_message(ClientMessage::StartedEditingFrame(
                            target_line_idx,
                            target_frame_idx,
                        ));
                        app.set_status_message(format!(
                            "Requested script Line {}, Frame {}",
                            target_line_idx, target_frame_idx
                        ));
                        return Ok(true);
                    } else {
                        app.set_status_message("scene not loaded, cannot navigate.".to_string());
                        return Ok(true);
                    }
                }
                KeyCode::Char('l') => {
                    let current_lang_opt = app
                        .editor
                        .scene
                        .as_ref()
                        .and_then(|s| s.lines.get(app.editor.active_line.line_index))
                        .and_then(|l| {
                            l.scripts
                                .iter()
                                .find(|scr| scr.index == app.editor.active_line.frame_index)
                        })
                        .map(|scr| scr.lang.clone());

                    if let Some(current_lang) = current_lang_opt {
                        if let Some(index) = app
                            .editor
                            .available_languages
                            .iter()
                            .position(|l| l == &current_lang)
                        {
                            app.editor.selected_lang_index = index;
                        }
                    } // else keep default index 0

                    app.editor.is_lang_popup_active = true;
                    app.set_status_message("Select language (↑/↓/Enter/Esc)".to_string());
                    return Ok(true);
                }
                _ => {}
            }
        }

        // --- Mode-Specific Input Handling ---
        let consumed_in_mode;
        match app.client_config.editing_mode {
            disk::EditingMode::Vim => {
                let input: Input = key_event.into();
                // Call the handler from the vim module
                consumed_in_mode = vim::handle_vim_input(app, input);
                // If Esc was pressed AND vim handler didn't consume it (meaning it was in Normal mode)
                if key_event.code == KeyCode::Esc
                    && !consumed_in_mode
                    && app.editor.vim_state.mode == vim::Mode::Normal
                // Check if in Normal mode
                {
                    // Then exit the editor
                    app.send_client_message(ClientMessage::StoppedEditingFrame(
                        app.editor.active_line.line_index,
                        app.editor.active_line.frame_index,
                    ));
                    app.editor.compilation_error = None;
                    app.events.sender.send(crate::event::Event::App(
                        crate::event::AppEvent::SwitchToGrid,
                    ))?;
                    app.set_status_message("Exited editor (Esc in Normal Mode).".to_string());
                    return Ok(true); // Exit handled
                }
            }
            disk::EditingMode::Normal => {
                // Call the handler from the normal module
                consumed_in_mode = normal::handle_normal_input(app, key_event);
                // In Normal mode, Esc exit is handled earlier (before mode-specific block)
            }
        };

        // --- Open Help Popup ---
        // Check *after* mode-specific handling, in case the key is used within a mode
        // Also ensure we aren't in a state where the key might be typed into an input (like search)
        if !consumed_in_mode
            && !app.editor.search_state.is_active
            && !app.editor.is_lang_popup_active
        {
            let should_open_help = match app.client_config.editing_mode {
                disk::EditingMode::Normal => {
                    key_event.code == KeyCode::Char('h')
                        && key_event.modifiers == KeyModifiers::CONTROL
                }
                disk::EditingMode::Vim => {
                    key_event.code == KeyCode::Char('?')
                        && key_event.modifiers == KeyModifiers::NONE
                }
            };

            if should_open_help {
                // Use the calculated boolean
                app.editor.is_help_popup_active = true;
                let open_key_str = match app.client_config.editing_mode {
                    disk::EditingMode::Normal => "Ctrl+H",
                    disk::EditingMode::Vim => "?",
                };
                app.set_status_message(format!(
                    "Help popup opened ({}). Press Esc or ? to close.",
                    open_key_str
                ));
                return Ok(true); // Consumed the key to open help
            }
        }

        Ok(consumed_in_mode) // Return result from mode-specific handler if '?' wasn't pressed
    }

    /// Renders the editor component to the terminal frame.
    ///
    /// This function handles the complex layout and rendering of the editor interface, including:
    /// - Main editor area with syntax highlighting
    /// - Line view panel
    /// - Bottom panels (search, error messages, command line)
    /// - Help text
    /// - Language selection popup
    ///
    /// # Arguments
    /// * `app` - The application state containing editor data and settings
    /// * `frame` - The terminal frame to render to
    /// * `area` - The rectangular area to render within
    ///
    /// # Layout Structure
    /// The editor is divided into several key areas:
    /// 1. Main editor block with title showing line/frame info
    /// 2. Horizontal split between main editor and line view
    /// 3. Vertical split of main editor into:
    ///    - Text area
    ///    - Bottom panels (search/error)
    ///    - Command line (when active)
    ///    - Help text
    ///
    /// # Features
    /// - Syntax highlighting based on current frame's language
    /// - Vim mode indicators and command line
    /// - Search functionality
    /// - Error message display
    /// - Context-aware help text
    /// - Line view visualization
    /// - Language selection popup
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let line_idx = app.editor.active_line.line_index;
        let frame_idx = app.editor.active_line.frame_index;

        let scene_opt = app.editor.scene.as_ref();
        let line_opt = scene_opt.and_then(|s| s.lines.get(line_idx));
        let playhead_pos_opt = app
            .server
            .current_frame_positions
            .as_ref()
            .and_then(|p| p.get(line_idx))
            .copied();

        let (_status_str, _length_str, is_enabled) = if let Some(line) = line_opt {
            if frame_idx < line.frames.len() {
                let enabled = line.is_frame_enabled(frame_idx);
                let length = line.frames[frame_idx];
                (
                    if enabled { "Enabled" } else { "Disabled" },
                    format!("Len: {:.2}", length),
                    enabled,
                )
            } else {
                ("Invalid Frame", "Len: N/A".to_string(), true)
            }
        } else {
            ("Invalid Line/Scene", "Len: N/A".to_string(), true)
        };

        let border_style = if is_enabled {
            CommonStyles::default_text_themed(&app.client_config.theme)
        } else {
            CommonStyles::description_themed(&app.client_config.theme)
        };

        let editor_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(border_style);

        frame.render_widget(editor_block.clone(), area);
        let inner_area = editor_block.inner(area);

        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        let line_view_width = 17;
        let actual_line_view_width = min(line_view_width, inner_area.width);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(actual_line_view_width),
                Constraint::Min(0),
            ])
            .split(inner_area);

        let line_view_area = horizontal_chunks[0];
        let main_editor_area = horizontal_chunks[1];

        if main_editor_area.width > 0 && main_editor_area.height > 0 {
            let help_area: Rect;
            let mut bottom_panel_area: Option<Rect> = None;

            let search_active = app.editor.search_state.is_active;
            let compilation_error_present = app.editor.compilation_error.is_some();
            let command_mode_active = app.client_config.editing_mode == disk::EditingMode::Vim
                && app.editor.vim_state.mode == vim::Mode::Command;
            let search_input_mode_active = app.client_config.editing_mode == disk::EditingMode::Vim
                && matches!(
                    app.editor.vim_state.mode,
                    vim::Mode::Search { .. }
                );

            let mut constraints = vec![Constraint::Min(0)];
            let mut bottom_panel_height = 0;
            let mut command_line_height = 0;

            // Determine heights, prioritizing Search/Error
            if search_active {
                bottom_panel_height = 3;
            } else if compilation_error_present {
                bottom_panel_height = 5;
            }
            bottom_panel_height = min(
                bottom_panel_height,
                main_editor_area.height.saturating_sub(1),
            );

            // Command line only if no search/error and space permits
            if !search_active
                && !compilation_error_present
                && (command_mode_active || search_input_mode_active)
            {
                command_line_height = min(
                    1,
                    main_editor_area
                        .height
                        .saturating_sub(bottom_panel_height + 1),
                ); // Needs 1 for command, 1 for help
            }

            // Push constraints
            if bottom_panel_height > 0 {
                constraints.push(Constraint::Length(bottom_panel_height));
            }
            if command_line_height > 0 {
                constraints.push(Constraint::Length(command_line_height));
            }
            let help_height = if main_editor_area.height > bottom_panel_height + command_line_height
            {
                1
            } else {
                0
            };
            if help_height > 0 {
                constraints.push(Constraint::Length(help_height));
            }

            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(main_editor_area);

            let editor_text_area: Rect = vertical_chunks[0];
            let mut current_index = 1;
            if bottom_panel_height > 0 {
                bottom_panel_area = Some(vertical_chunks[current_index]);
                current_index += 1;
            }
            let mut command_line_area: Option<Rect> = None;
            if command_line_height > 0 {
                command_line_area = Some(vertical_chunks[current_index]);
                current_index += 1;
            }
            if current_index < vertical_chunks.len() {
                help_area = vertical_chunks[current_index];
            } else {
                help_area = Rect::new(
                    main_editor_area.x,
                    main_editor_area.y + main_editor_area.height,
                    main_editor_area.width,
                    0,
                );
            }

            if let Some(panel_area) = bottom_panel_area {
                if panel_area.width > 0 && panel_area.height > 0 {
                    if search_active {
                        // Use the extracted function
                        search::render_search_panel(app, frame, panel_area);
                    } else if let Some(error_msg) = &app.editor.compilation_error {
                        let mut error_line_num = 0;
                        let mut error_col_num = 0;
                        let mut char_idx_count = 0;
                        let editor_lines = app.editor.textarea.lines();

                        for (i, line) in editor_lines.iter().enumerate() {
                            let line_len = line.chars().count();
                            let line_end_idx = char_idx_count + line_len;

                            if error_msg.from >= char_idx_count && error_msg.from < line_end_idx {
                                error_line_num = i;
                                error_col_num = error_msg.from - char_idx_count;
                                break;
                            }
                            let newline_offset = if i < editor_lines.len() - 1 { 1 } else { 0 };
                            char_idx_count = line_end_idx + newline_offset;

                            if error_msg.from == char_idx_count && i + 1 < editor_lines.len() {
                                error_line_num = i + 1;
                                error_col_num = 0;
                                break;
                            }
                        }

                        let error_block = Block::default()
                            .title(format!(
                                " Compilation Error ({}: Line {}, Col {}) ",
                                error_msg.lang,
                                error_line_num + 1,
                                error_col_num + 1
                            ))
                            .borders(Borders::ALL)
                            .border_type(BorderType::Plain)
                            .style(CommonStyles::error_themed(&app.client_config.theme));
                        let error_paragraph = Paragraph::new(error_msg.info.as_str())
                            .wrap(ratatui::widgets::Wrap { trim: true })
                            .block(error_block.clone());
                        frame.render_widget(error_paragraph, panel_area);
                    }
                }
            }

            // --- Render Command Line (if active) ---
            if let Some(cmd_area) = command_line_area {
                if cmd_area.width > 0 && cmd_area.height > 0 {
                    let buffer_text = &app.editor.vim_state.command_buffer;
                    let (prefix, style) = match app.editor.vim_state.mode {
                        vim::Mode::Command => (":", CommonStyles::warning_themed(&app.client_config.theme)),
                        vim::Mode::Search { forward: true } => {
                            ("/", CommonStyles::accent_magenta_themed(&app.client_config.theme))
                        }
                        vim::Mode::Search { forward: false } => {
                            ("?", CommonStyles::accent_magenta_themed(&app.client_config.theme))
                        }
                        _ => ("", CommonStyles::default_text_themed(&app.client_config.theme)), // Should not be reached if command_line_area is Some
                    };

                    if !prefix.is_empty() {
                        let display_text = format!("{}{}", prefix, buffer_text);
                        let paragraph = Paragraph::new(display_text).style(style);
                        frame.render_widget(paragraph, cmd_area);
                    }
                }
            }
            // --- End Render Command Line ---

            if editor_text_area.width > 0 && editor_text_area.height > 0 {
                let mut text_area = app.editor.textarea.clone();
                text_area.set_line_number_style(CommonStyles::description_themed(&app.client_config.theme));

                // --- Syntax Highlighting Configuration ---
                if let Some(highlighter) = app.editor.syntax_highlighter.as_ref() {
                    // Assuming highlighter is Option<Arc<SyntaxHighlighter>>
                    // 1. Determine the language of the current frame
                    let current_lang_opt: Option<String> = scene_opt
                        .and_then(|s| s.lines.get(line_idx))
                        .and_then(|l| l.scripts.iter().find(|scr| scr.index == frame_idx))
                        .map(|scr| scr.lang.clone());

                    // 2. Look up the syntect syntax name from the map
                    let syntax_name_opt: Option<String> = current_lang_opt
                        .and_then(|lang| app.editor.syntax_name_map.get(&lang).cloned());

                    // 3. Configure the TextArea
                    text_area.set_syntax_highlighter((**highlighter).clone());
                    text_area.set_syntax(syntax_name_opt);
                    // Use theme-appropriate syntax highlighting theme
                    let syntax_theme = get_syntax_theme(&app.client_config.theme);
                    text_area.set_theme(Some(syntax_theme));
                } else {
                    // Fallback if highlighter isn't loaded
                    text_area.set_syntax(None);
                }
                // --- End Syntax Highlighting Configuration ---

                frame.render_widget(&text_area, editor_text_area); // Render the configured text_area
            }

            if help_area.width > 0 && help_area.height > 0 {
                let help_style = CommonStyles::description_themed(&app.client_config.theme);
                let key_style = CommonStyles::key_binding_themed(&app.client_config.theme)
                    .add_modifier(Modifier::BOLD);

                let help_line = if search_active {
                    // Keep search help as is
                    Line::from(vec![
                        Span::styled(" Esc ", key_style),
                        Span::styled("Cancel | ", help_style),
                        Span::styled(" Enter ", key_style),
                        Span::styled("Find & Close | ", help_style),
                        Span::styled(" ^N/↓ ", key_style),
                        Span::styled("Next | ", help_style),
                        Span::styled(" ^P/↑ ", key_style),
                        Span::styled("Prev", help_style),
                    ])
                } else {
                    // Determine help text based on editing mode
                    let help_spans = match app.client_config.editing_mode {
                        disk::EditingMode::Vim => {
                            vec![
                                Span::styled("?", key_style),
                                Span::styled(": Help ", help_style), // Add padding space here
                            ]
                        }
                        disk::EditingMode::Normal => {
                            vec![
                                Span::styled("Ctrl+H", key_style),
                                Span::styled(": Help ", help_style), // Add padding space here
                            ]
                        }
                    };
                    Line::from(help_spans)
                };

                let help = Paragraph::new(help_line).alignment(ratatui::layout::Alignment::Right);
                frame.render_widget(help, help_area);
            }
        } else {
            frame.render_widget(
                Paragraph::new("Editor Area Too Small")
                    .centered()
                    .style(CommonStyles::error_themed(&app.client_config.theme)),
                main_editor_area,
            );
        }

        // Extract playhead frame index if the playhead is on the current line
        let playhead_frame_idx_opt = playhead_pos_opt
            .filter(|(line, _, _)| *line == line_idx)
            .map(|(_, frame, _)| frame);

        if line_view_area.width > 0 && line_view_area.height > 0 {
            line_view::render_single_line_view(
                app,
                frame,
                line_view_area,
                line_idx,
                frame_idx,
                playhead_frame_idx_opt, // Pass Option<usize>
            );
        } else if inner_area.width > 0 && inner_area.height > 0 {
            let indicator_area = Rect {
                x: inner_area.right() - 1,
                y: inner_area.top(),
                width: 1,
                height: 1,
            };
            frame.render_widget(
                Span::styled("…", CommonStyles::default_text_themed(&app.client_config.theme)),
                indicator_area,
            );
        }

        // --- Render Language Selection Popup (if active) ---
        lang_popup::render_lang_popup(app, frame, area);
        // --- End Language Selection Popup ---

        // --- Render Help Popup (if active) ---
        help::render_editor_help_popup(app, frame, area);
        // --- End Help Popup ---
    }
}

/// Get theme-appropriate syntax highlighting theme name
fn get_syntax_theme(theme: &crate::disk::Theme) -> String {
    use crate::disk::Theme;
    
    match theme {
        Theme::Classic => "base16-default.dark".to_string(),
        Theme::Ocean => "base16-ocean.dark".to_string(),
        Theme::Forest => "base16-eighties.dark".to_string(),
    }
}
