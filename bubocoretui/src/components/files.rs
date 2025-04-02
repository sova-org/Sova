use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect, Constraint, Layout, Direction},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph},
};
use std::error::Error;
use std::fs;
use std::path::{PathBuf, Component as PathComponent};
use directories::ProjectDirs;

pub struct FilesState {
    pub current_path: PathBuf,
    pub selected_index: usize,
    pub entries: Vec<PathBuf>,
}

impl FilesState {
    pub fn new() -> Self {
        let base_path = ProjectDirs::from("com", "BuboCore",  "BuboCore")
            .map(|proj_dirs| proj_dirs.config_dir().join("Projects"))
            .unwrap_or_else(|| PathBuf::from("."));
        
        if !base_path.exists() {
            let _ = fs::create_dir_all(&base_path);
        }

        let entries = Self::read_directory(&base_path);
        
        Self {
            current_path: base_path,
            selected_index: 0,
            entries,
        }
    }

    fn read_directory(path: &PathBuf) -> Vec<PathBuf> {
        match fs::read_dir(path) {
            Ok(entries) => {
                let mut paths = Vec::new();
                // Add parent directory if not at root
                if path.parent().is_some() {
                    paths.push(path.parent().unwrap().to_path_buf());
                }
                // Add directory entries
                for entry in entries {
                    if let Ok(entry) = entry {
                        paths.push(entry.path());
                    }
                }
                paths.sort_by(|a, b| {
                    // Directories first, then files
                    let a_is_dir = a.is_dir();
                    let b_is_dir = b.is_dir();
                    if a_is_dir != b_is_dir {
                        b_is_dir.cmp(&a_is_dir)
                    } else {
                        a.file_name().unwrap_or_default().cmp(b.file_name().unwrap_or_default())
                    }
                });
                paths
            }
            Err(_) => Vec::new(),
        }
    }

    pub fn enter_directory(&mut self, path: &PathBuf) {
        if path.is_dir() {
            self.current_path = path.clone();
            self.entries = Self::read_directory(&self.current_path);
            self.selected_index = 0;
        }
    }

    pub fn get_current_entry(&self) -> Option<&PathBuf> {
        self.entries.get(self.selected_index)
    }
}

pub struct FilesComponent;

impl FilesComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for FilesComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        // Files-specific key handling
        match key_event.code {
            KeyCode::Up => {
                if !app.interface.components.files_state.entries.is_empty() {
                    app.interface.components.files_state.selected_index = 
                        app.interface.components.files_state.selected_index.saturating_sub(1);
                }
                Ok(true)
            }
            KeyCode::Down => {
                if !app.interface.components.files_state.entries.is_empty() {
                    let len = app.interface.components.files_state.entries.len();
                    app.interface.components.files_state.selected_index = 
                        (app.interface.components.files_state.selected_index + 1).min(len - 1);
                }
                Ok(true)
            }
            KeyCode::Enter => {
                let current_entry_path = app.interface.components.files_state.get_current_entry().cloned();
                if let Some(path) = current_entry_path {
                    app.interface.components.files_state.enter_directory(&path);
                    if path.is_file() {
                        app.set_status_message(format!("Selected file: {}", path.display()));
                    }
                }
                Ok(true)
            }
            KeyCode::Backspace => {
                let parent_path = app.interface.components.files_state.current_path.parent().map(|p| p.to_path_buf());
                if let Some(path) = parent_path {
                   app.interface.components.files_state.enter_directory(&path);
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        // Extract only the last component of the path for the title
        let current_dir_name = app.interface.components.files_state.current_path
            .components()
            .last()
            .map_or_else(|| "?".to_string(), |c| 
                match c {
                    PathComponent::Normal(name) => name.to_string_lossy().to_string(),
                    PathComponent::RootDir => "/".to_string(),
                    _ => "?".to_string()
                }
            );
            
        let title = format!("Files - {}", current_dir_name);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        frame.render_widget(block, area); // Draw the block first

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // List area
                Constraint::Length(1), // Help text
            ])
            .split(inner_area); // Split the inner area

        let list_area = chunks[0];
        let help_area = chunks[1];

        // Create the list of files and directories
        let entries: Vec<ListItem> = app.interface.components.files_state.entries
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    // Special case for showing parent directory ".."
                    .unwrap_or_else(|| if path.components().last() == Some(PathComponent::ParentDir) { "..".to_string() } else { "?".to_string() }); 
                
                let style;
                let prefix;
                if path.is_dir() || name == ".." { // Treat ".." like a directory visually
                    style = if i == app.interface.components.files_state.selected_index {
                        Style::default().bg(Color::Blue).fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };
                    prefix = "📁 ";
                } else {
                     style = if i == app.interface.components.files_state.selected_index {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };
                     prefix = "📄 ";
                }
                
                ListItem::new(Text::from(format!("{}{}", prefix, name))).style(style)
            })
            .collect();

        let list = List::new(entries)
            .highlight_style(Style::default().bg(Color::Blue)); // Base highlight style
            // Note: Specific item styles override this, so we rely on the styles set in the map above

        // Render the list inside the list_area
        frame.render_widget(list, list_area);

        // Render help text in help_area
        let help_text = "↑↓: Navigate | Enter: Open | Backspace: Up";
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
