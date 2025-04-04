use crate::App;
use crate::components::Component;
use crate::event::{AppEvent, Event};
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, BorderType, Cell, Paragraph, Row, Table, Wrap},
};

/// Enum representing the different navigation tiles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavigationTile {
    Editor,
    Grid,
    Options,
    Help,
    Devices,
    Logs,
    Files,
    Empty,
}

impl NavigationTile {

    /// Get the corresponding word for each tile
    pub fn get_letter(&self) -> &str {
        match self {
            NavigationTile::Editor => "(E)ditor",
            NavigationTile::Grid => "(G)rid",
            NavigationTile::Options => "(O)ptions",
            NavigationTile::Help => "(H)elp",
            NavigationTile::Devices => "(D)evices",
            NavigationTile::Logs => "(L)ogs",
            NavigationTile::Files => "(F)iles",
            NavigationTile::Empty => "",
        }
    }

    /// Get the corresponding description for each tile
    pub fn get_description(&self) -> &str {
        match self {
            NavigationTile::Editor => " Edit and run code",
            NavigationTile::Grid => " Create and edit patterns",
            NavigationTile::Options => " Manage application settings",
            NavigationTile::Help => " Access BuboCoreTUI documentation",
            NavigationTile::Devices => " Manage connected devices",
            NavigationTile::Logs => " View application/server logs",
            NavigationTile::Files => " Manage project files",
            NavigationTile::Empty => "",
        }
    }

    /// Get the corresponding tile for a given character
    pub fn from_char(c: char) -> Self {
        match c.to_ascii_uppercase() {
            'E' => NavigationTile::Editor,
            'G' => NavigationTile::Grid,
            'O' => NavigationTile::Options,
            'H' => NavigationTile::Help,
            'D' => NavigationTile::Devices,
            'L' => NavigationTile::Logs,
            'F' => NavigationTile::Files,
            _ => NavigationTile::Empty,
        }
    }

}

pub struct NavigationComponent;

impl NavigationComponent {

    pub fn new() -> Self {
        Self {}
    }

    /// Get the grid of navigation tiles
    fn get_grid() -> [[NavigationTile; 2]; 6] {
        let mut grid = [[NavigationTile::Empty; 2]; 6];
        grid[0][0] = NavigationTile::Editor;
        grid[0][1] = NavigationTile::Grid;
        grid[1][0] = NavigationTile::Options;
        grid[1][1] = NavigationTile::Devices;
        grid[2][0] = NavigationTile::Logs;
        grid[2][1] = NavigationTile::Files;
        grid[3][0] = NavigationTile::Help;
        grid
    }

    /// Get the tile at a given position
    fn get_tile_at(cursor: (usize, usize)) -> NavigationTile {
        let grid = Self::get_grid();
        grid[cursor.0][cursor.1]
    }
}

impl Component for NavigationComponent {

    /// Function called before drawing the component
    /// 
    /// Unused for this component
    /// 
    /// # Arguments
    /// 
    /// * `app` - The application state
    /// 
    /// # Returns
    /// 
    fn before_draw(&mut self, _app: &mut App) -> EyreResult<()> {
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        match key_event.code {
            // Move the cursor up on the grid
            KeyCode::Up => {
                app.events.sender.send(Event::App(AppEvent::MoveNavigationCursor((-1, 0))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            // Move the cursor down on the grid
            KeyCode::Down => {
                app.events.sender.send(Event::App(AppEvent::MoveNavigationCursor((1, 0))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            // Move the cursor left on the grid
            KeyCode::Left => {
                app.events.sender.send(Event::App(AppEvent::MoveNavigationCursor((0, -1))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            // Move the cursor right on the grid
            KeyCode::Right => {
                app.events.sender.send(Event::App(AppEvent::MoveNavigationCursor((0, 1))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            // Select the tile at the cursor position (switch to the corresponding view)
            KeyCode::Enter => {
                let cursor = app.interface.components.navigation_cursor;
                let tile = Self::get_tile_at(cursor);
                if tile != NavigationTile::Empty {
                    let app_event_to_send = match tile {
                        NavigationTile::Editor => Some(AppEvent::SwitchToEditor),
                        NavigationTile::Grid => Some(AppEvent::SwitchToGrid),
                        NavigationTile::Options => Some(AppEvent::SwitchToOptions),
                        NavigationTile::Help => Some(AppEvent::SwitchToHelp),
                        NavigationTile::Devices => Some(AppEvent::SwitchToDevices),
                        NavigationTile::Logs => Some(AppEvent::SwitchToLogs),
                        NavigationTile::Files => Some(AppEvent::SwitchToFiles),
                        NavigationTile::Empty => None,
                    };
                    if let Some(app_event) = app_event_to_send {
                         app.events.sender.send(Event::App(app_event))
                            .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                    }
                }
                Ok(true)
            }
            // Select the tile using a character (switch to the corresponding view)
            KeyCode::Char(c) => {
                let tile = NavigationTile::from_char(c);
                if tile != NavigationTile::Empty {
                    let app_event_to_send = match tile {
                        NavigationTile::Editor => Some(AppEvent::SwitchToEditor),
                        NavigationTile::Grid => Some(AppEvent::SwitchToGrid),
                        NavigationTile::Options => Some(AppEvent::SwitchToOptions),
                        NavigationTile::Help => Some(AppEvent::SwitchToHelp),
                        NavigationTile::Devices => Some(AppEvent::SwitchToDevices),
                        NavigationTile::Logs => Some(AppEvent::SwitchToLogs),
                        NavigationTile::Files => Some(AppEvent::SwitchToFiles),
                        NavigationTile::Empty => None,
                    };
                    if let Some(app_event) = app_event_to_send {
                         app.events.sender.send(Event::App(app_event))
                            .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Function called to draw the component
    /// 
    /// # Arguments
    /// 
    /// * `app` - The application state
    /// * `frame` - The frame to draw on
    /// * `area` - The area to draw on
    /// 
    /// # Returns
    /// 
    /// * `EyreResult<()>` - The result of the draw operation
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let cursor = app.interface.components.navigation_cursor;
        let grid = Self::get_grid();

        // Main block where the navigation grid is drawn
        let navigation_block = Block::default()
            .title(" Navigation ")
            .border_type(BorderType::Thick)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        let inner_area = navigation_block.inner(area);
        frame.render_widget(navigation_block, area);

        // Split the main area into two parts, vertically
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(inner_area);
        
        let left_area = main_chunks[0];
        let right_area = main_chunks[1];

        // Map block where the navigation grid is drawn
        let map_block = Block::default()
            .border_type(BorderType::Thick)
            .borders(Borders::ALL);
        let inner_map_area = map_block.inner(left_area);
        frame.render_widget(map_block, left_area);

        // Calculate the height of each row in the grid
        let row_height = (inner_map_area.height / 6).max(1);

        // Create the constraints for the grid
        let grid_constraints = std::iter::repeat(Constraint::Percentage(50)).take(2).collect::<Vec<_>>();
        let mut grid_rows = Vec::new();
        for r in 0..6 {
            let mut cells = Vec::new();
            for c in 0..2 {
                let tile = grid[r][c];
                let is_cursor = (r, c) == cursor;
                let full_name_str = tile.get_letter(); // Get the &'static str

                // Create the content line first
                let content_line = if !full_name_str.is_empty() {
                    let full_name_owned = full_name_str.to_string();
                    let prefix = "(";
                    let letter_slice = full_name_owned.get(1..2).unwrap_or("");
                    let suffix = ")";
                    let rest_slice = full_name_owned.get(3..).unwrap_or("");
                    let base_fg_color = if is_cursor { Color::White } else { Color::White };
                    let bold_style = Style::default().fg(base_fg_color).add_modifier(Modifier::BOLD);
                    let normal_style = Style::default().fg(base_fg_color);
                    Line::from(vec![
                        Span::styled(prefix, normal_style),
                        Span::styled(letter_slice.to_string(), bold_style),
                        Span::styled(suffix, normal_style),
                        Span::styled(rest_slice.to_string(), normal_style),
                    ]).alignment(Alignment::Center)
                } else {
                    Line::from("   ").alignment(Alignment::Center)
                };

                // Calculate vertical padding
                let text_height = 1; // Our content is 1 line
                let top_padding = row_height.saturating_sub(text_height) / 2;
                let bottom_padding = row_height.saturating_sub(text_height) - top_padding;

                // Build lines with padding
                let mut lines = Vec::with_capacity(row_height as usize);
                for _ in 0..top_padding {
                    lines.push(Line::from(""));
                }
                lines.push(content_line);
                for _ in 0..bottom_padding {
                    lines.push(Line::from(""));
                }
                let cell_text = Text::from(lines);

                // Determine the cell's overall style (mainly for background)
                let cell_style = if is_cursor {
                    Style::default().bg(Color::Blue)
                } else if !full_name_str.is_empty() {
                    Style::default()
                } else {
                    Style::default().fg(Color::White)
                };

                cells.push(Cell::from(cell_text).style(cell_style));
            }
            // Use the pre-calculated row_height
            grid_rows.push(Row::new(cells).height(row_height));
        }
        let table = Table::new(grid_rows, &grid_constraints)
            .column_spacing(1);
        frame.render_widget(table, inner_map_area);

        // Split the right area into two parts, vertically
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)].as_ref())
            .split(right_area);
        
        let description_area = right_chunks[0];
        let info_area = right_chunks[1];

        // 6. Render Description block in the top-right part
        let current_tile = Self::get_tile_at(cursor);
        let description_block = Block::default()
            .border_type(BorderType::Thick)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));
        let description_text = Text::from(current_tile.get_description());
        let description_content = Paragraph::new(description_text)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center)
            .block(description_block);
        frame.render_widget(description_content, description_area);

        // 7. Render Info block in the bottom-right part
        let info_block = Block::default()
            .border_type(BorderType::Thick)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        let info_content = Paragraph::new(Text::from("Placeholder"))
            .alignment(Alignment::Left)
            .block(info_block);
        frame.render_widget(info_content, info_area);
    }
}

