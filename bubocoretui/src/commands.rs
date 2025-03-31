use crate::app::{App, LogLevel};
use crate::event::AppEvent;
use bubocorelib::server::client::ClientMessage;
use color_eyre::Result as EyreResult;

impl App {
    pub fn execute_command(&mut self, command: &str) -> EyreResult<()> {
        if command.is_empty() {
            return Ok(());
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "print" => {
                if !args.is_empty() {
                    // Let's print the pattern we got from the server
                    match args[0] {
                        "pattern" => {
                            self.add_log(LogLevel::Info, format!("Pattern: {:?}", self.editor.pattern));
                        }
                        _ => {
                            self.set_status_message(String::from("Unknown object"));
                        }
                    }
                } else {
                    self.set_status_message(String::from("Object required"));
                }
            }
            "name" => {
                if !args.is_empty() {
                    let name = args.join(" ");
                    self.send_client_message(ClientMessage::SetName(name.clone()));
                } else {
                    self.set_status_message(String::from("Name required"));
                }
            }
            "quit" | "q" | "exit" | "kill" => {
                self.events.send(AppEvent::Quit);
            }
            "help" | "?" => {
                self.events.send(AppEvent::SwitchToHelp);
            }
            "chat" => {
                if !args.is_empty() {
                    let message = args.join(" ");
                    self.send_client_message(ClientMessage::Chat(message.clone()));
                    self.add_log(LogLevel::Info, format!("Sent: {}", message));
                } else {
                    self.set_status_message(String::from("Chat message required"));
                }
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
                self.add_log(LogLevel::Error, format!("Unknown command: {}", cmd));
            }
        }
        Ok(())
    }
}
