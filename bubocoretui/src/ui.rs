use crate::app::{App, Mode};
use crate::components::*;
use color_eyre::Result as EyreResult;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Clear, Paragraph, Borders},
};
use crate::components::editor::EditorComponent;
use crate::components::grid::GridComponent;
use crate::components::help::HelpComponent;
use crate::components::navigation::NavigationComponent;
use crate::components::options::OptionsComponent;
use crate::components::splash::SplashComponent;
use crate::components::devices::DevicesComponent;
use crate::components::logs::LogsComponent;
use crate::components::files::FilesComponent;
use std::time::{Duration, Instant};
pub struct Flash {
    pub is_flashing: bool,
    pub flash_start: Option<Instant>,
    pub flash_duration: Duration,
}


/// Fonction principale de l'interface utilisateur qui gère le rendu de l'application
/// 
/// Cette fonction :
/// - Vérifie l'état du flash
/// - Configure la mise en page principale
/// - Dessine la barre supérieure
/// - Affiche le composant approprié selon le mode
/// - Dessine la barre inférieure
/// - Gère l'effet de flash si nécessaire
pub fn ui(frame: &mut Frame, app: &mut App) {
    check_flash_status(app);
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let top_bar = main_layout[0];
    let main_area = main_layout[1];
    let bottom_bar = main_layout[2];

    draw_top_bar(frame, app, top_bar);

    // Affiche le composant approprié selon le mode actuel
    match app.interface.screen.mode {
        Mode::Splash => SplashComponent::new().draw(app, frame, main_area),
        Mode::Editor => EditorComponent::new().draw(app, frame, main_area),
        Mode::Grid => GridComponent::new().draw(app, frame, main_area),
        Mode::Options => OptionsComponent::new().draw(app, frame, main_area),
        Mode::Help => HelpComponent::new().draw(app, frame, main_area),
        Mode::Devices => DevicesComponent::new().draw(app, frame, main_area),
        Mode::Logs => LogsComponent::new().draw(app, frame, main_area),
        Mode::Files => FilesComponent::new().draw(app, frame, main_area),
        Mode::Navigation => NavigationComponent::new().draw(app, frame, main_area),
    }

    draw_bottom_bar(frame, app, bottom_bar);

    // Gère l'effet de flash si nécessaire
    if app.interface.screen.flash.is_flashing {
        frame.render_widget(Clear, frame.area());
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::White)),
            frame.area(),
        );
    }
}

fn check_flash_status(app: &mut App) {
    if app.interface.screen.flash.is_flashing {
        if let Some(start_time) = app.interface.screen.flash.flash_start {
            if start_time.elapsed() > app.interface.screen.flash.flash_duration {
                app.interface.screen.flash.is_flashing = false;
                app.interface.screen.flash.flash_start = None;
            }
        }
    }
}

/// Dessine la barre inférieure de l'interface
/// 
/// Cette fonction gère l'affichage de la barre de statut en bas de l'écran.
/// Elle affiche soit :
/// - Le mode actuel, le message du bas, le tempo et le beat en mode normal
/// - Un prompt de commande en mode commande
pub fn draw_bottom_bar(frame: &mut Frame, app: &mut App, area: Rect) -> EyreResult<()> {
    // Mode commande actif : affiche le prompt
    if app.interface.components.command_mode.active {
        let cmd_block = Block::default().borders(Borders::ALL).title("Command");
        let cmd_area = cmd_block.inner(area);
        frame.render_widget(cmd_block, area);
        frame.render_widget(
            app.interface.components.command_mode.text_area.widget(),
            cmd_area,
        );
        // Optionally set cursor position for the command text area
        frame.set_cursor(
            cmd_area.x + app.interface.components.command_mode.text_area.cursor().0 as u16,
            cmd_area.y + app.interface.components.command_mode.text_area.cursor().1 as u16
        );
    } 
    // Mode commande inactif : affiche le nom de la vue actuelle, etc...
    else {
        // Affiche le nom de la vue actuelle
        let mode_text = match app.interface.screen.mode {
            Mode::Editor => "EDITOR",
            Mode::Grid => "GRID", 
            Mode::Options => "OPTIONS",
            Mode::Splash => "WELCOME",
            Mode::Help => "HELP",
            Mode::Devices => "DEVICES",
            Mode::Logs => "LOGS",
            Mode::Files => "FILES",
            Mode::Navigation => "NAVIGATION",
        };

        // Récupère les informations de tempo et de beat
        let phase = app.server.link.get_phase();
        let beat = phase.floor() + 1.0;
        let tempo = app.server.link.session_state.tempo();

        // Formate le texte de statut
        let status_text = format!(
            "[ {} ] | {} | {:.1} BPM | Beat {:.0}/{:.0}",
            mode_text, app.interface.components.bottom_message, tempo, beat, app.server.link.quantum
        );

        // Gère le troncage du texte si nécessaire
        let available_width = area.width as usize;
        let combined_text = if status_text.len() + 3 <= available_width {
            format!("{}", status_text)
        } else if status_text.len() + 3 < available_width {
            status_text
        } else {
            format!("{}...", &status_text[0..available_width.saturating_sub(3)])
        };

        // Affiche la barre de statut
        let bottom_bar = Paragraph::new(Text::from(combined_text))
            .style(Style::default().bg(Color::White).fg(Color::Black));

        frame.render_widget(bottom_bar, area);
    }
    Ok(()) // Return EyreResult Ok
}

/// Dessine la barre de progression en haut de l'interface
/// 
/// Cette fonction crée une barre de progression visuelle qui représente
/// l'avancement dans le cycle musical actuel. La barre se remplit de gauche
/// à droite en fonction de la phase actuelle par rapport au quantum.
fn draw_top_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    let phase = app.server.link.get_phase();
    let available_width = area.width as usize;
    let filled_width = ((phase / app.server.link.quantum) * available_width as f64) as usize;
    let mut bar = String::with_capacity(available_width);
    for i in 0..available_width {
        if i < filled_width {
            bar.push('█');
        } else {
            bar.push(' ');
        }
    }
    let top_bar = Paragraph::new(Text::from(bar))
        .style(Style::default().bg(Color::Green).fg(Color::Red));
    frame.render_widget(top_bar, area);
}
