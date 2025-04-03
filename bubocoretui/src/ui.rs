use crate::app::{App, Mode};
use crate::components::*;
use color_eyre::Result as EyreResult;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Clear, Paragraph},
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
    // Style général pour la barre (fond blanc, texte noir par défaut)
    let base_style = Style::default().bg(Color::White).fg(Color::Black);
    frame.render_widget(Block::default().style(base_style), area);

    // Mode commande actif
    if app.interface.components.command_mode.active {
        let command_block = Block::default().style(base_style);
        let command_area = command_block.inner(area);
        frame.render_widget(command_block, area);
        // Appliquer le style de base au textarea pour le contraste
        app.interface.components.command_mode.text_area.set_style(base_style);
        frame.render_widget(&app.interface.components.command_mode.text_area, command_area);
    } 
    // Mode commande inactif
    else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65), // Espace pour mode et message
                Constraint::Percentage(35), // Espace pour utilisateur et tempo
            ])
            .split(area);

        let left_area = chunks[0];
        let right_area = chunks[1];

        // --- Partie Gauche --- 
        let mode_text = match app.interface.screen.mode {
            Mode::Editor => "EDITOR",
            Mode::Grid => "GRID",
            Mode::Options => "OPTIONS",
            Mode::Splash => "WELCOME",
            Mode::Help => "HELP",
            Mode::Devices => "DEVICES",
            Mode::Logs => "LOGS",
            Mode::Files => "FILES",
            Mode::Navigation => "MENU",
        };
        
        // Style pour le mode : fond cyan, texte noir gras
        let mode_style = Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD);
        
        // Calcul espace message
        let mode_width = mode_text.len() + 2; // " MODE "
        let separator_width = 3; // " | "
        let max_message_width = left_area.width.saturating_sub(mode_width as u16 + separator_width as u16) as usize;
        let message = &app.interface.components.bottom_message;
        let truncated_message = if message.len() > max_message_width {
             format!("{}...", &message[..max_message_width.saturating_sub(3)])
        } else {
             message.to_string()
        };

        let left_text = Line::from(vec![
            Span::styled(format!(" {} ", mode_text), mode_style),
            Span::raw(" | "),
            Span::styled(truncated_message, Style::default().fg(Color::Black)), // Message en noir
        ]);
        let left_paragraph = Paragraph::new(left_text)
            .style(base_style)
            .alignment(Alignment::Left);
        frame.render_widget(left_paragraph, left_area);

        // --- Partie Droite --- 
        let tempo = app.server.link.session_state.tempo();
        let username = &app.server.username;

        // Calcul de l'espace max pour le username
        let tempo_text = format!("{:.1} BPM", tempo);
        let tempo_width = tempo_text.len() + 3; // " | TEMPO "
        let max_username_width = right_area.width.saturating_sub(tempo_width as u16) as usize;
        let truncated_username = if username.len() > max_username_width {
            format!("{}...", &username[..max_username_width.saturating_sub(3)])
        } else {
            username.clone()
        };

        let right_text = Line::from(vec![
            Span::styled(truncated_username, Style::default().fg(Color::Yellow)), // Username en jaune
            Span::raw(" | "),
            Span::styled(tempo_text, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(" "), // Padding droit
        ]).alignment(Alignment::Right);
        
        let right_paragraph = Paragraph::new(right_text)
            .style(base_style);
        frame.render_widget(right_paragraph, right_area);
    }
    Ok(())
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
