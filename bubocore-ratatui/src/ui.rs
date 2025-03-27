use crate::app::{App, Mode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Clear, Paragraph},
};
use std::time::Instant;

use crate::components::{editor, grid, options, splash};

pub fn flash_screen(app: &mut App) {
    app.screen_state.flash.is_flashing = true;
    app.screen_state.flash.flash_start = Some(Instant::now());
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    let flash = &mut app.screen_state.flash;
    if flash.is_flashing {
        if let Some(start_time) = flash.flash_start {
            if start_time.elapsed() > flash.flash_duration {
                flash.is_flashing = false;
                flash.flash_start = None;
            }
        }
    }
    // Layout avec barre du haut, contenu et barre du bas
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

    match app.screen_state.mode {
        Mode::Splash => splash::draw(frame, app, main_area),
        Mode::Editor => editor::draw(frame, app, main_area),
        Mode::Grid => grid::draw(frame, app, main_area),
        Mode::Options => options::draw(frame, app, main_area),
    }

    draw_bottom_bar(frame, app, bottom_bar);

    let flash = &mut app.screen_state.flash;
    if flash.is_flashing {
        frame.render_widget(Clear, frame.area());
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::White)),
            frame.area(),
        );
    }
}

fn draw_bottom_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    if !app.command_mode.active {
        // Affichage du nom de la vue actuelle !
        let mode_text = match app.screen_state.mode {
            Mode::Editor => "EDITOR",
            Mode::Grid => "GRID",
            Mode::Options => "OPTIONS",
            Mode::Splash => "SPLASH",
        };
        // Get current tempo and beat information
        let phase = app.link_client.get_phase();
        let beat = (phase / app.link_client.quantum * 4.0).floor() + 1.0;
        let tempo = app.link_client.session_state.tempo();

        let status_text = format!(
            "[ {} ] | {} | {:.1} BPM | Beat {:.0}/{:.0}",
            mode_text, app.status_message, tempo, beat, app.link_client.quantum
        );

        let available_width = area.width as usize;
        let combined_text = if status_text.len() + 3 <= available_width {
            format!("{}", status_text)
        } else if status_text.len() + 3 < available_width {
            status_text
        } else {
            format!("{}...", &status_text[0..available_width.saturating_sub(3)])
        };

        let bottom_bar = Paragraph::new(Text::from(combined_text))
            .style(Style::default().bg(Color::White).fg(Color::Black));

        frame.render_widget(bottom_bar, area);
    } else {
        // Command prompt mode
        let prompt_area = area;

        // Create a layout to add a small prompt indicator
        let prompt_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2), // Width of the prompt indicator
                Constraint::Min(1),    // Width of the input field
            ])
            .split(prompt_area);

        // Draw the prompt indicator
        let prompt =
            Paragraph::new(":").style(Style::default().bg(Color::DarkGray).fg(Color::White));
        frame.render_widget(prompt, prompt_layout[0]);

        // Draw the textarea for input
        app.command_mode
            .text_area
            .set_style(Style::default().bg(Color::DarkGray).fg(Color::White));
        frame.render_widget(&app.command_mode.text_area, prompt_layout[1]);
    }
}

fn draw_top_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    let phase = app.link_client.get_phase();

    // Représentation visuelle de la barre
    let available_width = area.width as usize;
    let filled_width = ((phase / app.link_client.quantum) * available_width as f64) as usize;
    let mut bar = String::with_capacity(available_width);
    for i in 0..available_width {
        if i < filled_width {
            bar.push('█');
        } else {
            bar.push(' ');
        }
    }

    let top_bar =
        Paragraph::new(Text::from(bar)).style(Style::default().bg(Color::Green).fg(Color::Red));

    frame.render_widget(top_bar, area);
}
