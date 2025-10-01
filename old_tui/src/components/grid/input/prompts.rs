//! Input prompt handling for the grid component.
//!
//! Handles all text input prompts like setting frame length, name, repetitions, etc.

use crate::app::App;
use color_eyre::Result as EyreResult;
use corelib::schedule::action_timing::ActionTiming;
use corelib::server::client::ClientMessage;
use crossterm::event::{KeyCode, KeyEvent};
use std::str::FromStr;
use tui_textarea::TextArea;

pub struct PromptHandler;

impl PromptHandler {
    /// Check if we're currently in any prompt mode
    pub fn is_in_prompt_mode(app: &App) -> bool {
        app.interface.components.is_setting_frame_length
            || app.interface.components.is_inserting_frame_duration
            || app.interface.components.is_setting_frame_name
            || app.interface.components.is_setting_scene_length
            || app.interface.components.is_setting_frame_repetitions
    }

    /// Handle input for any active prompt
    pub fn handle_prompt_input(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        if app.interface.components.is_setting_frame_length {
            Self::handle_set_length_input(app, key_event)
        } else if app.interface.components.is_inserting_frame_duration {
            Self::handle_insert_duration_input(app, key_event)
        } else if app.interface.components.is_setting_frame_name {
            Self::handle_set_name_input(app, key_event)
        } else if app.interface.components.is_setting_scene_length {
            Self::handle_set_scene_length_input(app, key_event)
        } else if app.interface.components.is_setting_frame_repetitions {
            Self::handle_set_repetitions_input(app, key_event)
        } else {
            Ok(false)
        }
    }

    fn handle_set_length_input(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let mut is_active = app.interface.components.is_setting_frame_length;
        let mut textarea = app.interface.components.frame_length_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            KeyCode::Esc => {
                status_msg_to_set = Some("Frame length setting cancelled.".to_string());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_str = textarea.lines()[0].trim();
                if let Some(scene) = app.editor.scene.as_ref() {
                    let current_selection = app.interface.components.grid_selection;
                    match input_str.parse::<f64>() {
                        Ok(new_length) if new_length > 0.0 => {
                            let ((top, left), (bottom, right)) = current_selection.bounds();
                            let mut modified_lines: std::collections::HashMap<usize, Vec<f64>> =
                                std::collections::HashMap::new();
                            let mut frames_changed = 0;

                            for col_idx in left..=right {
                                if let Some(line) = scene.lines.get(col_idx) {
                                    let mut current_frames = line.frames.clone();
                                    let mut was_modified = false;
                                    for row_idx in top..=bottom {
                                        if row_idx < current_frames.len() {
                                            current_frames[row_idx] = new_length;
                                            was_modified = true;
                                            frames_changed += 1;
                                        }
                                    }
                                    if was_modified {
                                        modified_lines.insert(col_idx, current_frames);
                                    }
                                }
                            }

                            for (col, updated_frames) in modified_lines {
                                app.send_client_message(ClientMessage::UpdateLineFrames(
                                    col,
                                    updated_frames,
                                    ActionTiming::Immediate,
                                ));
                            }

                            if frames_changed > 0 {
                                status_msg_to_set = Some(format!(
                                    "Set length to {:.2} for {} frame(s)",
                                    new_length, frames_changed
                                ));
                            } else {
                                status_msg_to_set =
                                    Some("No valid frames in selection to set length".to_string());
                            }
                            exit_mode = true;
                        }
                        _ => {
                            let error_message = format!(
                                "Invalid frame length: '{}'. Must be positive number.",
                                input_str
                            );
                            app.interface.components.bottom_message = error_message.clone();
                            app.interface.components.bottom_message_timestamp =
                                Some(std::time::Instant::now());
                            status_msg_to_set = Some(error_message);
                        }
                    }
                } else {
                    status_msg_to_set =
                        Some("Error: Scene not loaded while setting frame length.".to_string());
                    exit_mode = true;
                }
            }
            _ => {
                handled_textarea = textarea.input(key_event);
            }
        }

        if let Some(msg) = status_msg_to_set {
            app.set_status_message(msg);
        }

        if exit_mode {
            is_active = false;
            textarea = TextArea::default();
        }

        app.interface.components.is_setting_frame_length = is_active;
        app.interface.components.frame_length_input = textarea;

        Ok(exit_mode || handled_textarea)
    }

    fn handle_insert_duration_input(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let mut is_active = app.interface.components.is_inserting_frame_duration;
        let mut textarea = app.interface.components.insert_duration_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            KeyCode::Esc => {
                status_msg_to_set = Some("Frame insertion cancelled.".to_string());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_str = textarea.lines()[0].trim();
                match f64::from_str(input_str) {
                    Ok(new_duration) if new_duration > 0.0 => {
                        let (row_idx, col_idx) =
                            app.interface.components.grid_selection.cursor_pos();
                        let insert_pos = row_idx + 1;
                        app.send_client_message(ClientMessage::InsertFrame(
                            col_idx,
                            insert_pos,
                            new_duration,
                            ActionTiming::Immediate,
                        ));
                        status_msg_to_set = Some(format!(
                            "Requested inserting frame with duration {:.2} at ({}, {})",
                            new_duration, col_idx, insert_pos
                        ));
                        exit_mode = true;
                    }
                    _ => {
                        let error_message = format!(
                            "Invalid duration: '{}'. Must be a positive number.",
                            input_str
                        );
                        app.interface.components.bottom_message = error_message.clone();
                        app.interface.components.bottom_message_timestamp =
                            Some(std::time::Instant::now());
                        status_msg_to_set = Some(error_message);
                    }
                }
            }
            _ => {
                handled_textarea = textarea.input(key_event);
            }
        }

        if let Some(msg) = status_msg_to_set {
            app.set_status_message(msg);
        }

        if exit_mode {
            is_active = false;
            textarea = TextArea::default();
        }

        app.interface.components.is_inserting_frame_duration = is_active;
        app.interface.components.insert_duration_input = textarea;
        Ok(exit_mode || handled_textarea)
    }

    fn handle_set_name_input(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let mut is_active = app.interface.components.is_setting_frame_name;
        let mut textarea = app.interface.components.frame_name_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            KeyCode::Esc => {
                status_msg_to_set = Some("Frame naming cancelled.".to_string());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_name = textarea.lines()[0].trim().to_string();
                let (row_idx, col_idx) = app.interface.components.grid_selection.cursor_pos();

                let name_to_send = if input_name.is_empty() {
                    None
                } else {
                    Some(input_name.clone())
                };
                app.send_client_message(ClientMessage::SetFrameName(
                    col_idx,
                    row_idx,
                    name_to_send.clone(),
                    ActionTiming::Immediate,
                ));

                status_msg_to_set = if let Some(name) = name_to_send {
                    Some(format!(
                        "Requested setting name to '{}' for frame ({}, {})",
                        name, col_idx, row_idx
                    ))
                } else {
                    Some(format!(
                        "Requested clearing name for frame ({}, {})",
                        col_idx, row_idx
                    ))
                };
                exit_mode = true;
            }
            _ => {
                handled_textarea = textarea.input(key_event);
            }
        }

        if let Some(msg) = status_msg_to_set {
            app.set_status_message(msg);
        }

        if exit_mode {
            is_active = false;
            textarea = TextArea::default();
        }

        app.interface.components.is_setting_frame_name = is_active;
        app.interface.components.frame_name_input = textarea;

        Ok(exit_mode || handled_textarea)
    }

    fn handle_set_scene_length_input(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let mut is_active = app.interface.components.is_setting_scene_length;
        let mut textarea = app.interface.components.scene_length_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            KeyCode::Esc => {
                status_msg_to_set = Some("Scene length setting cancelled.".to_string());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_str = textarea.lines()[0].trim();
                match input_str.parse::<usize>() {
                    Ok(new_length) if new_length > 0 => {
                        app.send_client_message(ClientMessage::SetSceneLength(
                            new_length,
                            ActionTiming::EndOfScene,
                        ));
                        status_msg_to_set =
                            Some(format!("Requested setting scene length to {}", new_length));
                        exit_mode = true;
                    }
                    _ => {
                        let error_message = format!(
                            "Invalid length: '{}'. Must be a positive integer.",
                            input_str
                        );
                        app.interface.components.bottom_message = error_message.clone();
                        app.interface.components.bottom_message_timestamp =
                            Some(std::time::Instant::now());
                        status_msg_to_set = Some(error_message);
                    }
                }
            }
            _ => {
                handled_textarea = textarea.input(key_event);
            }
        }

        if let Some(msg) = status_msg_to_set {
            app.set_status_message(msg);
        }

        if exit_mode {
            is_active = false;
            textarea = TextArea::default();
        }
        app.interface.components.is_setting_scene_length = is_active;
        app.interface.components.scene_length_input = textarea;
        Ok(exit_mode || handled_textarea)
    }

    fn handle_set_repetitions_input(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let mut is_active = app.interface.components.is_setting_frame_repetitions;
        let mut textarea = app.interface.components.frame_repetitions_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            KeyCode::Esc => {
                status_msg_to_set = Some("Frame repetitions setting cancelled.".to_string());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_str = textarea.lines()[0].trim();
                match input_str.parse::<usize>() {
                    Ok(new_repetitions) if new_repetitions > 0 => {
                        let (row_idx, col_idx) =
                            app.interface.components.grid_selection.cursor_pos();
                        app.send_client_message(ClientMessage::SetFrameRepetitions(
                            col_idx,
                            row_idx,
                            new_repetitions,
                            ActionTiming::Immediate,
                        ));
                        status_msg_to_set = Some(format!(
                            "Set repetitions to {} for frame ({}, {})",
                            new_repetitions, col_idx, row_idx
                        ));
                        exit_mode = true;
                    }
                    _ => {
                        let error_message = format!(
                            "Invalid repetitions: '{}'. Must be a positive integer.",
                            input_str
                        );
                        app.interface.components.bottom_message = error_message.clone();
                        app.interface.components.bottom_message_timestamp =
                            Some(std::time::Instant::now());
                        status_msg_to_set = Some(error_message);
                    }
                }
            }
            _ => {
                // Only allow digits in the input
                let is_digit = matches!(key_event.code, KeyCode::Char(c) if c.is_ascii_digit());
                if is_digit || matches!(key_event.code, KeyCode::Backspace) {
                    handled_textarea = textarea.input(key_event);
                } else {
                    handled_textarea = true; // Mark as handled to prevent other actions
                }
            }
        }

        if let Some(msg) = status_msg_to_set {
            app.set_status_message(msg);
        }

        if exit_mode {
            is_active = false;
            textarea = TextArea::default();
        }

        app.interface.components.is_setting_frame_repetitions = is_active;
        app.interface.components.frame_repetitions_input = textarea;
        Ok(exit_mode || handled_textarea)
    }
}
