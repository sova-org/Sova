use crate::components::help::HelpState;
use ratatui::{style::Style, widgets::Block};
use rusty_link::{AblLink, SessionState};
use std::error::Error;
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
        text_area.set_block(Block::default());
        text_area.set_cursor_line_style(Style::default());
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
    pub flash_elapsed: Duration,
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
    /// Capturer l'état de l'horloge
    pub fn capture_app_state(&mut self) {
        self.link.capture_app_session_state(&mut self.session_state);
    }

    /// Pousser un nouvel état
    pub fn commit_app_state(&self) {
        self.link.commit_app_session_state(&self.session_state);
    }

    /// Pousser la synchronisation
    pub fn set_start_stop_sync(&self) {
        let state = self.link.is_start_stop_sync_enabled();
        self.link.enable_start_stop_sync(!state);
        self.commit_app_state();
    }

    // Récupérer la phase actuelle
    pub fn get_phase(&mut self) -> f64 {
        self.capture_app_state();
        let beat = self
            .session_state
            .beat_at_time(self.link.clock_micros(), self.quantum as f64);
        beat % self.quantum as f64
    }
}

pub struct App {
    pub screen_state: ScreenState,
    pub editor_data: EditorData,
    pub state: ServerState,
    pub status_message: String,
    pub link_client: Link,
    pub command_mode: CommandMode,
    pub help_state: Option<HelpState>,
    pub exit: bool,
}

impl App {
    pub fn new() -> App {
        let app = App {
            screen_state: ScreenState {
                mode: Mode::Splash,
                flash: Flash {
                    is_flashing: false,
                    flash_start: None,
                    flash_duration: Duration::from_micros(200_000),
                    flash_elapsed: Duration::from_secs(0),
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
            exit: false,
        };
        app.link_client.link.enable(true);
        app
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

    pub fn send_content(&self) -> Result<(), Box<dyn Error>> {
        // TODO: I probably should do something!
        Ok(())
    }

    pub fn execute_command(&mut self) -> Result<(), Box<dyn Error>> {
        let command = self.command_mode.get_command();

        if command.is_empty() {
            return Ok(());
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts[0];
        // TODO: do something with the arguments!
        let args = &parts[1..];

        match cmd {
            "quit" | "q" | "exit" | "kill" => {
                self.exit = true;
                Ok(())
            }
            "help" | "?" => {
                self.screen_state.mode = Mode::Help;
                Ok(())
            }
            "tempo" | "t" => {
                if let Some(tempo_str) = args.get(0) {
                    if let Ok(tempo) = tempo_str.parse::<f64>() {
                        if tempo >= 20.0 && tempo <= 999.0 {
                            self.link_client
                                .session_state
                                .set_tempo(tempo, self.link_client.link.clock_micros());
                            self.link_client.commit_app_state();
                            self.set_status_message(format!("Tempo set to {:.1} BPM", tempo));
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
                Ok(())
            }
            "quantum" => {
                if let Some(quantum_str) = args.get(0) {
                    if let Ok(quantum) = quantum_str.parse::<f64>() {
                        // FIX: There is a problem with quantum
                        if quantum > 0.0 && quantum <= 16.0 {
                            self.link_client.quantum = quantum;
                            self.link_client.capture_app_state();
                            self.link_client.commit_app_state();
                        }
                    } else {
                        self.set_status_message(String::from("Invalid quantum value"));
                    }
                } else {
                    self.set_status_message(String::from("Quantum value required"));
                }
                Ok(())
            }
            _ => {
                self.set_status_message(format!("Unknown command: {}", cmd));
                Ok(())
            }
        }
    }
}
