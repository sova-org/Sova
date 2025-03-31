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
use chrono::{DateTime, Local};
use std::collections::VecDeque;
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
}

pub struct ServerState {
    pub network: NetworkManager,
    pub is_connected: bool,
    pub is_connecting: bool,
    pub connection_state: Option<ConnectionState>,
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
    pub help_state: Option<HelpState>,
    pub bottom_message: String,
    pub bottom_message_timestamp: Option<Instant>,
}

#[derive(Clone, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

const MAX_LOGS: usize = 100;

pub struct App {
    pub running: bool,
    pub interface: InterfaceState,
    pub editor: EditorData,
    pub server: ServerState,
    pub events: EventHandler,
    pub logs: VecDeque<LogEntry>,
}

impl App {
    pub fn new(ip: String, port: u16, username: String) -> Self {
        let events = EventHandler::new();
        let event_sender = events.sender.clone();
        let mut app = Self {
            running: true,
            editor: EditorData {
                content: String::new(),
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
                connection_state: None,
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
                    command_mode: CommandMode::new(),
                    help_state: None,
                    bottom_message: String::from("Press ENTER to start!"),
                    bottom_message_timestamp: None,
                },
            },
            events,
            logs: VecDeque::with_capacity(MAX_LOGS),
        };
        app.server.link.link.enable(true);
        app.init_connection_state();
        app
    }

    pub fn init_connection_state(&mut self) {
        let (ip, port) = self.server.network.get_connection_info();
        self.server.connection_state = Some(ConnectionState::new(&ip, port, &self.server.username));
    }

    pub async fn run<B: Backend>(&mut self, mut terminal: Terminal<B>) -> EyreResult<()> {
        while self.running {
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
            ServerMessage::Chat(msg) => {
                self.add_log(LogLevel::Info, format!("Received: {}", msg.to_string()));
            }
            ServerMessage::PeersUpdated(peers) => {
                self.server.peers = peers;
                self.add_log(LogLevel::Info, format!("Peers updated: {}", self.server.peers.join(", ")));
            }
            ServerMessage::Hello { pattern, devices, clients } => {
                self.set_status_message(format!("Handshake successful for {}", self.server.username));
                self.editor.pattern = Some(pattern);
                self.server.devices = devices.iter().map(|(name, _)| name.clone()).collect();
                self.server.peers = clients;
                self.server.is_connected = true;
                self.server.is_connecting = false;

                if matches!(self.interface.screen.mode, Mode::Splash) {
                    self.events.send(AppEvent::SwitchToEditor);
                }
            }
            ServerMessage::ClockState(tempo, _beat, _micros, quantum) => {
                self.set_status_message(format!("Clock sync: {:.1} BPM", tempo));
                let timestamp = self.server.link.link.clock_micros();
                self.server.link.session_state.set_tempo(tempo, timestamp);
                self.server.link.quantum = quantum;
                self.add_log(LogLevel::Info, format!("Tempo updated: {:.1} BPM", tempo));
            }
            ServerMessage::PatternValue(_pattern) => {
                self.set_status_message(String::from("Received pattern update"));
            }
            ServerMessage::StepPosition(_positions) => {
            }
            ServerMessage::PatternLayout(_layout) => {
            }
            ServerMessage::Success => {}
            ServerMessage::InternalError(message) => {
                self.add_log(LogLevel::Error, message);
            }
            ServerMessage::LogMessage(message) => {
                self.add_log(LogLevel::Info, message.to_string());
            }
        }
    }

    pub fn add_log(&mut self, level: LogLevel, message: String) {
        if self.logs.len() == MAX_LOGS {
            self.logs.pop_front();
        }
        self.logs.push_back(LogEntry {
            timestamp: Local::now(),
            level,
            message,
        });
        self.editor.line_count = self.editor.content.lines().count().max(1);
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

    fn tick(&mut self) {
        // Remove bottom message after 3 seconds
        if let Some(timestamp) = self.interface.components.bottom_message_timestamp {
            if timestamp.elapsed() > Duration::from_secs(3) {
                self.interface.components.bottom_message = String::new();
                self.interface.components.bottom_message_timestamp = None;
            }
        }
    }

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
            }
            AppEvent::UpdateQuantum(quantum) => {
                self.server.link.quantum = quantum;
                self.server.link.capture_app_state();
                self.server.link.commit_app_state();
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
            // Optionally, call handle_common_keys or other default logic here
            // For now, we'll just leave it empty as before
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
        self.interface.components.bottom_message_timestamp = Some(Instant::now());
    }

    pub fn send_content(&self) -> EyreResult<()> {
        Ok(())
    }
}
