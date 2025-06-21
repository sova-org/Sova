use crate::app::App;
use crate::components::grid::cell_style::GridCellStyles;
use crate::components::grid::{GridCellData, GridSelection};
use ratatui::prelude::*;
use ratatui::text::Line;
use ratatui::widgets::Cell;

/// A renderer for individual cells in the timeline grid.
///
/// This struct handles the visual rendering of grid cells, including their styles and content.
/// It maintains consistent styling across the grid and provides methods to determine
/// how each cell should be displayed based on its state.
///
/// # Fields
///
/// * `styles` - The collection of visual styles for different cell states
/// * `bar_char_active` - The character used to represent an active frame in the grid
/// * `bar_char_inactive` - The character used to represent an inactive frame in the grid
pub struct GridCellRenderer {
    pub styles: GridCellStyles,
    pub bar_char_active: &'static str,
    pub bar_char_inactive: &'static str,
}

impl GridCellRenderer {
    pub fn new() -> Self {
        Self {
            styles: GridCellStyles::default_styles(),
            bar_char_active: "▌",
            bar_char_inactive: " ",
        }
    }

    /// Determines the visual style and optional content override for a grid cell based on its state.
    ///
    /// This function evaluates the cell's position relative to local and peer cursors, selections,
    /// and editing states to determine how it should be displayed.
    ///
    /// # Arguments
    ///
    /// * `data` - The grid cell data containing position and line information
    /// * `app` - The application state containing cursor positions and peer information
    /// * `base_style` - The default style to use if no special states apply
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * The final style to apply to the cell
    /// * An optional content override (typically used for peer cursor indicators)
    pub fn determine_cell_style_and_content<'a>(
        &self,
        data: &GridCellData<'a>,
        app: &App,
        base_style: Style,
    ) -> (Style, Option<Span<'static>>) {
        let styles = &self.styles;

        let is_local_cursor =
            (data.frame_idx, data.col_idx) == app.interface.components.grid_selection.cursor_pos();
        let ((selection_top, selection_left), (selection_bottom, selection_right)) =
            app.interface.components.grid_selection.bounds();
        let is_selected_locally = data.frame_idx >= selection_top
            && data.frame_idx <= selection_bottom
            && data.col_idx >= selection_left
            && data.col_idx <= selection_right;

        let peer_on_cell: Option<(String, GridSelection)> = app
            .server
            .peer_sessions
            .iter()
            .filter_map(|(name, peer_state)| {
                peer_state.grid_selection.map(|sel| (name.clone(), sel))
            })
            .find(|(_, peer_selection)| {
                (data.frame_idx, data.col_idx) == peer_selection.cursor_pos()
            });
        let is_being_edited_by_peer = app
            .server
            .peer_sessions
            .values()
            .any(|peer_state| peer_state.editing_frame == Some((data.col_idx, data.frame_idx)));

        let mut final_style;
        let mut content_override = None;

        if is_local_cursor || is_selected_locally {
            final_style = styles.cursor;
        } else if let Some((ref peer_name, _)) = peer_on_cell {
            final_style = styles.peer_cursor;
            let name_fragment = peer_name.clone().chars().take(4).collect::<String>();
            content_override = Some(Span::raw(format!("{:<4}", name_fragment)));
        } else {
            final_style = base_style;
        }

        if is_being_edited_by_peer && !(is_local_cursor || is_selected_locally) {
            // Apply animation only if there's peer content or local cursor/selection isn't overriding
            let should_animate = is_local_cursor || is_selected_locally || peer_on_cell.is_some();
            if should_animate {
                let phase = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
                    % 500;
                let current_fg = final_style.fg.unwrap_or(Color::White);
                let animated_fg = if phase < 250 { current_fg } else { Color::Red };
                final_style = final_style.fg(animated_fg);
            }
        }

        (final_style, content_override)
    }

    /// Renders a single cell in the grid with all its visual elements and states.
    ///
    /// This function handles the complex rendering logic for a grid cell, including:
    /// - Frame state (enabled/disabled)
    /// - Playback markers (current frame, end of line)
    /// - Start/end range indicators
    /// - Peer cursor and editing indicators
    /// - Frame names and duration values
    ///
    /// # Arguments
    ///
    /// * `data` - Contains the cell's position and associated line data
    /// * `app` - Reference to the application state for accessing peer and playback info
    ///
    /// # Returns
    ///
    /// A `Cell` containing the rendered content with appropriate styling
    ///
    /// # Notes
    ///
    /// The cell layout consists of:
    /// - Left side: Range bar, play marker, frame name
    /// - Right side: Frame duration
    /// - Padding in between to fill the column width
    ///
    /// Special cases handled:
    /// - Empty cells (no line data)
    /// - Out of bounds frames
    /// - Peer cursors and editing states
    /// - Playback position indicators
    pub fn render<'a>(&self, data: GridCellData<'a>, app: &App) -> Cell<'static> {
        let styles = &self.styles; // Use styles from the renderer instance

        if let Some(line) = data.line {
            if data.frame_idx < line.frames.len() {
                let frame_val = line.frames[data.frame_idx];
                let frame_name = line.frame_names.get(data.frame_idx).cloned().flatten();
                let is_enabled = line.is_frame_enabled(data.frame_idx);
                // Get repetitions for this frame
                let total_repetitions = line
                    .frame_repetitions
                    .get(data.frame_idx)
                    .copied()
                    .unwrap_or(1)
                    .max(1); // Ensure at least 1

                // Determine base style
                let base_style = if is_enabled {
                    styles.enabled
                } else {
                    styles.disabled
                };

                let (final_style, content_override) =
                    self.determine_cell_style_and_content(&data, app, base_style);
                let current_frame_info = app
                    .server
                    .current_frame_positions
                    .as_ref()
                    .and_then(|positions| positions.iter().find(|(l, _, _)| *l == data.col_idx));

                let (current_frame_idx_for_line, current_repetition_idx) = current_frame_info
                    .map(|(_, f, r)| (*f, *r))
                    .unwrap_or((usize::MAX, 0)); // Default if line not found

                let is_head_past_last_frame = current_frame_idx_for_line == usize::MAX;
                let is_this_the_last_frame = data.frame_idx == line.frames.len().saturating_sub(1);

                // Determine Play Marker
                let is_head_on_this_frame = current_frame_idx_for_line == data.frame_idx;
                let play_marker = if is_this_the_last_frame && is_head_past_last_frame {
                    "⏳"
                } else if is_head_on_this_frame {
                    "▶"
                } else {
                    " "
                };
                let play_marker_span = Span::raw(play_marker);

                // Determine Start/End Bar
                let should_draw_bar = if let Some(start) = line.start_frame {
                    if let Some(end) = line.end_frame {
                        data.frame_idx >= start && data.frame_idx <= end
                    } else {
                        data.frame_idx >= start
                    }
                } else {
                    line.end_frame.is_some_and(|end| data.frame_idx <= end)
                };
                let bar_char = if should_draw_bar {
                    self.bar_char_active
                } else {
                    self.bar_char_inactive
                };
                let bar_span = Span::styled(
                    bar_char,
                    if should_draw_bar {
                        styles.start_end_marker
                    } else {
                        Style::default()
                    },
                );

                // Build left part (Bar, Play, Space, Content)
                // Use content_override (peer name) if present, otherwise use frame_name
                let main_content_str = content_override.map_or_else(
                    || frame_name.unwrap_or_else(|| "".to_string()),
                    |span| span.content.to_string(),
                );

                // Build repetition string ONLY if playhead is on this frame
                let repetition_span_opt: Option<Span> =
                    if is_head_on_this_frame && total_repetitions > 1 {
                        // Use a slightly less intrusive style for repetition count
                        Some(Span::styled(
                            // Style with gray background like duration
                            format!(" ({}/{})", current_repetition_idx + 1, total_repetitions),
                            Style::default().fg(Color::White).bg(Color::DarkGray), // Match duration style
                        ))
                    } else {
                        None // No repetition info shown otherwise
                    };

                let mut left_spans = vec![bar_span, play_marker_span, Span::raw(" ")];
                left_spans.push(Span::raw(main_content_str));
                let left_width = left_spans.iter().map(|s| s.width()).sum::<usize>();

                // Build right part (duration)
                let duration_str = format!(" {:.1} ", frame_val);
                let duration_style = Style::default().fg(Color::White).bg(Color::DarkGray);
                let duration_span = Span::styled(duration_str.clone(), duration_style); // Clone style for potential reuse
                let duration_width = duration_span.width();

                // Calculate repetition width
                let repetition_width = repetition_span_opt.as_ref().map_or(0, |s| s.width()); // No extra space needed if styled

                // Calculate padding
                let available_width = data.col_width;
                let padding_needed = available_width
                    .saturating_sub(left_width as u16)
                    .saturating_sub(duration_width as u16)
                    .saturating_sub(repetition_width as u16);
                let padding_span = Span::raw(" ".repeat(padding_needed as usize));

                // Assemble final spans
                let mut cell_line_spans = left_spans;
                cell_line_spans.push(padding_span);
                cell_line_spans.push(duration_span);
                if let Some(rep_span) = repetition_span_opt {
                    cell_line_spans.push(rep_span); // Push the Span directly
                }

                // Create cell
                let cell_content =
                    Line::from(cell_line_spans).alignment(ratatui::layout::Alignment::Left);
                Cell::from(cell_content).style(final_style)
            } else {
                self.render_empty(data, app)
            }
        } else {
            self.render_empty(data, app)
        }
    }

    /// Renders an empty cell in the grid.
    ///
    /// This function creates a cell with no frame content, using the terminal's default background.
    /// It may display peer cursor information if present, otherwise renders an empty space.
    ///
    /// # Arguments
    ///
    /// * `data` - The grid cell data containing position and line information
    /// * `app` - The application state for determining cell styling and peer information
    ///
    /// # Returns
    ///
    /// A styled `Cell` that can be rendered in the grid
    fn render_empty<'a>(&self, data: GridCellData<'a>, app: &App) -> Cell<'static> {
        let base_style = Style::default().bg(Color::Reset);
        let (final_style, content_override) =
            self.determine_cell_style_and_content(&data, app, base_style);
        let cell_content_span = content_override.unwrap_or_else(|| Span::raw(""));
        let cell_content =
            Line::from(cell_content_span).alignment(ratatui::layout::Alignment::Left);
        Cell::from(cell_content).style(final_style)
    }
}
