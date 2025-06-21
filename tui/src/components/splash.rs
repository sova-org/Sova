use crate::app::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tui_big_text::{BigText, PixelSize};
use tui_textarea::TextArea;

/// Manages the state for connection input fields in the splash screen.
///
/// This struct holds the input fields for IP address, port, and username,
/// along with tracking which field currently has focus. Each input field
/// is implemented as a `TextArea` widget for text input functionality.
///
/// # Fields
///
/// * `ip_input` - Text input area for the IP address
/// * `port_input` - Text input area for the port number
/// * `username_input` - Text input area for the username
/// * `focus` - Indicates which input field currently has focus
pub struct ConnectionState {
    pub ip_input: TextArea<'static>,
    pub port_input: TextArea<'static>,
    pub username_input: TextArea<'static>,
    pub focus: ConnectionField,
}

#[derive(Debug, PartialEq, Clone, Copy)]
/// Represents the different input fields in the connection form.
///
/// This enum is used to track which field currently has focus in the connection
/// input form, allowing for proper navigation and input handling between the
/// different fields.
///
/// # Variants
///
/// * `IpAddress` - The IP address input field
/// * `Port` - The port number input field
/// * `Username` - The username input field
pub enum ConnectionField {
    IpAddress,
    Port,
    Username,
}

impl ConnectionState {
    pub fn new(initial_ip: &str, initial_port: u16, initial_username: &str) -> Self {
        let mut ip_input = TextArea::new(vec![initial_ip.to_string()]);

        // IP address input
        ip_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("IP Address")
                .border_type(BorderType::Thick)
                .style(Style::default().fg(Color::Green).fg(Color::White)),
        );

        // Port input
        let mut port_input = TextArea::new(vec![initial_port.to_string()]);
        port_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Port")
                .border_type(BorderType::Thick)
                .style(Style::default().fg(Color::Green).fg(Color::White)),
        );

        // Username selection
        let mut username_input = TextArea::new(vec![initial_username.to_string()]);
        username_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Username")
                .border_type(BorderType::Thick)
                .style(Style::default().fg(Color::Green).fg(Color::White)),
        );

        let mut state = Self {
            ip_input,
            port_input,
            username_input,
            focus: ConnectionField::Username,
        };
        state.update_focus_style();
        state
    }

    /// Validate the IP address
    ///
    /// Checks if the IP address is in the correct format (xxx.xxx.xxx.xxx) and that each octet
    /// is a valid number between 0-255.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>`:
    ///     - `Ok(())` if the IP address is valid
    ///     - `Err(String)` with an error message if the IP address is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// let state = ConnectionState::new("192.168.1.1", 8080, "user");
    /// assert!(state.validate_ip().is_ok());
    /// ```
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

    /// Validate the port number
    ///
    /// Checks if the port number is a valid u16 value between 1-65535.
    ///
    /// # Returns
    ///
    /// * `Result<u16, String>`:
    ///     - `Ok(u16)` containing the valid port number if validation succeeds
    ///     - `Err(String)` with an error message if the port is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// let state = ConnectionState::new("192.168.1.1", 8080, "user");
    /// assert!(state.validate_port().is_ok());
    /// ```
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

    /// Validate the username
    ///
    /// Checks if the username meets the following criteria:
    /// - Not empty
    /// - Contains only alphanumeric characters or hyphens
    ///
    /// # Returns
    ///
    /// * `Result<(), String>`:
    ///     - `Ok(())` if the username is valid
    ///     - `Err(String)` with an error message if the username is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// let state = ConnectionState::new("192.168.1.1", 8080, "user");
    /// assert!(state.validate_username().is_ok());
    /// ```
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

    pub fn previous_field(&mut self) {
        self.focus = match self.focus {
            ConnectionField::Username => ConnectionField::Port,
            ConnectionField::Port => ConnectionField::IpAddress,
            ConnectionField::IpAddress => ConnectionField::Username,
        };
        self.update_focus_style();
    }

    pub fn update_focus_style(&mut self) {
        // Reset styles
        self.ip_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("IP Address")
                .style(Style::default().fg(Color::White)),
        );

        self.port_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Port")
                .style(Style::default().fg(Color::White)),
        );

        self.username_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Username")
                .style(Style::default().fg(Color::White)),
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

/// A component that handles the initial connection screen of the application.
///
/// The `SplashComponent` manages the user interface for connecting to a server,
/// including input fields for IP address, port, and username. It provides
/// validation of connection parameters and handles the connection process.
///
/// This component is typically the first screen shown to users when they launch
/// the application, allowing them to establish their initial connection to a
/// BuboCore server.
pub struct SplashComponent;

impl Default for SplashComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl SplashComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for SplashComponent {
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        if app.server.connection_state.is_none() {
            app.init_connection_state();
        }

        if let Some(connection_state) = &mut app.server.connection_state {
            match key_event.code {
                // Connect to the server
                KeyCode::Enter => match connection_state.validate_username() {
                    Ok(_) => match connection_state.validate_ip() {
                        Ok(_) => match connection_state.validate_port() {
                            Ok(port) => {
                                let ip = connection_state.get_ip();
                                let username = connection_state.get_username();
                                let _ = app
                                    .server
                                    .network
                                    .update_connection_info(ip, port, username);
                                app.server.is_connecting = true;
                                app.set_status_message("Connecting...".to_string());
                                Ok(true)
                            }
                            Err(msg) => {
                                app.set_status_message(msg);
                                Ok(true)
                            }
                        },
                        Err(msg) => {
                            app.set_status_message(msg);
                            Ok(true)
                        }
                    },
                    Err(msg) => {
                        app.set_status_message(msg);
                        Ok(true)
                    }
                },
                // Switch to the next field
                KeyCode::Tab => {
                    connection_state.next_field();
                    Ok(true)
                }
                // Move to the next field with Down arrow
                KeyCode::Down => {
                    connection_state.next_field();
                    Ok(true)
                }
                // Move to the previous field with Up arrow
                KeyCode::Up => {
                    connection_state.previous_field();
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

    /// Draw the splash component
    ///
    /// # Arguments
    ///
    /// * `app`: The application state
    /// * `frame`: The frame to draw on
    /// * `area`: The area to draw on
    ///
    /// # Returns
    ///
    /// * `()`
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(9),
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
            .pixel_size(PixelSize::Full)
            .style(Style::default().fg(Color::White))
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

            let instructions = Paragraph::new("Press TAB to switch fields, ENTER to connect")
                .style(Style::default().fg(Color::White))
                .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(instructions, vertical_layout[5]);
        }
    }
}
