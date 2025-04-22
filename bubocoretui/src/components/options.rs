use crate::app::App;
use crate::components::Component;
use crate::app::EditorKeymapMode;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, BorderType, ListState},
};

pub struct OptionsComponent;

impl OptionsComponent {
    pub fn new() -> Self {
        Self {} 
    }
}

impl Component for OptionsComponent {
    // Change signature back to &mut self to satisfy the trait
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        let selected_index = &mut app.interface.components.options_selected_index;
        let num_options = app.interface.components.options_num_options; // Read num_options

        match key_event.code {
            KeyCode::Up => {
                *selected_index = selected_index.saturating_sub(1);
                return Ok(true);
            }
            KeyCode::Down => {
                *selected_index = (*selected_index + 1).min(num_options - 1);
                return Ok(true);
            }
            KeyCode::Enter => {
                match *selected_index { // Dereference selected_index to match
                    0 => { // Toggle Phase Bar
                        app.settings.show_phase_bar = !app.settings.show_phase_bar;
                        app.set_status_message(format!(
                            "Phase bar visibility set to {}",
                            if app.settings.show_phase_bar { "On" } else { "Off" }
                        ));
                    }
                    1 => { // Toggle Editor Keymap Mode
                        app.settings.editor_keymap_mode = match app.settings.editor_keymap_mode {
                            EditorKeymapMode::Normal => EditorKeymapMode::Vim,
                            EditorKeymapMode::Vim => EditorKeymapMode::Normal,
                        };
                        app.set_status_message(format!(
                            "Editor Keymap set to {:?}",
                            app.settings.editor_keymap_mode
                        ));
                    }
                    _ => {} // Should not happen if num_options is correct
                }
                return Ok(true);
            }
            _ => {}
        }
        Ok(false)
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Options ")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // List of options
                Constraint::Length(1), // Help
            ])
            .split(inner_area);

        let options_area = chunks[0];
        let help_area = chunks[1];

        // List of available options
        let options = vec![
            ListItem::new(Line::from(vec![
                Span::raw("Show Phase Bar: "),
                Span::styled(
                    if app.settings.show_phase_bar { "On" } else { "Off" },
                    Style::default().fg(if app.settings.show_phase_bar { Color::Green } else { Color::Red }),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("Editor Keymap:  "),
                Span::styled(
                    format!("{:?}", app.settings.editor_keymap_mode),
                    Style::default().fg(Color::Cyan),
                ),
            ])),
        ];

        let options_list = List::new(options)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
            .highlight_symbol("> ");

        // Create a ListState to manage the selection using app state
        let mut list_state = ListState::default();
        list_state.select(Some(app.interface.components.options_selected_index)); // Read from app state

        frame.render_stateful_widget(options_list, options_area, &mut list_state);

        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("↑↓", key_style), Span::styled(": Navigate | ", help_style),
            Span::styled("Enter", key_style), Span::styled(": Toggle Option", help_style),
        ];
        let help = Paragraph::new(Line::from(help_spans))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
