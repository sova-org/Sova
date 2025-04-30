use crate::app::App;
use ratatui::{
    Frame,
    prelude::{Rect, Color, Style, Modifier},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthStr;

/// Renders a single line's view in the editor, showing its frames and their states.
///
/// This function displays a line's frames in a bordered area, showing various states like
/// enabled/disabled frames, playhead position, start/end markers, and the currently edited frame.
/// Each frame is displayed with its name (if available) and index, with appropriate styling
/// to indicate its current state.
///
/// # Arguments
///
/// * `app` - Reference to the main application state containing the scene data
/// * `frame` - Mutable reference to the terminal frame for rendering
/// * `area` - The rectangular area where the line view should be rendered
/// * `line_idx` - The index of the line to render within the scene
/// * `current_edit_frame_idx` - The index of the frame currently being edited
/// * `playhead_pos_opt` - Optional playhead position to highlight the current frame
///
/// # Notes
///
/// The function handles empty lines by displaying a centered "Line is empty" message.
/// For non-empty lines, it renders each frame with appropriate visual indicators for:
/// - Enabled/disabled state
/// - Playhead position
/// - Start/end markers
/// - Currently edited frame
pub fn render_single_line_view(
    app: &App,
    frame: &mut Frame,
    area: Rect,
    line_idx: usize,
    current_edit_frame_idx: usize,
    playhead_pos_opt: Option<usize>,
) {
    let line_view_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let inner_area = line_view_block.inner(area);
    frame.render_widget(line_view_block, area);

    if inner_area.width == 0 || inner_area.height == 0 {
        return;
    }

    if let Some(scene) = app.editor.scene.as_ref() {
        if let Some(line) = scene.lines.get(line_idx) {
            if line.frames.is_empty() {
                frame.render_widget(
                    Paragraph::new("Line is empty")
                        .centered()
                        .style(Style::default().fg(Color::DarkGray)),
                    inner_area,
                );
                return;
            }

            let items: Vec<ListItem> = line
                .frames
                .iter()
                .enumerate()
                .map(|(i, _frame_val)| {
                    let is_enabled = line.is_frame_enabled(i);
                    let is_playhead = playhead_pos_opt == Some(i);
                    let is_start = line.start_frame == Some(i);
                    let is_end = line.end_frame == Some(i);
                    let is_current_edit = i == current_edit_frame_idx;

                    // Fixed elements width calculation
                    let playhead_width = 1;
                    let marker_width = 1;
                    let index_width = 3; // " {:<2}" -> " 1", " 10", "100" might need adjustment
                    let fixed_spacers_width = 2; // Between playhead/marker, marker/name, name/index
                    let total_fixed_width = playhead_width
                        + marker_width
                        + index_width
                        + fixed_spacers_width;
                    let max_name_width =
                        (inner_area.width as usize).saturating_sub(total_fixed_width);

                    // Fetch and truncate name
                    let frame_name = line.frame_names.get(i).cloned().flatten();
                    let name_str = frame_name.unwrap_or_default();
                    let truncated_name: String = if name_str.width() > max_name_width {
                        name_str
                            .chars()
                            .take(max_name_width.saturating_sub(1))
                            .collect::<String>()
                            + "…"
                    } else {
                        name_str
                    };
                    let name_span = Span::raw(format!(
                        "{:<width$}",
                        truncated_name,
                        width = max_name_width
                    ));

                    // Build Spans
                    let playhead_span = Span::raw(if is_playhead { "▶" } else { " " });
                    let marker_span = Span::raw(if is_start {
                        "b"
                    } else if is_end {
                        "e"
                    } else {
                        " "
                    });
                    let index_span = Span::raw(format!(" {:<2}", i));

                    // Build Style
                    let (bg_color, fg_color) = if is_enabled {
                        (Color::Green, Color::White)
                    } else {
                        (Color::Red, Color::White)
                    };

                    let item_style = Style::default().bg(bg_color).fg(fg_color);

                    // Style the index span specifically if it's the current edit
                    let styled_index_span = if is_current_edit {
                        index_span.style(Style::default().add_modifier(Modifier::REVERSED))
                    } else {
                        index_span // Inherits fg from item_style
                    };

                    ListItem::new(Line::from(vec![
                        playhead_span,
                        marker_span,
                        Span::raw(" "), // Spacer 1
                        name_span,      // Truncated Name
                        Span::raw(" "), // Spacer 2
                        styled_index_span,
                    ]))
                    .style(item_style)
                })
                .collect();

            let list = List::new(items);

            frame.render_widget(list, inner_area);
        } else {
            frame.render_widget(
                Paragraph::new("Invalid Line")
                    .centered()
                    .style(Style::default().fg(Color::Red)),
                inner_area,
            );
        }
    } else {
        frame.render_widget(
            Paragraph::new("No Scene")
                .centered()
                .style(Style::default().fg(Color::Gray)),
            inner_area,
        );
    }
} 