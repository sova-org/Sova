use std::time::Instant;

/// Tracks actual time progression within frames
#[derive(Clone, Debug)]
pub struct TimeSystem {
    pub frame_start_times: Vec<Option<Instant>>, // Per frame start time
    pub frame_durations: Vec<f64>,               // Per frame duration in seconds
}

impl TimeSystem {
    pub fn new(frame_count: usize) -> Self {
        Self {
            frame_start_times: vec![None; frame_count],
            frame_durations: vec![1.0; frame_count], // Default 1 second per frame
        }
    }

    /// Get time progression for a frame (0.0 to 1.0)
    pub fn get_progression(&self, frame_index: usize) -> Option<f32> {
        if frame_index >= self.frame_start_times.len() {
            return None;
        }

        let start_time = self.frame_start_times[frame_index]?;
        let duration = self
            .frame_durations
            .get(frame_index)
            .copied()
            .unwrap_or(1.0);

        let elapsed = start_time.elapsed().as_secs_f64();
        let progression = (elapsed / duration).clamp(0.0, 1.0);

        Some(progression as f32)
    }
}
