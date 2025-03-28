use crate::App;
use crate::components::{Component, handle_common_keys, inner_area};
use crate::event::AppEvent;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::error::Error;

pub struct HelpState {
    pub topics: Vec<String>,
    pub contents: Vec<String>,
    pub selected_index: usize,
}

impl HelpState {
    pub fn new() -> Self {
        let mut topics = Vec::new();
        let mut contents = Vec::new();

        // Différents sujets et contenu !
        topics.push("About".to_string());
        contents.push("BuboCore is a live coding environment.".to_string());

        topics.push("Navigation".to_string());
        contents.push(
            "F1 - Switch to Editor view\n\
                      F2 - Switch to Grid view\n\
                      F3 - Switch to Options view\n\
                      Tab - Cycle between views\n\
                      Ctrl+P - Open command prompt\n\
                      Ctrl+C - Exit application"
                .to_string(),
        );

        topics.push("Commands".to_string());
        contents.push(
            "Type Ctrl+P to open the command prompt, then enter commands:\n\n\
                      quit or q - Exit the application\n\
                      help or ? - Show this help screen\n\
                      tempo [bpm] - Set the tempo in beats per minute\n\
                      quantum [beats] - Set the quantum (measure length) in beats"
                .to_string(),
        );

        topics.push("Editor".to_string());
        contents.push(
            "The Editor lets you write and edit code or patterns.\n\n\
                      Ctrl+E - Parse and execute the current content"
                .to_string(),
        );

        topics.push("Grid".to_string());
        contents.push("The Grid provides a matrix interface for creating patterns.\n".to_string());

        HelpState {
            topics,
            contents,
            selected_index: 0,
        }
    }

    // Navigation dans les sujets
    pub fn next_topic(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.topics.len();
    }

    pub fn prev_topic(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = self.topics.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }
}

pub struct HelpComponent;

impl HelpComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for HelpComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> Result<bool, Box<dyn Error + 'static>> {
        // First try common key handlers
        if handle_common_keys(app, key_event)? {
            return Ok(true);
        }

        // Help-specific key handling
        match key_event.code {
            KeyCode::Tab => {
                app.events.send(AppEvent::SwitchToEditor);
                Ok(true)
            }
            KeyCode::Up => {
                if let Some(help_state) = &mut app.interface.components.help_state {
                    help_state.prev_topic();
                }
                Ok(true)
            }
            KeyCode::Down => {
                if let Some(help_state) = &mut app.interface.components.help_state {
                    help_state.next_topic();
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        if app.interface.components.help_state.is_none() {
            return; // Should not happen, but just in case
        }

        let help_state = app.interface.components.help_state.as_ref().unwrap();

        // Création du layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        // Sidebar (20%)
        let sidebar_area = chunks[0];
        let sidebar_block = Block::default()
            .title("Topics")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(sidebar_block, sidebar_area);

        // Liste à partir des sujets
        let items: Vec<ListItem> = help_state
            .topics
            .iter()
            .map(|topic| {
                ListItem::new(Line::from(vec![Span::styled(
                    topic,
                    Style::default().fg(Color::White),
                )]))
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(help_state.selected_index));

        let list = List::new(items)
            .block(Block::default())
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        let inner_sidebar = inner_area(sidebar_area);
        frame.render_stateful_widget(list, inner_sidebar, &mut list_state);

        // Contenu (droite - 80%)
        let content_area = chunks[1];
        let title = help_state
            .topics
            .get(help_state.selected_index)
            .unwrap_or(&"Help".to_string())
            .clone();
        let content_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(content_block, content_area);

        // Contenu sélectionné
        let content = help_state
            .contents
            .get(help_state.selected_index)
            .unwrap_or(&"No content available.".to_string())
            .clone();

        // Rendu du contenu
        let content_paragraph = Paragraph::new(Text::from(content))
            .style(Style::default().fg(Color::White))
            .block(Block::default());

        let inner_content = inner_area(content_area);
        frame.render_widget(content_paragraph, inner_content);
    }
}
