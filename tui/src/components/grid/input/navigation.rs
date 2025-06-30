//! Navigation input handling for the grid component.
//!
//! Handles arrow keys, scrolling, and basic navigation within the grid.

use crate::app::App;
use crate::components::grid::utils::GridRenderInfo;
use color_eyre::Result as EyreResult;
use corelib::scene::Scene;
use corelib::shared_types::GridSelection;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cmp::min;

pub struct NavigationHandler;

impl NavigationHandler {
    /// Handle help mode key events
    pub fn handle_help_mode(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                app.interface.components.grid_show_help = false;
                app.set_status_message("Closed help.".to_string());
                Ok(true)
            }
            _ => Ok(true), // Consume all other input when help is shown
        }
    }

    /// Handle navigation-related key events
    pub fn handle_navigation(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let scene_opt = app.editor.scene.as_ref();
        let num_cols = scene_opt.map_or(0, |p| p.lines.len());
        
        // Get current selection and scroll info
        let initial_selection = app.interface.components.grid_selection;
        let mut current_selection = initial_selection;
        let render_info = app.interface.components.last_grid_render_info;
        let mut current_scroll_offset = app.interface.components.grid_scroll_offset;
        let mut handled = false;

        match key_event.code {
            KeyCode::Esc => {
                if !current_selection.is_single() {
                    current_selection = GridSelection::single(
                        current_selection.start.0, 
                        current_selection.start.1
                    );
                    app.set_status_message("Selection reset to single cell (at start)".to_string());
                    handled = true;
                }
            }
            KeyCode::PageDown => {
                if let Some(info) = render_info {
                    if info.visible_height > 0 && info.max_frames > info.visible_height {
                        let page_size = info.visible_height.saturating_sub(1).max(1);
                        let max_scroll = info.max_frames.saturating_sub(info.visible_height);
                        current_scroll_offset = (current_scroll_offset + page_size).min(max_scroll);

                        // Move cursor to the top of the new page
                        if let Some(scene) = scene_opt {
                            let current_col = current_selection.cursor_pos().1;
                            let new_row = current_scroll_offset;
                            let frames_in_col = scene.lines.get(current_col).map_or(0, |l| l.frames.len());
                            let clamped_row = new_row.min(frames_in_col.saturating_sub(1));
                            current_selection = GridSelection::single(clamped_row, current_col);
                        }
                        handled = true;
                    }
                }
            }
            KeyCode::PageUp => {
                if let Some(info) = render_info {
                    if info.visible_height > 0 {
                        let page_size = info.visible_height.saturating_sub(1).max(1);
                        current_scroll_offset = current_scroll_offset.saturating_sub(page_size);

                        // Move cursor to the top of the new page
                        if let Some(scene) = scene_opt {
                            let current_col = current_selection.cursor_pos().1;
                            let new_row = current_scroll_offset;
                            let frames_in_col = scene.lines.get(current_col).map_or(0, |l| l.frames.len());
                            let clamped_row = new_row.min(frames_in_col.saturating_sub(1));
                            current_selection = GridSelection::single(clamped_row, current_col);
                        }
                        handled = true;
                    }
                }
            }
            KeyCode::Down | KeyCode::Up | KeyCode::Left | KeyCode::Right => {
                if let Some(scene) = scene_opt {
                    if num_cols > 0 {
                        let is_shift_pressed = key_event.modifiers.contains(KeyModifiers::SHIFT);
                        let (next_selection, changed) = Self::calculate_next_selection(
                            current_selection,
                            key_event.code,
                            is_shift_pressed,
                            scene,
                            num_cols,
                        );
                        if changed {
                            current_selection = next_selection;
                            handled = true;
                        }
                    }
                }
            }
            KeyCode::Char('?') => {
                app.interface.components.grid_show_help = true;
                app.set_status_message("Opened help (Esc or ? to close).".to_string());
                handled = true;
            }
            _ => {}
        }

        // Handle auto-scroll based on cursor position
        if handled {
            current_scroll_offset = Self::auto_scroll_for_cursor(
                current_selection.cursor_pos().0,
                current_scroll_offset,
                render_info,
            );
        }

        // Update app state if anything changed
        let selection_changed = initial_selection != current_selection;
        let scroll_changed = current_scroll_offset != app.interface.components.grid_scroll_offset;

        if selection_changed || scroll_changed {
            app.interface.components.grid_scroll_offset = current_scroll_offset;
            app.interface.components.grid_selection = current_selection;

            // Update render info for next frame
            if let Some(scene) = &app.editor.scene {
                let max_frames = scene.lines.iter().map(|line| line.frames.len()).max().unwrap_or(0);
                app.interface.components.last_grid_render_info = Some(GridRenderInfo {
                    visible_height: render_info.map_or(0, |info| info.visible_height),
                    max_frames,
                });
            }

            if selection_changed {
                use corelib::server::client::ClientMessage;
                app.send_client_message(ClientMessage::UpdateGridSelection(current_selection));
            }
        }

        Ok(handled || selection_changed || scroll_changed)
    }

    fn calculate_next_selection(
        current_selection: GridSelection,
        key_code: KeyCode,
        is_shift_pressed: bool,
        scene: &Scene,
        num_cols: usize,
    ) -> (GridSelection, bool) {
        let mut end_pos = current_selection.end;
        let mut changed = true;

        match key_code {
            KeyCode::Down => {
                if let Some(line) = scene.lines.get(end_pos.1) {
                    let frames_in_col = line.frames.len();
                    if frames_in_col > 0 {
                        end_pos.0 = min(end_pos.0 + 1, frames_in_col - 1);
                    } else {
                        changed = false;
                    }
                } else {
                    changed = false;
                }
            }
            KeyCode::Up => {
                end_pos.0 = end_pos.0.saturating_sub(1);
            }
            KeyCode::Left => {
                let next_col = end_pos.1.saturating_sub(1);
                if next_col != end_pos.1 {
                    let frames_in_next_col = scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                    end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1));
                    end_pos.1 = next_col;
                } else {
                    changed = false;
                }
            }
            KeyCode::Right => {
                let next_col = min(end_pos.1 + 1, num_cols.saturating_sub(1));
                if next_col != end_pos.1 {
                    let frames_in_next_col = scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                    end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1));
                    end_pos.1 = next_col;
                } else {
                    changed = false;
                }
            }
            _ => {
                changed = false;
            }
        }

        let final_selection = if changed {
            if is_shift_pressed {
                let mut modified_selection = current_selection;
                modified_selection.end = end_pos;
                modified_selection
            } else {
                GridSelection::single(end_pos.0, end_pos.1)
            }
        } else {
            current_selection
        };

        let actually_changed = final_selection != current_selection;
        (final_selection, actually_changed)
    }

    fn auto_scroll_for_cursor(
        cursor_row: usize,
        current_scroll_offset: usize,
        render_info: Option<GridRenderInfo>,
    ) -> usize {
        if let Some(info) = render_info {
            let visible_height = info.visible_height;
            let max_frames = info.max_frames;
            let mut desired_offset = current_scroll_offset;

            if cursor_row < desired_offset {
                // Cursor moved above visible area
                desired_offset = cursor_row;
            } else if visible_height > 0 && cursor_row >= desired_offset + visible_height {
                // Cursor moved below visible area
                desired_offset = cursor_row.saturating_sub(visible_height.saturating_sub(1));
            }

            // Clamp desired_offset
            let max_scroll = max_frames.saturating_sub(visible_height);
            desired_offset.min(max_scroll)
        } else {
            current_scroll_offset
        }
    }
}