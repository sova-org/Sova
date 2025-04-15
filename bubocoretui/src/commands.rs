use crate::app::App;
use crate::event::AppEvent;
use crate::components::logs::LogLevel;
use bubocorelib::server::client::ClientMessage;
use bubocorelib::schedule::ActionTiming;
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

    /// Parses an optional timing argument string into an ActionTiming enum.
    /// Defaults to Immediate and logs a warning for unrecognized inputs.
    fn parse_timing_arg(&mut self, arg: Option<&str>) -> ActionTiming {
        arg.map_or(ActionTiming::Immediate, |timing_str| {
            match timing_str.to_lowercase().as_str() {
                "immediate" | "now" => ActionTiming::Immediate,
                "end" | "loop" => ActionTiming::EndOfScene,
                _ => {
                    if let Ok(beat) = timing_str.parse::<u64>() {
                        ActionTiming::AtBeat(beat)
                    } else {
                        self.add_log(LogLevel::Warn, format!("Unrecognized timing '{}', defaulting to immediate.", timing_str));
                        ActionTiming::Immediate
                    }
                }
            }
        })
    }

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

            // Quitte l'application
            "quit" | "q" | "exit" | "kill" => {
                self.events.send(AppEvent::Quit);
            }

            // Change le nom du client et le propage aux autres clients
            "setname" => {
                if args.is_empty() {
                    self.set_status_message(String::from("Name required"));
                    return Ok(());
                }
                let name = args.join(" ");
                self.send_client_message(ClientMessage::SetName(name.clone()));
                self.server.username = name;
                self.set_status_message(format!("Set name to '{}'", self.server.username));
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

            // Affiche la liste des périphériques
            "devices" => {
                self.events.send(AppEvent::SwitchToDevices);
            }

            // Affiche le journal
            "logs" => {
                self.events.send(AppEvent::SwitchToLogs);
            }

            // Affiche la vue d'aide
            "help" | "?" => {
                self.events.send(AppEvent::SwitchToHelp);
            }

            // Affiche la vue de la liste des fichiers
            "files" => {
                self.events.send(AppEvent::SwitchToSaveLoad);
            }

            // Communication rudimentaire entre clients
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
                            self.send_client_message(ClientMessage::SetTempo(tempo, ActionTiming::Immediate));
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

            // Set the scene length
            "scenelength" | "sl" => {
                 if let Some(length_str) = args.get(0) {
                    if let Ok(length) = length_str.parse::<usize>() {
                        let timing = self.parse_timing_arg(args.get(1).copied());
                        self.send_client_message(ClientMessage::SetSceneLength(length, timing));
                        self.set_status_message(format!("Requested setting scene length to {} ({:?})", length, timing));
                    } else {
                        self.set_status_message(String::from("Invalid length value (must be a positive integer)"));
                    }
                } else {
                    self.set_status_message(String::from("Length value required (e.g., 'sl 16' or 'sl 8 end')"));
                }
            }

            // Set a custom length for a line
            "linelen" | "ll" => {
                if args.len() < 2 {
                    self.set_status_message(String::from("Usage: linelen <line_index> <length|scene> [timing]"));
                    return Ok(());
                }
                if let Ok(user_line_idx) = args[0].parse::<usize>() {
                    if user_line_idx == 0 { // Check for 0 index
                         self.set_status_message(String::from("Line index must be 1 or greater."));
                         return Ok(());
                    }
                    let line_idx = user_line_idx - 1; // Convert to 0-based index

                    let length_arg = args[1].to_lowercase();
                    let length_opt: Option<f64> = if length_arg == "scene" {
                        None
                    } else if let Ok(len) = length_arg.parse::<f64>() {
                        if len > 0.0 { Some(len) } else { None } // Ensure positive length
                    } else {
                        self.set_status_message(String::from("Invalid length argument: use a positive number or 'scene'"));
                        return Ok(());
                    };

                    let timing = self.parse_timing_arg(args.get(2).copied());

                    self.send_client_message(ClientMessage::SetLineLength(line_idx, length_opt, timing));
                    self.set_status_message(format!("Requested setting Line {} length to {:?} ({:?})", user_line_idx, length_opt, timing)); // Use user_line_idx in message

                } else {
                     self.set_status_message(String::from("Invalid line index"));
                }
            }

            // Set the playback speed factor for a line
            "linespeed" | "ls" => {
                if args.len() < 2 {
                    self.set_status_message(String::from("Usage: linespeed <line_index> <speed_factor> [timing]"));
                    return Ok(());
                }
                if let Ok(user_line_idx) = args[0].parse::<usize>() {
                     if user_line_idx == 0 { // Check for 0 index
                         self.set_status_message(String::from("Line index must be 1 or greater."));
                         return Ok(());
                     }
                     let line_idx = user_line_idx - 1; // Convert to 0-based index

                    if let Ok(speed_factor) = args[1].parse::<f64>() {
                         if speed_factor <= 0.0 {
                            self.set_status_message(String::from("Speed factor must be positive."));
                            return Ok(());
                         }

                        let timing = self.parse_timing_arg(args.get(2).copied());

                        self.send_client_message(ClientMessage::SetLineSpeedFactor(line_idx, speed_factor, timing));
                        self.set_status_message(format!("Requested setting Line {} speed factor to x{:.2} ({:?})", user_line_idx, speed_factor, timing));

                    } else {
                         self.set_status_message(String::from("Invalid speed factor (must be a number)"));
                    }
                } else {
                     self.set_status_message(String::from("Invalid line index"));
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
