use crate::components::{
    Component,
    editor::EditorComponent,
    grid::GridComponent,
    help::{HelpComponent, HelpState},
    options::OptionsComponent,
    splash::ConnectionState,
    splash::SplashComponent,
};
use crate::event::{AppEvent, Event, EventHandler};
use crate::link::Link;
use crate::network::NetworkManager;
use bubocorelib::pattern::Pattern;
use bubocorelib::server::{ServerMessage, client::ClientMessage};
use color_eyre::Result as EyreResult;
use ratatui::{
    Terminal,
    backend::Backend,
    crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use std::time::{Duration, Instant};
use tui_textarea::TextArea;

pub enum Mode {
    Editor,
    Grid,
    Options,
    Splash,
    Help,
}

pub struct CommandMode {
    pub active: bool,
    pub text_area: TextArea<'static>,
}

impl CommandMode {
    pub fn new() -> Self {
        let mut text_area = TextArea::default();
        text_area.set_block(ratatui::widgets::Block::default());
        text_area.set_cursor_line_style(ratatui::style::Style::default());
        text_area.set_placeholder_text("Type a command (like 'help')...");
        CommandMode {
            active: false,
            text_area,
        }
    }

    pub fn enter(&mut self) {
        self.active = true;
        self.text_area.delete_line_by_head();
        self.text_area.move_cursor(tui_textarea::CursorMove::End);
    }

    pub fn exit(&mut self) {
        self.active = false;
    }

    pub fn get_command(&self) -> String {
        self.text_area.lines().join("").trim().to_string()
    }
}

pub struct Flash {
    pub is_flashing: bool,
    pub flash_start: Option<Instant>,
    pub flash_duration: Duration,
}

pub struct ScreenState {
    pub mode: Mode,
    pub flash: Flash,
}

pub struct UserPosition {
    pub pattern: usize,
    pub script: usize,
}

pub struct EditorData {
    pub active_sequence: UserPosition,
    pub line_count: usize,
    pub content: String,
    pub textarea: TextArea<'static>,
    pub pattern: Option<Pattern>,
    pub devices: Vec<(String, String)>,
}

pub struct ServerState {
    pub network: NetworkManager,
    pub is_connected: bool,
    pub is_connecting: bool,
    pub username: String,
    pub peers: Vec<String>,
    pub devices: Vec<String>,
    pub link: Link,
}

pub struct InterfaceState {
    pub screen: ScreenState,
    pub components: ComponentState,
}

pub struct ComponentState {
    pub command_mode: CommandMode,
    pub connection_state: Option<ConnectionState>,
    pub help_state: Option<HelpState>,
    pub bottom_message: String,
}

pub struct App {
    pub running: bool,
    pub interface: InterfaceState,
    pub editor: EditorData,
    pub server: ServerState,
    pub events: EventHandler,
}

impl App {
    pub fn new(ip: String, port: u16, username: String) -> Self {
        let events = EventHandler::new();
        let event_sender = events.sender.clone();
        // FIX: get patterns before application starts
        let mut app = Self {
            running: true,
            editor: EditorData {
                content: String::new(),
                devices: vec![],
                line_count: 1,
                active_sequence: UserPosition {
                    pattern: 0,
                    script: 0,
                },
                textarea: TextArea::default(),
                pattern: None,
            },
            server: ServerState {
                is_connected: false,
                is_connecting: false,
                link: Link::new(),
                peers: Vec::new(),
                devices: Vec::new(),
                username: username.clone(),
                network: NetworkManager::new(ip, port, username, event_sender),
            },
            interface: InterfaceState {
                screen: ScreenState {
                    mode: Mode::Splash,
                    flash: Flash {
                        is_flashing: false,
                        flash_start: None,
                        flash_duration: Duration::from_micros(200_000),
                    },
                },
                components: ComponentState {
                    connection_state: None,
                    command_mode: CommandMode::new(),
                    help_state: None,
                    bottom_message: String::from("Press ENTER to start!"),
                },
            },
            events,
        };
        app.server.link.link.enable(true);
        app.init_connection_state();
        app
    }

    pub fn init_connection_state(&mut self) {
        let (ip, port) = self.server.network.get_connection_info();
        self.interface.components.connection_state = Some(ConnectionState::new(&ip, port, &self.server.username));
    }

    pub async fn run<B: Backend>(&mut self, mut terminal: Terminal<B>) -> EyreResult<()> {
        while self.running {
            // Draw a frame
            terminal.draw(|frame| crate::ui::ui(frame, self))?;

            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    CrosstermEvent::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Release {
                            continue;
                        }
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => self.handle_app_event(app_event)?,
                Event::Network(message) => self.handle_server_message(message),
            }
        }
        Ok(())
    }

    fn handle_server_message(&mut self, message: ServerMessage) {
        match message {
            // Handshake from server
            ServerMessage::Hello { pattern, devices, clients } => {
                self.set_status_message(format!("Handshake successful for {}", self.server.username));
                self.editor.pattern = Some(pattern);
                self.server.devices = devices.iter().map(|(name, _)| name.clone()).collect();
                self.server.peers = clients;
                self.server.is_connected = true;
                self.server.is_connecting = false;

                // Switch to editor only if we were connecting from the splash screen
                if matches!(self.interface.screen.mode, Mode::Splash) {
                    self.events.send(AppEvent::SwitchToEditor);
                }
            }
            ServerMessage::ClockState(tempo, _beat, _micros, quantum) => {
                self.set_status_message(format!("Clock sync: {:.1} BPM", tempo));
                let timestamp = self.server.link.link.clock_micros();
                self.server.link.session_state.set_tempo(tempo, timestamp);
                self.server.link.quantum = quantum;
            }
            ServerMessage::PatternValue(_pattern) => {
                self.set_status_message(String::from("Received pattern update"));
            }
            ServerMessage::StepPosition(_positions) => {
                // Update the current step positions in your grid view
            }
            ServerMessage::PatternLayout(_layout) => {
                // Update the grid layout
            }
            ServerMessage::Success => {
                self.set_status_message(String::from("Command executed successfully"));
            }
            ServerMessage::InternalError => {
                self.set_status_message(String::from("Server error occurred"));
            }
            ServerMessage::LogMessage(message) => {
                self.set_status_message(format!("Server message: {:?}", message));
            }
        }
    }

    pub fn send_client_message(&mut self, message: ClientMessage) {
        match self.server.network.send(message) {
            Ok(_) => {}
            Err(e) => {
                self.set_status_message(format!("Failed to send message: {}", e));
                self.server.is_connected = false;
            }
        }
    }

    fn tick(&mut self) {}

    fn handle_app_event(&mut self, event: AppEvent) -> EyreResult<()> {
        match event {
            AppEvent::SwitchToEditor => self.interface.screen.mode = Mode::Editor,
            AppEvent::SwitchToGrid => self.interface.screen.mode = Mode::Grid,
            AppEvent::SwitchToOptions => self.interface.screen.mode = Mode::Options,
            AppEvent::SwitchToHelp => {
                self.interface.screen.mode = Mode::Help;
                if self.interface.components.help_state.is_none() {
                    self.interface.components.help_state = Some(HelpState::new());
                }
            }
            AppEvent::NextScreen => {
                self.interface.screen.mode = match self.interface.screen.mode {
                    Mode::Editor => Mode::Grid,
                    Mode::Grid => Mode::Options,
                    Mode::Options => Mode::Editor,
                    Mode::Help => Mode::Editor,
                    Mode::Splash => Mode::Editor,
                };
            }
            AppEvent::EnterCommandMode => {
                self.interface.components.command_mode.enter();
            }
            AppEvent::ExitCommandMode => {
                self.interface.components.command_mode.exit();
            }
            AppEvent::ExecuteCommand(cmd) => {
                match self.execute_command(&cmd) {
                    Ok(_) => {}
                    Err(e) => {
                        self.set_status_message(format!("Error: {}", e));
                    }
                }
                self.interface.components.command_mode.exit();
            }
            AppEvent::ExecuteContent => {
                self.flash_screen();
                match self.send_content() {
                    Ok(_) => {
                        self.set_status_message(String::from("Content sent successfully!"));
                    }
                    Err(e) => {
                        self.set_status_message(format!("Error sending content: {}", e));
                    }
                }
            }
            AppEvent::UpdateTempo(tempo) => {
                self.server
                    .link
                    .session_state
                    .set_tempo(tempo, self.server.link.link.clock_micros());
                self.server.link.commit_app_state();
                self.set_status_message(format!("Tempo set to {:.1} BPM", tempo));
            }
            AppEvent::UpdateQuantum(quantum) => {
                self.server.link.quantum = quantum;
                self.server.link.capture_app_state();
                self.server.link.commit_app_state();
                self.set_status_message(format!("Quantum set to {}", quantum));
            }
            AppEvent::ToggleStartStopSync => {
                self.server.link.toggle_start_stop_sync();
                let state = self.server.link.link.is_start_stop_sync_enabled();
                self.set_status_message(format!(
                    "Start/Stop sync {}",
                    if state { "enabled" } else { "disabled" }
                ));
            }
            AppEvent::Quit => {
                self.quit();
            }
        }
        Ok(())
    }

    fn handle_key_events(&mut self, key_event: KeyEvent) -> EyreResult<()> {
        // Handle global command mode toggle first
        if key_event.code == KeyCode::Char('p')
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
        {
            if self.interface.components.command_mode.active {
                self.events.send(AppEvent::ExitCommandMode);
            } else {
                self.events.send(AppEvent::EnterCommandMode);
            }
            return Ok(());
        }

        // Handle command mode input
        if self.interface.components.command_mode.active {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('c')
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.events.send(AppEvent::ExitCommandMode);
                }
                KeyCode::Enter => {
                    let cmd = self.interface.components.command_mode.get_command();
                    self.events.send(AppEvent::ExecuteCommand(cmd));
                }
                _ => {
                    self.interface
                        .components
                        .command_mode
                        .text_area
                        .input(key_event);
                }
            }
            return Ok(());
        }

        let handled = match self.interface.screen.mode {
            Mode::Splash => SplashComponent::new()
                .handle_key_event(self, key_event)
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))?,
            Mode::Editor => EditorComponent::new()
                .handle_key_event(self, key_event)
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))?,
            Mode::Grid => GridComponent::new()
                .handle_key_event(self, key_event)
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))?,
            Mode::Options => OptionsComponent::new()
                .handle_key_event(self, key_event)
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))?,
            Mode::Help => HelpComponent::new()
                .handle_key_event(self, key_event)
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))?,
        };

        if !handled {
            // Handle any unhandled keys here if needed
        }

        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn flash_screen(&mut self) {
        self.interface.screen.flash.is_flashing = true;
        self.interface.screen.flash.flash_start = Some(Instant::now());
    }

    pub fn set_content(&mut self, content: String) {
        self.editor.content = content;
        self.editor.line_count = self.editor.content.lines().count().max(1);
    }

    pub fn set_status_message(&mut self, message: String) {
        self.interface.components.bottom_message = message;
    }

    pub fn send_content(&self) -> EyreResult<()> {
        // TODO: Implement content sending logic
        Ok(())
    }

    pub fn execute_command(&mut self, command: &str) -> EyreResult<()> {
        if command.is_empty() {
            return Ok(());
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "quit" | "q" | "exit" | "kill" => {
                self.events.send(AppEvent::Quit);
            }
            "help" | "?" => {
                self.events.send(AppEvent::SwitchToHelp);
            }
            "tempo" | "t" => {
                if let Some(tempo_str) = args.get(0) {
                    if let Ok(tempo) = tempo_str.parse::<f64>() {
                        if tempo >= 20.0 && tempo <= 999.0 {
                            self.events.send(AppEvent::UpdateTempo(tempo));
                            self.send_client_message(ClientMessage::SetTempo(tempo));
                        } else {
                            self.set_status_message(String::from(
                                "Tempo must be between 20 and 999 BPM",
                            ));
                        }
                    } else {
                        self.set_status_message(String::from("Invalid tempo value"));
                    }
                } else {
                    self.set_status_message(String::from("Tempo value required"));
                }
            }
            "sync" => {
                self.send_client_message(ClientMessage::GetClock);
                self.set_status_message(String::from("Synchronizing with server..."));
            }
            "connect" => match self.server.network.reconnect() {
                Ok(_) => self.set_status_message(String::from("Reconnecting...")),
                Err(e) => self.set_status_message(format!("Failed to reconnect: {}", e)),
            },
            "quantum" => {
                if let Some(quantum_str) = args.get(0) {
                    if let Ok(quantum) = quantum_str.parse::<f64>() {
                        if quantum > 0.0 && quantum <= 16.0 {
                            self.events.send(AppEvent::UpdateQuantum(quantum));
                        } else {
                            self.set_status_message(String::from(
                                "Quantum must be between 0 and 16",
                            ));
                        }
                    } else {
                        self.set_status_message(String::from("Invalid quantum value"));
                    }
                } else {
                    self.set_status_message(String::from("Quantum value required"));
                }
            }
            _ => {
                self.set_status_message(format!("Unknown command: {}", cmd));
            }
        }
        Ok(())
    }
}
