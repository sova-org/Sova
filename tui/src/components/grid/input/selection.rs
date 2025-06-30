//! Selection and clipboard handling for the grid component.
//!
//! Handles copy/paste operations, frame duplication, and selection management.

use crate::app::{App, ClipboardFrameData, ClipboardState};
use color_eyre::Result as EyreResult;
use corelib::scene::Scene;
use corelib::schedule::action_timing::ActionTiming;
use corelib::server::client::ClientMessage;
use corelib::shared_types::{GridSelection, PastedFrameData};
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashSet;

pub struct SelectionHandler;

impl SelectionHandler {
    /// Handle selection-related key events
    pub fn handle_selection(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let mut current_selection = app.interface.components.grid_selection;
        let mut handled = false;

        // For selection operations, we need a valid scene
        let has_scene = app.editor.scene.is_some();
        if !has_scene {
            return Ok(false);
        }

        match key_event.code {
            KeyCode::Char('c') => {
                handled = Self::handle_copy_action(app, current_selection);
            }
            KeyCode::Char('p') => {
                handled = Self::handle_paste_action(app, &mut current_selection);
            }
            KeyCode::Char('a') => {
                handled = Self::handle_duplicate_action(app, current_selection, true);
            }
            KeyCode::Char('d') => {
                handled = Self::handle_duplicate_action(app, current_selection, false);
            }
            _ => {}
        }

        // Update selection if it was modified
        if current_selection != app.interface.components.grid_selection {
            app.interface.components.grid_selection = current_selection;
            app.send_client_message(ClientMessage::UpdateGridSelection(current_selection));
        }

        Ok(handled)
    }

    fn handle_copy_action(app: &mut App, current_selection: GridSelection) -> bool {
        if let Some(scene) = &app.editor.scene {
            match Self::collect_copy_data(current_selection, scene) {
                Ok((new_clipboard_state, status_msg, messages_to_send)) => {
                    app.clipboard = new_clipboard_state;
                    app.set_status_message(status_msg);
                    for msg in messages_to_send {
                        app.send_client_message(msg);
                    }
                    true
                }
                Err(status_msg) => {
                    app.set_status_message(status_msg);
                    app.clipboard = ClipboardState::Empty;
                    false
                }
            }
        } else {
            app.set_status_message("Cannot copy: Scene not loaded.".to_string());
            false
        }
    }

    fn collect_copy_data(
        current_selection: GridSelection,
        scene: &Scene,
    ) -> Result<(ClipboardState, String, Vec<ClientMessage>), String> {
        let ((src_top, src_left), (src_bottom, src_right)) = current_selection.bounds();
        let mut collected_data: Vec<Vec<ClipboardFrameData>> = Vec::new();
        let mut pending_scripts = HashSet::new();
        let mut messages_to_send = Vec::new();
        let mut has_valid_frames = false;

        for col_idx in src_left..=src_right {
            let mut col_vec = Vec::new();
            let line_opt = scene.lines.get(col_idx);
            for row_idx in src_top..=src_bottom {
                let frame_data = if let Some(line) = line_opt {
                    if row_idx < line.frames.len() {
                        has_valid_frames = true;
                        if pending_scripts.insert((col_idx, row_idx)) {
                            messages_to_send.push(ClientMessage::GetScript(col_idx, row_idx));
                        }
                        ClipboardFrameData {
                            length: line.frames[row_idx],
                            is_enabled: line.is_frame_enabled(row_idx),
                            script_content: None,
                            frame_name: line.frame_names.get(row_idx).cloned().flatten(),
                        }
                    } else {
                        ClipboardFrameData::default()
                    }
                } else {
                    ClipboardFrameData::default()
                };
                col_vec.push(frame_data);
            }
            collected_data.push(col_vec);
        }

        if has_valid_frames {
            let new_clipboard_state = ClipboardState::FetchingScripts {
                pending: pending_scripts,
                collected_data,
                origin_top_left: (src_top, src_left),
            };
            let status_msg = format!(
                "Requesting scripts for copy [({}, {})..({}, {})]...",
                src_left, src_top, src_right, src_bottom
            );
            Ok((new_clipboard_state, status_msg, messages_to_send))
        } else {
            Err("Cannot copy: Selection contains no valid frames.".to_string())
        }
    }

    fn handle_paste_action(app: &mut App, current_selection: &mut GridSelection) -> bool {
        match app.clipboard.clone() {
            ClipboardState::ReadyMulti { data } => {
                let (target_row, target_col) = current_selection.cursor_pos();
                *current_selection = GridSelection::single(target_row, target_col);

                let num_cols_pasted = data.len();
                let num_rows_pasted = data.first().map_or(0, |col| col.len());

                if num_cols_pasted > 0 && num_rows_pasted > 0 {
                    let pasted_data: Vec<Vec<PastedFrameData>> = data
                        .iter()
                        .map(|col_data| {
                            col_data
                                .iter()
                                .map(|frame_data| PastedFrameData {
                                    length: frame_data.length,
                                    is_enabled: frame_data.is_enabled,
                                    script_content: frame_data.script_content.clone(),
                                    name: frame_data.frame_name.clone(),
                                    repetitions: None,
                                })
                                .collect()
                        })
                        .collect();

                    app.send_client_message(ClientMessage::PasteDataBlock {
                        data: pasted_data,
                        target_row,
                        target_col,
                        timing: ActionTiming::Immediate,
                    });
                    app.set_status_message(format!(
                        "Requested pasting {}x{} block at ({}, {})...",
                        num_cols_pasted, num_rows_pasted, target_col, target_row
                    ));
                    app.clipboard = ClipboardState::Empty;
                    true
                } else {
                    app.set_status_message("Cannot paste empty clipboard data.".to_string());
                    false
                }
            }
            ClipboardState::FetchingScripts { pending, .. } => {
                app.set_status_message(format!(
                    "Still fetching {} scripts from server to copy...",
                    pending.len()
                ));
                false
            }
            ClipboardState::Empty => {
                app.set_status_message("Clipboard is empty. Use 'c' to copy first.".to_string());
                false
            }
        }
    }

    fn handle_duplicate_action(
        app: &mut App,
        current_selection: GridSelection,
        insert_before: bool,
    ) -> bool {
        let ((top, left), (bottom, right)) = current_selection.bounds();

        let (target_cursor_row, target_cursor_col, desc) = if insert_before {
            (top, left, "before")
        } else {
            (bottom + 1, left, "after")
        };

        app.send_client_message(ClientMessage::RequestDuplicationData {
            src_top: top,
            src_left: left,
            src_bottom: bottom,
            src_right: right,
            target_cursor_row,
            target_cursor_col,
            insert_before,
            timing: ActionTiming::Immediate,
        });

        app.set_status_message(format!(
            "Requested duplication ({}) for selection [({}, {})..({}, {})]",
            desc, left, top, right, bottom
        ));

        true
    }
}