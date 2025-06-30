use crate::app::App;
use crate::disk::Theme;
use crate::utils::styles::CommonStyles;
use ratatui::{
    Frame,
    prelude::{Color, Modifier, Rect, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use unicode_width::UnicodeWidthStr;

/// Grid colors adapted for the line view
struct GridColors {
    enabled: Color,
    disabled: Color,
    playing: Color,
    user_cursor: Color,
    text_dark: Color,
    text_light: Color,
}

impl GridColors {
    fn for_theme(theme: &Theme) -> Self {
        Self {
            enabled: CommonStyles::accent_cyan_themed(theme)
                .fg
                .unwrap_or(Color::Green),
            disabled: CommonStyles::description_themed(theme)
                .fg
                .unwrap_or(Color::Gray),
            playing: CommonStyles::warning_themed(theme)
                .fg
                .unwrap_or(Color::Yellow),
            user_cursor: CommonStyles::selected_item_themed(theme)
                .bg
                .unwrap_or(Color::White),
            text_dark: Color::Black,
            text_light: Color::White,
        }
    }

    fn text_for_background(&self, bg: Color) -> Color {
        match bg {
            Color::White | Color::Yellow => self.text_dark,
            Color::Rgb(r, g, b) if (r as u16 + g as u16 + b as u16) > 400 => self.text_dark,
            _ => self.text_light,
        }
    }
}

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
    let theme = &app.client_config.theme;
    let grid_colors = GridColors::for_theme(theme);
    let line_view_block = Block::default()
        .borders(Borders::RIGHT)
        .style(CommonStyles::default_text_themed(&app.client_config.theme));

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
                        .style(CommonStyles::description_themed(&app.client_config.theme)),
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
                    let _is_start = line.start_frame == Some(i);
                    let _is_end = line.end_frame == Some(i);
                    let is_current_edit = i == current_edit_frame_idx;

                    // Calculate time progression for this specific frame
                    let frame_progression = if is_playhead {
                        // Create a more visible animation using current_phase
                        // Convert phase to a pulsing 0-1 value with sine wave
                        let phase = app.server.current_phase;
                        let pulse = ((phase * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5) as f32;
                        Some(pulse)
                    } else {
                        None
                    };

                    // Fixed elements width calculation
                    let bar_width = 1;
                    let playhead_width = 1;
                    let index_width = 3; // " {:<2}" -> " 1", " 10", "100" might need adjustment
                    let fixed_spacers_width = 2; // Spacer between bar/playhead, and playhead/name
                    let total_fixed_width =
                        playhead_width + bar_width + index_width + fixed_spacers_width;
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
                    // Determine Start/End Bar character and style
                    let is_in_range = match (line.start_frame, line.end_frame) {
                        (Some(start), Some(end)) => i >= start && i <= end,
                        (Some(start), None) => i >= start,
                        (None, Some(end)) => i <= end,
                        (None, None) => false, // No range defined
                    };

                    let bar_char = if is_in_range { "▐" } else { " " };
                    let bar_style = if is_in_range {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default() // Default style for inactive bar
                    };
                    let bar_span = Span::styled(bar_char, bar_style);

                    let playhead_span = Span::raw(if is_playhead { "▶" } else { " " });
                    let index_span = Span::raw(format!(" {:<2}", i));

                    // Determine base colors based on state
                    let base_bg = if is_current_edit {
                        grid_colors.user_cursor
                    } else if is_playhead {
                        grid_colors.playing
                    } else if is_enabled {
                        grid_colors.enabled
                    } else {
                        grid_colors.disabled
                    };

                    // Apply gradient if this frame is playing with progression
                    let (bg_color, fg_color) = if is_playhead && frame_progression.is_some() {
                        let progress = frame_progression.unwrap_or(0.0);
                        let gradient_bg = create_gradient_color(base_bg, progress);
                        (gradient_bg, grid_colors.text_for_background(gradient_bg))
                    } else {
                        (base_bg, grid_colors.text_for_background(base_bg))
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
                        name_span,      // Truncated Name
                        Span::raw(" "), // Spacer 1 (Playhead <-> Name)
                        bar_span,       // Start/End Bar (Moved)
                        Span::raw(" "), // Spacer 2 (Bar <-> Index)
                        styled_index_span,
                    ]))
                    .style(item_style)
                })
                .collect();

            let list = List::new(items);

            // Reserve space for the language name at the bottom if possible
            let mut list_area = inner_area;
            let mut lang_area: Option<Rect> = None;
            if inner_area.height > 1 {
                // Check if there's space for the language row
                let chunks = ratatui::prelude::Layout::vertical([
                    ratatui::prelude::Constraint::Min(1),
                    ratatui::prelude::Constraint::Length(1),
                ])
                .split(inner_area);
                list_area = chunks[0];
                lang_area = Some(chunks[1]);
            }

            frame.render_widget(list, list_area);

            // Render the language name at the bottom
            if let Some(area) = lang_area {
                let lang_name = line
                    .scripts
                    .iter()
                    .find(|scr| scr.index == current_edit_frame_idx)
                    .map_or("N/A", |scr| scr.lang.as_str());
                let lang_text = Paragraph::new(Line::from(vec![
                    Span::styled(
                        "Lang: ",
                        CommonStyles::default_text_themed(&app.client_config.theme),
                    ),
                    Span::styled(
                        lang_name,
                        CommonStyles::description_themed(&app.client_config.theme),
                    ),
                ]))
                .centered();
                frame.render_widget(lang_text, area);
            }
        } else {
            frame.render_widget(
                Paragraph::new("Invalid Line")
                    .centered()
                    .style(CommonStyles::error_themed(&app.client_config.theme)),
                inner_area,
            );
        }
    } else {
        frame.render_widget(
            Paragraph::new("No Scene")
                .centered()
                .style(CommonStyles::description_themed(&app.client_config.theme)),
            inner_area,
        );
    }
}

/// Create a gradient color based on progression
fn create_gradient_color(base_color: Color, progress: f32) -> Color {
    match base_color {
        Color::Rgb(r, g, b) => {
            // Create a gradient from base color to bright white/yellow
            let progress = progress.clamp(0.0, 1.0);

            // Target bright color (warm white/yellow)
            let target_r = 255;
            let target_g = 255;
            let target_b = 200; // Slightly warm

            // Interpolate between base and target
            let new_r = (r as f32 + (target_r as f32 - r as f32) * progress * 0.6) as u8;
            let new_g = (g as f32 + (target_g as f32 - g as f32) * progress * 0.6) as u8;
            let new_b = (b as f32 + (target_b as f32 - b as f32) * progress * 0.6) as u8;

            Color::Rgb(new_r, new_g, new_b)
        }
        _ => {
            // For non-RGB colors, convert to approximate RGB first
            let rgb_color = match base_color {
                Color::Green => Color::Rgb(0, 255, 0),
                Color::Yellow => Color::Rgb(255, 255, 0),
                Color::Cyan => Color::Rgb(0, 255, 255),
                Color::Blue => Color::Rgb(0, 0, 255),
                Color::Red => Color::Rgb(255, 0, 0),
                Color::Gray => Color::Rgb(128, 128, 128),
                Color::DarkGray => Color::Rgb(64, 64, 64),
                Color::White => Color::Rgb(255, 255, 255),
                Color::Black => Color::Rgb(0, 0, 0),
                Color::Magenta => Color::Rgb(255, 0, 255),
                _ => Color::Rgb(128, 128, 128), // Default gray
            };
            create_gradient_color(rgb_color, progress)
        }
    }
}

/// Brighten a color by a factor
fn brighten_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        ),
        _ => color,
    }
}
