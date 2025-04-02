use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Layout, Direction, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

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
            "ESC - Open Navigation Grid\n\
             In Navigation Grid:\n\
             - Arrow keys: Move cursor\n\
             - A-Z: Quick jump to view\n\
             - ESC: Close grid\n\n\
             F1 - Switch to Editor view\n\
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

        topics.push("Devices".to_string());
        contents.push(
            "The Devices view shows available MIDI and OSC devices.\n\n\
             ↑↓ - Navigate through devices\n\
             Enter - Select/Connect device\n\
             Tab - Back to Editor"
                .to_string(),
        );

        topics.push("Logs".to_string());
        contents.push(
            "The Logs view displays application messages and errors.\n\n\
             ↑↓ - Scroll through logs\n\
             Ctrl+C - Clear logs\n\
             Tab - Back to Editor"
                .to_string(),
        );

        topics.push("Files".to_string());
        contents.push(
            "The Files view lets you browse and manage files.\n\n\
             ↑↓ - Navigate through files\n\
             Enter - Open directory/file\n\
             Backspace - Go up one directory\n\
             Tab - Back to Editor"
                .to_string(),
        );

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
    ) -> EyreResult<bool> {
        // Help-specific key handling
        match key_event.code {
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
            return;
        }

        let help_state = app.interface.components.help_state.as_ref().unwrap();

        // Create the horizontal layout directly on the input `area`
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area); // Split area directly

        // --- Sidebar (Left - 20%) --- 
        let sidebar_area = inner_chunks[0];
        let sidebar_block = Block::default()
            .title(" Internal Help ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan)); 
        frame.render_widget(sidebar_block.clone(), sidebar_area); // Draw block first
        let inner_sidebar = sidebar_block.inner(sidebar_area); // Use block.inner()

        // Render list inside the inner sidebar area (no change needed here)
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
        frame.render_stateful_widget(list, inner_sidebar, &mut list_state);

        // --- Content Area (Right - 80%) --- 
        let content_area = inner_chunks[1];
        let title = help_state
            .topics
            .get(help_state.selected_index)
            .unwrap_or(&"Help".to_string())
            .clone();
        let content_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan)); 
        frame.render_widget(content_block.clone(), content_area); // Draw block first
        // Get inner area of the content block
        let inner_content_area = content_block.inner(content_area); 

        // Split the inner content area for the text and the help line
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Main text area
                Constraint::Length(1), // Help text line
            ])
            .split(inner_content_area);

        let main_content_text_area = content_chunks[0];
        let content_help_area = content_chunks[1];

        // Render selected content text
        let content = help_state
            .contents
            .get(help_state.selected_index)
            .unwrap_or(&"No content available.".to_string())
            .clone();
        let content_paragraph = Paragraph::new(Text::from(content))
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: true }); // Added wrap for long text
            // Removed .block(Block::default())
        frame.render_widget(content_paragraph, main_content_text_area); // Render in top part

        // Render help text inside the content block
        let help_text = "↑↓: Navigate Topics"; 
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, content_help_area); // Render in bottom part

        // REMOVED incorrect help text rendering at the very bottom
    }
}
