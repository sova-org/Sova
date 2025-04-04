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
use crate::components::saveload::SaveLoadComponent;
use std::time::{Duration, Instant};
pub struct Flash {
    pub is_flashing: bool,
    pub flash_start: Option<Instant>,
    pub flash_duration: Duration,
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    check_flash_status(app);

    // Ajuster les contraintes en fonction de l'affichage de la barre de phase
    let top_bar_height = if app.settings.show_phase_bar { 1 } else { 0 };

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_bar_height), // Barre du haut (phase)
            Constraint::Min(1),             // Zone principale
            Constraint::Length(1),             // Barre du bas (statut)
        ])
        .split(frame.area());

    let top_bar = main_layout[0];
    let main_area = main_layout[1];
    let bottom_bar = main_layout[2];

    draw_top_bar(frame, app, top_bar);

    // Obtenir une référence mutable aux composants stockés dans App
    // (Nécessite que App contienne ces instances, par exemple dans app.interface.components)
    // Exemple: let components = &mut app.interface.components;

    // --- Appel de before_draw sur le composant actif --- 
    let before_draw_result = match app.interface.screen.mode {
        // Mode::Splash => components.splash_component.before_draw(app),
        // Mode::Editor => components.editor_component.before_draw(app),
        // Mode::Grid => components.grid_component.before_draw(app),
        // Mode::Options => components.options_component.before_draw(app),
        // Mode::Help => components.help_component.before_draw(app),
        // Mode::Devices => components.devices_component.before_draw(app),
        // Mode::Logs => components.logs_component.before_draw(app),
        Mode::SaveLoad => SaveLoadComponent::new().before_draw(app), // TEMPORAIRE: Utilise encore ::new() 
                                                                     // mais appelle before_draw. Nécessite refactoring d'App.
        // Mode::Navigation => components.navigation_component.before_draw(app),
        _ => Ok(()), // Gérer les autres modes ou retourner une erreur par défaut
    };

    // Gérer l'erreur de before_draw si nécessaire
    if let Err(e) = before_draw_result {
        // Logguer ou afficher l'erreur, ex:
        app.add_log(crate::app::LogLevel::Error, format!("Error in before_draw: {}", e));
    }
    // ----------------------------------------------------

    // --- Dessin du composant actif --- 
    match app.interface.screen.mode {
        Mode::Splash => SplashComponent::new().draw(app, frame, main_area),
        Mode::Editor => EditorComponent::new().draw(app, frame, main_area),
        Mode::Grid => GridComponent::new().draw(app, frame, main_area),
        Mode::Options => OptionsComponent::new().draw(app, frame, main_area),
        Mode::Help => HelpComponent::new().draw(app, frame, main_area),
        Mode::Devices => DevicesComponent::new().draw(app, frame, main_area),
        Mode::Logs => LogsComponent::new().draw(app, frame, main_area),
        Mode::SaveLoad => SaveLoadComponent::new().draw(app, frame, main_area),
        Mode::Navigation => NavigationComponent::new().draw(app, frame, main_area),
        // _ => {} // Gérer les autres cas si nécessaire
    }

    if let Err(e) = draw_bottom_bar(frame, app, bottom_bar) {
        app.add_log(crate::app::LogLevel::Error, format!("Error drawing bottom bar: {}", e));
    }

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
/// - Le mode actuel, le message du bas, le nom d'utilisateur, une mini barre de phase, le tempo et le beat en mode normal
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
                Constraint::Percentage(60), // Adjusted: Less space for left side
                Constraint::Percentage(40), // Adjusted: More space for right side (user, phase, tempo)
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
            Mode::Navigation => "MENU",
            Mode::SaveLoad => "FILES", // Changed to FILES for consistency? Or keep SAVE?
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
        let phase = app.server.link.get_phase();
        let quantum = app.server.link.quantum.max(1.0); // Avoid division by zero/infinitesimal quantum
        let username = &app.server.username;

        // Mini Phase Bar Calculation
        let mini_bar_width = 10; // Fixed width for the mini bar
        let filled_ratio = (phase / quantum).clamp(0.0, 1.0); // Ensure ratio is between 0 and 1
        let filled_count = (filled_ratio * mini_bar_width as f64).round() as usize;
        let empty_count = mini_bar_width - filled_count;
        let mini_bar_str = format!("{}{}", "█".repeat(filled_count), " ".repeat(empty_count));
        let mini_bar_style = Style::default().fg(Color::Green); // Style for the mini bar

        // Calcul espace restant pour username et tempo
        let tempo_text = format!("{:.1} BPM", tempo);
        let tempo_width = tempo_text.len() + 1; // " TEMPO" + space padding right
        let phase_bar_width = mini_bar_width + 2 + 2; // "[" + bar + "]" + " | " + " "
        let reserved_width = tempo_width + phase_bar_width;
        let max_username_width = right_area.width.saturating_sub(reserved_width as u16) as usize;

        let truncated_username = if username.len() > max_username_width {
            format!("{}...", &username[..max_username_width.saturating_sub(3)])
        } else {
            username.clone()
        };

        let right_text = Line::from(vec![
            Span::styled(truncated_username, Style::default().fg(Color::Yellow)), // Username en jaune
            Span::raw(" | "),
            Span::styled(mini_bar_str, mini_bar_style), // Mini Phase Bar
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
    if !app.settings.show_phase_bar {
        return; // Ne rien dessiner si l'option est désactivée
    }

    let phase = app.server.link.get_phase();
    let quantum = app.server.link.quantum.max(1.0); // Prevent division by zero
    let available_width = area.width as usize;
    // Ensure phase calculation doesn't lead to NaN or Inf if quantum is tiny
    let filled_ratio = if quantum > 0.0 { (phase / quantum).clamp(0.0, 1.0) } else { 0.0 };
    let filled_width = (filled_ratio * available_width as f64).round() as usize;

    let mut bar = String::with_capacity(available_width);
    for i in 0..available_width {
        if i < filled_width {
            bar.push('█');
        } else {
            bar.push(' ');
        }
    }
    let top_bar = Paragraph::new(Text::from(bar))
        .style(Style::default().bg(Color::DarkGray).fg(Color::Green));
    frame.render_widget(top_bar, area);
}
