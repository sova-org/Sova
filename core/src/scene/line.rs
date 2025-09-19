use crate::scene::{default_speed_factor, script::Script, Frame};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    clock::{Clock, SyncTime},
    lang::variable::VariableStore,
    log_eprintln, log_println,
    scene::script,
};

/// Represents a sequence of timed frames within a scene, each with associated scripts and properties.
///
/// A `Line` defines a linear progression of events, where each event (frame) has a duration
/// specified in musical beats. Lines can have their playback speed adjusted, contain variables,
/// and support looping, repetition of individual frames, and enabling/disabling specific frames.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Line {
    /// Frames of the line
    pub frames: Vec<Frame>,
    /// A multiplier applied to the duration of beats. `1.0` is normal speed, `< 1.0` is slower, `> 1.0` is faster.
    #[serde(default = "default_speed_factor")]
    pub speed_factor: f64,
    /// A store for variables specific to this line's execution context.
    #[serde(default)]
    pub vars: VariableStore,
    /// The index of this line within its parent container (e.g., a `Scene`). Should be managed externally.
    #[serde(default)]
    pub index: usize,
    /// If set, playback starts at this frame index (inclusive). Overrides the default start at index 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<usize>,
    /// If set, playback ends at this frame index (inclusive). Overrides the default end at the last frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_frame: Option<usize>,
    /// If set, defines a custom total loop duration in beats for this line, overriding the calculated sum of its frames.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_length: Option<f64>,
    // --- Runtime State (Not Serialized) ---
    /// The index of the currently active frame during playback.
    #[serde(skip)]
    pub current_frame: usize,
    /// The index where the first *actual* playback iteration started (considering `start_frame` and enabled frames).
    #[serde(skip)]
    pub first_iteration_index: usize,
    /// The current loop iteration number for the line.
    #[serde(skip)]
    pub current_iteration: usize,
    /// The current repetition count for the currently active frame (0-based). Resets when moving to a new frame.
    #[serde(skip)]
    pub current_repetition: usize,
    /// Total number of frames that have been *executed* (started) during playback, including repetitions.
    #[serde(skip)]
    pub frames_executed: usize,
    /// Total number of *unique* frames whose duration has fully elapsed during playback.
    #[serde(skip)]
    pub frames_passed: usize,
    /// The absolute time (`SyncTime`) when this line started its current playback loop.
    #[serde(skip)]
    pub start_date: SyncTime,
}

impl Line {
    /// Creates a new `Line` with the given frame durations.
    ///
    /// Initializes associated Frames
    /// to default values matching the number of frames provided.
    /// Runtime state fields are initialized to indicate no playback has started.
    pub fn new(frames_dur: Vec<f64>) -> Self {
        let mut line = Line {
            frames: frames_dur.into_iter().map(|d| d.into()).collect(),
            index: usize::MAX,
            vars: VariableStore::new(),
            speed_factor: 1.0f64,
            current_frame: 0,
            frames_executed: 0,
            frames_passed: 0,
            start_date: SyncTime::MAX,
            first_iteration_index: usize::MAX,
            current_iteration: usize::MAX,
            current_repetition: 0,
            start_frame: None,
            end_frame: None,
            custom_length: None,
        };
        line.make_consistent();
        line
    }

    /// Ensures the consistency of the line's internal state.
    ///
    /// This method synchronizes the lengths of `enabled_frames`, `scripts`, `frame_names`, and `frame_repetitions`
    /// with the length of the `frames` vector. It adds default values if vectors are too short, or truncates them
    /// if they are too long.
    ///
    /// It also ensures that `script.index` matches the frame index, clones scripts if necessary to update the index,
    /// guarantees `frame_repetitions` are at least 1, and validates `start_frame` and `end_frame` boundaries.
    ///
    /// This should be called after any operation that might change the number of frames or related vector lengths
    /// directly (e.g., deserialization, manual modification).
    pub fn make_consistent(&mut self) {
        let n_frames = self.n_frames();

        for (i,frame) in self.frames.iter_mut().enumerate() {
            if frame.repetitions == 0 {
                frame.repetitions = 1;
            }
            if frame.script.index != i || frame.script.line_index != self.index {
                let new_script = Arc::make_mut(&mut frame.script);
                new_script.index = i;
                new_script.line_index = self.index;
            }
        }

        if let Some(start) = self.start_frame {
            if start >= n_frames {
                self.start_frame = None;
            }
        }
        if let Some(end) = self.end_frame {
            if end >= n_frames {
                self.end_frame = if n_frames > 0 {
                    Some(n_frames - 1)
                } else {
                    None
                };
            }
        }
        if let (Some(start), Some(end)) = (self.start_frame, self.end_frame) {
            if start > end {
                self.start_frame = None;
                self.end_frame = None;
            }
        }
    }

    /// Calculates the expected absolute end time of the line's current playback cycle.
    ///
    /// This is based on the `start_date` and the total duration of *all* frames in beats
    /// (as returned by `beats_len`), converted to microseconds using the provided `clock`.
    /// Note: This does *not* account for `speed_factor`, `custom_length`, frame repetitions,
    /// or the effective start/end frames. It represents the theoretical end time if played
    /// sequentially from start to finish once at normal speed.
    pub fn expected_end_date(&self, clock: &Clock) -> SyncTime {
        self.start_date + clock.beats_to_micros(self.length())
    }

    pub fn length(&self) -> f64 {
        if let Some(len) = self.custom_length {
            return len;
        }
        let start = self.start_frame.unwrap_or(0);
        let end = self.start_frame.unwrap_or(self.n_frames() - 1);
        let mut len = 0.0;
        for i in start..=end {
            len += self.frame(i).effective_duration();
        }
        len
    }

    /// Returns the total number of frames in this line.
    #[inline]
    pub fn n_frames(&self) -> usize {
        self.frames.len()
    }

    /// Returns true if line has no frame
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn add_frame_if_empty(&mut self) {
        if self.is_empty() {
            self.frames.push(Frame::default());
        }
    }

    /// Returns the frame at given index. Handles overflow by rotating back to vector beginning.
    #[inline]
    pub fn frame(&self, index: usize) -> &Frame {
        &self.frames[index % self.n_frames()]
    }

    /// Returns the frame at given index. Handles overflow by rotating back to vector beginning.
    #[inline]
    pub fn frame_mut(&mut self, index: usize) -> &mut Frame {
        let index = index % self.n_frames();
        &mut self.frames[index]
    }

    /// Calculates the total duration of the line in beats by summing the durations of *all* frames.
    ///
    /// Note: This does *not* consider `enabled_frames`, `frame_repetitions`, `start_frame`, `end_frame`,
    /// or `custom_length`. Use `effective_beats_len` for the duration considering the playback range.
    ///
    /// Uses high-precision rational arithmetic to eliminate cumulative floating-point rounding errors.
    // #[inline]
    // pub fn beats_len(&self) -> f64 {
    //     use crate::util::decimal_operations::precise_sum;
    //     precise_sum(self.frames.iter().copied())
    // }

    /// Returns an iterator over the durations (`f64`) of all frames in the line.
    #[inline]
    pub fn frames_iter(&self) -> impl Iterator<Item = &Frame> {
        self.frames.iter()
    }

    /// Returns a mutable iterator over the durations (`f64`) of all frames in the line.
    #[inline]
    pub fn frames_iter_mut(&mut self) -> impl Iterator<Item = &mut Frame> {
        self.frames.iter_mut()
    }

    /// Returns an iterator over the scripts of all frames in the line.
    #[inline]
    pub fn scripts_iter(&self) -> impl Iterator<Item = &Script> {
        self.frames_iter().map(|f| &*f.script)
    }

    /// Returns a mutable iterator over the arc of scripts of all frames in the line.
    #[inline]
    pub fn scripts_iter_mut(&mut self) -> impl Iterator<Item = &mut Arc<Script>> {
        self.frames_iter_mut().map(|f| &mut f.script)
    }

    /// Returns a slice containing the durations of all frames.
    #[inline]
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    /// Replaces the entire set of frame durations with `new_frames`.
    ///
    /// This also adjusts the lengths of `enabled_frames`, `scripts`, `frame_names`, and `frame_repetitions`
    /// to match the new number of frames, potentially adding defaults or truncating.
    /// Finally, it calls `make_consistent` to ensure internal state integrity.
    pub fn set_frames(&mut self, new_frames: Vec<f64>) {
        let n_frames = self.n_frames();
        for (i, duration) in new_frames.into_iter().enumerate() {
            if i < n_frames {
                self.frame_mut(i).duration = duration;
            } else {
                self.frames.push(duration.into());
            }
        }
        self.make_consistent();
    }

    /// Inserts a new frame with the given duration (`value`) at the specified `position`.
    ///
    /// Shifts existing frames at and after `position` one step to the right.
    /// Inserts corresponding default values into `enabled_frames` (true), `scripts` (default script),
    /// `frame_names` (None), and `frame_repetitions` (1) at the same `position`.
    /// Calls `make_consistent` afterwards to update indices and ensure validity.
    ///
    /// # Panics
    /// Although the code prints an error, it does not panic if `position > self.frames.len()`.
    /// It simply returns early in that case. Insertion is allowed at `position == self.frames.len()`.
    pub fn insert_frame(&mut self, position: usize, value: Frame) {
        if position > self.frames.len() {
            // Allow inserting at the end (position == len)
            log_eprintln!("[!] Frame::insert_frame: Invalid position {}", position);
            return;
        }
        // Insert into frames
        self.frames.insert(position, value);
        // Ensure consistency (updates indices, bounds, etc.)
        self.make_consistent();
    }

    /// Removes the frame at the specified `position`.
    ///
    /// Shifts existing frames after `position` one step to the left.
    /// Also removes the corresponding elements from `enabled_frames`, `scripts`, `frame_names`,
    /// and `frame_repetitions`.
    /// Calls `make_consistent` afterwards to update indices and ensure validity.
    ///
    /// # Panics
    /// Although the code prints an error, it does not panic if `position >= self.frames.len()`.
    /// It simply returns early in that case.
    pub fn remove_frame(&mut self, position: usize) {
        if position >= self.frames.len() {
            log_eprintln!("[!] Frame::remove_frame: Invalid position {}", position);
            return;
        }

        // Remove from vectors
        log_println!(
            "[LINE DEBUG] remove_frame({}): BEFORE - frames={}",
            position,
            self.frames.len()
        );
        self.frames.remove(position);
        log_println!(
            "[LINE DEBUG] remove_frame({}): AFTER - frames={}",
            position,
            self.frames.len()
        );

        // Ensure consistency (updates indices, bounds, etc.)
        self.make_consistent();
    }

    /// Changes the duration of the frame at the specified `index` to `value`.
    ///
    /// Handles wrapping: if `index` is out of bounds, it wraps around using the modulo operator.
    /// Does nothing if the line has no frames.
    pub fn change_frame(&mut self, index: usize, value: f64) {
        if self.frames.is_empty() {
            return;
        }
        self.frame_mut(index).duration = value;
    }

    /// Associates the given `script` with the frame at the specified `index`.
    ///
    /// Takes ownership of the `script` and wraps it in an `Arc`.
    /// Sets the `script.index` field to the provided `index`.
    /// Handles wrapping: if `index` is out of bounds, it wraps around using the modulo operator.
    /// Does nothing if the line has no frames.
    pub fn set_script(&mut self, index: usize, mut script: script::Script) {
        if self.frames.is_empty() {
            return;
        }
        script.index = index;
        script.line_index = self.index;
        self.frame_mut(index).script = Arc::new(script);
    }

    /// Returns a reference to the frame's script at a given index.
    /// If the line contains no frame, then the function returns None.
    /// Handles wrapping: if `index` is out of bounds, it wraps around using the modulo operator.
    pub fn script(&self, index: usize) -> Option<&Script> {
        if self.frames.is_empty() {
            return None;
        }
        return Some(&self.frame(index).script)
    }

    /// Returns the frame absolute duration, without repetitions of enabledness
    pub fn frame_len(&self, index: usize) -> f64 {
        if self.frames.is_empty() {
            return f64::INFINITY;
        }
        self.frame(index).duration
    }

    /// Returns the effective duration of the frame
    pub fn effective_frame_len(&self, index: usize) -> f64 {
        if self.frames.is_empty() {
            return f64::INFINITY;
        }
        self.frame(index).effective_duration()
    }

    /// Enables the frame at the specified `frame` index for playback.
    ///
    /// Handles wrapping: if `frame` index is out of bounds, it wraps around using the modulo operator.
    /// Does nothing if the line has no frames.
    pub fn enable_frame(&mut self, frame: usize) {
        if self.frames.is_empty() {
            return;
        }
        self.frame_mut(frame).enabled = true;
    }

    /// Disables the frame at the specified `frame` index for playback.
    ///
    /// Disabled frames are skipped during iteration.
    /// Handles wrapping: if `frame` index is out of bounds, it wraps around using the modulo operator.
    /// Does nothing if the line has no frames.
    pub fn disable_frame(&mut self, frame: usize) {
        if self.frames.is_empty() {
            return;
        }
        self.frame_mut(frame).enabled = false;
    }

    /// Enables multiple frames specified by their indices in the `frames` slice.
    ///
    /// Handles wrapping for each index in the slice.
    /// Does nothing if the line has no frames. Skips indices that are out of bounds after wrapping
    /// (though wrapping ensures they map to a valid index if `n_frames > 0`).
    pub fn enable_frames(&mut self, frames: &[usize]) {
        if self.frames.is_empty() {
            return;
        }
        for &frame_index in frames {
            self.enable_frame(frame_index);
        }
    }

    /// Disables multiple frames specified by their indices in the `frames` slice.
    ///
    /// Handles wrapping for each index in the slice.
    /// Does nothing if the line has no frames. Skips indices that are out of bounds after wrapping
    /// (though wrapping ensures they map to a valid index if `n_frames > 0`).
    pub fn disable_frames(&mut self, frames: &[usize]) {
        if self.frames.is_empty() {
            return;
        }
        for &frame_index in frames {
            self.disable_frame(frame_index);
        }
    }

    /// Checks if the frame at the specified `index` is currently enabled.
    ///
    /// Handles wrapping: if `index` is out of bounds, it wraps around using the modulo operator.
    /// Returns `false` if the line has no frames.
    pub fn is_frame_enabled(&self, index: usize) -> bool {
        if self.frames.is_empty() {
            return false;
        }
        self.frame(index).enabled
    }

    /// Gets the effective start frame index for playback.
    /// Returns the value of `start_frame` if set, otherwise defaults to `0`.
    pub fn get_effective_start_frame(&self) -> usize {
        self.start_frame.unwrap_or(0)
    }

    /// Gets the effective end frame index (inclusive) for playback.
    /// Returns the value of `end_frame` if set, otherwise defaults to the index of the last frame
    /// (`n_frames - 1`). Returns `0` if `n_frames` is `0`. Uses `saturating_sub` for safety.
    pub fn get_effective_end_frame(&self) -> usize {
        let n_frames = self.n_frames();
        self.end_frame.unwrap_or(n_frames.saturating_sub(1))
    }

    /// Returns the number of frames within the effective playback range [`start_frame`, `end_frame`].
    /// Considers the values returned by `get_effective_start_frame` and `get_effective_end_frame`.
    /// Returns `0` if the line has no frames.
    pub fn get_effective_num_frames(&self) -> usize {
        if self.n_frames() == 0 {
            return 0;
        }
        let start = self.get_effective_start_frame();
        let end = self.get_effective_end_frame();
        end.saturating_sub(start) + 1
    }

    /// Returns a slice representing the frame durations within the effective playback range.
    /// Uses the indices determined by `get_effective_start_frame` and `get_effective_end_frame`.
    /// Returns an empty slice if the line has no frames.
    pub fn get_effective_frames(&self) -> &[Frame] {
        if self.n_frames() == 0 {
            return &[];
        }
        let start = self.get_effective_start_frame();
        let end = self.get_effective_end_frame();
        &self.frames[start..=end]
    }

    /// Calculates the total beat length of the frames within the effective playback range.
    /// Sums the durations of the frames returned by `get_effective_frames`.
    /// Does *not* account for `frame_repetitions` or `speed_factor`.
    /// Uses high-precision rational arithmetic to eliminate cumulative floating-point rounding errors.
    pub fn effective_beats_len(&self) -> f64 {
        use crate::util::decimal_operations::precise_sum;
        precise_sum(self.get_effective_frames().iter().map(|f| f.duration))
    }

    /// Sets the optional name for the frame at the specified `frame_index`.
    ///
    /// If `frame_index` is out of bounds, an error is printed to stderr and no change is made.
    pub fn set_frame_name(&mut self, frame_index: usize, name: Option<String>) {
        if !self.frames.is_empty() {
            self.frame_mut(frame_index).name = name;
        } else {
            log_eprintln!("[!] Line::set_frame_name: No frame in line");
        }
    }

    pub fn calculate_frame_index(
        &self,
        clock: &Clock,
        date: SyncTime,
    ) -> (usize, usize, usize, SyncTime, SyncTime) {
        let effective_loop_length_beats = self.length();

        if effective_loop_length_beats <= 0.0 {
            return (usize::MAX, usize::MAX, 0, SyncTime::MAX, SyncTime::MAX);
        }

        let current_absolute_beat = clock.beat_at_date(date);
        if current_absolute_beat < 0.0 {
            return (usize::MAX, usize::MAX, 0, SyncTime::MAX, SyncTime::MAX);
        }

        use crate::util::decimal_operations::precise_beat_modulo;

        let beat_in_effective_loop =
            precise_beat_modulo(current_absolute_beat, effective_loop_length_beats);
        let loop_iteration = (current_absolute_beat / effective_loop_length_beats).floor() as usize;

        let effective_start_frame = self.get_effective_start_frame();
        let effective_num_frames = self.get_effective_num_frames();

        if effective_num_frames == 0 {
            return (usize::MAX, loop_iteration, 0, SyncTime::MAX, SyncTime::MAX);
        }

        let mut cumulative_beats_in_line = 0.0;
        for frame_idx_in_range in 0..effective_num_frames {
            let absolute_frame_index = effective_start_frame + frame_idx_in_range;

            let speed_factor = if self.speed_factor == 0.0 {
                1.0
            } else {
                self.speed_factor
            };
            use crate::util::decimal_operations::precise_beat_division;
            let single_rep_len_beats =
                precise_beat_division(self.frame_len(absolute_frame_index), speed_factor);
            let total_repetitions = self.frame(absolute_frame_index).repetitions;

            let total_frame_len_beats = single_rep_len_beats * total_repetitions as f64;

            if single_rep_len_beats <= 0.0 {
                continue;
            }

            let frame_end_beat_in_line = cumulative_beats_in_line + total_frame_len_beats;

            if beat_in_effective_loop >= cumulative_beats_in_line
                && beat_in_effective_loop < frame_end_beat_in_line
            {
                let beat_within_frame = beat_in_effective_loop - cumulative_beats_in_line;
                let current_repetition_index =
                    (beat_within_frame / single_rep_len_beats).floor().max(0.0) as usize;
                let current_repetition_index = current_repetition_index.min(total_repetitions - 1);

                let absolute_beat_at_loop_start = loop_iteration as f64 * effective_loop_length_beats;
                let frame_first_rep_start_beat_absolute =
                    absolute_beat_at_loop_start + cumulative_beats_in_line;
                let current_rep_start_beat_absolute = frame_first_rep_start_beat_absolute
                    + (current_repetition_index as f64 * single_rep_len_beats);
                let current_repetition_start_date = clock.date_at_beat(current_rep_start_beat_absolute);

                let current_rep_end_beat_in_line = cumulative_beats_in_line
                    + (single_rep_len_beats * (current_repetition_index + 1) as f64);
                let remaining_beats_in_rep = current_rep_end_beat_in_line - beat_in_effective_loop;
                let remaining_micros_in_rep = clock.beats_to_micros(remaining_beats_in_rep);

                let remaining_beats_in_loop = effective_loop_length_beats - beat_in_effective_loop;
                let remaining_micros_in_loop = clock.beats_to_micros(remaining_beats_in_loop);

                let next_event_delay = remaining_micros_in_rep.min(remaining_micros_in_loop);

                return (
                    absolute_frame_index,
                    loop_iteration,
                    current_repetition_index,
                    current_repetition_start_date,
                    next_event_delay,
                );
            }

            cumulative_beats_in_line += total_frame_len_beats;
        }

        let remaining_beats_in_loop = effective_loop_length_beats - beat_in_effective_loop;
        let remaining_micros_in_loop = clock.beats_to_micros(remaining_beats_in_loop);
        (
            usize::MAX,
            loop_iteration,
            0,
            SyncTime::MAX,
            remaining_micros_in_loop,
        )
    }
}
