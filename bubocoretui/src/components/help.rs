use crate::App;
use crate::components::Component;
use crate::markdown::parser::parse_markdown;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Layout, Direction, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, BorderType},
};

/// State for the help component
pub struct HelpState {
    pub topics: Vec<String>,
    pub contents: Vec<String>,
    pub selected_index: usize,
    pub scroll_offset: u16,
}

impl HelpState {

    /// Create a new help state
    /// 
    /// # Returns
    /// 
    /// A new help state with the default topics and contents
    pub fn new() -> Self {
        let topics = vec![
            "Welcome".to_string(),
            "Navigation".to_string(),
            "Editor".to_string(),
            "Grid".to_string(),
            "Commands".to_string(),
            "Logs".to_string(),
            "Devices".to_string(),
            "Files".to_string(),
            "About".to_string(),
        ];

        // Load content from markdown files. They are in the static/help directory.
        // The files are named like the topics, but with a .md extension.
        let contents = vec![
            include_str!("../../static/help/welcome.md").to_string(),
            include_str!("../../static/help/navigation.md").to_string(),
            include_str!("../../static/help/commands.md").to_string(),
            include_str!("../../static/help/editor.md").to_string(),
            include_str!("../../static/help/grid.md").to_string(),
            include_str!("../../static/help/devices.md").to_string(),
            include_str!("../../static/help/logs.md").to_string(),
            include_str!("../../static/help/files.md").to_string(),
            include_str!("../../static/help/about.md").to_string(),
        ];

        HelpState {
            topics,
            contents,
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    // Navigate to the next topic
    pub fn next_topic(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.topics.len();
        self.scroll_offset = 0;
    }

    // Navigating to the previous topic
    pub fn prev_topic(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = self.topics.len() - 1;
        } else {
            self.selected_index -= 1;
        }
        self.scroll_offset = 0;
    }

    // Scroll down the content
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    // Scroll up the content
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
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
        if let Some(help_state) = &mut app.interface.components.help_state {
            match key_event.code {
                // Navigate to the previous topic
                KeyCode::Up => {
                    help_state.prev_topic();
                    Ok(true)
                }
                // Navigate to the next topic
                KeyCode::Down => {
                    help_state.next_topic();
                    Ok(true)
                }
                // Scroll the content up
                KeyCode::PageUp | KeyCode::Left => {
                    help_state.scroll_up();
                    Ok(true)
                }
                // Scroll the content down
                KeyCode::PageDown | KeyCode::Right => {
                    help_state.scroll_down();
                    Ok(true)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    /// Draw the help component
    /// 
    /// # Arguments
    /// 
    /// * `app`: The application state
    /// * `frame`: The frame to draw on
    /// * `area`: The area to draw on
    ///
    /// # Returns
    /// 
    /// * `()`
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {

        // We have nothing to draw if the help state is not set
        if app.interface.components.help_state.is_none() {
            return;
        }

        // Get the help state
        let help_state = app.interface.components.help_state.as_ref().unwrap();

        // --- Main Horizontal Layout (Sidebar | Content) ---
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        // Navigation sidebar
        let sidebar_area = inner_chunks[0];
        let content_area = inner_chunks[1];

        // --- Sidebar Rendering ---
        let sidebar_block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White)); 
        frame.render_widget(sidebar_block.clone(), sidebar_area);
        let inner_sidebar = sidebar_block.inner(sidebar_area);

        // Topic list
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
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, inner_sidebar, &mut list_state);

        // --- Content Area Rendering ---
        let title = help_state.topics.get(help_state.selected_index).unwrap_or(&"Help".to_string()).clone();
        let content_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White)); 
        frame.render_widget(content_block.clone(), content_area);
        let inner_content_area = content_block.inner(content_area); 

        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Central text
                Constraint::Length(1), // Help text
            ])
            .split(inner_content_area);

        let text_render_base_area = content_chunks[0]; // Area for text + padding
        let content_help_area = content_chunks[1];

        // Add Vertical Padding (Top) 
        let vertical_padded_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 1 line padding top
                Constraint::Min(0),    // Area for horizontally padded text
            ])
            .split(text_render_base_area); // Split the base area vertically

        let horizontal_padding_area = vertical_padded_layout[1]; // Use the bottom chunk for horizontal padding

        // Add Horizontal Padding (Left/Right)
        let horizontal_padded_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1), // 1 column padding left
                Constraint::Min(0),    // Actual text content area
                Constraint::Length(1), // 1 column padding right
            ])
            .split(horizontal_padding_area); // Split the area remaining after vertical padding

        let actual_text_area = horizontal_padded_layout[1]; // Render text in the middle chunk

        // Get the content for the current topic
        let content_str = help_state
            .contents
            .get(help_state.selected_index)
            .map_or("No content available.", |s| s.as_str());
        
        // Parse the markdown content (currently placeholder)
        let content = parse_markdown(content_str); 

        let content_paragraph = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((help_state.scroll_offset, 0)); // Apply vertical scroll offset
        frame.render_widget(content_paragraph, actual_text_area); // Render into the padded area

        // Display the help text (keybindings)
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("↑↓", key_style), Span::styled(": Topics | ", help_style),
            Span::styled("←/PgUp", key_style), Span::styled(": Scroll Up | ", help_style),
            Span::styled("→/PgDn", key_style), Span::styled(": Scroll Down", help_style),
        ];
        let help = Paragraph::new(Line::from(help_spans))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, content_help_area); 
    }
}