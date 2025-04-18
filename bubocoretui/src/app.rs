use crate::components::{
    Component,
    command_palette::{CommandPaletteComponent, PaletteAction},
    editor::EditorComponent,
    grid::GridComponent,
    help::{HelpComponent, HelpState},
    options::OptionsComponent,
    splash::{ConnectionState, SplashComponent},
    navigation::NavigationComponent,
    logs::{LogsComponent, LogEntry, LogLevel},
    devices::{DevicesComponent, DevicesState},
    saveload::{SaveLoadComponent, SaveLoadState},
    editor::SearchState,
};
use crate::event::{AppEvent, Event, EventHandler};
use crate::link::Link;
use crate::network::NetworkManager;
use crate::ui::Flash;
use crate::disk;
use bubocorelib::scene::Scene;
use bubocorelib::server::{ServerMessage, client::ClientMessage};
use bubocorelib::shared_types::{DeviceInfo, GridSelection, DeviceKind};
use color_eyre::Result as EyreResult;
use ratatui::{
    Terminal,
    style::Color,
    backend::Backend,
    crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use std::time::{Duration, Instant};
use chrono::Local;
use tui_textarea::TextArea;
use std::collections::{VecDeque, HashMap};
use bubocorelib::compiler::CompilationError;

/// Maximum number of log entries to keep.
const MAX_LOGS: usize = 100;

/// Represents the different primary views or screens of the application.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Mode {
    Editor,
    Grid,
    Options,
    Splash,
    Help,
    Devices,
    Logs,
    Navigation,
    SaveLoad,
} 

#[derive(Clone, Debug)]
pub struct CopiedFrameData {
    pub length: f64,
    pub is_enabled: bool,
    pub script_content: Option<String>, 
    pub source_col: usize,
    pub source_row: usize,
}

#[derive(Clone, Debug, Default)]
pub enum ClipboardState {
    #[default]
    Empty,
    // Stores length/state immediately, waits for script
    FetchingScript {
        col: usize,
        row: usize,
        length: f64,
        is_enabled: bool,
    },
    // All available data is ready
    Ready(CopiedFrameData),
}

/// Represents the observable state of a connected peer.
#[derive(Debug, Clone, Default)]
pub struct PeerSessionState {
    /// The peer's last known grid cursor/selection state.
    pub grid_selection: Option<GridSelection>,
    /// The specific frame the peer is currently editing (if any).
    pub editing_frame: Option<(usize, usize)>, // (line_idx, frame_idx)
    // Add other states later, e.g.:
    // pub current_focus: Option<FocusArea>,
    // pub editing_status: Option<EditingStatus>,
}

/// State related to screen rendering and navigation history.
pub struct ScreenState {
    /// The currently active application mode (view).
    pub mode: Mode,
    /// State for the screen flash effect.
    pub flash: Flash,
    /// Stores the previous mode when an overlay (like Navigation) is active.
    pub previous_mode: Option<Mode>,
}

/// Represents the user's current position within the scene (line and frame).
pub struct UserPosition {
    pub line_index: usize,
    pub frame_index: usize,
}

/// State specific to the text editor component.
pub struct EditorData {
    /// The line and frame currently being edited or viewed.
    pub active_line: UserPosition,
    /// The `tui_textarea` widget state for the editor.
    pub textarea: TextArea<'static>,
    /// The currently loaded scene data.
    pub scene: Option<Scene>,
    /// Stores the last compilation error related to the currently viewed script.
    pub compilation_error: Option<CompilationError>,
    /// Holds the state for the search functionality within the editor.
    pub search_state: SearchState,
}

/// State related to the server connection, clock sync, and shared data.
pub struct ServerState {
    /// Manages the network connection to the server.
    pub network: NetworkManager,
    /// Flag indicating if the WebSocket connection is currently established.
    pub is_connected: bool,
    /// Flag indicating if a connection attempt is in progress.
    pub is_connecting: bool,
    /// State specifically for the splash screen connection display.
    pub connection_state: Option<ConnectionState>,
    /// This client's username.
    pub username: String,
    /// List of usernames of other connected clients.
    pub peers: Vec<String>,
    /// List of device names managed by the server.
    pub devices: Vec<DeviceInfo>,
    /// State related to Ableton Link synchronization.
    pub link: Link,
    /// Current frame index for each line, updated by the server.
    pub current_frame_positions: Option<Vec<usize>>,
    /// Stores the last known state of other connected peers.
    pub peer_sessions: HashMap<String, PeerSessionState>,
    /// Flag indicating if the server transport is currently playing.
    pub is_transport_playing: bool,
}

/// Holds the primary state categories of the application interface.
pub struct InterfaceState {
    /// State related to the overall screen and mode.
    pub screen: ScreenState,
    /// State specific to different UI components.
    pub components: ComponentState,
}

/// Aggregates the state for various interactive UI components.
pub struct ComponentState {
    /// State for the command palette component.
    pub command_palette: CommandPaletteComponent,
    /// State for the help screen component.
    pub help_state: Option<HelpState>,
    /// Current message displayed in the bottom status bar.
    pub bottom_message: String,
    /// Timestamp when the bottom message was set (for potential auto-clearing).
    pub bottom_message_timestamp: Option<Instant>,
    /// User's current selection within the scene grid.
    pub grid_selection: GridSelection,
    /// State for the devices list component.
    pub devices_state: DevicesState,
    /// State for the logs view component.
    pub logs_state: LogsState,
    /// State for the save/load component.
    pub save_load_state: SaveLoadState,
    /// Name of the project being saved via command palette.
    pub pending_save_name: Option<String>,
    /// Cursor position within the navigation overlay.
    pub navigation_cursor: (usize, usize),
    /// Flag indicating if the user is currently inputting a frame length.
    pub is_setting_frame_length: bool,
    /// Text area for frame length input.
    pub frame_length_input: TextArea<'static>,
}

/// Application-wide settings.
#[derive(Clone, Copy, Debug)]
pub struct AppSettings {
    /// Whether to display the phase progress bar at the top.
    pub show_phase_bar: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { show_phase_bar: false }
    }
}

/// Main application state structure.
pub struct App {
    pub clipboard: ClipboardState,
    /// Controls the main application loop. Set to `false` to exit.
    pub running: bool,
    /// Holds the state related to the UI layout and components.
    pub interface: InterfaceState,
    /// State specific to the script editor.
    pub editor: EditorData,
    /// State related to the server connection and synchronization.
    pub server: ServerState,
    /// Handles event queuing and dispatching.
    pub events: EventHandler,
    /// A queue of log messages displayed in the Logs view.
    pub logs: VecDeque<LogEntry>,
    /// User-configurable application settings.
    pub settings: AppSettings,
}

impl App {
    /// Creates a new `App` instance.
    /// 
    /// # Arguments
    /// 
    /// * `ip` - The server's IP address.
    /// * `port` - The server's port.
    /// * `username` - The username for this client.
    pub fn new(ip: String, port: u16, username: String) -> Self {
        let events = EventHandler::new();
        let event_sender = events.sender.clone();
        let mut app = Self {
            running: true,
            editor: EditorData {
                active_line: UserPosition {
                    line_index: 0,
                    frame_index: 0,
                },
                textarea: TextArea::default(),
                scene: None,
                compilation_error: None,
                search_state: SearchState::new(),
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
                current_frame_positions: None,
                peer_sessions: HashMap::new(),
                is_transport_playing: false,
            },
            interface: InterfaceState {
                screen: ScreenState {
                    mode: Mode::Splash,
                    flash: Flash {
                        is_flashing: false,
                        flash_start: None,
                        flash_color: Color::White,
                        flash_duration: Duration::from_micros(20_000),
                    },
                    previous_mode: None,
                },
                components: ComponentState {
                    command_palette: CommandPaletteComponent::new(),
                    help_state: None,
                    bottom_message: String::from("Press ENTER to start! or Ctrl+P for commands"),
                    bottom_message_timestamp: None,
                    grid_selection: GridSelection::single(0, 0),
                    devices_state: DevicesState::new(),
                    logs_state: LogsState::new(),
                    save_load_state: SaveLoadState::new(),
                    pending_save_name: None,
                    navigation_cursor: (0, 0),
                    is_setting_frame_length: false,
                    frame_length_input: TextArea::default(),
                },
            },
            events,
            logs: VecDeque::with_capacity(MAX_LOGS),
            settings: AppSettings::default(),
            clipboard: ClipboardState::default(),
        };
        // Enable Ableton Link synchronization.
        app.server.link.link.enable(true);
        // Initialize the splash screen connection state display.
        app.init_connection_state();
        app
    }

    /// Runs the main application loop.
    /// 
    /// This function handles the application's lifecycle:
    /// - Processes events (tick, keyboard, application, network).
    /// - Draws the UI based on the current state.
    /// - Continues until `self.running` is set to `false`.
    /// 
    /// # Arguments
    /// 
    /// * `terminal` - The terminal backend used for rendering.
    /// 
    /// # Returns
    /// 
    /// - `Ok(())` if the application exits normally.
    /// - `Err` if an error occurs during execution.
    pub async fn run<B: Backend>(&mut self, mut terminal: Terminal<B>) -> EyreResult<()> {
        while self.running {
            // Process the next event FIRST
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    CrosstermEvent::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Release {
                            continue;
                        }
                        let _ = self.handle_key_events(key_event)?;
                    }
                    _ => {}
                },
                Event::App(app_event) => self.handle_app_event(app_event)?,
                Event::Network(message) => self.handle_server_message(message),
            }

            // Only draw if still running after handling the event
            if !self.running {
                break;
            }

            // THEN draw the UI based on the updated state
            terminal.draw(|frame| {
                crate::ui::ui(frame, self);
                self.interface.components.command_palette.draw(frame);
            })?;
        }
        Ok(())
    }

    /// Initializes the connection state display for the splash screen.
    pub fn init_connection_state(&mut self) {
        let (ip, port) = self.server.network.get_connection_info();
        self.server.connection_state = Some(ConnectionState::new(&ip, port, &self.server.username));
    }

    /// Handles messages received from the server.
    /// 
    /// Updates the application state based on the content of the `ServerMessage`.
    /// 
    /// # Arguments
    /// 
    /// * `message` - The `ServerMessage` to process.
    fn handle_server_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::TransportStarted => {
                self.add_log(LogLevel::Info, "Transport started".to_string());
                self.server.is_transport_playing = true;
            }
            ServerMessage::TransportStopped => {
                self.add_log(LogLevel::Info, "Transport stopped".to_string());
                self.server.is_transport_playing = false;
            }
            ServerMessage::CompilationErrorOccurred(error) => {
                self.editor.compilation_error = Some(error.clone());
                self.add_log(LogLevel::Error, format!("Compilation error: {}", error));
            }
            // Received a chat message from another peer.
            ServerMessage::Chat(msg) => {
                self.add_log(LogLevel::Info, format!("Received: {}", msg.to_string()));
            }
            // Received an updated list of connected peers.
            ServerMessage::PeersUpdated(peers) => {
                self.server.peers = peers.clone(); // Clone for log message
                self.add_log(LogLevel::Info, format!("Peers updated: {}", self.server.peers.join(", ")));

                // Also update peer_sessions based on PeersUpdated
                let current_peer_set: std::collections::HashSet<_> = peers.into_iter().collect();
                self.server.peer_sessions.retain(|username, _| current_peer_set.contains(username));
                self.add_log(LogLevel::Debug, format!("Peer sessions map cleaned. Size: {}", self.server.peer_sessions.len())); // Debug log
            }
            // Initial state synchronization after connecting.
            ServerMessage::Hello { username, scene, devices, peers, link_state, is_playing: _ } => {
                self.set_status_message(format!("Handshake successful for {}", username));
                // Store the initial scene
                self.editor.scene = Some(scene.clone());
                // Directly assign the Vec<DeviceInfo>
                self.server.devices = devices;
                self.server.is_connected = true;
                self.server.is_connecting = false;

                // Update Link state from Hello message
                let (tempo, _beat, _phase, num_peers, is_enabled) = link_state;
                let timestamp = self.server.link.link.clock_micros(); // Get current time for tempo setting
                self.server.link.session_state.set_tempo(tempo, timestamp);
                // Set enabled status using the link instance
                self.server.link.link.enable(is_enabled);
                // Log num_peers but don't store it in app.server.link
                self.add_log(LogLevel::Debug, format!("Link status from Hello: Tempo={}, Peers={}, Enabled={}", tempo, num_peers, is_enabled));
                // Removed: self.server.link.num_peers = num_peers; 
                // Removed: self.server.link.is_enabled = is_enabled;
                // Also update quantum if available (maybe add to Hello?)
                // self.server.link.quantum = quantum;

                // Initialize peer sessions map based on initial client list
                self.server.peer_sessions.clear(); // Clear any old state
                for peer_name in peers.iter() { 
                    if peer_name != &username { // Don't add self
                        self.server.peer_sessions.insert(peer_name.clone(), PeerSessionState::default());
                    }
                }
                self.add_log(LogLevel::Debug, format!("Peer sessions map initialized after Hello. Size: {}", self.server.peer_sessions.len())); 

                // Assign username and peers
                self.server.username = username;
                self.server.peers = peers; 

                // Check if we can request the first script (Line 0, Frame 0)
                let mut request_first_script = false;
                if let Some(first_line) = scene.lines.get(0) {
                    if !first_line.frames.is_empty() {
                        request_first_script = true;
                    }
                }

                if request_first_script {
                    self.add_log(LogLevel::Info, "Requesting script for Line 0, Frame 0 after handshake.".to_string());
                    self.send_client_message(ClientMessage::GetScript(0, 0));
                } else {
                     self.add_log(LogLevel::Info, "No script requested after handshake (scene empty or line 0 has no frames).".to_string());
                    if matches!(self.interface.screen.mode, Mode::Splash) {
                         let _ = self.events.sender.send(Event::App(AppEvent::SwitchToGrid))
                            .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e));
                    }
                }
            }
            // Received clock state update from the server.
            // This might become less important if Link state is in Hello
            ServerMessage::ClockState(tempo, _beat, _micros, quantum) => {
                self.set_status_message(format!("Clock sync: {:.1} BPM", tempo));
                let timestamp = self.server.link.link.clock_micros();
                self.server.link.session_state.set_tempo(tempo, timestamp);
                self.server.link.quantum = quantum;
                self.add_log(LogLevel::Info, format!("Tempo updated: {:.1} BPM", tempo));
            }
            ServerMessage::SceneValue(new_scene) => {
                self.set_status_message(String::from("Received scene update"));
                self.editor.scene = Some(new_scene);
            }
            ServerMessage::FramePosition(positions) => {
                if let Some(scene) = &self.editor.scene {
                    let num_lines = scene.lines.len();
                    let mut current_frames = self.server.current_frame_positions
                        .take()
                        .unwrap_or_else(|| vec![usize::MAX; num_lines]);

                    if current_frames.len() != num_lines {
                        self.add_log(LogLevel::Warn, format!("Resizing current_frame_positions from {} to {}", current_frames.len(), num_lines));
                        current_frames.resize(num_lines, usize::MAX);
                    }

                    for (line_idx, frame_idx) in positions {
                        if line_idx < current_frames.len() {
                            current_frames[line_idx] = frame_idx;
                        } else {
                            self.add_log(LogLevel::Warn, format!("Received FramePosition for invalid line index: {} (max is {})", line_idx, current_frames.len() - 1));
                        }
                    }
                    self.server.current_frame_positions = Some(current_frames);
                } else {
                    self.add_log(LogLevel::Warn, "Received FramePosition but no scene loaded, clearing state.".to_string());
                    self.server.current_frame_positions = None;
                }
            }
            ServerMessage::Success => {}
            ServerMessage::InternalError(message) => {
                self.add_log(LogLevel::Error, message);
            }
            // Use LogString instead of LogMessage
            ServerMessage::LogString(message) => {
                self.add_log(LogLevel::Info, message);
            }
            ServerMessage::ScriptContent { line_idx, frame_idx, content } => {
                self.add_log(LogLevel::Debug, format!("Received script for ({}, {})", line_idx, frame_idx));

                // Check if this matches an ongoing clipboard fetch
                let match_clipboard = if let ClipboardState::FetchingScript { col, row, .. } = self.clipboard {
                    col == line_idx && row == frame_idx
                } else {
                    false
                };

                if match_clipboard {
                    // Consume content into the clipboard state
                    if let ClipboardState::FetchingScript { col, row, length, is_enabled } = self.clipboard {
                         self.clipboard = ClipboardState::Ready(CopiedFrameData {
                             length,
                             is_enabled,
                             script_content: Some(content), // Move content here
                             source_col: col,
                             source_row: row,
                         });
                         self.set_status_message("Script copied to clipboard.".to_string());
                         self.add_log(LogLevel::Info, format!("Stored script for ({},{}) in clipboard.", col, row));
                    } else {
                        // Should be unreachable due to `match_clipboard` check, but handle defensively
                         self.add_log(LogLevel::Error, "Clipboard state mismatch during ScriptContent handling!".to_string());
                    }
                } else {
                    // Assume it's for the editor: consume content here
                    self.add_log(LogLevel::Info, format!("Loading script for ({}, {}) into editor.", line_idx, frame_idx));
                    self.editor.compilation_error = None;
                    self.editor.textarea = TextArea::new(content.lines().map(|s| s.to_string()).collect()); // Move content here
                    self.editor.active_line.line_index = line_idx;
                    self.editor.active_line.frame_index = frame_idx;
                    // Switch to editor view
                    let _ = self.events.sender.send(Event::App(AppEvent::SwitchToEditor))
                        .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e));
                    self.set_status_message(format!("Loaded script for Line {}, Frame {} into editor", line_idx, frame_idx));
                }
            }
            // Received a snapshot from the server, usually after a `GetSnapshot` request.
            ServerMessage::Snapshot(snapshot) => {
                self.add_log(LogLevel::Info, "Received snapshot from server for saving.".to_string());

                // Check if a save was initiated via command palette
                let project_name = self.interface.components.pending_save_name.take()
                    .or_else(|| {
                        // Fallback: Check if SaveLoad view initiated it
                        let input_text = self.interface.components.save_load_state.input_area.lines()[0].trim();
                        if !input_text.is_empty() {
                            Some(input_text.to_string())
                        } else {
                            None
                        }
                    });

                if let Some(proj_name) = project_name {
                    if !proj_name.is_empty() {
                        self.add_log(LogLevel::Info, format!("Saving snapshot as project: {}", proj_name));
                        let event_sender = self.events.sender.clone();
                        let proj_name_clone = proj_name.clone(); // Clone for async task

                        tokio::spawn(async move {
                            match disk::save_project(&snapshot, &proj_name_clone).await {
                                Ok(_) => {
                                    let refresh_result = disk::list_projects().await;
                                    let event_result = refresh_result.map_err(|e| e.to_string());
                                    let _ = event_sender.send(Event::App(AppEvent::ProjectListLoaded(event_result)));
                                }
                                Err(e) => {
                                    eprintln!("Error saving project '{}': {}", proj_name_clone, e);
                                    // Optionally send an error event back to app
                                    // let _ = event_sender.send(Event::App(AppEvent::ProjectSaveError(e.to_string())));
                                }
                            }
                        });
                        self.interface.components.save_load_state.status_message = format!("Project '{}' saved.", proj_name);
                        self.set_status_message(format!("Project '{}' saved successfully.", proj_name));
                        // Clear the input area in SaveLoadState if it was used as fallback
                        self.interface.components.save_load_state.input_area = TextArea::default(); 
                    } else {
                         self.add_log(LogLevel::Warn, "Received snapshot but project name was empty.".to_string());
                         self.interface.components.save_load_state.status_message = "Save failed: Project name empty.".to_string();
                    }
                } else {
                    self.add_log(LogLevel::Warn, "Received snapshot but no project name was stored or provided for saving.".to_string());
                    self.interface.components.save_load_state.status_message = "Save failed: No project name.".to_string();
                }
            }
            // Received a grid selection update from another peer.
            ServerMessage::PeerGridSelectionUpdate(username, selection) => {
                if username != self.server.username { // Don't process updates about self
                    self.add_log(LogLevel::Debug, format!("Received grid selection update for peer '{}': {:?}", username, selection)); // Use Debug level
                    // Get or insert the peer's state entry
                    let peer_state = self.server.peer_sessions.entry(username.clone()).or_default();
                    // Update the grid selection field
                    peer_state.grid_selection = Some(selection);
                }
            }
            // Received notification that a peer started editing a frame
            ServerMessage::PeerStartedEditing(username, line_idx, frame_idx) => {
                if username != self.server.username {
                    self.add_log(LogLevel::Debug, format!("Peer '{}' started editing Line {}, Frame {}", username, line_idx, frame_idx));
                    let peer_state = self.server.peer_sessions.entry(username.clone()).or_default();
                    peer_state.editing_frame = Some((line_idx, frame_idx));
                }
            }
            // Received notification that a peer stopped editing a frame
            ServerMessage::PeerStoppedEditing(username, line_idx, frame_idx) => {
                 if username != self.server.username {
                     self.add_log(LogLevel::Debug, format!("Peer '{}' stopped editing Line {}, Frame {}", username, line_idx, frame_idx));
                     let peer_state = self.server.peer_sessions.entry(username.clone()).or_default();
                     // Only clear if they stopped editing the *same* frame we thought they were editing
                     if peer_state.editing_frame == Some((line_idx, frame_idx)) {
                         peer_state.editing_frame = None;
                     }
                 }
            }
            ServerMessage::SceneLength(length) => {
                self.add_log(LogLevel::Info, format!("Scene length updated to: {}", length));
                if let Some(scene) = &mut self.editor.scene {
                    scene.length = length;
                } else {
                    self.add_log(LogLevel::Warn, "Received SceneLength update but no scene is currently loaded.".to_string());
                }
            }
            ServerMessage::DeviceList(devices) => {
                self.add_log(LogLevel::Info, format!("Received updated device list ({} devices)", devices.len()));
                
                // 1. Update the main device list
                self.server.devices = devices.clone(); 
                
                // 2. Extract and update the slot assignments map in DevicesState
                let slot_assignments_clone;
                let midi_selected_index_clone;
                let osc_selected_index_clone;
                let tab_index_clone;
                {
                    // Scope for the mutable borrow of state
                    let state = &mut self.interface.components.devices_state;
                    state.slot_assignments.clear();
                    for device in devices.iter() {
                        if device.id != 0 {
                            state.slot_assignments.insert(device.id, device.name.clone());
                        }
                    }
                    // Clone necessary state before releasing the borrow
                    slot_assignments_clone = state.slot_assignments.clone();
                    midi_selected_index_clone = state.midi_selected_index;
                    osc_selected_index_clone = state.osc_selected_index;
                    tab_index_clone = state.tab_index;
                } // state borrow ends here

                // Call add_log without state being borrowed
                self.add_log(LogLevel::Debug, format!("Updated slot assignments: {:?}", slot_assignments_clone));

                // 3. Clamp selection indices using cloned values
                let midi_count = devices.iter().filter(|d| d.kind == DeviceKind::Midi).count();
                let osc_count = devices.iter().filter(|d| d.kind == DeviceKind::Osc).count();

                let new_midi_selected_index = midi_selected_index_clone.min(midi_count.saturating_sub(1));
                let new_osc_selected_index = osc_selected_index_clone.min(osc_count.saturating_sub(1));
                
                // Re-borrow state mutably to update the clamped indices
                {
                    let state = &mut self.interface.components.devices_state;
                    state.midi_selected_index = new_midi_selected_index;
                    state.osc_selected_index = new_osc_selected_index;

                    if tab_index_clone == 0 {
                        state.selected_index = state.midi_selected_index;
                    } else {
                         state.selected_index = state.osc_selected_index;
                    }
                 } // state borrow ends here
            }
            // Re-add ScriptCompiled handler
            ServerMessage::ScriptCompiled { line_idx, frame_idx } => {
                self.add_log(LogLevel::Info, format!("Server confirmed script compiled for ({}, {})", line_idx, frame_idx));
                if self.editor.active_line.line_index == line_idx && self.editor.active_line.frame_index == frame_idx {
                    self.editor.compilation_error = None;
                }
            }
             // Re-add ConnectionRefused handler
            ServerMessage::ConnectionRefused(reason) => {
                 self.add_log(LogLevel::Error, format!("Connection refused: {}", reason));
                 self.server.is_connected = false;
                 self.server.is_connecting = false;
                 self.set_status_message(format!("Connection failed: {}", reason)); 
            }
        }
    }

    /// Adds a log entry to the application's log queue.
    ///
    /// If the log view is currently set to follow, adjusts the scroll position.
    /// 
    /// # Arguments
    /// 
    /// * `level` - The severity level of the log message.
    /// * `message` - The log message to add.
    /// 
    /// # Returns
    /// 
    /// - `()` if the log was added successfully.
    pub fn add_log(&mut self, level: LogLevel, message: String) {
        // Check if we are currently following before modifying logs
        let should_follow = self.interface.components.logs_state.is_following;

        if self.logs.len() == MAX_LOGS {
            self.logs.pop_front();
        }
        self.logs.push_back(LogEntry {
            timestamp: Local::now(),
            level,
            message,
        });

        // If we were following, update scroll position to the (conceptual) new end
        if should_follow {
            let new_len = self.logs.len();
            // The draw function will handle clamping based on height
            self.interface.components.logs_state.scroll_position = new_len.saturating_sub(1);
            // Ensure is_following remains true if we add a log while following
            self.interface.components.logs_state.is_following = true;
        }
    }

    /// Sends a `ClientMessage` to the server via the `NetworkManager`.
    /// 
    /// Handles potential send errors by logging and updating connection status.
    /// 
    /// # Arguments
    /// 
    /// * `message` - The `ClientMessage` to send.
    /// 
    /// # Returns
    /// 
    /// This function doesn't return a value but handles errors internally.
    pub fn send_client_message(&mut self, message: ClientMessage) {
        match self.server.network.send(message) {
            Ok(_) => {}
            Err(e) => {
                self.set_status_message(format!("Failed to send message: {}", e));
                self.server.is_connected = false;
            }
        }
    }

    /// Periodic update function, called on each `Event::Tick`.
    /// 
    /// Currently used to clear the status bar message after a delay.
    fn tick(&mut self) {
        if let Some(timestamp) = self.interface.components.bottom_message_timestamp {
            if timestamp.elapsed() > Duration::from_secs(3) {
                self.interface.components.bottom_message = String::new();
                self.interface.components.bottom_message_timestamp = None;
            }
        }
    }

    /// Handles internal `AppEvent` messages.
    /// 
    /// Dispatches events to the appropriate handlers or updates application state.
    /// 
    /// # Arguments
    /// 
    /// * `event` - The `AppEvent` to handle.
    /// 
    /// # Returns
    /// 
    /// - `Ok(())` if the event was handled successfully.
    /// - `Err` if an error occurred during handling.
    fn handle_app_event(&mut self, event: AppEvent) -> EyreResult<()> {
        match event {
            AppEvent::ProjectDeleted(project_name) => {
                self.add_log(LogLevel::Info, format!("Project '{}' deleted.", project_name));
                // Trigger refresh directly after deletion confirmation
                let event_sender = self.events.sender.clone();
                tokio::spawn(async move {
                    let refresh_result = disk::list_projects().await;
                    let event_result = refresh_result.map_err(|e| e.to_string());
                    let _ = event_sender.send(Event::App(AppEvent::ProjectListLoaded(event_result)));
                });
            },
            AppEvent::ProjectDeleteError(err_msg) => {
                self.add_log(LogLevel::Error, format!("Error deleting project: {}", err_msg));
            },
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
            AppEvent::UpdateTempo(tempo) => {
                self.server.link.session_state.set_tempo(tempo, self.server.link.link.clock_micros());
                self.server.link.commit_app_state();
            },
            AppEvent::UpdateQuantum(quantum) => {
                self.server.link.quantum = quantum;
                self.server.link.capture_app_state();
                self.server.link.commit_app_state();
            },
            AppEvent::Quit => {
                self.quit();
            },
            AppEvent::ProjectListLoaded(result) => {
                self.add_log(LogLevel::Debug, format!("Handling ProjectListLoaded event: {:?}", result)); // LOG
                let state = &mut self.interface.components.save_load_state;
                match result {
                    Ok(projects_with_metadata) => {
                        state.projects = projects_with_metadata;
                        state.selected_index = state.selected_index.min(state.projects.len().saturating_sub(1));
                        state.status_message = format!("{} projects found.", state.projects.len());
                    }
                    Err(e) => {
                        state.projects.clear();
                        state.selected_index = 0;
                        state.status_message = format!("Error listing projects: {}", e);
                    }
                }
            },
            AppEvent::ProjectLoadError(err_msg) => {
                self.interface.components.save_load_state.status_message = format!("Load failed: {}", err_msg);
                self.set_status_message(format!("Error loading project: {}", err_msg));
            },
            AppEvent::LoadProject(snapshot, timing) => {
                 self.set_status_message(format!("Applying loaded project ({:?})...", timing));
                 self.add_log(LogLevel::Info, format!("Applying snapshot (Tempo: {}, Scene: {} lines)", snapshot.tempo, snapshot.scene.lines.len()));

                 // 1. Update local state IMMEDIATELY
                 self.editor.scene = Some(snapshot.scene.clone()); // Update local scene data
                 self.server.link.session_state.set_tempo(snapshot.tempo, self.server.link.link.clock_micros()); // Update local tempo
                 self.interface.components.grid_selection = GridSelection::single(0, 0); // Reset grid selection

                 // 2. Send messages to server with the specified timing
                 self.send_client_message(ClientMessage::SetTempo(snapshot.tempo, timing));
                 self.send_client_message(ClientMessage::SetScene(snapshot.scene, timing)); // Send scene again (server might validate)
                 self.send_client_message(ClientMessage::UpdateGridSelection(self.interface.components.grid_selection)); // Send reset selection
                 
                 self.add_log(LogLevel::Info, "Project load messages sent to server.".to_string());
                 
                 // 3. Switch view after applying locally and sending messages
                 self.interface.screen.mode = Mode::Grid; 
            },
            AppEvent::SwitchToSaveLoad => {
                 self.add_log(LogLevel::Debug, "Handling SwitchToSaveLoad event, triggering refresh.".to_string());
                 self.interface.screen.mode = Mode::SaveLoad;
                 // Trigger refresh when switching to this view
                 let event_sender = self.events.sender.clone();
                 tokio::spawn(async move {
                     let refresh_result = disk::list_projects().await;
                     // Map the disk error to string for the event
                     let event_result = refresh_result.map_err(|e| e.to_string());
                     // Send the loaded list (or error) back to the app event loop
                     let _ = event_sender.send(Event::App(AppEvent::ProjectListLoaded(event_result)));
                 });
            },
            AppEvent::SaveProjectRequest(name_opt) => {
                 if let Some(name) = name_opt {
                     // Name provided via palette
                     self.interface.components.pending_save_name = Some(name.clone());
                     self.add_log(LogLevel::Info, format!("Requesting snapshot to save as '{}'...", name));
                     self.send_client_message(ClientMessage::GetSnapshot);
                 } else {
                     // No name provided, maybe check current project or switch to SaveLoad view?
                     // For now, let's require a name from the palette or use the SaveLoad view UI.
                     self.set_status_message("Save command requires a project name, or use the Files view.".to_string());
                     // Optionally, switch to SaveLoad view and activate saving mode:
                     // self.interface.screen.mode = Mode::SaveLoad;
                     // self.interface.components.save_load_state.is_saving = true;
                     // self.interface.components.save_load_state.input_area = TextArea::default(); // Clear it
                     // self.interface.components.save_load_state.status_message = "Enter project name to save:".to_string();
                 }
            },
            AppEvent::LoadProjectRequest(project_name, timing) => {
                 self.add_log(LogLevel::Info, format!("Attempting to load project '{}' ({:?}) from disk...", project_name, timing));
                 let event_sender = self.events.sender.clone();
                 let proj_name_clone = project_name.clone(); // Clone for async task

                 tokio::spawn(async move {
                     match disk::load_project(&proj_name_clone).await {
                         Ok(snapshot) => {
                             // Send the existing LoadProject event upon successful disk read
                             let _ = event_sender.send(Event::App(AppEvent::LoadProject(snapshot, timing)));
                         }
                         Err(e) => {
                             // Send the existing ProjectLoadError event
                             let _ = event_sender.send(Event::App(AppEvent::ProjectLoadError(e.to_string())));
                         }
                     }
                 });
            },
        }
        Ok(())
    }

    /// Handles keyboard events.
    ///
    /// Processing order:
    /// 1. Global quit (`Ctrl+C`).
    /// 2. Command palette toggle (`Ctrl+P`).
    /// 3. Global function key shortcuts (`F1`-`F8`).
    /// 4. Navigation overlay toggle (`Tab`).
    /// 5. Delegate to the active component's `handle_key_event` method.
    fn handle_key_events(&mut self, key_event: KeyEvent) -> EyreResult<bool> {
        let key_code = key_event.code;
        let key_modifiers = key_event.modifiers;

        // 1. Give priority to the Command Palette if it's visible
        if self.interface.components.command_palette.is_visible {
            let palette_result = self.interface.components.command_palette.handle_key_event(key_event)?;

            match palette_result {
                Some(action) => {
                    // Execute the action here
                    match action {
                        PaletteAction::Dispatch(event) => {
                            let _ = self.events.sender.send(Event::App(event));
                        }
                        PaletteAction::ParseArgs(func) => {
                            let input_clone = self.interface.components.command_palette.input.clone();
                            let exec_result = func(self, &input_clone);
                            if let Err(e) = exec_result {
                                self.add_log(LogLevel::Error, format!("Error executing command: {}", e));
                            }
                        }
                    }
                    return Ok(true);
                }
                None => {
                    return Ok(true);
                }
            }
        }

        // 2. Global quit (`Ctrl+C`) (now reachable even if palette is open, if palette returns None).
        if key_modifiers == KeyModifiers::CONTROL && key_code == KeyCode::Char('c') {
            self.events.sender.send(Event::App(AppEvent::Quit))?;
            return Ok(true);
        }

        // 3. Global Command Palette toggle (`Ctrl+P`).
        if key_modifiers == KeyModifiers::CONTROL && key_code == KeyCode::Char('p') {
            self.interface.components.command_palette.toggle();
            return Ok(true); // Consume Ctrl+P
        }

        // 4. Global function key shortcuts for switching modes.
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
                self.events.sender.send(Event::App(AppEvent::SwitchToSaveLoad))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                 return Ok(true);
            }
            KeyCode::F(8) => { // This maps to SwitchToSaveLoad
                self.events.sender.send(Event::App(AppEvent::SwitchToSaveLoad))
                    .map_err(|e| color_eyre::eyre::eyre!("Send Error: {}", e))?;
                 return Ok(true);
            }
            _ => {} // Continue if not an F-key
        }

        // 5. Navigation overlay toggle (`Tab`).
        if key_code == KeyCode::Tab {
             if self.interface.screen.mode == Mode::Navigation {
                 self.events.sender.send(Event::App(AppEvent::ExitNavigation))?;
                 return Ok(true);
             } else if self.interface.screen.mode != Mode::Splash { 
                 self.interface.screen.previous_mode = Some(self.interface.screen.mode);
                 self.interface.screen.mode = Mode::Navigation;
                 return Ok(true);
             }
        }

        // 6. Delegate to the active component.
        let handled = match self.interface.screen.mode {
            Mode::Navigation => NavigationComponent::new().handle_key_event(self, key_event)?,
            Mode::Editor => EditorComponent::new().handle_key_event(self, key_event)?,
            Mode::Grid => GridComponent::new().handle_key_event(self, key_event)?,
            Mode::Options => OptionsComponent::new().handle_key_event(self, key_event)?,
            Mode::Splash => SplashComponent::new().handle_key_event(self, key_event)?,
            Mode::Help => HelpComponent::new().handle_key_event(self, key_event)?,
            Mode::Devices => DevicesComponent::new().handle_key_event(self, key_event)?,
            Mode::Logs => LogsComponent::new().handle_key_event(self, key_event)?,
            Mode::SaveLoad => SaveLoadComponent::new().handle_key_event(self, key_event)?,
        };
        
        Ok(handled)
    }

    /// Signals the application to exit the main loop.
    /// 
    /// This function disables the main loop of the application.
    /// 
    /// # Returns
    /// 
    /// Un `Result` containing:
    /// * `Ok(())` if the application has been closed successfully
    /// * `Err` if an error occurred during closure
    /// 
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Triggers a screen flash effect.
    pub fn flash_screen(&mut self) {
        self.interface.screen.flash.is_flashing = true;
        self.interface.screen.flash.flash_start = Some(Instant::now());
    }

    /// Sets the message displayed in the bottom status bar.
    ///
    /// Also records the timestamp for potential auto-clearing.
    pub fn set_status_message(&mut self, message: String) {
        self.interface.components.bottom_message = message;
        self.interface.components.bottom_message_timestamp = Some(Instant::now());
    }
}

/// State for the Logs view component.
#[derive(Debug, Clone, Copy)]
pub struct LogsState {
    /// The current line number scrolled to (0 is the top).
    pub scroll_position: usize,
    /// Whether the view should automatically scroll to the bottom on new logs.
    pub is_following: bool,
}

impl LogsState {
    /// Creates a new default `LogsState`.
    pub fn new() -> Self {
        Self {
            scroll_position: 0,
            is_following: true,
        }
    }
}
