use crate::app::App;
use crate::components::Component;
use crate::components::logs::LogLevel;
use crate::event::{AppEvent, Event};
use crate::components::markdownparser::parse_markdown;
use bubocorelib::shared_types::DeviceKind;
use chrono::{DateTime, Local};
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table, Wrap},
};

/// Enum representing the different navigation tiles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavigationTile {
    Editor,
    Scene,
    Options,
    Help,
    Devices,
    Logs,
    SaveLoad,
    Empty,
}

impl NavigationTile {
    /// Get the corresponding word for each tile
    pub fn get_letter(&self) -> &str {
        match self {
            NavigationTile::Editor => "(E)ditor",
            NavigationTile::Scene => "(S)cene",
            NavigationTile::Options => "(O)ptions",
            NavigationTile::Help => "(H)elp",
            NavigationTile::Devices => "(D)evices",
            NavigationTile::Logs => "(L)ogs",
            NavigationTile::SaveLoad => "(F)iles",
            NavigationTile::Empty => "",
        }
    }

    /// Get the corresponding description for each tile
    pub fn get_description(&self) -> &str {
        match self {
            NavigationTile::Editor => " Edit and run code",
            NavigationTile::Scene => " Create and edit scenes",
            NavigationTile::Options => " Manage application settings",
            NavigationTile::Help => " Access BuboCoreTUI documentation",
            NavigationTile::Devices => " Manage connected devices",
            NavigationTile::Logs => " View application/server logs",
            NavigationTile::SaveLoad => " Manage project files",
            NavigationTile::Empty => "",
        }
    }

    /// Get the corresponding tile for a given character
    pub fn from_char(c: char) -> Self {
        match c.to_ascii_uppercase() {
            'E' => NavigationTile::Editor,
            'S' => NavigationTile::Scene,
            'O' => NavigationTile::Options,
            'H' => NavigationTile::Help,
            'D' => NavigationTile::Devices,
            'L' => NavigationTile::Logs,
            'F' => NavigationTile::SaveLoad,
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
        grid[0][1] = NavigationTile::Scene;
        grid[1][0] = NavigationTile::Options;
        grid[1][1] = NavigationTile::Devices;
        grid[2][0] = NavigationTile::Logs;
        grid[2][1] = NavigationTile::SaveLoad;
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
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        match key_event.code {
            // Move the cursor up on the grid
            KeyCode::Up => {
                app.events
                    .sender
                    .send(Event::App(AppEvent::MoveNavigationCursor((-1, 0))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            // Move the cursor down on the grid
            KeyCode::Down => {
                app.events
                    .sender
                    .send(Event::App(AppEvent::MoveNavigationCursor((1, 0))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            // Move the cursor left on the grid
            KeyCode::Left => {
                app.events
                    .sender
                    .send(Event::App(AppEvent::MoveNavigationCursor((0, -1))))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                Ok(true)
            }
            // Move the cursor right on the grid
            KeyCode::Right => {
                app.events
                    .sender
                    .send(Event::App(AppEvent::MoveNavigationCursor((0, 1))))
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
                        NavigationTile::Scene => Some(AppEvent::SwitchToGrid),
                        NavigationTile::Options => Some(AppEvent::SwitchToOptions),
                        NavigationTile::Help => Some(AppEvent::SwitchToHelp),
                        NavigationTile::Devices => Some(AppEvent::SwitchToDevices),
                        NavigationTile::Logs => Some(AppEvent::SwitchToLogs),
                        NavigationTile::SaveLoad => Some(AppEvent::SwitchToSaveLoad),
                        NavigationTile::Empty => None,
                    };
                    if let Some(app_event) = app_event_to_send {
                        app.events
                            .sender
                            .send(Event::App(app_event))
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
                        NavigationTile::Scene => Some(AppEvent::SwitchToGrid),
                        NavigationTile::Options => Some(AppEvent::SwitchToOptions),
                        NavigationTile::Help => Some(AppEvent::SwitchToHelp),
                        NavigationTile::Devices => Some(AppEvent::SwitchToDevices),
                        NavigationTile::Logs => Some(AppEvent::SwitchToLogs),
                        NavigationTile::SaveLoad => Some(AppEvent::SwitchToSaveLoad),
                        NavigationTile::Empty => None,
                    };
                    if let Some(app_event) = app_event_to_send {
                        app.events
                            .sender
                            .send(Event::App(app_event))
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
        let grid_constraints = std::iter::repeat(Constraint::Percentage(50))
            .take(2)
            .collect::<Vec<_>>();
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
                    let base_fg_color = if is_cursor {
                        Color::White
                    } else {
                        Color::White
                    };
                    let bold_style = Style::default()
                        .fg(base_fg_color)
                        .add_modifier(Modifier::BOLD);
                    let normal_style = Style::default().fg(base_fg_color);
                    Line::from(vec![
                        Span::styled(prefix, normal_style),
                        Span::styled(letter_slice.to_string(), bold_style),
                        Span::styled(suffix, normal_style),
                        Span::styled(rest_slice.to_string(), normal_style),
                    ])
                    .alignment(Alignment::Center)
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
        let table = Table::new(grid_rows, &grid_constraints).column_spacing(1);
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
            .style(Style::default().fg(Color::White))
            .title(format!(" Info ({}) ", current_tile.get_letter()))
            .padding(Padding {
                left: 1,
                right: 0,
                top: 1,
                bottom: 0,
            });

        let inner_info_area = info_block.inner(info_area);
        frame.render_widget(info_block, info_area);

        let info_text = match current_tile {
            NavigationTile::Editor => {
                let label_style = Style::default().fg(Color::Yellow);
                let value_style = Style::default().fg(Color::White);

                // --- Top Info Block (Single Line) ---
                let line_idx = app.editor.active_line.line_index;
                let frame_idx = app.editor.active_line.frame_index;
                
                let script_status_text = app.editor.scene.as_ref()
                    .and_then(|s| s.lines.get(line_idx))
                    .map(|l| l.is_frame_enabled(frame_idx))
                    .map(|enabled| if enabled { "[Enabled]" } else { "[Disabled]" })
                    .unwrap_or("[No Scene/Frame]");
                let status_style = if script_status_text == "[Enabled]" {
                    Style::default().fg(Color::Green)
                } else if script_status_text == "[Disabled]" {
                     Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let top_line = Line::from(vec![
                    Span::styled("Active: ", label_style),
                    Span::styled(format!("Line {}, Frame {} ", line_idx, frame_idx), value_style),
                    Span::styled(script_status_text, status_style), // Add status here
                ]);

                // Render top info in its own paragraph with background
                let top_info_text = Text::from(vec![top_line]); // Only one line now
                let top_info_paragraph = Paragraph::new(top_info_text)
                    .style(Style::default().bg(Color::DarkGray).fg(Color::White)) // Use DarkGray bg
                    .alignment(Alignment::Left);
                frame.render_widget(top_info_paragraph, Rect { 
                    x: inner_info_area.x,
                    y: inner_info_area.y,
                    width: inner_info_area.width,
                    height: 1, // Height for the single line
                });

                // --- Editor Content Preview ---
                // Calculate remaining area for text editor preview
                let editor_preview_area = Rect {
                    x: inner_info_area.x,
                    y: inner_info_area.y + 1, // Start directly below the top info block
                    width: inner_info_area.width,
                    height: inner_info_area.height.saturating_sub(1), // Adjust height
                };

                let editor_lines: Vec<Line> = app
                    .editor
                    .textarea
                    .lines()
                    .iter()
                    .take(editor_preview_area.height as usize) // Only take lines that fit
                    .map(|line_str| Line::from(line_str.clone()))
                    .collect();
                // Render editor preview directly (no wrapping needed here)
                frame.render_widget(Paragraph::new(Text::from(editor_lines)), editor_preview_area);
                // Return empty text as we've rendered directly
                Text::from("") 
            }
            NavigationTile::Logs => {
                let available_height = inner_info_area.height;
                let log_lines: Vec<Line> = app
                    .logs
                    .iter()
                    .rev()
                    .take(available_height as usize) // Take full height now
                    .rev()
                    .map(|log_entry| {
                        let time_str = log_entry.timestamp.format("%H:%M:%S").to_string();
                        let (level_str, level_style) = match log_entry.level {
                            LogLevel::Info => {
                                (" INFO ", Style::default().fg(Color::Black).bg(Color::White))
                            }
                            LogLevel::Warn => (
                                " WARN ",
                                Style::default().fg(Color::Black).bg(Color::Yellow),
                            ),
                            LogLevel::Error => (
                                " ERROR ",
                                Style::default()
                                    .fg(Color::White)
                                    .bg(Color::Red)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            LogLevel::Debug => (
                                " DEBUG ",
                                Style::default().fg(Color::White).bg(Color::Magenta),
                            ),
                        };

                        Line::from(vec![
                            Span::styled(
                                time_str,
                                Style::default().bg(Color::White).fg(Color::Black),
                            ),
                            Span::styled(level_str, level_style),
                            Span::raw(" "),
                            Span::raw(&log_entry.message),
                        ])
                    })
                    .collect();
                // Remove total count and follow status header
                Text::from(log_lines)
            }
            NavigationTile::Scene => {
                let available_height = inner_info_area.height;
                let available_width = inner_info_area.width;

                if let Some(scene) = &app.editor.scene {
                    if scene.lines.is_empty() {
                        Text::from("scene has no lines.")
                    } else {
                        // 4 chars per line. Format: '[ ] G ' (Begin, End, Status/Playhead, Space)
                        let max_lines_to_show = (available_width / 4).max(1) as usize;
                        let max_frames_to_show = available_height.saturating_sub(1) as usize; // Minus header line

                        let mut lines = Vec::new();

                        // Header: S1  S2  S3  ...
                        let header_spans: Vec<Span> = (0..scene.lines.len().min(max_lines_to_show))
                            .map(|i| {
                                Span::styled(
                                    format!("S{}  ", i + 1),
                                    Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::BOLD),
                                )
                            })
                            .collect();
                        lines.push(Line::from(header_spans));

                        // Grid content
                        let max_frames_in_scene = scene
                            .lines
                            .iter()
                            .map(|s| s.frames.len())
                            .max()
                            .unwrap_or(0);

                        for frame_idx in 0..max_frames_in_scene.min(max_frames_to_show) {
                            let mut frame_spans = Vec::new();
                            for line_idx in 0..scene.lines.len().min(max_lines_to_show) {
                                if let Some(line) = scene.lines.get(line_idx) {
                                    if frame_idx < line.frames.len() {
                                        let is_enabled = line.is_frame_enabled(frame_idx);
                                        let is_current = app
                                            .server
                                            .current_frame_positions
                                            .as_ref()
                                            .and_then(|p| p.get(line_idx))
                                            .map_or(false, |&current| current == frame_idx);

                                        // Range Marker (like grid.rs)
                                        let should_draw_range_bar =
                                            if let Some(start) = line.start_frame {
                                                if let Some(end) = line.end_frame {
                                                    frame_idx >= start && frame_idx <= end
                                                } else {
                                                    frame_idx >= start
                                                }
                                            } else {
                                                if let Some(end) = line.end_frame {
                                                    frame_idx <= end
                                                } else {
                                                    false
                                                }
                                            };
                                        let range_marker =
                                            if should_draw_range_bar { "▌" } else { " " };

                                        // Playhead Marker
                                        let playhead_marker = if is_current { "│" } else { " " };
                                        let playhead_style = Style::default().fg(Color::White);

                                        // Status Bar
                                        let status_char = '█';
                                        let status_color =
                                            if is_enabled { Color::Green } else { Color::Red };
                                        let status_style = Style::default().fg(status_color);

                                        // Assemble the cell: R P S Space
                                        frame_spans.push(Span::raw(range_marker)); // Default style
                                        frame_spans
                                            .push(Span::styled(playhead_marker, playhead_style));
                                        frame_spans.push(Span::styled(
                                            status_char.to_string(),
                                            status_style,
                                        ));
                                        frame_spans.push(Span::raw(" ")); // Padding
                                    } else {
                                        frame_spans.push(Span::raw("    ")); // Empty slot with 4 spaces
                                    }
                                } else {
                                    frame_spans.push(Span::raw("    "));
                                }
                            }
                            lines.push(Line::from(frame_spans));
                        }
                        Text::from(lines)
                    }
                } else {
                    Text::from("No scene loaded.")
                }
            }
            NavigationTile::Options => {
                let mut lines = Vec::new();
                let label_style = Style::default().fg(Color::Yellow);
                let value_style = Style::default().fg(Color::White);

                // Tempo
                let tempo = app.server.link.session_state.tempo(); // Use session_state.tempo()
                lines.push(Line::from(vec![
                    Span::styled("Tempo: ", label_style),
                    Span::styled(format!("{:.2}", tempo), value_style),
                    Span::styled(" BPM", Style::default().fg(Color::DarkGray)),
                ]));

                // Quantum
                let quantum = app.server.link.quantum;
                lines.push(Line::from(vec![
                    Span::styled("Quantum: ", label_style),
                    Span::styled(quantum.to_string(), value_style),
                ]));

                // Phase Bar Setting
                lines.push(Line::from(vec![
                    Span::styled("Show Phase Bar: ", label_style),
                    Span::styled(if app.settings.show_phase_bar { "Yes" } else { "No" }, value_style),
                ]));

                // Placeholder for more settings
                lines.push(Line::from(""));

                Text::from(lines)
            }
            NavigationTile::Devices => {
                let mut lines = Vec::new();
                let label_style = Style::default().fg(Color::Yellow);
                let value_style = Style::default().fg(Color::White);
                let device_style = Style::default().fg(Color::Cyan);
                let type_style = Style::default().fg(Color::DarkGray);

                let midi_count = app.server.devices.iter().filter(|d| d.kind == DeviceKind::Midi).count();
                let osc_count = app.server.devices.iter().filter(|d| d.kind == DeviceKind::Osc).count();

                // Add Server Connection Status
                let connection_status = if app.server.is_connected {
                    Span::styled("Connected", Style::default().fg(Color::Green))
                } else if app.server.is_connecting {
                    Span::styled("Connecting...", Style::default().fg(Color::Yellow))
                } else {
                    Span::styled("Disconnected", Style::default().fg(Color::Red))
                };
                lines.push(Line::from(vec![
                    Span::styled("Server: ", label_style),
                    connection_status,
                ]));
                lines.push(Line::from("")); // Spacer

                lines.push(Line::from(vec![
                    Span::styled("MIDI Devices: ", label_style),
                    Span::styled(midi_count.to_string(), value_style),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("OSC Devices: ", label_style),
                    Span::styled(osc_count.to_string(), value_style),
                ]));
                lines.push(Line::from("")); // Spacer

                lines.push(Line::from(Span::styled("Discovered (Top 5):", label_style)));

                for device in app.server.devices.iter().take(5) {
                    let type_label = match device.kind {
                        DeviceKind::Midi => "[MIDI]",
                        DeviceKind::Osc => " [OSC]",
                        // Add a catch-all arm for safety
                        _ => " [?]", 
                    };
                    // Add slot info if assigned
                    let slot_info = app.interface.components.devices_state.slot_assignments
                        .iter()
                        .find(|(_, name)| *name == &device.name)
                        .map(|(id, _)| format!(" (Slot {})", id))
                        .unwrap_or_default();

                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}", device.name), device_style),
                        Span::styled(format!(" {}", type_label), type_style),
                        Span::styled(slot_info, Style::default().fg(Color::Yellow)), // Show slot in Yellow
                    ]));
                }
                if app.server.devices.len() > 5 {
                    lines.push(Line::from(Span::styled("  ...", Style::default().fg(Color::DarkGray))));
                }
                if app.server.devices.is_empty() {
                    lines.push(Line::from(Span::styled("  None found", Style::default().fg(Color::DarkGray))));
                }

                Text::from(lines)
            }
            NavigationTile::SaveLoad => {
                let state = &app.interface.components.save_load_state;
                let available_height = inner_info_area.height;

                let total_projects = state.projects.len();
                let mut info_lines = vec![Line::from(vec![
                    Span::styled("Total Projects: ", Style::default().fg(Color::Yellow)),
                    Span::raw(total_projects.to_string()),
                ])];

                if state.projects.is_empty() {
                    info_lines.push(Line::from("  No projects found."));
                } else {
                    let project_lines: Vec<Line> = state
                        .projects
                        .iter()
                        .take((available_height as usize).saturating_sub(1)) // Adjust take for header line
                        .enumerate()
                        .map(|(idx, (name, created_at, updated_at, tempo, line_count))| {
                            // Zebra striping
                            let bg_color = if idx % 2 == 0 { Color::Reset } else { Color::DarkGray };
                            let item_style = Style::default().bg(bg_color);

                            let mut spans = vec![Span::styled(format!("  {:<15}", name), Style::default().fg(Color::Cyan))]; 
 
                            let meta_style_label = Style::default().fg(Color::DarkGray);
                            let meta_style_value = Style::default().fg(Color::Gray);

                            // Tempo
                            spans.push(Span::styled(" T:", meta_style_label));
                            let tempo_str = tempo.map_or_else(|| "-".to_string(), |t| format!("{:.0}", t));
                            spans.push(Span::styled(format!("{:<4}", tempo_str), meta_style_value)); // Pad tempo

                            // Line Count
                            spans.push(Span::styled(" L:", meta_style_label));
                            let lines_str = line_count.map_or_else(|| "-".to_string(), |lc| lc.to_string());
                            spans.push(Span::styled(format!("{:<3}", lines_str), meta_style_value)); // Pad line count

                            let time_style = Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC);
                            let time_format = "%y-%m-%d %H:%M"; // Shorter format

                            if let Some(updated) = updated_at {
                                let local_updated: DateTime<Local> = (*updated).into();
                                spans.push(Span::styled(format!(" (S: {})", local_updated.format(time_format)), time_style));
                            } else if let Some(created) = created_at {
                                let local_created: DateTime<Local> = (*created).into();
                                spans.push(Span::styled(format!(" (C: {})", local_created.format(time_format)), time_style));
                            }
                            Line::from(spans).style(item_style) // Apply style to the whole line
                        })
                        .collect();
                    info_lines.extend(project_lines);
                }
                Text::from(info_lines)
            }
            NavigationTile::Help => {
                if let Some(help_state) = &app.interface.components.help_state {
                    if let Some(welcome_content) = help_state.contents.get(0) {
                        parse_markdown(welcome_content)
                    } else {
                        Text::from("Welcome section content is empty or missing.")
                    }
                } else {
                    Text::from("Help content not loaded yet. Press (H) to view.")
                }
            }
            _ => Text::from("Placeholder"),
        };

        let info_content = Paragraph::new(info_text)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(info_content, inner_info_area);
    }
}
