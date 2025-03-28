use crate::App;
use crate::components::Component;
use crate::event::AppEvent;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::error::Error;
use tui_big_text::{BigText, PixelSize};
use tui_textarea::TextArea;

pub struct ConnectionState {
    pub ip_input: TextArea<'static>,
    pub port_input: TextArea<'static>,
    pub focus: ConnectionField,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConnectionField {
    IpAddress,
    Port,
}

impl ConnectionState {
    pub fn new(initial_ip: &str, initial_port: u16) -> Self {
        let mut ip_input = TextArea::new(vec![initial_ip.to_string()]);
        ip_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("IP Address")
                .style(Style::default().fg(Color::Blue)),
        );

        let mut port_input = TextArea::new(vec![initial_port.to_string()]);
        port_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Port")
                .style(Style::default().fg(Color::Blue)),
        );

        Self {
            ip_input,
            port_input,
            focus: ConnectionField::IpAddress,
        }
    }
    pub fn validate_ip(&self) -> Result<(), String> {
        let ip = self.get_ip();
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return Err("IP must have 4 octets (xxx.xxx.xxx.xxx)".to_string());
        }
        for part in parts {
            match part.parse::<u8>() {
                Ok(_) => {}
                Err(_) => return Err("Each octet must be a number between 0-255".to_string()),
            }
        }
        Ok(())
    }

    pub fn validate_port(&self) -> Result<u16, String> {
        let port_str = self.port_input.lines().join("");
        match port_str.parse::<u16>() {
            Ok(port) => {
                if port == 0 {
                    Err("Port cannot be 0".to_string())
                } else {
                    Ok(port)
                }
            }
            Err(_) => Err("Port must be a valid number between 1-65535".to_string()),
        }
    }

    pub fn next_field(&mut self) {
        self.focus = match self.focus {
            ConnectionField::IpAddress => ConnectionField::Port,
            ConnectionField::Port => ConnectionField::IpAddress,
        };
        self.update_focus_style();
    }

    pub fn update_focus_style(&mut self) {
        // Reset styles
        self.ip_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("IP Address")
                .style(Style::default().fg(Color::Blue)),
        );

        self.port_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Port")
                .style(Style::default().fg(Color::Blue)),
        );

        // Set focused style
        match self.focus {
            ConnectionField::IpAddress => {
                self.ip_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("IP Address")
                        .style(
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                );
            }
            ConnectionField::Port => {
                self.port_input.set_block(
                    Block::default().borders(Borders::ALL).title("Port").style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
            }
        }
    }

    pub fn get_ip(&self) -> String {
        self.ip_input.lines().join("")
    }

    pub fn get_port(&self) -> Result<u16, std::num::ParseIntError> {
        self.port_input.lines().join("").parse::<u16>()
    }
}

pub struct SplashComponent;

impl SplashComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for SplashComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> Result<bool, Box<dyn Error + 'static>> {
        if app.interface.components.connection_state.is_none() {
            app.init_connection_state();
        }

        if let Some(connection_state) = &mut app.interface.components.connection_state {
            match key_event.code {
                KeyCode::Enter => match connection_state.validate_ip() {
                    Ok(_) => match connection_state.validate_port() {
                        Ok(port) => {
                            let ip = connection_state.get_ip();
                            match app.server.network.update_connection_info(ip.clone(), port) {
                                Ok(_) => {
                                    app.interface.components.bottom_message =
                                        format!("Connecting to {}:{}...", ip, port);
                                    app.events.send(AppEvent::SwitchToEditor);
                                    return Ok(true);
                                }
                                Err(e) => {
                                    app.interface.components.bottom_message =
                                        format!("Connection error: {}", e);
                                    return Ok(true);
                                }
                            }
                        }
                        Err(msg) => {
                            app.interface.components.bottom_message = msg;
                            return Ok(true);
                        }
                    },
                    Err(msg) => {
                        app.interface.components.bottom_message = msg;
                        return Ok(true);
                    }
                },
                KeyCode::Tab => {
                    connection_state.next_field();
                    connection_state.update_focus_style();
                    Ok(true)
                }
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.events.send(AppEvent::Quit);
                    Ok(true)
                }
                KeyCode::Backspace | KeyCode::Delete => {
                    // Handle backspace and delete keys
                    match connection_state.focus {
                        ConnectionField::IpAddress => {
                            connection_state.ip_input.input(key_event);
                        }
                        ConnectionField::Port => {
                            connection_state.port_input.input(key_event);
                        }
                    }
                    Ok(true)
                }
                KeyCode::Char(c) => {
                    // Handle character input
                    match connection_state.focus {
                        ConnectionField::IpAddress => {
                            connection_state.ip_input.input(key_event);
                        }
                        ConnectionField::Port => {
                            // Only allow digits for port
                            if c.is_ascii_digit() {
                                connection_state.port_input.input(key_event);
                            }
                        }
                    }
                    Ok(true)
                }
                _ => {
                    // Handle other keys (arrows, etc)
                    match connection_state.focus {
                        ConnectionField::IpAddress => {
                            connection_state.ip_input.input(key_event);
                        }
                        ConnectionField::Port => {
                            connection_state.port_input.input(key_event);
                        }
                    }
                    Ok(true)
                }
            }
        } else {
            Ok(false)
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        // Create a vertical layout with proper space allocation
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Padding
                Constraint::Length(12), // Titre
                Constraint::Length(3),  // IP field
                Constraint::Length(3),  // Port field
                Constraint::Length(2),  // Instructions
                Constraint::Min(1),     // Padding
                Constraint::Length(1),  // Ligne statut
            ])
            .split(area);

        let big_text = BigText::builder()
            .centered()
            .pixel_size(PixelSize::Full)
            .style(Style::default().fg(Color::Cyan))
            .lines(vec!["BuboCore".into()])
            .build();

        frame.render_widget(big_text, vertical_layout[1]);

        if let Some(connection_state) = &app.interface.components.connection_state {
            let ip_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ])
                .split(vertical_layout[2]);

            frame.render_widget(&connection_state.ip_input, ip_layout[1]);

            let port_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ])
                .split(vertical_layout[3]);

            frame.render_widget(&connection_state.port_input, port_layout[1]);

            let instructions =
                Paragraph::new("Press TAB to switch fields, ENTER to connect, Ctrl+C to quit")
                    .style(Style::default().fg(Color::White))
                    .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(instructions, vertical_layout[4]);
        }
    }
}
