use crate::app::{App, EditorKeymapMode};
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

/// Component responsible for displaying and handling application settings.
pub struct OptionsComponent;

impl OptionsComponent {
    /// Creates a new `OptionsComponent`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for OptionsComponent {
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let selected_index = &mut app.interface.components.options_selected_index;
        // Read num_options from the app state where it should be managed
        let num_options = app.interface.components.options_num_options;

        match key_event.code {
            KeyCode::Up => {
                *selected_index = selected_index.saturating_sub(1);
                Ok(true)
            }
            KeyCode::Down => {
                *selected_index = (*selected_index + 1).min(num_options.saturating_sub(1));
                Ok(true)
            }
            KeyCode::Enter => {
                // Handle selection based on the current index
                match *selected_index {
                    0 => { // Index 0 is now Editor Keymap
                        app.settings.editor_keymap_mode = match app.settings.editor_keymap_mode {
                            EditorKeymapMode::Normal => EditorKeymapMode::Vim,
                            EditorKeymapMode::Vim => EditorKeymapMode::Normal,
                        };
                        app.set_status_message(format!(
                            "Editor Keymap set to {:?}",
                            app.settings.editor_keymap_mode
                        ));
                    }
                    _ => {} // Other indices are currently unused
                }
                Ok(true)
            }
            _ => Ok(false), // Return false if the key wasn't handled here
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title(" Options ")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // List of options
                Constraint::Length(1), // Help text at the bottom
            ])
            .split(inner_area);

        let options_area = chunks[0];
        let help_area = chunks[1];

        // Define the list items for available options
        let options = vec![
            // Removed "Show Phase Bar" option
            ListItem::new(Line::from(vec![
                Span::raw("Editor Keymap:  "),
                Span::styled(
                    format!("{:?}", app.settings.editor_keymap_mode),
                    Style::default().fg(Color::Cyan), // Keep distinctive color
                ),
            ])),
            // Add more options here in the future
        ];

        // Create the list widget
        let options_list = List::new(options)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray),
            )
            .highlight_symbol("> ");

        // Manage list state based on app state
        let mut list_state = ListState::default();
        list_state.select(Some(app.interface.components.options_selected_index));

        frame.render_stateful_widget(options_list, options_area, &mut list_state);

        // Draw help text
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("↑↓", key_style),
            Span::styled(": Navigate | ", help_style),
            Span::styled("Enter", key_style),
            Span::styled(": Toggle Option", help_style),
        ];
        let help = Paragraph::new(Line::from(help_spans)).alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
