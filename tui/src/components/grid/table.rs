use crate::app::App;
use crate::components::grid::{GridCellData, GridCellRenderer};
use corelib::scene::Scene;
use ratatui::prelude::*;
use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Widget};
use std::iter::once;

/// A widget that renders the grid table in the timeline view.
///
/// This struct holds the necessary data to render the grid table, including:
/// - A reference to the application state for styling and configuration
/// - A reference to the scene data containing the timeline content
/// - The current scroll offset for vertical scrolling
/// - The visible height of the grid in the current viewport
///
/// The widget implements the `Widget` trait to render the grid table within a specified area.
///
/// # Lifetime Parameters
///
/// * `'a` - The lifetime of the references to `App` and `Scene`
pub struct GridTableWidget<'a> {
    pub app: &'a App,
    pub scene: &'a Scene,
    pub scroll_offset: usize,
    pub visible_height: usize,
}

impl<'a> GridTableWidget<'a> {
    pub fn new(
        app: &'a App,
        scene: &'a Scene,
        scroll_offset: usize,
        visible_height: usize,
    ) -> Self {
        Self {
            app,
            scene,
            scroll_offset,
            visible_height,
        }
    }

    pub fn render_empty_state(buf: &mut Buffer, area: Rect, message: &str) {
        let paragraph = Paragraph::new(message).yellow().centered();
        paragraph.render(area, buf);
    }
}

impl<'a> Widget for GridTableWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = &self.scene.lines;
        let num_lines = lines.len();
        if num_lines == 0 {
            Self::render_empty_state(buf, area, "No lines in scene. Shift+A to add.");
            return;
        }

        let cell_renderer = GridCellRenderer::new();
        let max_frames = lines
            .iter()
            .map(|line| line.frames.len())
            .max()
            .unwrap_or(0);

        let start_row = self.scroll_offset;
        let end_row = self.scroll_offset.saturating_add(self.visible_height);

        if max_frames == 0 && self.visible_height > 0 {
            Self::render_empty_state(buf, area, "Lines have no frames. 'i' to insert.");
        } else if self.visible_height == 0 {
            Self::render_empty_state(buf, area, "Area too small for grid data");
            if area.height < 1 {
                return;
            }
        }

        let header_style = Style::default().fg(Color::White).bg(Color::Blue).bold();
        let header_cells = lines.iter().enumerate().map(|(i, line)| {
            let length_display = line
                .custom_length
                .map_or("(Scene)".to_string(), |len| format!("({:.1}b)", len));
            let speed_display = format!("x{:.1}", line.speed_factor);
            let text = format!("LINE {} {} {}", i + 1, length_display, speed_display);
            Cell::from(Line::from(text).alignment(Alignment::Center)).style(header_style)
        });
        let header = Row::new(header_cells).height(1).style(header_style);

        let padding_cells = std::iter::repeat_n(
            Cell::from("").style(Style::default().bg(Color::Reset)),
            num_lines,
        );
        let padding_row = Row::new(padding_cells).height(1);

        // Data Rows - Iterate over the *entire visible range*, not just max_frames
        let data_rows = (start_row..end_row).map(|frame_idx| {
            let cells = lines.iter().enumerate().map(|(col_idx, line)| {
                // Use area.width for column width calculation
                let col_width = if num_lines > 0 {
                    (area.width / num_lines as u16).max(6)
                } else {
                    area.width
                };

                let cell_data = GridCellData {
                    frame_idx,
                    col_idx,
                    line: Some(line),
                    col_width,
                };

                cell_renderer.render(cell_data, self.app)
            });
            Row::new(cells).height(1)
        });

        // Combine Rows: Header, Padding, Data
        let combined_rows = once(padding_row).chain(data_rows);

        // Calculate Column Widths using area.width
        let col_width_constraint = if num_lines > 0 {
            Constraint::Min((area.width / num_lines as u16).max(6))
        } else {
            Constraint::Min(area.width)
        };
        let widths: Vec<Constraint> =
            std::iter::repeat_n(col_width_constraint, num_lines).collect();

        // Create Table
        let table = Table::new(combined_rows, &widths)
            .header(header)
            .column_spacing(1)
            .block(Block::default());

        ratatui::widgets::Widget::render(table, area, buf);
    }
}
