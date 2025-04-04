use crate::App;
use crate::components::Component;
use crate::markdown::parser::parse_markdown;
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
        let topics = vec![
            "About".to_string(),
            "Navigation".to_string(),
            "Commands".to_string(),
            "Editor".to_string(),
            "Grid".to_string(),
            "Devices".to_string(),
            "Logs".to_string(),
            "Files".to_string(),
        ];

        // Load content from markdown files
        let contents = vec![
            include_str!("../../static/help/about.md").to_string(),
            include_str!("../../static/help/navigation.md").to_string(),
            include_str!("../../static/help/commands.md").to_string(),
            include_str!("../../static/help/editor.md").to_string(),
            include_str!("../../static/help/grid.md").to_string(),
            include_str!("../../static/help/devices.md").to_string(),
            include_str!("../../static/help/logs.md").to_string(),
            include_str!("../../static/help/files.md").to_string(),
        ];

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

    fn before_draw(&mut self, _app: &mut App) -> EyreResult<()> {
        Ok(())
    }

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

        // Disposition horizontale (liste des sujets à gauche 20%, contenu à droite 80%)
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        // Navigation
        let sidebar_area = inner_chunks[0];
        let sidebar_block = Block::default()
            .title(" Internal Help ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan)); 
        frame.render_widget(sidebar_block.clone(), sidebar_area);
        let inner_sidebar = sidebar_block.inner(sidebar_area);

        // Liste des sujets
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

        // Contenu (80% de la largeur)
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
        frame.render_widget(content_block.clone(), content_area);
        let inner_content_area = content_block.inner(content_area); 

        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Texte
                Constraint::Length(1), // Aide
            ])
            .split(inner_content_area);

        let main_content_text_area = content_chunks[0];
        let content_help_area = content_chunks[1];

        // Rendu du texte
        let content_str = help_state
            .contents
            .get(help_state.selected_index)
            .map_or("No content available.", |s| s.as_str());
        
        // Parse the markdown content (currently placeholder)
        let content = parse_markdown(content_str); 

        let content_paragraph = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: true }); // On wrap
        frame.render_widget(content_paragraph, main_content_text_area);

        // Indication des touches
        let help_text = "↑↓: Navigate Topics"; 
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, content_help_area); 
    }
}
