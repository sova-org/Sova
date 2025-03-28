use crate::components::help::HelpState;
use crate::components::{
    Component, editor::EditorComponent, grid::GridComponent, help::HelpComponent,
    options::OptionsComponent, splash::SplashComponent,
};
use crate::event::{AppEvent, Event, EventHandler};
use crate::network::NetworkManager;
use bubocorelib::server::{ServerMessage, client::ClientMessage};
use color_eyre::Result as EyreResult;
use ratatui::{
    Terminal,
    backend::Backend,
    crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use rusty_link::{AblLink, SessionState};
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

pub struct EditorData {
    pub content: String,
    pub line_count: usize,
    pub cursor_position: (u16, u16),
    pub textarea: TextArea<'static>,
}

pub struct ServerState {
    pub is_connected: bool,
    pub peers: Vec<String>,
    pub devices: Vec<String>,
}

pub struct Link {
    pub link: AblLink,
    pub session_state: SessionState,
    pub quantum: f64,
}

impl Link {
    pub fn capture_app_state(&mut self) {
        self.link.capture_app_session_state(&mut self.session_state);
    }

    pub fn commit_app_state(&self) {
        self.link.commit_app_session_state(&self.session_state);
    }

    pub fn toggle_start_stop_sync(&mut self) {
        let state = self.link.is_start_stop_sync_enabled();
        self.link.enable_start_stop_sync(!state);
        self.commit_app_state();
    }

    pub fn get_phase(&mut self) -> f64 {
        self.capture_app_state();
        let beat = self
            .session_state
            .beat_at_time(self.link.clock_micros(), self.quantum as f64);
        beat % self.quantum as f64
    }
}

pub struct App {
    pub running: bool,
    pub screen_state: ScreenState,
    pub editor_data: EditorData,
    pub state: ServerState,
    pub status_message: String,
    pub link_client: Link,
    pub command_mode: CommandMode,
    pub help_state: Option<HelpState>,
    pub events: EventHandler,
    pub network: NetworkManager,
}

impl App {
    pub fn new(ip: String, port: u16) -> Self {
        let app = Self {
            network: NetworkManager::new(ip, port),
            running: true,
            screen_state: ScreenState {
                mode: Mode::Splash,
                flash: Flash {
                    is_flashing: false,
                    flash_start: None,
                    flash_duration: Duration::from_micros(200_000),
                },
            },
            editor_data: EditorData {
                content: String::new(),
                line_count: 0,
                cursor_position: (0, 0),
                textarea: TextArea::default(),
            },
            state: ServerState {
                is_connected: false,
                peers: Vec::new(),
                devices: Vec::new(),
            },
            status_message: String::from("Press ENTER to start!"),
            link_client: Link {
                link: AblLink::new(120.0),
                session_state: SessionState::new(),
                quantum: 4.0,
            },
            command_mode: CommandMode::new(),
            help_state: None,
            events: EventHandler::new(),
        };
        app.link_client.link.enable(true);
        app
    }

    pub async fn run<B: Backend>(&mut self, mut terminal: Terminal<B>) -> EyreResult<()> {
        while self.running {
            while let Some(message) = self.network.try_receive() {
                self.handle_server_message(message);
            }
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
            }
        }
        Ok(())
    }

    fn handle_server_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::ClockState(tempo, _beat, _micros, quantum) => {
                self.set_status_message(format!("Clock sync: {:.1} BPM", tempo));
                let timestamp = self.link_client.link.clock_micros();
                self.link_client.session_state.set_tempo(tempo, timestamp);
                self.link_client.quantum = quantum;
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
        }
    }

    pub fn send_client_message(&mut self, message: ClientMessage) {
        match self.network.send(message) {
            Ok(_) => {}
            Err(e) => {
                self.set_status_message(format!("Failed to send message: {}", e));
                self.state.is_connected = false;
            }
        }
    }

    fn tick(&mut self) {}

    fn handle_app_event(&mut self, event: AppEvent) -> EyreResult<()> {
        match event {
            AppEvent::SwitchToEditor => self.screen_state.mode = Mode::Editor,
            AppEvent::SwitchToGrid => self.screen_state.mode = Mode::Grid,
            AppEvent::SwitchToOptions => self.screen_state.mode = Mode::Options,
            AppEvent::SwitchToHelp => {
                self.screen_state.mode = Mode::Help;
                if self.help_state.is_none() {
                    self.help_state = Some(HelpState::new());
                }
            }
            AppEvent::NextScreen => {
                self.screen_state.mode = match self.screen_state.mode {
                    Mode::Editor => Mode::Grid,
                    Mode::Grid => Mode::Options,
                    Mode::Options => Mode::Editor,
                    Mode::Help => Mode::Editor,
                    Mode::Splash => Mode::Editor,
                };
            }
            AppEvent::EnterCommandMode => {
                self.command_mode.enter();
            }
            AppEvent::ExitCommandMode => {
                self.command_mode.exit();
            }
            AppEvent::ExecuteCommand(cmd) => {
                match self.execute_command(&cmd) {
                    Ok(_) => {}
                    Err(e) => {
                        self.set_status_message(format!("Error: {}", e));
                    }
                }
                self.command_mode.exit();
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
                self.link_client
                    .session_state
                    .set_tempo(tempo, self.link_client.link.clock_micros());
                self.link_client.commit_app_state();
                self.set_status_message(format!("Tempo set to {:.1} BPM", tempo));
            }
            AppEvent::UpdateQuantum(quantum) => {
                self.link_client.quantum = quantum;
                self.link_client.capture_app_state();
                self.link_client.commit_app_state();
                self.set_status_message(format!("Quantum set to {}", quantum));
            }
            AppEvent::ToggleStartStopSync => {
                self.link_client.toggle_start_stop_sync();
                let state = self.link_client.link.is_start_stop_sync_enabled();
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
            if self.command_mode.active {
                self.events.send(AppEvent::ExitCommandMode);
            } else {
                self.events.send(AppEvent::EnterCommandMode);
            }
            return Ok(());
        }

        // Handle command mode input
        if self.command_mode.active {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('c')
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.events.send(AppEvent::ExitCommandMode);
                }
                KeyCode::Enter => {
                    let cmd = self.command_mode.get_command();
                    self.events.send(AppEvent::ExecuteCommand(cmd));
                }
                _ => {
                    self.command_mode.text_area.input(key_event);
                }
            }
            return Ok(());
        }

        // Delegate key handling to the current component based on mode
        let handled = match self.screen_state.mode {
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
        self.screen_state.flash.is_flashing = true;
        self.screen_state.flash.flash_start = Some(Instant::now());
    }

    pub fn set_content(&mut self, content: String) {
        self.editor_data.content = content;
        self.editor_data.line_count = self.editor_data.content.lines().count().max(1);
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) {
        self.editor_data.cursor_position = (x, y);
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = message;
    }

    pub fn set_flash_duration(&mut self, microseconds: u64) {
        self.screen_state.flash.flash_duration = Duration::from_micros(microseconds);
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
            "connect" => match self.network.reconnect() {
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
