use crate::app::{App, LogLevel};
use crate::event::AppEvent;
use bubocorelib::server::client::ClientMessage;
use color_eyre::Result as EyreResult;
use tui_textarea::TextArea;
/// Structure représentant le mode de commande de l'application (command prompt).
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


impl App {

    /// Exécute une commande de l'interface utilisateur.
    /// 
    /// Cette fonction analyse la commande entrante, la valide et l'exécute.
    /// Elle gère les commandes courantes telles que l'impression de motifs,
    /// le changement de nom, la sortie, l'aide, le chat, le tempo, la synchronisation,
    /// la connexion et la quantité.
    /// 
    /// # Arguments
    /// 
    /// * `command` - La commande à exécuter.
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant :
    /// * `Ok(())` si la commande a été exécutée avec succès.
    /// * `Err` si une erreur s'est produite lors de l'exécution de la commande.
    pub fn execute_command(&mut self, command: &str) -> EyreResult<()> {
        // Si la commande est vide, retourne Ok()
        if command.is_empty() {
            return Ok(());
        }

        // Analyse la commande pour extraire la commande principale et ses arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        let (cmd, args) = (parts[0], &parts[1..]);

        match cmd {

            // Utilisé principalement pour le débogage
            "print" => {
                if !args.is_empty() {
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

            // Change le nom du client et le propage aux autres clients
            "name" => {
                if !args.is_empty() {
                    let name = args.join(" ");
                    self.send_client_message(ClientMessage::SetName(name.clone()));
                } else {
                    self.set_status_message(String::from("Name required"));
                }
            }

            // Quitte l'application
            "quit" | "q" | "exit" | "kill" => {
                self.events.send(AppEvent::Quit);
            }

            // Affiche la vue d'aide
            "help" | "?" => {
                self.events.send(AppEvent::SwitchToHelp);
            }

            // Affiche la vue de l'éditeur
            "editor" => {
                self.events.send(AppEvent::SwitchToEditor);
            }

            // Affiche la vue de la grille
            "grid" => {
                self.events.send(AppEvent::SwitchToGrid);
            }

            // Affiche la vue des options
            "options" => {
                self.events.send(AppEvent::SwitchToOptions);
            }

            // Autorise une forme de communication rudimentaire entre les clients
            "chat" => {
                if !args.is_empty() {
                    let message = args.join(" ");
                    self.send_client_message(ClientMessage::Chat(message.clone()));
                    self.add_log(LogLevel::Info, format!("Sent: {}", message));
                } else {
                    self.set_status_message(String::from("Chat message required"));
                }
            }

            // Modifie le tempo, notifie les autres clients
            "tempo" | "t" => {
                if let Some(tempo_str) = args.get(0) {
                    if let Ok(tempo) = tempo_str.parse::<f64>() {
                        // Ableton Link is imposing a limit to tempo range
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

            // Essaie de reconnecter le client au serveur
            "connect" => match self.server.network.reconnect() {
                Ok(_) => self.set_status_message(String::from("Reconnecting...")),
                Err(e) => self.set_status_message(format!("Failed to reconnect: {}", e)),
            },

            // Modifie le quantum utilisé par l'horloge Link
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

            // Commande inconnue, affiche un message d'erreur
            _ => {
                self.set_status_message(format!("Unknown command: {}", cmd));
                self.add_log(LogLevel::Error, format!("Unknown command: {}", cmd));
            }
        }
        Ok(())
    }
}
