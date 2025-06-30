use ratatui::prelude::*;
use ratatui::widgets::Cell;

/// Pure data for rendering a single grid cell
#[derive(Clone, Debug)]
pub struct CellData {
    pub frame_value: f64,
    pub frame_name: Option<String>,
    pub is_enabled: bool,
    pub is_playing: bool,
    pub time_progression: Option<f32>, // 0.0 to 1.0
    pub interaction: CellInteraction,
    pub repetitions: usize, // Number of repetitions for this frame
}

#[derive(Clone, Debug)]
pub enum CellInteraction {
    None,
    LocalCursor,
    LocalSelection,
    Peer { name: String, color_index: usize, blink_visible: bool },
}

/// Computed visual style for a cell
#[derive(Clone)]
pub struct CellStyle {
    pub background: Color,
    pub text: Color,
    pub accent: Color,
}

/// Renders individual cells with proper separation of concerns
#[derive(Clone)]
pub struct CellRenderer {
    pub cell_height: u16,
}

impl CellRenderer {
    pub fn new() -> Self {
        Self { cell_height: 3 }
    }

    pub fn render(&self, data: &CellData, style: &CellStyle, width: u16) -> Cell<'static> {
        let content = self.build_content(data, width);
        let final_style = self.apply_progression_style(style, data.time_progression);
        
        Cell::from(content).style(Style::default().bg(final_style.background).fg(final_style.text))
    }

    fn build_content(&self, data: &CellData, width: u16) -> ratatui::text::Text<'static> {
        let play_marker = if data.is_playing { "▶" } else { " " };
        
        // Remove selection marker characters - background color already indicates selection
        let selection_marker = " ";
        
        let content_text = match &data.interaction {
            CellInteraction::Peer { name, .. } => {
                let peer_name = name.chars().take(3).collect::<String>();
                format!("{:<3}", peer_name)
            }
            _ => {
                let name = data.frame_name.clone().unwrap_or_default();
                if name.len() > 4 {
                    format!("{:.4}", name)
                } else {
                    format!("{:<4}", name)
                }
            }
        };
        
        // Format duration with repetitions if > 1
        let duration_text = if data.repetitions > 1 {
            format!("{:.1}x{}", data.frame_value, data.repetitions)
        } else {
            format!("{:.1}", data.frame_value)
        };
        
        // Build content for middle line
        let available_width = width.saturating_sub(2); // Account for margins
        let content_with_marker = format!("{}{}", selection_marker, content_text);
        let padding_width = available_width.saturating_sub(content_with_marker.len() as u16 + duration_text.len() as u16 + 1); // +1 for play marker
        let padding = " ".repeat(padding_width as usize);
        
        let middle_line = format!("{}{}{}{}", play_marker, content_with_marker, padding, duration_text);
        
        // Build time progression bar for bottom line
        let progress_line = self.build_progress_line(data.time_progression, width);
        
        ratatui::text::Text::from(vec![
            Line::from(" ".repeat(width as usize)), // Top line (empty)
            Line::from(middle_line),                // Middle line (content)
            progress_line,                          // Bottom line (time progression)
        ])
    }

    fn build_progress_line(&self, progression: Option<f32>, width: u16) -> Line<'static> {
        use ratatui::text::Span;
        use ratatui::style::{Color, Style};
        
        match progression {
            Some(progress) if progress >= 0.0 => {
                let progress = progress.clamp(0.0, 1.0);
                // Calculate playhead position across tile width
                let playhead_pos = (width as f32 * progress) as usize;
                let playhead_pos = playhead_pos.min(width as usize - 1);
                
                let mut spans = Vec::new();
                
                // Build the playhead visualization
                for i in 0..width as usize {
                    if i == playhead_pos && progress < 1.0 {
                        // Current playhead position - bright indicator
                        spans.push(Span::styled(
                            "█", // Full block for playhead
                            Style::default().fg(Color::Yellow).bg(Color::Red)
                        ));
                    } else if i < playhead_pos {
                        // Already played portion - filled
                        spans.push(Span::styled(
                            "▓", // Medium shade for played
                            Style::default().fg(Color::Green)
                        ));
                    } else {
                        // Not yet played portion - empty
                        spans.push(Span::styled(
                            "░", // Light shade for unplayed
                            Style::default().fg(Color::DarkGray)
                        ));
                    }
                }
                
                Line::from(spans)
            }
            _ => {
                // No progression - show completely empty bar
                Line::from(Span::styled(
                    "░".repeat(width as usize), // Light shade for entire width
                    Style::default().fg(Color::DarkGray)
                ))
            }
        }
    }

    fn build_progress_bar(&self, progression: Option<f32>, width: u16) -> String {
        match progression {
            Some(progress) if progress > 0.0 => {
                let progress = progress.clamp(0.0, 1.0);
                let filled_width = ((width as f32 * progress) as u16).min(width);
                let empty_width = width.saturating_sub(filled_width);
                
                // Use block characters for a clear progress bar
                let filled_char = "█"; // Full block for filled portion
                let empty_char = "▁";  // Bottom eighth block for empty portion
                
                format!("{}{}", 
                    filled_char.repeat(filled_width as usize),
                    empty_char.repeat(empty_width as usize)
                )
            }
            _ => {
                // No progression - show subtle empty bar
                "▁".repeat(width as usize)
            }
        }
    }

    fn apply_progression_style(&self, base_style: &CellStyle, progression: Option<f32>) -> CellStyle {
        match progression {
            Some(progress) if progress > 0.0 => {
                // Create gradient effect based on progression
                let gradient_color = self.create_gradient_color(base_style.background, progress);
                CellStyle {
                    background: gradient_color,
                    ..*base_style
                }
            }
            _ => base_style.clone(),
        }
    }

    fn create_gradient_color(&self, base_color: Color, progress: f32) -> Color {
        match base_color {
            Color::Rgb(r, g, b) => {
                // Create a gradient from base color to bright white/yellow
                let progress = progress.clamp(0.0, 1.0);
                
                // Target bright color (warm white/yellow)
                let target_r = 255;
                let target_g = 255;
                let target_b = 200; // Slightly warm
                
                // Interpolate between base and target
                let new_r = (r as f32 + (target_r as f32 - r as f32) * progress * 0.6) as u8;
                let new_g = (g as f32 + (target_g as f32 - g as f32) * progress * 0.6) as u8;
                let new_b = (b as f32 + (target_b as f32 - b as f32) * progress * 0.6) as u8;
                
                Color::Rgb(new_r, new_g, new_b)
            }
            _ => {
                // For non-RGB colors, use simple brightening
                self.brighten_color(base_color, 1.0 + progress * 0.5)
            }
        }
    }

    fn brighten_color(&self, color: Color, factor: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => Color::Rgb(
                ((r as f32 * factor) as u8).min(255),
                ((g as f32 * factor) as u8).min(255),
                ((b as f32 * factor) as u8).min(255),
            ),
            _ => color,
        }
    }
}