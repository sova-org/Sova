use crate::app::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use markdownparser::parse_markdown;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget,
    },
    Frame,
};

pub mod markdownparser;

/// Manages the state for the `HelpComponent`.
///
/// Holds help topics, content, selection/scroll state, and search functionality state.
#[derive(Clone)]
pub struct HelpState {
    /// Topic titles.
    pub topics: Vec<String>,
    /// Markdown content corresponding to each topic by index.
    pub contents: Vec<String>,
    /// Index of the currently selected topic in the potentially filtered list.
    pub selected_index: usize,
    /// Vertical scroll offset for the content paragraph.
    pub scroll_offset: u16,
    /// Current search query entered by the user to filter topics.
    pub search_query: String,
    /// `true` if the user is currently entering a search query.
    pub is_searching: bool,
}

impl HelpState {
    /// Creates a new `HelpState` instance, loading topics and content from static files.
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

        let contents = vec![
            include_str!("../../static/help/welcome.md").to_string(),
            include_str!("../../static/help/navigation.md").to_string(),
            include_str!("../../static/help/editor.md").to_string(),
            include_str!("../../static/help/grid.md").to_string(),
            include_str!("../../static/help/commands.md").to_string(),
            include_str!("../../static/help/logs.md").to_string(),
            include_str!("../../static/help/devices.md").to_string(),
            include_str!("../../static/help/files.md").to_string(),
            include_str!("../../static/help/about.md").to_string(),
        ];

        HelpState {
            topics,
            contents,
            selected_index: 0,
            scroll_offset: 0,
            search_query: String::new(),
            is_searching: false,
        }
    }

    /// Scrolls the content view down by one line.
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scrolls the content view up by one line.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
}

/// The Help component responsible for displaying help topics and content.
///
/// Implements `StatefulWidget` to manage rendering based on `HelpState` and
/// `Component` for event handling and application integration.
#[derive(Clone)]
pub struct HelpComponent;

impl HelpComponent {
    /// Creates a new `HelpComponent`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for HelpComponent {
    /// Handles key events for the Help component.
    ///
    /// Prioritizes search input mode if active. Otherwise handles navigation,
    /// scrolling, and entering search mode.
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        if let Some(help_state) = &mut app.interface.components.help_state {
            // Handle Searching Input Mode first if active
            if help_state.is_searching {
                match key_event.code {
                    KeyCode::Esc => {
                        help_state.is_searching = false;
                        help_state.search_query.clear();
                        help_state.selected_index = 0; // Reset selection to top of unfiltered list
                        Ok(true)
                    }
                    KeyCode::Enter => {
                        help_state.is_searching = false;
                        help_state.selected_index = 0; // Reset selection to top of filtered list
                        Ok(true)
                    }
                    KeyCode::Backspace => {
                        if !help_state.search_query.is_empty() {
                            help_state.search_query.pop();
                            help_state.selected_index = 0; // Reset selection when query changes
                        }
                        Ok(true)
                    }
                    KeyCode::Char(c) => {
                        // Add character if it's not modified by Ctrl/Alt
                        if !key_event
                            .modifiers
                            .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
                        {
                            help_state.search_query.push(c);
                            help_state.selected_index = 0; // Reset selection when query changes
                        }
                        Ok(true)
                    }
                    // Consume navigation keys while typing search query
                    KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown => Ok(true),
                    _ => Ok(false), // Pass unhandled keys through
                }
            } else {
                // Handle Normal Navigation/Action Mode
                match key_event.code {
                    KeyCode::Char('/') => {
                        help_state.is_searching = true;
                        help_state.search_query.clear();
                        help_state.selected_index = 0;
                        Ok(true)
                    }
                    KeyCode::Up => {
                        // Actual index update happens in render_sidebar based on filtered count
                        help_state.selected_index = help_state.selected_index.saturating_sub(1);
                        help_state.scroll_offset = 0;
                        Ok(true)
                    }
                    KeyCode::Down => {
                        // Actual index update and clamping happens in render_sidebar
                        help_state.selected_index = help_state.selected_index.saturating_add(1);
                        help_state.scroll_offset = 0;
                        Ok(true)
                    }
                    KeyCode::PageUp | KeyCode::Left => {
                        help_state.scroll_up();
                        Ok(true)
                    }
                    KeyCode::PageDown | KeyCode::Right => {
                        help_state.scroll_down();
                        Ok(true)
                    }
                    _ => Ok(false), // Event not handled by this component in this mode
                }
            }
        } else {
            Ok(false) // No state, event not handled
        }
    }

    /// Draws the Help component UI.
    ///
    /// Delegates the actual rendering to the `StatefulWidget::render` implementation
    /// by calling `frame.render_stateful_widget`.
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        if let Some(help_state) = &mut app.interface.components.help_state.clone() {
            // Clone state temporarily to pass mutably, avoiding borrowing conflicts
            // with the immutable `app` reference in this `draw` method signature.
            frame.render_stateful_widget(self.clone(), area, help_state);
        }
    }
}

impl StatefulWidget for HelpComponent {
    type State = HelpState;

    /// Renders the Help component UI based on the `HelpState`.
    ///
    /// Draws the sidebar, content area, and an optional search bar.
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Define Main Layout: potentially splits area for search bar at the bottom
        let (search_bar_area, main_content_area) = if state.is_searching {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),    // Main area for sidebar/content
                    Constraint::Length(3), // Search bar height
                ])
                .split(area);
            (Some(main_chunks[1]), main_chunks[0])
        } else {
            (None, area) // No search bar, use full area
        };

        // Split main content area into Sidebar | Content
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(main_content_area);

        let sidebar_area = chunks[0];
        let content_area = chunks[1];

        // --- Render Sidebar --- //
        self.render_sidebar(buf, sidebar_area, state);

        // --- Render Content Area --- //
        self.render_content(buf, content_area, state);

        // --- Render Search Bar (if active) --- //
        if let Some(search_area) = search_bar_area {
            self.render_search_bar(buf, search_area, state);
        }
    }
}

impl HelpComponent {
    /// Renders the sidebar containing the filtered list of help topics.
    fn render_sidebar(&self, buf: &mut Buffer, area: Rect, state: &mut HelpState) {
        let sidebar_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White));
        let inner_sidebar = sidebar_block.inner(area);
        sidebar_block.render(area, buf);

        // Filter topics based on search query (case-insensitive check on title and content)
        let search_term = state.search_query.to_lowercase();
        let filtered_topics: Vec<(usize, &String)> = state
            .topics
            .iter()
            .enumerate()
            .filter(|(index, topic_name)| {
                if search_term.is_empty() {
                    true // Show all if search is empty
                } else {
                    topic_name.to_lowercase().contains(&search_term)
                        || state.contents[*index]
                            .to_lowercase()
                            .contains(&search_term)
                }
            })
            .collect();

        let filtered_count = filtered_topics.len();

        // Clamp selected index based on the filtered list size
        if filtered_count > 0 {
            state.selected_index = state.selected_index.min(filtered_count - 1);
        } else {
            state.selected_index = 0;
        }

        // Create list items from filtered topics
        let items: Vec<ListItem> = filtered_topics
            .iter()
            .map(|(_, topic_name)| {
                ListItem::new(Line::from(Span::styled(
                    *topic_name,
                    Style::default().fg(Color::White),
                )))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default())
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_index));

        StatefulWidget::render(list, inner_sidebar, buf, &mut list_state);
    }

    /// Renders the main content area, displaying the selected topic's markdown content.
    fn render_content(&self, buf: &mut Buffer, area: Rect, state: &HelpState) {
        let content_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White));
        let inner_content_area = content_block.inner(area);
        content_block.render(area, buf);

        // Layout for Content Area: Text + Help Footer
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Central text area
                Constraint::Length(1), // Help text footer
            ])
            .split(inner_content_area);

        let text_render_base_area = content_chunks[0];
        let content_help_area = content_chunks[1];

        // Add Padding (Top, Left, Right) to the text area
        let actual_text_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)]) // Top padding
            .split(text_render_base_area)[1];
        let actual_text_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1), // Left padding
                Constraint::Min(0),
                Constraint::Length(1), // Right padding
            ])
            .split(actual_text_area)[1];

        // Determine the original index of the selected topic from the filtered list
        // This filtering logic is duplicated from render_sidebar; could be optimized.
        let search_term = state.search_query.to_lowercase();
        let filtered_topics: Vec<(usize, &String)> = state
            .topics
            .iter()
            .enumerate()
            .filter(|(index, topic_name)| {
                if search_term.is_empty() {
                    true
                } else {
                    topic_name.to_lowercase().contains(&search_term)
                        || state.contents[*index]
                            .to_lowercase()
                            .contains(&search_term)
                }
            })
            .collect();

        let original_index = if filtered_topics.is_empty() {
            None
        } else {
            let current_selected = state.selected_index.min(filtered_topics.len() - 1);
            filtered_topics.get(current_selected).map(|(idx, _)| *idx)
        };

        // Get content based on original index and parse markdown
        let content_str = original_index
            .and_then(|idx| state.contents.get(idx))
            .map_or("No content available.", |s| s.as_str());
        let parsed_content = parse_markdown(content_str);

        let content_paragraph = Paragraph::new(parsed_content)
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((state.scroll_offset, 0));

        content_paragraph.render(actual_text_area, buf);

        // Render the help text footer
        self.render_footer(buf, content_help_area);
    }

    /// Renders the footer help text with keybindings.
    fn render_footer(&self, buf: &mut Buffer, area: Rect) {
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("↑↓", key_style),
            Span::styled(": Topics | ", help_style),
            Span::styled("/", key_style),
            Span::styled(": Search | ", help_style),
            Span::styled("←/PgUp", key_style),
            Span::styled(": Scroll Up | ", help_style),
            Span::styled("→/PgDn", key_style),
            Span::styled(": Scroll Down", help_style),
        ];
        let help_paragraph = Paragraph::new(Line::from(help_spans)).alignment(Alignment::Center);

        help_paragraph.render(area, buf);
    }

    /// Renders the search input bar when active.
    fn render_search_bar(&self, buf: &mut Buffer, area: Rect, state: &HelpState) {
        let title = " Search Help (Type, Esc: Clear, Enter: Exit) ";
        let search_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(Color::Yellow));

        // Get inner area to render paragraph inside
        let inner_area = search_block.inner(area);

        // Render block frame first
        search_block.render(area, buf);

        // Render search query paragraph inside the block
        let search_paragraph = Paragraph::new(state.search_query.as_str())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);

        search_paragraph.render(inner_area, buf);
    }
}
