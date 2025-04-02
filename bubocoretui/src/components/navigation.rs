use crate::App;
use crate::components::Component;
use crate::event::{AppEvent, Event};
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, BorderType, Cell, Paragraph, Row, Table, Wrap},
};
use std::error::Error;

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
    pub fn get_letter(&self) -> char {
        match self {
            NavigationTile::Editor => 'E',
            NavigationTile::Grid => 'G',
            NavigationTile::Options => 'O',
            NavigationTile::Help => 'H',
            NavigationTile::Devices => 'D',
            NavigationTile::Logs => 'L',
            NavigationTile::Files => 'F',
            NavigationTile::Empty => ' ',
        }
    }

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

    pub fn name(&self) -> &str {
        match self {
            NavigationTile::Editor => "Editor Blabla",
            NavigationTile::Grid => "Grid",
            NavigationTile::Options => "Options",
            NavigationTile::Help => "Help",
            NavigationTile::Devices => "Devices",
            NavigationTile::Logs => "Logs",
            NavigationTile::Files => "Files",
            NavigationTile::Empty => "",
        }
    }
}

pub struct NavigationComponent;

impl NavigationComponent {
    pub fn new() -> Self {
        Self {}
    }

    fn get_grid() -> [[NavigationTile; 5]; 5] {
        let mut grid = [[NavigationTile::Empty; 5]; 5];
        grid[0][0] = NavigationTile::Devices;
        grid[0][1] = NavigationTile::Options;
        grid[0][2] = NavigationTile::Logs;
        grid[1][2] = NavigationTile::Grid;
        grid[2][0] = NavigationTile::Editor;
        grid[2][2] = NavigationTile::Files;
        grid[2][4] = NavigationTile::Help;
        grid
    }

    fn get_tile_at(cursor: (usize, usize)) -> NavigationTile {
        let grid = Self::get_grid();
        grid[cursor.0][cursor.1]
    }
}

impl Component for NavigationComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        match key_event.code {
            KeyCode::Up => {
                app.events.sender.send(Event::App(AppEvent::MoveNavigationCursor((-1, 0))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            KeyCode::Down => {
                app.events.sender.send(Event::App(AppEvent::MoveNavigationCursor((1, 0))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            KeyCode::Left => {
                app.events.sender.send(Event::App(AppEvent::MoveNavigationCursor((0, -1))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            KeyCode::Right => {
                app.events.sender.send(Event::App(AppEvent::MoveNavigationCursor((0, 1))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
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
                Ok(true) // Consume Enter key
            }
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
                Ok(true) // Consume char input
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let cursor = app.interface.components.navigation_cursor;
        let grid = Self::get_grid();

        // 1. Create the main outer block for the entire navigation view
        let navigation_block = Block::default()
            .title(" Navigation (ESC to exit) ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Cyan));
        // Get the area inside the main block's borders
        let inner_area = navigation_block.inner(area);
        // Render the main block first, covering the whole area
        frame.render_widget(navigation_block, area);

        // 2. Split the INNER area horizontally
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            // Use the inner_area for the split
            .split(inner_area);
        
        let left_area = main_chunks[0];
        let right_area = main_chunks[1];

        // 3. Create and render the Map block in the left area
        let map_block = Block::default()
            .title(" Map ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_map_area = map_block.inner(left_area); // Area inside the map block
        frame.render_widget(map_block, left_area);

        // 4. Render the Grid Table inside the Map block's inner area
        let grid_constraints = std::iter::repeat(Constraint::Percentage(20)).take(5).collect::<Vec<_>>();
        let mut grid_rows = Vec::new();
        for r in 0..5 {
            let mut cells = Vec::new();
            for c in 0..5 {
                let tile = grid[r][c];
                let is_cursor = (r, c) == cursor;
                let letter = tile.get_letter();
                let content_str = if letter != ' ' { format!(" {} ", letter) } else { "   ".to_string() };
                let style = if is_cursor {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else if letter != ' ' {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let cell_content = Text::from(content_str).alignment(Alignment::Center);
                cells.push(Cell::from(cell_content).style(style));
            }
            // Calculate height based on inner_map_area
            let row_height = (inner_map_area.height / 5).max(1); 
            grid_rows.push(Row::new(cells).height(row_height));
        }
        let table = Table::new(grid_rows, &grid_constraints)
            .column_spacing(1);
        // Render the table in the inner_map_area
        frame.render_widget(table, inner_map_area);

        // 5. Split the RIGHT part of the inner area vertically
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            // Use the right_area for the split
            .split(right_area);
        
        let description_area = right_chunks[0];
        let info_area = right_chunks[1];

        // 6. Render Description block in the top-right part
        let current_tile = Self::get_tile_at(cursor);
        let description_title = format!(" {} - {} ", current_tile.get_letter(), current_tile.name());
        let description_block = Block::default()
            .title(description_title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Green));
        let description_text = Text::from(current_tile.get_description());
        let description_content = Paragraph::new(description_text)
            .wrap(Wrap { trim: true })
            .block(description_block);
        frame.render_widget(description_content, description_area);

        // 7. Render Info block in the bottom-right part
        let info_block = Block::default()
            .title(" Info ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Magenta));
        let info_content = Paragraph::new(Text::from("General info, stats, or tips could go here."))
            .alignment(Alignment::Center)
            .block(info_block);
        frame.render_widget(info_content, info_area);
    }
}

