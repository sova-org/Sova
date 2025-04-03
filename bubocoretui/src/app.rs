use crate::components::{
    Component,
    editor::EditorComponent,
    grid::GridComponent,
    help::{HelpComponent, HelpState},
    options::OptionsComponent,
    splash::{ConnectionState, SplashComponent},
    navigation::NavigationComponent,
    devices::{DevicesComponent, DevicesState},
    logs::{LogsComponent, LogsState},
    files::{FilesComponent, FilesState},
};
use crate::event::{AppEvent, Event, EventHandler};
use crate::link::Link;
use crate::network::NetworkManager;
use crate::commands::CommandMode;
use crate::ui::Flash;
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

/// Taille maximale de la liste des logs
const MAX_LOGS: usize = 100;

/// Enumération représentant les différentes vues disponibles dans l'application.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Mode {
    Editor,
    Grid,
    Options,
    Splash,
    Help,
    Devices,
    Logs,
    Files,
    Navigation,
} 

pub struct ScreenState {
    /// Vue active de l'application
    pub mode: Mode,
    /// Effet de flash
    pub flash: Flash,
    /// Stocke le mode précédent lorsque l'overlay de navigation est ouvert
    pub previous_mode: Option<Mode>,
}

pub struct UserPosition {
    pub sequence_index: usize,
    pub step_index: usize,
}

/// Structure représentant l'état de l'éditeur de texte intégré
pub struct EditorData {
    pub active_sequence: UserPosition,
    pub line_count: usize,
    pub content: String,
    pub textarea: TextArea<'static>,
    pub pattern: Option<Pattern>,
}

/// Structure représentant l'état du serveur (horloge et réseau)
pub struct ServerState {
    /// Gestionnaire de réseau
    pub network: NetworkManager,
    /// Indique si le client est connecté au serveur
    pub is_connected: bool,
    /// Indique si le client est en train de se connecter au serveur
    pub is_connecting: bool,
    /// État de la connexion au serveur
    pub connection_state: Option<ConnectionState>,
    /// Nom du client
    pub username: String,
    /// Liste des pairs (autres clients)
    pub peers: Vec<String>,
    /// Liste des périphériques gérés par le serveur (MIDI, OSC, etc.)
    pub devices: Vec<String>,
    /// Horloge Ableton Link (le serveur possède aussi sa propre horloge)
    pub link: Link,
    /// Current step index for each sequence, updated by the server.
    pub current_step_positions: Option<Vec<usize>>,
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
    pub grid_cursor: (usize, usize),
    pub devices_state: DevicesState,
    pub logs_state: LogsState,
    pub files_state: FilesState,
    pub navigation_cursor: (usize, usize),
}

/// Enumération représentant les différents niveaux de logging possibles
#[derive(Clone, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

/// Structure représentant une entrée de log
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

/// Structure principale de l'application TUI
pub struct App {
    /// Indique si l'application est en cours d'exécution
    pub running: bool,
    /// État de l'interface utilisateur
    pub interface: InterfaceState,
    /// État de l'éditeur de texte intégré
    pub editor: EditorData,
    /// État du serveur (horloge et réseau)
    pub server: ServerState,
    /// Gestionnaire d'événements
    pub events: EventHandler,
    /// Liste des logs
    pub logs: VecDeque<LogEntry>,
}

impl App {
    /// Crée une nouvelle instance de l'application
    /// 
    /// # Arguments
    /// 
    /// * `ip` - L'adresse IP du serveur
    /// * `port` - Le port du serveur
    /// * `username` - Le nom du client
    pub fn new(ip: String, port: u16, username: String) -> Self {
        let events = EventHandler::new();
        let event_sender = events.sender.clone();
        let mut app = Self {
            running: true,
            editor: EditorData {
                content: String::new(),
                line_count: 1,
                active_sequence: UserPosition {
                    sequence_index: 0,
                    step_index: 0,
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
                current_step_positions: None,
            },
            interface: InterfaceState {
                screen: ScreenState {
                    mode: Mode::Splash,
                    flash: Flash {
                        is_flashing: false,
                        flash_start: None,
                        flash_duration: Duration::from_micros(20_000),
                    },
                    previous_mode: None,
                },
                components: ComponentState {
                    command_mode: CommandMode::new(),
                    help_state: None,
                    bottom_message: String::from("Press ENTER to start!"),
                    bottom_message_timestamp: None,
                    grid_cursor: (0, 0),
                    devices_state: DevicesState::new(),
                    logs_state: LogsState::new(),
                    files_state: FilesState::new(),
                    navigation_cursor: (0, 0),
                },
            },
            events,
            logs: VecDeque::with_capacity(MAX_LOGS),
        };
        // Active la synchronisation Link
        app.server.link.link.enable(true);
        // Initialise la connexion au serveur
        app.init_connection_state();
        app
    }

    /// Exécute la boucle principale de l'application.
    /// 
    /// Cette fonction gère le cycle de vie principal de l'application :
    /// - Dessine l'interface utilisateur.
    /// - Traite les événements (tick, clavier, application, réseau).
    /// - Continue jusqu'à ce que l'application soit interrompue.
    /// 
    /// # Arguments
    /// 
    /// * `terminal` - Le terminal utilisé pour le rendu de l'interface
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant :
    /// * `Ok(())` si l'application s'est terminée normalement
    /// * `Err` si une erreur s'est produite pendant l'exécution
    pub async fn run<B: Backend>(&mut self, mut terminal: Terminal<B>) -> EyreResult<()> {
        while self.running {
            terminal.draw(|frame| crate::ui::ui(frame, self))?;
            match self.events.next().await? {
                // Fonction périodique (vitesse du rafraîchissement)
                Event::Tick => self.tick(),
                // Gestion des événements clavier ou terminal
                Event::Crossterm(event) => match event {
                    CrosstermEvent::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Release {
                            continue;
                        }
                        let _ = self.handle_key_events(key_event)?;
                    }
                    _ => {}
                },
                // Gestion des événements liés à l'application
                Event::App(app_event) => self.handle_app_event(app_event)?,
                // Gestion des événements liés au réseau
                Event::Network(message) => self.handle_server_message(message),
            }
        }
        Ok(())
    }

    /// Initialise l'état de la connexion au serveur
    pub fn init_connection_state(&mut self) {
        let (ip, port) = self.server.network.get_connection_info();
        self.server.connection_state = Some(ConnectionState::new(&ip, port, &self.server.username));
    }

    /// Gère les messages reçus du serveur.
    /// 
    /// Cette fonction traite les différents types de messages que le serveur peut envoyer aux clients :
    /// - Messages de chat (en provenance des autres clients)
    /// - Mises à jour de la liste des pairs connectés
    /// - Handshake: initialisation de l'état de l'application à partir des informations reçues du serveur
    /// - État de l'horloge et synchronisation
    /// - Messages d'erreur et de log, etc.
    /// 
    /// # Arguments
    /// 
    /// * `message` - Le message reçu du serveur à traiter
    fn handle_server_message(&mut self, message: ServerMessage) {
        match message {
            // Messages de chat (en provenance des autres clients)
            ServerMessage::Chat(msg) => {
                self.add_log(LogLevel::Info, format!("Received: {}", msg.to_string()));
            }
            // Mise à jour de la liste des pairs connectés
            ServerMessage::PeersUpdated(peers) => {
                self.server.peers = peers;
                self.add_log(LogLevel::Info, format!("Peers updated: {}", self.server.peers.join(", ")));
            }
            // Handshake: le serveur envoie toutes les informations nécessaires à l'initialisation de l'état
            // de l'application. Ce message est requis pour toute première connexion au serveur par un client.
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
            // État de l'horloge et synchronisation
            ServerMessage::ClockState(tempo, _beat, _micros, quantum) => {
                self.set_status_message(format!("Clock sync: {:.1} BPM", tempo));
                let timestamp = self.server.link.link.clock_micros();
                self.server.link.session_state.set_tempo(tempo, timestamp);
                self.server.link.quantum = quantum;
                self.add_log(LogLevel::Info, format!("Tempo updated: {:.1} BPM", tempo));
            }
            ServerMessage::PatternValue(new_pattern) => {
                self.set_status_message(String::from("Received pattern update"));
                self.add_log(LogLevel::Debug, "Received PatternValue update.".to_string());
                self.editor.pattern = Some(new_pattern);
            }
            ServerMessage::StepPosition(positions) => {
                self.server.current_step_positions = Some(positions);
            }
            ServerMessage::PatternLayout(_layout) => {
            }
            // Message de succès (le serveur a réussi à traiter la requête souhaitée)
            ServerMessage::Success => {}
            // Message d'erreur interne (le serveur a rencontré une erreur interne et la signale)
            ServerMessage::InternalError(message) => {
                self.add_log(LogLevel::Error, message);
            }
            // Message de log (le serveur émet un message à destination des logs du client)
            ServerMessage::LogMessage(message) => {
                self.add_log(LogLevel::Info, message.to_string());
            }
            ServerMessage::StepEnabled(_a, _b) => {
            },
            ServerMessage::StepDisabled(_a, _b) => {

            },
            // Receive script content from server
            ServerMessage::ScriptContent { sequence_idx, step_idx, content } => {
                self.add_log(LogLevel::Info, format!("Received script for Seq {}, Step {}", sequence_idx, step_idx));
                // Update the textarea
                self.editor.textarea = TextArea::new(content.lines().map(|s| s.to_string()).collect());
                // Update active sequence/step
                self.editor.active_sequence.sequence_index = sequence_idx; // Store the sequence index
                self.editor.active_sequence.step_index = step_idx;       // Store the step index
                // Switch to editor view
                let _ = self.events.sender.send(Event::App(AppEvent::SwitchToEditor))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e));
                self.set_status_message(format!("Loaded script for Seq {}, Step {} into editor", sequence_idx, step_idx));
            }
        }
    }

    /// Ajoute un message de log à la liste des logs.
    /// 
    /// Cette fonction ajoute un message de log à la liste des logs de l'application.
    /// Elle vérifie également que la liste des logs ne dépasse pas la taille maximale autorisée.
    /// 
    /// # Arguments
    /// 
    /// * `level` - Le niveau de log (Info, Warn, Error, Debug)
    /// * `message` - Le message à ajouter à la liste des logs
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant :
    /// * `Ok(())` si le message a été ajouté à la liste des logs
    /// * `Err` si une erreur s'est produite pendant l'ajout
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

    /// Envoie un message au serveur.
    /// 
    /// Cette fonction envoie un message au serveur via le gestionnaire de réseau.
    /// Elle gère également les erreurs de connexion.
    /// 
    /// # Arguments
    /// 
    /// * `message` - Le message à envoyer au serveur
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant :
    /// * `Ok(())` si le message a été envoyé au serveur
    /// * `Err` si une erreur s'est produite pendant l'envoi
    pub fn send_client_message(&mut self, message: ClientMessage) {
        match self.server.network.send(message) {
            Ok(_) => {}
            Err(e) => {
                self.set_status_message(format!("Failed to send message: {}", e));
                self.server.is_connected = false;
            }
        }
    }

    /// Fonction exécutée périodiquement par l'application pour chaque frame du cycle événementiel.
    /// 
    /// - Cette fonction gère la suppression du message dans la barre inférieure après 3 secondes.
    fn tick(&mut self) {
        if let Some(timestamp) = self.interface.components.bottom_message_timestamp {
            if timestamp.elapsed() > Duration::from_secs(3) {
                self.interface.components.bottom_message = String::new();
                self.interface.components.bottom_message_timestamp = None;
            }
        }
    }

    /// Gère les événements de l'application.
    /// 
    /// Cette fonction gère les différents types d'événements que l'application peut recevoir :
    /// - Événements de bas niveau (clavier, terminal)
    /// - Événements de l'interface utilisateur (switch de mode, commandes, etc.)
    /// 
    /// # Arguments
    /// 
    /// * `event` - L'événement à traiter
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant :
    /// * `Ok(())` si l'événement a été traité avec succès
    /// * `Err` si une erreur s'est produite pendant le traitement
    /// 
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
            },
            AppEvent::SwitchToDevices => self.interface.screen.mode = Mode::Devices,
            AppEvent::SwitchToLogs => self.interface.screen.mode = Mode::Logs,
            AppEvent::SwitchToFiles => self.interface.screen.mode = Mode::Files,
            AppEvent::MoveNavigationCursor((dy, dx)) => {
                let (max_row, max_col) = (5, 1);
                let current_cursor = self.interface.components.navigation_cursor;
                let new_row = (current_cursor.0 as i32 + dy).clamp(0, max_row as i32) as usize;
                let new_col = (current_cursor.1 as i32 + dx).clamp(0, max_col as i32) as usize;
                self.interface.components.navigation_cursor = (new_row, new_col);
            },
            AppEvent::ExitNavigation => {
                 if let Some(prev_mode) = self.interface.screen.previous_mode.take() {
                    self.interface.screen.mode = prev_mode;
                 }
            },
            AppEvent::ExecuteCommand(cmd) => {
                self.interface.components.command_mode.exit();
                self.execute_command(&cmd)?;
            },
            AppEvent::SendScript(_script) => {},
            AppEvent::GetScript(_pattern_id, _step_id) => {},
            AppEvent::UpdateTempo(tempo) => {
                self.server.link.session_state.set_tempo(tempo, self.server.link.link.clock_micros());
                self.server.link.commit_app_state();
            },
            AppEvent::UpdateQuantum(quantum) => {
                self.server.link.quantum = quantum;
                self.server.link.capture_app_state();
                self.server.link.commit_app_state();
            },
            AppEvent::ToggleStartStopSync => {
                self.server.link.toggle_start_stop_sync();
                let state = self.server.link.link.is_start_stop_sync_enabled();
                self.set_status_message(format!("Start/Stop sync {}", if state { "enabled" } else { "disabled" }));
            },
            AppEvent::Quit => {
                self.quit();
            }
        }
        Ok(())
    }

    /// Gère les événements clavier.
    /// Priorité de gestion :
    /// 1. Quitter (Ctrl+C)
    /// 2. Mode Commande (Ctrl+P pour ouvrir, ESC pour fermer, Enter pour exec)
    /// 3. Raccourcis F1-F7
    /// 4. Navigation (ESC pour ouvrir/fermer, puis touches spécifiques si actif)
    /// 5. Délégation au composant de la vue active
    fn handle_key_events(&mut self, key_event: KeyEvent) -> EyreResult<bool> {
        let key_code = key_event.code;
        let key_modifiers = key_event.modifiers;

        // 1. Mode commande (Ctrl+P)
        if self.interface.components.command_mode.active {
            match key_code {
                KeyCode::Esc => {
                    self.interface.components.command_mode.exit(); 
                    return Ok(true);
                }
                KeyCode::Enter => {
                    let command = self.interface.components.command_mode.get_command();
                    self.events.sender.send(Event::App(AppEvent::ExecuteCommand(command)))?;
                    return Ok(true);
                }
                 // Ctrl+P also exits if already active
                KeyCode::Char('p') if key_modifiers == KeyModifiers::CONTROL => {
                    self.interface.components.command_mode.exit();
                    return Ok(true); // Consume Ctrl+P
                }
                 _ => { 
                    let handled_by_textarea = self.interface.components.command_mode.text_area.input(key_event);
                    return Ok(handled_by_textarea);
                }
            }
        }
        if key_modifiers == KeyModifiers::CONTROL && key_code == KeyCode::Char('p') {
             // We already handled the case where it was active above, so here it must be inactive
             self.interface.components.command_mode.enter();
             return Ok(true); // Consume Ctrl+P
        }


        // 2. Quitter l'application (Ctrl+C)
        if key_modifiers == KeyModifiers::CONTROL && key_code == KeyCode::Char('c') {
            self.events.sender.send(Event::App(AppEvent::Quit))?;
            return Ok(true);
        }
 
        // 4. Autres actions globales (Touches de fonction, etc.)
        match key_code {
            KeyCode::F(1) => {
                self.events.sender.send(Event::App(AppEvent::SwitchToEditor))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                return Ok(true);
            }
            KeyCode::F(2) => {
                self.events.sender.send(Event::App(AppEvent::SwitchToGrid))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                return Ok(true); 
            }
            KeyCode::F(3) => {
                self.events.sender.send(Event::App(AppEvent::SwitchToOptions))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                return Ok(true); 
            }
            KeyCode::F(4) => {
                self.events.sender.send(Event::App(AppEvent::SwitchToHelp))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                 return Ok(true);
            }
            KeyCode::F(5) => {
                self.events.sender.send(Event::App(AppEvent::SwitchToDevices))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                 return Ok(true);
            }
            KeyCode::F(6) => {
                self.events.sender.send(Event::App(AppEvent::SwitchToLogs))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                 return Ok(true);
            }
            KeyCode::F(7) => {
                self.events.sender.send(Event::App(AppEvent::SwitchToFiles))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                 return Ok(true);
            }
            KeyCode::Tab => {
                if self.interface.screen.mode == Mode::Navigation {
                    self.events.sender.send(Event::App(AppEvent::ExitNavigation))
                        .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                    return Ok(true); 
                } 
            }
            _ => {}
        }

        // 5. Touche Tab pour quitter le mode de navigation
        if key_code == KeyCode::Tab && self.interface.screen.mode != Mode::Navigation {
            self.interface.screen.previous_mode = Some(self.interface.screen.mode);
            self.interface.screen.mode = Mode::Navigation;
            return Ok(true);
        }

        // 6. Déléguer au composant actif
        let handled = match self.interface.screen.mode {
            Mode::Navigation => NavigationComponent::new().handle_key_event(self, key_event)?,
            Mode::Editor => EditorComponent::new().handle_key_event(self, key_event)?,
            Mode::Grid => GridComponent::new().handle_key_event(self, key_event)?,
            Mode::Options => OptionsComponent::new().handle_key_event(self, key_event)?,
            Mode::Splash => SplashComponent::new().handle_key_event(self, key_event)?,
            Mode::Help => HelpComponent::new().handle_key_event(self, key_event)?,
            Mode::Devices => {
                let mut comp = DevicesComponent::new();
                comp.handle_key_event(self, key_event)?
            }
            Mode::Logs => LogsComponent::new().handle_key_event(self, key_event)?,
            Mode::Files => FilesComponent::new().handle_key_event(self, key_event)?,
        };
        
        Ok(handled)
    }

    /// Fonction de fermeture de l'application.
    /// 
    /// Cette fonction désactive la boucle principale de l'application.
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant :
    /// * `Ok(())` si l'application a été fermée avec succès
    /// * `Err` si une erreur s'est produite pendant la fermeture
    /// 
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn flash_screen(&mut self) {
        self.interface.screen.flash.is_flashing = true;
        self.interface.screen.flash.flash_start = Some(Instant::now());
    }

    /// Définit un message à afficher dans la barre inférieure.
    pub fn set_status_message(&mut self, message: String) {
        self.interface.components.bottom_message = message;
        self.interface.components.bottom_message_timestamp = Some(Instant::now());
    }
}
