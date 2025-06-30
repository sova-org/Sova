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
    Peer {
        name: String,
        color_index: usize,
        blink_visible: bool,
    },
}

/// Computed visual style for a cell
#[derive(Clone)]
pub struct CellStyle {
    pub background: Color,
    pub text: Color,
    pub accent: Color,
}

/// Cache for expensive grid progression calculations
#[derive(Clone, Debug)]
pub struct GridProgressionCache {
    scene_hash: u64,
    frame_data: Vec<Vec<FrameCacheEntry>>, // [line_idx][frame_idx]
    last_update: std::time::Instant,
}

#[derive(Clone, Debug)]
pub struct FrameCacheEntry {
    start_beat: f64,
    total_beats: f64,
    single_rep_beats: f64,
    repetitions: usize,
}

impl GridProgressionCache {
    pub fn new() -> Self {
        Self {
            scene_hash: 0,
            frame_data: Vec::new(),
            last_update: std::time::Instant::now(),
        }
    }

    pub fn is_valid(&self, scene: &corelib::scene::Scene) -> bool {
        let current_hash = self.calculate_scene_hash(scene);
        current_hash == self.scene_hash && self.last_update.elapsed().as_millis() < 16 // Invalidate after 16ms (60 FPS)
    }

    pub fn update(&mut self, scene: &corelib::scene::Scene) {
        self.scene_hash = self.calculate_scene_hash(scene);
        self.frame_data.clear();

        for line in &scene.lines {
            let mut line_frames = Vec::new();
            let mut cumulative_beats = 0.0;
            let speed_factor = if line.speed_factor == 0.0 {
                1.0
            } else {
                line.speed_factor
            };

            for (frame_idx, &frame_value) in line.frames.iter().enumerate() {
                let single_rep_beats = frame_value / speed_factor;
                let repetitions = line
                    .frame_repetitions
                    .get(frame_idx)
                    .copied()
                    .unwrap_or(1)
                    .max(1);
                let total_frame_beats = single_rep_beats * repetitions as f64;

                line_frames.push(FrameCacheEntry {
                    start_beat: cumulative_beats,
                    total_beats: total_frame_beats,
                    single_rep_beats,
                    repetitions,
                });

                cumulative_beats += total_frame_beats;
            }

            self.frame_data.push(line_frames);
        }

        self.last_update = std::time::Instant::now();
    }

    pub fn get_progression(
        &self,
        line_idx: usize,
        frame_idx: usize,
        scene: &corelib::scene::Scene,
        current_beat: f64,
    ) -> Option<f32> {
        let line = scene.lines.get(line_idx)?;
        let frame_cache = self.frame_data.get(line_idx)?.get(frame_idx)?;

        let effective_loop_length_beats = line.custom_length.unwrap_or(scene.length() as f64);
        if effective_loop_length_beats <= 0.0 {
            return None;
        }

        let beat_in_loop = if current_beat >= 0.0 {
            current_beat % effective_loop_length_beats
        } else {
            0.0
        };

        let frame_start_beat = frame_cache.start_beat;
        let frame_end_beat = frame_start_beat + frame_cache.total_beats;

        if beat_in_loop >= frame_start_beat && beat_in_loop < frame_end_beat {
            let beat_within_frame = beat_in_loop - frame_start_beat;
            let current_rep = (beat_within_frame / frame_cache.single_rep_beats)
                .floor()
                .max(0.0) as usize;
            let current_rep = current_rep.min(frame_cache.repetitions - 1);

            let rep_start_beat = current_rep as f64 * frame_cache.single_rep_beats;
            let beat_within_rep = beat_within_frame - rep_start_beat;
            let rep_progress = if frame_cache.single_rep_beats > 0.0 {
                beat_within_rep / frame_cache.single_rep_beats
            } else {
                0.0
            };

            Some(rep_progress.clamp(0.0, 1.0) as f32)
        } else {
            None
        }
    }

    fn calculate_scene_hash(&self, scene: &corelib::scene::Scene) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        scene.length().hash(&mut hasher);

        for line in &scene.lines {
            line.frames.len().hash(&mut hasher);
            line.custom_length.is_some().hash(&mut hasher);
            if let Some(length) = line.custom_length {
                length.to_bits().hash(&mut hasher);
            }
            line.speed_factor.to_bits().hash(&mut hasher);
            for &frame in &line.frames {
                frame.to_bits().hash(&mut hasher);
            }
            for &rep in &line.frame_repetitions {
                rep.hash(&mut hasher);
            }
        }

        hasher.finish()
    }
}

/// Optimized string formatting utilities to reduce allocations
pub struct CellStringCache;

impl CellStringCache {
    /// Efficiently format duration text
    pub fn format_duration(frame_value: f64, repetitions: usize) -> String {
        if repetitions > 1 {
            format!("{:.1} * {}", frame_value, repetitions)
        } else {
            format!("{:.1}", frame_value)
        }
    }

    /// Get pre-allocated progress characters
    pub fn get_progress_char(char_type: usize) -> &'static str {
        match char_type {
            0 => "█", // Full block
            1 => "▓", // Medium shade
            2 => "░", // Light shade
            _ => " ", // Space
        }
    }

    /// Efficiently create spaces without allocation for small widths
    pub fn create_spaces(width: usize) -> String {
        if width <= 20 {
            // Use a pre-allocated constant for small widths
            "                    "[..width].to_string()
        } else {
            " ".repeat(width)
        }
    }
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

        Cell::from(content).style(
            Style::default()
                .bg(final_style.background)
                .fg(final_style.text),
        )
    }

    fn build_content(&self, data: &CellData, width: u16) -> ratatui::text::Text<'static> {
        use ratatui::style::{Color, Style};
        use ratatui::text::Span;

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

        // Format duration with repetitions if > 1 - use optimized formatter
        let duration_text = CellStringCache::format_duration(data.frame_value, data.repetitions);

        // Build content for middle line with spans for white background on duration
        let available_width = width.saturating_sub(2); // Account for margins
        let content_with_marker = format!("{}{}", selection_marker, content_text);
        let padding_width = available_width
            .saturating_sub(content_with_marker.len() as u16 + duration_text.len() as u16 + 1); // +1 for play marker
        let padding = CellStringCache::create_spaces(padding_width as usize);

        // Create middle line with styled spans
        let middle_line = Line::from(vec![
            Span::raw(format!("{}{}{}", play_marker, content_with_marker, padding)),
            Span::styled(
                duration_text,
                Style::default().bg(Color::White).fg(Color::Black),
            ),
        ]);

        // Build time progression bar for bottom line
        let progress_line = self.build_progress_line(data.time_progression, width);

        ratatui::text::Text::from(vec![
            Line::from(CellStringCache::create_spaces(width as usize)), // Top line (empty)
            middle_line,   // Middle line (content with styled duration)
            progress_line, // Bottom line (time progression)
        ])
    }

    fn build_progress_line(&self, progression: Option<f32>, width: u16) -> Line<'static> {
        use ratatui::style::{Color, Style};
        use ratatui::text::Span;

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
                            CellStringCache::get_progress_char(0), // Full block for playhead
                            Style::default().fg(Color::Yellow).bg(Color::Red),
                        ));
                    } else if i < playhead_pos {
                        // Already played portion - filled
                        spans.push(Span::styled(
                            CellStringCache::get_progress_char(1), // Medium shade for played
                            Style::default().fg(Color::Green),
                        ));
                    } else {
                        // Not yet played portion - empty
                        spans.push(Span::styled(
                            CellStringCache::get_progress_char(2), // Light shade for unplayed
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                }

                Line::from(spans)
            }
            _ => {
                // No progression - show completely empty bar
                let light_shade_char = CellStringCache::get_progress_char(2);
                Line::from(Span::styled(
                    light_shade_char.repeat(width as usize), // Light shade for entire width
                    Style::default().fg(Color::DarkGray),
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
                let filled_char = CellStringCache::get_progress_char(0); // Full block for filled portion
                let empty_char = "▁"; // Bottom eighth block for empty portion

                format!(
                    "{}{}",
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

    fn apply_progression_style(
        &self,
        base_style: &CellStyle,
        progression: Option<f32>,
    ) -> CellStyle {
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
