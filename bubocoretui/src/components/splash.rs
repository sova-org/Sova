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
    pub username_input: TextArea<'static>,
    pub focus: ConnectionField,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConnectionField {
    IpAddress,
    Port,
    Username,
}

impl ConnectionState {
    pub fn new(initial_ip: &str, initial_port: u16, initial_username: &str) -> Self {
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

        let mut username_input = TextArea::new(vec![initial_username.to_string()]);
        username_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Username")
                .style(Style::default().fg(Color::Blue)),
        );

        Self {
            ip_input,
            port_input,
            username_input,
            focus: ConnectionField::Username,
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

    pub fn validate_username(&self) -> Result<(), String> {
        let username = self.get_username();
        if username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err("Username must contain only letters, numbers, or hyphens".to_string());
        }
        Ok(())
    }

    pub fn next_field(&mut self) {
        self.focus = match self.focus {
            ConnectionField::Username => ConnectionField::IpAddress,
            ConnectionField::IpAddress => ConnectionField::Port,
            ConnectionField::Port => ConnectionField::Username,
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

        self.username_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Username")
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
            ConnectionField::Username => {
                self.username_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Username")
                        .style(
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

    pub fn get_username(&self) -> String {
        self.username_input.lines().join("")
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
        if app.server.connection_state.is_none() {
            app.init_connection_state();
        }

        if let Some(connection_state) = &mut app.server.connection_state {
            match key_event.code {
                KeyCode::Enter => {
                    match connection_state.validate_username() {
                        Ok(_) => match connection_state.validate_ip() {
                            Ok(_) => match connection_state.validate_port() {
                                Ok(port) => {
                                    let ip = connection_state.get_ip();
                                    let username = connection_state.get_username();
                                    match app
                                        .server
                                        .network
                                        .update_connection_info(ip.clone(), port, username.clone())
                                    {
                                        Ok(_) => {
                                            app.server.is_connecting = true;
                                            app.server.username = username.clone();
                                            app.interface.components.bottom_message = format!(
                                                "Attempting to connect to {}:{} as {}...",
                                                ip,
                                                port,
                                                username
                                            );
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
                        Err(msg) => {
                            app.interface.components.bottom_message = msg;
                            return Ok(true);
                        }
                    }
                }
                KeyCode::Tab => {
                    connection_state.next_field();
                    Ok(true)
                }
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.events.send(AppEvent::Quit);
                    Ok(true)
                }
                KeyCode::Backspace | KeyCode::Delete => {
                    match connection_state.focus {
                        ConnectionField::IpAddress => {
                            connection_state.ip_input.input(key_event);
                        }
                        ConnectionField::Port => {
                            connection_state.port_input.input(key_event);
                        }
                        ConnectionField::Username => {
                            connection_state.username_input.input(key_event);
                        }
                    }
                    Ok(true)
                }
                KeyCode::Char(c) => {
                    match connection_state.focus {
                        ConnectionField::IpAddress => {
                            if c.is_ascii_digit() || c == '.' {
                                connection_state.ip_input.input(key_event);
                            }
                        }
                        ConnectionField::Port => {
                            if c.is_ascii_digit() {
                                connection_state.port_input.input(key_event);
                            }
                        }
                        ConnectionField::Username => {
                            if c.is_alphanumeric() {
                                connection_state.username_input.input(key_event);
                            }
                        }
                    }
                    Ok(true)
                }
                _ => {
                    match connection_state.focus {
                        ConnectionField::IpAddress => {
                            connection_state.ip_input.input(key_event);
                        }
                        ConnectionField::Port => {
                            connection_state.port_input.input(key_event);
                        }
                        ConnectionField::Username => {
                            connection_state.username_input.input(key_event);
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
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(8),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

        let big_text = BigText::builder()
            .centered()
            .pixel_size(PixelSize::Sextant)
            .style(Style::default().fg(Color::Cyan))
            .lines(vec!["BuboCore".into()])
            .build();

        frame.render_widget(big_text, vertical_layout[1]);

        if let Some(connection_state) = &app.server.connection_state {
            let horizontal_center_layout = |area: Rect| {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(25),
                        Constraint::Percentage(50),
                        Constraint::Percentage(25),
                    ])
                    .split(area)[1]
            };

            let username_area = horizontal_center_layout(vertical_layout[2]);
            frame.render_widget(&connection_state.username_input, username_area);

            let ip_area = horizontal_center_layout(vertical_layout[3]);
            frame.render_widget(&connection_state.ip_input, ip_area);

            let port_area = horizontal_center_layout(vertical_layout[4]);
            frame.render_widget(&connection_state.port_input, port_area);

            let instructions =
                Paragraph::new("Press TAB to switch fields, ENTER to connect, Ctrl+C to quit")
                    .style(Style::default().fg(Color::White))
                    .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(instructions, vertical_layout[5]);
        }
    }
}
