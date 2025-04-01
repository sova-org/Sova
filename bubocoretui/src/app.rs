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
pub enum Mode {
    Editor,
    Grid,
    Options,
    Splash,
    Help,
}

pub struct ScreenState {
    /// Vue active de l'application
    pub mode: Mode,
    /// Effet de flash
    pub flash: Flash,
}

pub struct UserPosition {
    pub pattern: usize,
    pub script: usize,
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
                    pattern: 0 as usize,
                    script: 0 as usize,
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
                        flash_duration: Duration::from_micros(200_000),
                    },
                },
                components: ComponentState {
                    command_mode: CommandMode::new(),
                    help_state: None,
                    bottom_message: String::from("Press ENTER to start!"),
                    bottom_message_timestamp: None,
                    grid_cursor: (0, 0),
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
                // 60FPS  
                Event::Tick => self.tick(),
                // Gestion des événements clavier ou terminal
                Event::Crossterm(event) => match event {
                    CrosstermEvent::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Release {
                            continue;
                        }
                        self.handle_key_events(key_event)?
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
            // Mises à jour de la liste des pairs connectés
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
                // Add log to confirm reception
                self.add_log(LogLevel::Debug, "Received and processing PatternValue update.".to_string());
                self.editor.pattern = Some(new_pattern);
            }
            ServerMessage::StepPosition(positions) => {
                // Optional: Log reception for debugging
                // self.add_log(LogLevel::Debug, format!("Received step positions: {:?}", positions));
                self.server.current_step_positions = Some(positions);
            }
            ServerMessage::PatternLayout(_layout) => {
            }
            // Message de succès (le serveur a réussi à traiter la requête souhaitée)
            ServerMessage::Success => {} // This is likely received after sending a command, but doesn't update state.
            // Message d'erreur interne (le serveur a rencontré une erreur interne et la signale)
            ServerMessage::InternalError(message) => {
                self.add_log(LogLevel::Error, message);
            }
            // Message de log (le serveur émet un message à destination des logs du client)
            ServerMessage::LogMessage(message) => {
                self.add_log(LogLevel::Info, message.to_string());
            }
            ServerMessage::StepEnabled(a, b) => {},
            ServerMessage::StepDisabled(a, b) => {},
            /* // Commenting out as server doesn't send these; updates come via PatternValue
            ServerMessage::StepEnabled(sequence_index, step_index) => {
                if let Some(pattern) = self.editor.pattern.as_mut() {
                   if let Some(sequence) = pattern.sequences.get_mut(sequence_index) {
                       sequence.enable_step(step_index);
                       self.set_status_message(format!("Server confirmed: Step enabled [Seq: {}, Step: {}]", sequence_index, step_index));
                   } else {
                        self.add_log(LogLevel::Warn, format!("Received StepEnabled for invalid sequence index: {}", sequence_index));
                   }
                } else {
                    self.add_log(LogLevel::Warn, "Received StepEnabled but no pattern is loaded locally.".to_string());
                }
            }
            ServerMessage::StepDisabled(sequence_index, step_index) => {
                 if let Some(pattern) = self.editor.pattern.as_mut() {
                   if let Some(sequence) = pattern.sequences.get_mut(sequence_index) {
                       sequence.disable_step(step_index);
                       self.set_status_message(format!("Server confirmed: Step disabled [Seq: {}, Step: {}]", sequence_index, step_index));
                   } else {
                       self.add_log(LogLevel::Warn, format!("Received StepDisabled for invalid sequence index: {}", sequence_index));
                   }
                 } else {
                     self.add_log(LogLevel::Warn, "Received StepDisabled but no pattern is loaded locally.".to_string());
                 }
             }
             */
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
            AppEvent::SwitchToGrid => {
                self.interface.screen.mode = Mode::Grid;
            },
            AppEvent::SwitchToOptions => self.interface.screen.mode = Mode::Options,
            // Bascule vers la vue d'aide
            AppEvent::SwitchToHelp => {
                self.interface.screen.mode = Mode::Help;
                // Une initialisation est nécessaire pour la première utilisation
                if self.interface.components.help_state.is_none() {
                    self.interface.components.help_state = Some(HelpState::new());
                }
            }
            // Bascule vers la vue suivante suivant le mode actif
            AppEvent::NextScreen => {
                self.interface.screen.mode = match self.interface.screen.mode {
                    Mode::Editor => Mode::Grid,
                    Mode::Grid => Mode::Options,
                    Mode::Options => Mode::Editor,
                    Mode::Help => Mode::Editor,
                    Mode::Splash => Mode::Editor,
                };
            }
            // Active le mode permettant l'affichage du command prompt
            AppEvent::EnterCommandMode => {
                self.interface.components.command_mode.enter();
            }
            // Désactive le mode permettant l'affichage du command prompt
            AppEvent::ExitCommandMode => {
                self.interface.components.command_mode.exit();
            }
            // Exécution d'une commande interne via le command prompt
            AppEvent::ExecuteCommand(cmd) => {
                match self.execute_command(&cmd) {
                    Ok(_) => {}
                    Err(e) => {
                        self.set_status_message(format!("Error: {}", e));
                    }
                }
                self.interface.components.command_mode.exit();
            }
            AppEvent::SendScript(_script) => {

            }
            AppEvent::GetScript(_pattern_id, _step_id) => {

            }
            // Mise à jour du temp de l'horloge Ableton Link
            AppEvent::UpdateTempo(tempo) => {
                self.server
                    .link
                    .session_state
                    .set_tempo(tempo, self.server.link.link.clock_micros());
                self.server.link.commit_app_state();
            }
            // Mise à jour du quantum de l'horloge Ableton Link
            AppEvent::UpdateQuantum(quantum) => {
                self.server.link.quantum = quantum;
                self.server.link.capture_app_state();
                self.server.link.commit_app_state();
            }
            // Activation/désactivation de la synchronisation start/stop (Ableton Link)
            AppEvent::ToggleStartStopSync => {
                self.server.link.toggle_start_stop_sync();
                let state = self.server.link.link.is_start_stop_sync_enabled();
                self.set_status_message(format!(
                    "Start/Stop sync {}",
                    if state { "enabled" } else { "disabled" }
                ));
            }
            // Arrêt de l'application
            AppEvent::Quit => {
                self.quit();
            }
        }
        Ok(())
    }

    /// Gère les événements clavier.
    /// 
    /// Cette fonction gère les différents types d'événements clavier que l'application peut recevoir :
    /// 
    /// # Arguments
    /// 
    /// * `key_event` - L'événement clavier à traiter
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant :
    /// * `Ok(())` si l'événement a été traité avec succès
    /// * `Err` si une erreur s'est produite pendant le traitement
    fn handle_key_events(&mut self, key_event: KeyEvent) -> EyreResult<()> {
        // Ouverture/fermeture du command prompt
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
        // Traitement des événements liés au command prompt
        if self.interface.components.command_mode.active {
            match key_event.code {
                // Fermeture du command prompt avant envoi de la commande
                KeyCode::Esc | KeyCode::Char('c')
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.events.send(AppEvent::ExitCommandMode);
                }
                // Exécution de la commande saisie
                KeyCode::Enter => {
                    let cmd = self.interface.components.command_mode.get_command();
                    self.events.send(AppEvent::ExecuteCommand(cmd));
                }
                // Toute autre touche est traitée comme une entrée de caractère
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

        // Traitement des événements en lien avec chacun des modes actifs
        // FIX: est-il nécessaire de reconstruire les composants à chaque fois ?
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
        if !handled { }

        Ok(())
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

    /// Active l'effet de flash sur l'écran.
    /// 
    /// Cette fonction active l'effet de flash sur l'écran de l'application.
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant :
    /// * `Ok(())` si l'effet de flash a été activé avec succès
    /// * `Err` si une erreur s'est produite pendant l'activation de l'effet de flash
    /// 
    pub fn flash_screen(&mut self) {
        self.interface.screen.flash.is_flashing = true;
        self.interface.screen.flash.flash_start = Some(Instant::now());
    }

    pub fn set_content(&mut self, content: String) {
        self.editor.content = content;
        self.editor.line_count = self.editor.content.lines().count().max(1);
    }

    /// Définit un message à afficher dans la barre inférieure.
    /// 
    /// 
    pub fn set_status_message(&mut self, message: String) {
        self.interface.components.bottom_message = message;
        self.interface.components.bottom_message_timestamp = Some(Instant::now());
    }
}
