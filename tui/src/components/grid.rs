use crate::app::App;
use crate::utils::styles::CommonStyles;
use crate::components::grid::{
    cell_renderer::GridCellRenderer,
    help::GridHelpPopupWidget,
    input_prompt::InputPromptWidget,
    table::GridTableWidget,
    utils::{GridCellData, GridRenderInfo},
};
use corelib::shared_types::GridSelection;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::{prelude::*, widgets::*};

mod cell_renderer;
mod cell_style;
mod help;
mod input;
mod input_prompt;
mod table;
pub mod utils;

/// A component that renders and manages the grid view of the timeline.
///
/// This component is responsible for displaying the timeline grid, handling user interactions,
/// and managing the visual representation of frames, lines, and their states. It coordinates
/// with the application state to render the grid and process user input.
pub struct GridComponent;

/// Defines the layout areas for the grid component's various UI elements.
///
/// This struct holds the rectangular areas (Rect) for different parts of the grid interface:
/// - `table_area`: The main area where the grid table is rendered
/// - `length_prompt_area`: Area for the frame length input prompt
/// - `insert_prompt_area`: Area for the frame insertion duration prompt
/// - `name_prompt_area`: Area for the frame name input prompt
/// - `scene_length_prompt_area`: Area for the scene length input prompt
/// - `repetitions_prompt_area`: Area for the frame repetitions input prompt
struct GridLayoutAreas {
    table_area: Rect,
    length_prompt_area: Rect,
    insert_prompt_area: Rect,
    name_prompt_area: Rect,
    scene_length_prompt_area: Rect,
    repetitions_prompt_area: Rect,
}

impl Default for GridComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl GridComponent {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, app: &mut App, frame: &mut Frame, area: Rect) {
        // Get the current scene length from the scene object
        let scene_length = app.editor.scene.as_ref().map_or(0, |s| s.length());

        // --- 1. Render Outer Block and Calculate Layout ---
        let layout_areas = match self.calculate_layout(app, area) {
            Some(areas) => areas,
            None => {
                // Render a simple block even if area is too small, but nothing inside
                let outer_block = Block::default().borders(Borders::ALL).title(" Grid ");
                frame.render_widget(outer_block, area);
                return;
            }
        };

        // --- Calculate max_frames (needed for outer block potentially) ---
        let max_frames = app.editor.scene.as_ref().map_or(0, |s| {
            s.lines
                .iter()
                .map(|line| line.frames.len())
                .max()
                .unwrap_or(0)
        });

        // --- Calculate visible height ---
        let table_height = layout_areas.table_area.height as usize;
        let header_rows = 1;
        let padding_rows = 1;
        let visible_height = table_height.saturating_sub(header_rows + padding_rows);

        // --- Scrolling (Offset fixed to 0 for now, key handling deferred) ---
        // Read current offset and clamp based on current render info
        let max_scroll = max_frames.saturating_sub(visible_height);
        app.interface.components.grid_scroll_offset =
            app.interface.components.grid_scroll_offset.min(max_scroll);
        let scroll_offset = app.interface.components.grid_scroll_offset; // Use the potentially clamped value
        let render_info = GridRenderInfo {
            visible_height,
            max_frames,
        }; // For title indicators
        // Store render info back into app state
        app.interface.components.last_grid_render_info = Some(render_info);

        self.render_outer_block(app, frame, area, scene_length, scroll_offset, Some(render_info));
        self.render_input_prompts(app, frame, &layout_areas);
        if let Some(scene) = &app.editor.scene {
            let grid_table_widget = GridTableWidget::new(app, scene, scroll_offset, visible_height);
            frame.render_widget(grid_table_widget, layout_areas.table_area);
        } else {
            // Render empty state directly in the table area if no scene
            // Note: GridTableWidget::render_empty_state is static, so we call it like this
            // We need a buffer though... rendering directly to frame might be easier here.
            let empty_paragraph = Paragraph::new("No scene loaded from server.")
                .style(CommonStyles::warning_themed(&app.client_config.theme))
                .centered();
            frame.render_widget(empty_paragraph, layout_areas.table_area);
            // Ensure render info is cleared if no scene
            app.interface.components.last_grid_render_info = None;
        }

        // --- Render Help Indicator (if help popup is NOT showing) ---
        if !app.interface.components.grid_show_help {
            let key_style = CommonStyles::key_binding_themed(&app.client_config.theme)
                .add_modifier(Modifier::BOLD);
            let help_text_string = "?: Help ";
            let help_text_width = help_text_string.len() as u16;
            // Use layout_areas.table_area for positioning
            let target_area = layout_areas.table_area;
            if target_area.width >= help_text_width && target_area.height > 0 {
                let help_text_area = Rect::new(
                    target_area.right().saturating_sub(help_text_width),
                    target_area.bottom().saturating_sub(1), // Position at the bottom of the table area
                    help_text_width,
                    1,
                );
                let help_spans = vec![
                    Span::styled("?", CommonStyles::default_text_themed(&app.client_config.theme)),
                    Span::styled(": Help ", key_style),
                ];
                let help_paragraph =
                    Paragraph::new(Line::from(help_spans)).alignment(Alignment::Right);
                frame.render_widget(help_paragraph, help_text_area);
            }
        }

        // --- 5. Render Help Popup (if active) ---
        if app.interface.components.grid_show_help {
            frame.render_widget(GridHelpPopupWidget, area);

            // Hide main cursor if help is shown and not in an input mode
            if !app.interface.components.is_setting_frame_length
                && !app.interface.components.is_inserting_frame_duration
                && !app.interface.components.is_setting_frame_name
                && !app.interface.components.is_setting_scene_length
                && !app.interface.components.is_setting_frame_repetitions
            {
                frame.set_cursor_position(Rect::default());
            }
        }
    }

    fn calculate_layout(&self, app: &App, area: Rect) -> Option<GridLayoutAreas> {
        // Need at least some space for borders + title + content (Thick border = 2 horiz, 2 vert)
        if area.width < 2 || area.height < 2 {
            return None;
        }

        // Calculate the actual inner area after accounting for Thick borders
        let inner_area = area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        // Check if inner area is valid for content
        if inner_area.width < 1 || inner_area.height < 1 {
            return None;
        }

        // Determine heights based on which prompts are active (Help height removed)
        let length_prompt_height = if app.interface.components.is_setting_frame_length {
            3
        } else {
            0
        };
        let insert_prompt_height = if app.interface.components.is_inserting_frame_duration {
            3
        } else {
            0
        };
        let name_prompt_height = if app.interface.components.is_setting_frame_name {
            3
        } else {
            0
        };
        let scene_length_prompt_height = if app.interface.components.is_setting_scene_length {
            3
        } else {
            0
        };
        let repetitions_prompt_height = if app.interface.components.is_setting_frame_repetitions {
            3
        } else {
            0
        };
        let prompt_height = length_prompt_height
            + insert_prompt_height
            + name_prompt_height
            + scene_length_prompt_height
            + repetitions_prompt_height; // Add new height

        // Split inner area: Table takes remaining space, prompt(s)
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),                // Table area
                Constraint::Length(prompt_height), // Combined Prompt area (0 if inactive)
            ])
            .split(inner_area);

        let table_area = main_chunks[0];
        let prompt_area = main_chunks[1];

        // Split the prompt area if both prompts could potentially be active
        let prompt_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(length_prompt_height),
                Constraint::Length(insert_prompt_height),
                Constraint::Length(name_prompt_height),
                Constraint::Length(scene_length_prompt_height),
                Constraint::Length(repetitions_prompt_height), // Add new constraint
            ])
            .split(prompt_area);

        let length_prompt_area = prompt_layout[0];
        let insert_prompt_area = prompt_layout[1];
        let name_prompt_area = prompt_layout[2];
        let scene_length_prompt_area = prompt_layout[3];
        let repetitions_prompt_area = prompt_layout[4]; // Assign new area

        Some(GridLayoutAreas {
            table_area,
            length_prompt_area,
            insert_prompt_area,
            name_prompt_area,
            scene_length_prompt_area,
            repetitions_prompt_area,
        })
    }

    fn render_outer_block(
        &self,
        app: &App,
        frame: &mut Frame,
        area: Rect,
        scene_length: usize,
        scroll_offset: usize,
        render_info: Option<GridRenderInfo>,
    ) {
        let mut title = format!(" Length: {} ", scene_length);
        if let Some(info) = render_info {
            if info.max_frames > info.visible_height {
                // Calculate max_scroll accurately here
                let max_scroll = info.max_frames.saturating_sub(info.visible_height);
                let scroll_perc = if max_scroll > 0 {
                    (scroll_offset * 100) / max_scroll
                } else {
                    0
                };
                title = format!(
                    " Scene Grid L:{} F:{} {} {}{} {}% ",
                    scene_length,                              // 1
                    info.max_frames,                           // 2
                    if scroll_offset > 0 { '↑' } else { ' ' }, // 3
                    if scroll_offset + info.visible_height < info.max_frames {
                        '↓'
                    } else {
                        ' '
                    }, // 4
                    scroll_perc,                               // 5
                    "" // Need a 6th argument for the last placeholder, maybe scroll position like "(row {}/{})" later?
                );
            }
        }
        let outer_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(CommonStyles::default_text_themed(&app.client_config.theme));
        let inner_area = outer_block.inner(area);
        frame.render_widget(outer_block.clone(), area);

        // Need at least some space to draw anything inside
        if inner_area.width < 1 || inner_area.height < 2 {}
    }

    fn render_input_prompts(&self, app: &App, frame: &mut Frame, layout: &GridLayoutAreas) {
        // Render input prompt for setting length if active
        if app.interface.components.is_setting_frame_length {
            let prompt_widget = InputPromptWidget::new(
                &app.interface.components.frame_length_input,
                "Set Frame Length (Enter: Confirm, Esc: Cancel)".to_string(),
                CommonStyles::warning_themed(&app.client_config.theme),
            );
            frame.render_widget(prompt_widget, layout.length_prompt_area);
        }

        // Render input prompt for inserting frame if active
        if app.interface.components.is_inserting_frame_duration {
            let prompt_widget = InputPromptWidget::new(
                &app.interface.components.insert_duration_input,
                "Insert Frame Duration (Enter: Confirm, Esc: Cancel)".to_string(),
                CommonStyles::accent_cyan_themed(&app.client_config.theme),
            );
            frame.render_widget(prompt_widget, layout.insert_prompt_area);
        }

        if app.interface.components.is_setting_frame_name {
            let prompt_widget = InputPromptWidget::new(
                &app.interface.components.frame_name_input,
                "Set Frame Name (Enter: Confirm, Esc: Cancel)".to_string(),
                CommonStyles::accent_magenta_themed(&app.client_config.theme),
            );
            frame.render_widget(prompt_widget, layout.name_prompt_area);
        }

        if app.interface.components.is_setting_scene_length {
            let prompt_widget = InputPromptWidget::new(
                &app.interface.components.scene_length_input,
                "Set Scene Length (Enter: Confirm, Esc: Cancel)".to_string(),
                CommonStyles::warning_themed(&app.client_config.theme),
            );
            frame.render_widget(prompt_widget, layout.scene_length_prompt_area);
        }

        // Render input prompt for setting repetitions if active
        if app.interface.components.is_setting_frame_repetitions {
            let prompt_widget = InputPromptWidget::new(
                &app.interface.components.frame_repetitions_input,
                "Set Repetitions (1-N, Enter: Confirm, Esc: Cancel)".to_string(),
                CommonStyles::value_text_themed(&app.client_config.theme),
            );
            frame.render_widget(prompt_widget, layout.repetitions_prompt_area);
        }
    }
}
