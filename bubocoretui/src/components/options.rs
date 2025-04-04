use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, List, ListItem},
};

pub struct OptionsComponent {
    /// Index de l'option actuellement sélectionnée
    selected_option_index: usize,
}

impl OptionsComponent {
    pub fn new() -> Self {
        Self { selected_option_index: 0 }
    }
}

impl Component for OptionsComponent {

    fn before_draw(&mut self, _app: &mut App) -> EyreResult<()> {
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        // Pour l'instant, nous n'avons qu'une seule option (index 0)
        // La navigation haut/bas sera ajoutée lorsque nous aurons plus d'options.

        match key_event.code {
            KeyCode::Enter => {
                // Basculer l'option "Show Phase Bar"
                if self.selected_option_index == 0 {
                    app.settings.show_phase_bar = !app.settings.show_phase_bar;
                    app.set_status_message(format!(
                        "Phase bar visibility set to {}",
                        if app.settings.show_phase_bar { "On" } else { "Off" }
                    ));
                    return Ok(true);
                }
            }
            // TODO: Ajouter KeyCode::Up et KeyCode::Down pour la navigation
            _ => {}
        }
        Ok(false)
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Options ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Liste des options
                Constraint::Length(1), // Aide
            ])
            .split(inner_area);

        let options_area = chunks[0];
        let help_area = chunks[1];

        // Définir les options disponibles
        let options = vec![
            ListItem::new(Line::from(vec![
                Span::raw("Show Phase Bar: "),
                Span::styled(
                    if app.settings.show_phase_bar { "On" } else { "Off" },
                    Style::default().fg(if app.settings.show_phase_bar { Color::Green } else { Color::Red }),
                ),
            ])),
            // Ajouter d'autres options ici...
        ];

        // Créer la liste des options
        let options_list = List::new(options)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray));

        // Pour l'instant, nous ne dessinons pas la sélection car il n'y a qu'une option.
        // Nous utiliserons `ListState` lorsque nous aurons plusieurs options.
        frame.render_widget(options_list, options_area);

        let help_text = "Enter: Toggle Option"; // TODO: Ajouter Up/Down
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
