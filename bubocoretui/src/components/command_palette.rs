use crate::app::App;
use crate::components::logs::LogLevel;
use crate::event::{AppEvent, Event};
use bubocorelib::schedule::action_timing::ActionTiming;
use bubocorelib::server::client::ClientMessage;
use color_eyre::Result as EyreResult;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

/// Represents a single command available in the command palette.
#[derive(Clone, Debug)]
pub struct PaletteCommand {
    /// The primary name/keyword to trigger the command.
    pub keyword: String,
    /// Alternative keywords or aliases.
    pub aliases: Vec<String>,
    /// A short description of what the command does.
    pub description: String,
    /// The application event to dispatch when the command is executed.
    /// This might need refinement to handle commands with arguments.
    pub action: PaletteAction,
}

/// Represents the action to take when a command is selected.
#[derive(Clone, Debug)]
pub enum PaletteAction {
    /// Dispatch a simple AppEvent (no arguments needed from palette input).
    Dispatch(AppEvent),
    /// Requires parsing arguments from the palette input string.
    ParseArgs(fn(&mut App, &str) -> EyreResult<()>),
}

/// State for the Command Palette component.
#[derive(Default)]
pub struct CommandPaletteComponent {
    /// Whether the palette is currently visible.
    pub is_visible: bool,
    /// The current text entered by the user for filtering.
    pub input: String,
    /// The list of commands matching the current input filter.
    pub filtered_commands: Vec<PaletteCommand>,
    /// The index of the currently selected command in the filtered list.
    pub selected_index: usize,
    /// The full list of available commands.
    available_commands: Vec<PaletteCommand>,
    /// State for the list widget selection highlight.
    list_state: ListState,
}

impl CommandPaletteComponent {
    pub fn new() -> Self {
        let mut palette = Self {
            available_commands: Self::get_all_commands(),
            list_state: ListState::default(),
            ..Default::default()
        };
        palette.filter_commands();
        palette.list_state.select(Some(palette.selected_index));
        palette
    }

    /// Toggles the visibility of the command palette.
    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
        if self.is_visible {
            self.input.clear();
            self.selected_index = 0;
            self.filter_commands();
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Hides the command palette.
    pub fn hide(&mut self) {
        if self.is_visible {
            self.is_visible = false;
        }
    }

    /// Updates the `filtered_commands` list based on the current `input`.
    fn filter_commands(&mut self) {
        let query = self.input.to_lowercase();
        if query.is_empty() {
            self.filtered_commands = self.available_commands.clone();
        } else {
            let query_parts: Vec<&str> = query.splitn(2, ' ').collect();
            let query_command = query_parts.get(0).cloned().unwrap_or("");

            let mut scored_commands: Vec<(i32, PaletteCommand)> = self
                .available_commands
                .iter()
                .filter_map(|cmd| {
                    let keyword_lower = cmd.keyword.to_lowercase();
                    // Collect aliases to avoid re-iterating multiple times
                    let aliases_lower: Vec<String> =
                        cmd.aliases.iter().map(|a| a.to_lowercase()).collect();
                    let description_lower = cmd.description.to_lowercase();
                    let mut score = 0;

                    if keyword_lower == query {
                        score = 5;
                    } else if aliases_lower.iter().any(|a| a == &query) {
                        score = 4;
                    } else if keyword_lower == query_command && query.starts_with(&keyword_lower) {
                        if query == keyword_lower
                            || query.starts_with(&(keyword_lower.clone() + " "))
                        {
                            score = 3;
                        }
                    } else if let Some(matching_alias) =
                        aliases_lower.iter().find(|a| *a == query_command)
                    {
                        if query.starts_with(matching_alias) {
                            score = 2;
                            if query == *matching_alias
                                || query.starts_with(&(matching_alias.clone() + " "))
                            {
                                score = 2;
                            }
                        }
                    }

                    if score == 0 {
                        if keyword_lower.contains(&query)
                            || aliases_lower.iter().any(|a| a.contains(&query))
                            || description_lower.contains(&query)
                        {
                            score = 1;
                        }
                    }

                    if score > 0 {
                        Some((score, cmd.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            scored_commands.sort_by(|a, b| b.0.cmp(&a.0));

            self.filtered_commands = scored_commands.into_iter().map(|(_, cmd)| cmd).collect();
        }

        if self.filtered_commands.is_empty() {
            self.selected_index = 0;
            self.list_state.select(None);
        } else {
            self.selected_index = self.selected_index.min(self.filtered_commands.len() - 1);
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Moves the selection up in the filtered list.
    fn select_previous(&mut self) {
        if !self.filtered_commands.is_empty() {
            let new_index = if self.selected_index == 0 {
                self.filtered_commands.len() - 1
            } else {
                self.selected_index - 1
            };
            self.selected_index = new_index;
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Moves the selection down in the filtered list.
    fn select_next(&mut self) {
        if !self.filtered_commands.is_empty() {
            let new_index = (self.selected_index + 1) % self.filtered_commands.len();
            self.selected_index = new_index;
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Gets the currently selected command, if any.
    fn get_selected_command(&self) -> Option<&PaletteCommand> {
        self.filtered_commands.get(self.selected_index)
    }

    /// Handles key events when the command palette is active.
    /// Returns `Ok(Some(action))` to execute, `Ok(None)` if
    /// handled locally (input, navigation), or `Err`.
    pub fn handle_key_event(
        &mut self,
        key_event: crossterm::event::KeyEvent,
    ) -> EyreResult<Option<PaletteAction>> {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key_event.kind != KeyEventKind::Press {
            return Ok(None);
        }

        match key_event.code {
            KeyCode::Esc => {
                self.hide();
                Ok(None)
            }
            KeyCode::Enter => {
                let action_to_execute = if let Some(command) = self.get_selected_command() {
                    Some(command.action.clone())
                } else {
                    None
                };
                self.hide();
                Ok(action_to_execute)
            }
            KeyCode::Up => {
                self.select_previous();
                Ok(None)
            }
            KeyCode::Down => {
                self.select_next();
                Ok(None)
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                self.filter_commands();
                Ok(None)
            }
            KeyCode::Backspace => {
                if !self.input.is_empty() {
                    self.input.pop();
                    self.filter_commands();
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Draws the command palette overlay.
    pub fn draw(&mut self, frame: &mut Frame) {
        if !self.is_visible {
            return;
        }

        let area = frame.area();

        const MIN_LIST_HEIGHT: u16 = 3;
        let min_inner_height = 1 + MIN_LIST_HEIGHT;

        let desired_list_height = self.filtered_commands.len() as u16;
        let inner_height = (1 + desired_list_height).max(min_inner_height);

        // Calculate final popup dimensions
        let width = (area.width as f32 * 0.8).min(area.width as f32) as u16;
        let height = (inner_height + 2) // +2 for top/bottom borders
            .min(15) // Max overall height
            .min(area.height); // Cannot exceed screen height

        // Calculate centered popup area
        let popup_area = Rect {
            x: (area.width.saturating_sub(width)) / 2, // Center horizontally
            y: (area.height.saturating_sub(height)) / 2, // Center vertically
            width,
            height,
        };

        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);

        let outer_block = Block::default()
            .title(" Command Palette (Ctrl+P) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        // Calculate inner area *before* moving outer_block
        let inner_area = outer_block.inner(popup_area);

        // Now render the outer_block
        frame.render_widget(outer_block, popup_area);

        // Check if inner_area has any height before splitting
        if inner_area.height == 0 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(inner_area);

        // 1. Render Input Line
        let input_paragraph = Paragraph::new(format!("> {}", self.input))
            .style(Style::default().fg(Color::LightCyan));
        frame.render_widget(input_paragraph, chunks[0]);

        // 2. Render Filtered Commands List (only if there's space)
        if chunks.len() > 1 && chunks[1].height > 0 {
            let list_items: Vec<ListItem> = self
                .filtered_commands
                .iter()
                .map(|cmd| {
                    let keyword_style = Style::default().fg(Color::White).bold();
                    let desc_style = Style::default().fg(Color::DarkGray);

                    let content = Line::from(vec![
                        Span::styled(cmd.keyword.clone(), keyword_style),
                        Span::raw(" "),
                        Span::styled(format!("({})", cmd.description), desc_style),
                    ]);
                    ListItem::new(content)
                })
                .collect();

            let list = List::new(list_items)
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
                .highlight_symbol(">> ");

            frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
        }
    }

    // Placeholder for defining all commands
    fn get_all_commands() -> Vec<PaletteCommand> {
        vec![
            // --- Navigation & View Switching ---
            PaletteCommand {
                keyword: "editor".to_string(),
                aliases: vec!["edit".to_string(), "script".to_string()],
                description: "Switch to the script editor view".to_string(),
                action: PaletteAction::Dispatch(AppEvent::SwitchToEditor),
            },
            PaletteCommand {
                keyword: "grid".to_string(),
                aliases: vec!["scene".to_string()],
                description: "Switch to the scene grid view".to_string(),
                action: PaletteAction::Dispatch(AppEvent::SwitchToGrid),
            },
            PaletteCommand {
                keyword: "options".to_string(),
                aliases: vec!["settings".to_string()],
                description: "Switch to the options view".to_string(),
                action: PaletteAction::Dispatch(AppEvent::SwitchToOptions),
            },
            PaletteCommand {
                keyword: "devices".to_string(),
                aliases: vec!["devs".to_string()],
                description: "Switch to the connected devices view".to_string(),
                action: PaletteAction::Dispatch(AppEvent::SwitchToDevices),
            },
            PaletteCommand {
                keyword: "logs".to_string(),
                aliases: vec![],
                description: "Switch to the application logs view".to_string(),
                action: PaletteAction::Dispatch(AppEvent::SwitchToLogs),
            },
            PaletteCommand {
                keyword: "files".to_string(),
                aliases: vec![
                    "projects".to_string(),
                    "save".to_string(),
                    "load".to_string(),
                ],
                description: "Switch to the save/load projects view".to_string(),
                action: PaletteAction::Dispatch(AppEvent::SwitchToSaveLoad),
            },
            PaletteCommand {
                keyword: "help".to_string(),
                aliases: vec!["?".to_string(), "docs".to_string()],
                description: "Switch to the help view".to_string(),
                action: PaletteAction::Dispatch(AppEvent::SwitchToHelp),
            },
            // --- Application ---
            PaletteCommand {
                keyword: "mode".to_string(),
                aliases: vec![],
                description: "[normal|vim] Switch editor keymap mode".to_string(),
                action: PaletteAction::ParseArgs(execute_set_editor_mode),
            },
            PaletteCommand {
                keyword: "quit".to_string(),
                aliases: vec!["q".to_string(), "exit".to_string()],
                description: "Quit the application (alias: q, exit)".to_string(),
                action: PaletteAction::Dispatch(AppEvent::Quit),
            },
            // --- Project Management ---
            PaletteCommand {
                keyword: "save".to_string(),
                aliases: vec![],
                description: "Save current session as a project [optional_name]".to_string(),
                action: PaletteAction::ParseArgs(execute_save),
            },
            PaletteCommand {
                keyword: "load".to_string(),
                aliases: vec![],
                description: "Load project by name [timing]".to_string(),
                action: PaletteAction::ParseArgs(execute_load),
            },
            // --- Server/Scene Actions ---
            PaletteCommand {
                keyword: "setname".to_string(),
                aliases: vec!["name".to_string()],
                description: "[<name>] Set username (e.g., 'setname BuboBubo')".to_string(),
                action: PaletteAction::ParseArgs(execute_set_name),
            },
            PaletteCommand {
                keyword: "chat".to_string(),
                aliases: vec!["say".to_string()],
                description:
                    "[<message>] Send chat message to other peers (e.g., 'chat Hello how are you?')"
                        .to_string(),
                action: PaletteAction::ParseArgs(execute_chat),
            },
            PaletteCommand {
                keyword: "tempo".to_string(),
                aliases: vec!["t".to_string(), "bpm".to_string()],
                description: "[<bpm> [now|end|<beat>]] Set tempo (e.g., 'tempo 120', 20-900 BPM)"
                    .to_string(),
                action: PaletteAction::ParseArgs(execute_set_tempo),
            },
            PaletteCommand {
                keyword: "quantum".to_string(),
                aliases: vec![],
                description: "[<beats>] Set Link quantum (e.g., 'quantum 4', >0 <=16)".to_string(),
                action: PaletteAction::ParseArgs(execute_set_quantum),
            },
            PaletteCommand {
                keyword: "play".to_string(),
                aliases: vec![],
                description: "[[now|end|<beat>]] Start the transport".to_string(),
                action: PaletteAction::ParseArgs(execute_play),
            },
            PaletteCommand {
                keyword: "stop".to_string(),
                aliases: vec!["pause".to_string()],
                description: "[[now|end|<beat>]] Stop the transport".to_string(),
                action: PaletteAction::ParseArgs(execute_stop),
            },
            PaletteCommand {
                keyword: "scenelength".to_string(),
                aliases: vec!["sl".to_string()],
                description: "[<length> [now|end|<beat>]] Set scene length".to_string(),
                action: PaletteAction::ParseArgs(execute_set_scene_length),
            },
            PaletteCommand {
                keyword: "linelength".to_string(),
                aliases: vec!["ll".to_string()],
                description: "[<line> <len|scene> [now|end|<beat>]] Set line length".to_string(),
                action: PaletteAction::ParseArgs(execute_set_line_length),
            },
            PaletteCommand {
                keyword: "linespeed".to_string(),
                aliases: vec!["ls".to_string()],
                description: "[<line> <factor> [now|end|<beat>]] Set line speed factor".to_string(),
                action: PaletteAction::ParseArgs(execute_set_line_speed),
            },
        ]
    }
}

// --- Command Execution Functions ---
// These functions parse args from the full input string provided by the palette.
// They need access to `App` state, so they are defined here temporarily,
// but might be better placed within `App` or a dedicated command execution module.

// Helper function to parse timing - needs to be accessible by execute functions.
// Consider moving this to App or a shared utility module.
fn parse_timing_arg(app: &mut App, arg: Option<&str>) -> ActionTiming {
    arg.map_or(ActionTiming::Immediate, |timing_str| {
        match timing_str.to_lowercase().as_str() {
            "immediate" | "now" => ActionTiming::Immediate,
            "end" | "loop" => ActionTiming::EndOfScene,
            _ => {
                if let Ok(beat) = timing_str.parse::<u64>() {
                    ActionTiming::AtBeat(beat)
                } else {
                    app.add_log(
                        LogLevel::Warn,
                        format!(
                            "Unrecognized timing '{}', defaulting to immediate.",
                            timing_str
                        ),
                    );
                    ActionTiming::Immediate
                }
            }
        }
    })
}

fn execute_set_name(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() > 1 {
        let name = parts[1..].join(" ");
        app.send_client_message(ClientMessage::SetName(name.clone()));
        app.server.username = name;
        app.set_status_message(format!("Set name to '{}'", app.server.username));
    } else {
        app.set_status_message("Usage: setname <your_name>".to_string());
    }
    Ok(())
}

fn execute_chat(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() > 1 {
        let message = parts[1..].join(" ");
        app.send_client_message(ClientMessage::Chat(message.clone()));
        app.add_log(LogLevel::Info, format!("Sent: {}", message));
    } else {
        app.set_status_message("Usage: chat <message>".to_string());
    }
    Ok(())
}

fn execute_set_tempo(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if let Some(tempo_str) = parts.get(1) {
        if let Ok(tempo) = tempo_str.parse::<f64>() {
            if tempo >= 20.0 && tempo <= 999.0 {
                // Send AppEvent for local Link state update first
                let _ = app
                    .events
                    .sender
                    .send(Event::App(AppEvent::UpdateTempo(tempo)));
                // Then send message to server
                // Timing arg could be added here too, e.g., "tempo 120 end"
                let timing = parse_timing_arg(app, parts.get(2).copied());
                app.send_client_message(ClientMessage::SetTempo(tempo, timing));
                app.set_status_message(format!("Set tempo to {:.1} BPM ({:?})", tempo, timing));
            } else {
                app.set_status_message("Tempo must be between 20 and 999 BPM".to_string());
            }
        } else {
            app.set_status_message("Invalid tempo value".to_string());
        }
    } else {
        app.set_status_message("Usage: tempo <bpm> [timing]".to_string());
    }
    Ok(())
}

fn execute_set_quantum(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if let Some(quantum_str) = parts.get(1) {
        if let Ok(quantum) = quantum_str.parse::<f64>() {
            if quantum > 0.0 && quantum <= 16.0 {
                let _ = app
                    .events
                    .sender
                    .send(Event::App(AppEvent::UpdateQuantum(quantum)));
                app.set_status_message(format!("Set quantum to {}", quantum));
            } else {
                app.set_status_message("Quantum must be > 0 and <= 16".to_string());
            }
        } else {
            app.set_status_message("Invalid quantum value".to_string());
        }
    } else {
        app.set_status_message("Usage: quantum <value>".to_string());
    }
    Ok(())
}

fn execute_play(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let timing = parse_timing_arg(app, parts.get(1).copied());
    app.send_client_message(ClientMessage::TransportStart(timing));
    app.set_status_message(format!("Requested transport start ({:?})", timing));
    Ok(())
}

fn execute_stop(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let timing = parse_timing_arg(app, parts.get(1).copied());
    app.send_client_message(ClientMessage::TransportStop(timing));
    app.set_status_message(format!("Requested transport stop ({:?})", timing));
    Ok(())
}

fn execute_set_scene_length(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if let Some(length_str) = parts.get(1) {
        if let Ok(length) = length_str.parse::<usize>() {
            if length == 0 {
                app.set_status_message("Scene length must be greater than 0".to_string());
                return Ok(());
            }
            let timing = parse_timing_arg(app, parts.get(2).copied());
            app.send_client_message(ClientMessage::SetSceneLength(length, timing));
            app.set_status_message(format!("Requested scene length {} ({:?})", length, timing));
        } else {
            app.set_status_message("Invalid length value (must be a positive integer)".to_string());
        }
    } else {
        app.set_status_message("Usage: scenelength <length> [timing]".to_string());
    }
    Ok(())
}

fn execute_set_line_length(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() < 3 {
        app.set_status_message("Usage: linelength <line_idx> <length|scene> [timing]".to_string());
        return Ok(());
    }
    if let Ok(user_line_idx) = parts[1].parse::<usize>() {
        if user_line_idx == 0 {
            app.set_status_message("Line index must be 1 or greater.".to_string());
            return Ok(());
        }
        let line_idx = user_line_idx - 1; // Convert to 0-based index

        let length_arg = parts[2].to_lowercase();
        let length_opt: Option<f64> = if length_arg == "scene" {
            None
        } else if let Ok(len) = length_arg.parse::<f64>() {
            if len > 0.0 {
                Some(len)
            } else {
                app.set_status_message("Length must be positive if specified.".to_string());
                return Ok(());
            }
        } else {
            app.set_status_message("Invalid length: use a positive number or 'scene'".to_string());
            return Ok(());
        };

        let timing = parse_timing_arg(app, parts.get(3).copied());

        app.send_client_message(ClientMessage::SetLineLength(line_idx, length_opt, timing));
        app.set_status_message(format!(
            "Requested Line {} length {:?} ({:?})",
            user_line_idx, length_opt, timing
        ));
    } else {
        app.set_status_message("Invalid line index".to_string());
    }
    Ok(())
}

fn execute_set_line_speed(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() < 3 {
        app.set_status_message("Usage: linespeed <line_idx> <speed_factor> [timing]".to_string());
        return Ok(());
    }
    if let Ok(user_line_idx) = parts[1].parse::<usize>() {
        if user_line_idx == 0 {
            app.set_status_message("Line index must be 1 or greater.".to_string());
            return Ok(());
        }
        let line_idx = user_line_idx - 1;

        if let Ok(speed_factor) = parts[2].parse::<f64>() {
            if speed_factor <= 0.0 {
                app.set_status_message("Speed factor must be positive.".to_string());
                return Ok(());
            }

            let timing = parse_timing_arg(app, parts.get(3).copied());

            app.send_client_message(ClientMessage::SetLineSpeedFactor(
                line_idx,
                speed_factor,
                timing,
            ));
            app.set_status_message(format!(
                "Requested Line {} speed x{:.2} ({:?})",
                user_line_idx, speed_factor, timing
            ));
        } else {
            app.set_status_message("Invalid speed factor (must be a number)".to_string());
        }
    } else {
        app.set_status_message("Invalid line index".to_string());
    }
    Ok(())
}

// Add execute_save
fn execute_save(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let project_name = parts.get(1).map(|s| s.to_string()); // Optional name

    // Send request to app
    let _ = app
        .events
        .sender
        .send(Event::App(AppEvent::SaveProjectRequest(project_name)));
    app.set_status_message("Requesting save...".to_string());
    Ok(())
}

fn execute_load(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if let Some(project_name) = parts.get(1) {
        let timing = parse_timing_arg(app, parts.get(2).copied());
        let _ = app
            .events
            .sender
            .send(Event::App(AppEvent::LoadProjectRequest(
                project_name.to_string(),
                timing,
            )));
        app.set_status_message(format!(
            "Requesting load for '{}' ({:?})...",
            project_name, timing
        ));
    } else {
        app.set_status_message("Usage: load <project_name> [timing]".to_string());
    }
    Ok(())
}

// --- Reinstated and Updated Command Execution --- 
fn execute_set_editor_mode(app: &mut App, input: &str) -> EyreResult<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if let Some(mode_arg) = parts.get(1) {
        match mode_arg.to_lowercase().as_str() {
            "normal" => {
                app.client_config.editing_mode = crate::disk::EditingMode::Normal;
                app.set_status_message("Editor mode set to Normal".to_string());
            }
            "vim" => {
                app.client_config.editing_mode = crate::disk::EditingMode::Vim;
                app.set_status_message("Editor mode set to Vim".to_string());
            }
            _ => {
                app.set_status_message("Usage: mode [normal|vim]".to_string());
            }
        }
    } else {
        // If no argument, maybe toggle or show current? For now, show usage.
        app.set_status_message(format!(
            "Current mode: {}. Usage: mode [normal|vim]",
            app.client_config.editing_mode
        ));
    }
    Ok(())
}
