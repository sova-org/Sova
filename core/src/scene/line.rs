use crate::scene::{default_speed_factor, script::Script};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    clock::{Clock, SyncTime},
    lang::variable::VariableStore,
    scene::script,
    log_println, log_eprintln,
};

/// Represents a sequence of timed frames within a scene, each with associated scripts and properties.
///
/// A `Line` defines a linear progression of events, where each event (frame) has a duration
/// specified in musical beats. Lines can have their playback speed adjusted, contain variables,
/// and support looping, repetition of individual frames, and enabling/disabling specific frames.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Line {
    /// The duration of each frame in beats.
    /// The core data defining the timing of the line.
    pub frames: Vec<f64>,
    /// Tracks whether each corresponding frame in `frames` is currently active for playback.
    /// Disabled frames are skipped during iteration. Must have the same length as `frames`.
    pub enabled_frames: Vec<bool>,
    /// Scripts associated with each frame. Executed when the frame becomes active.
    /// Stored in `Arc` for potentially shared ownership or cheaper cloning. Must have the same length as `frames`.
    pub scripts: Vec<Arc<Script>>,
    /// Optional user-defined names for each frame. Useful for identification in UIs or debugging.
    /// Must have the same length as `frames`. Defaults to `None` for each frame.
    #[serde(default)]
    pub frame_names: Vec<Option<String>>,
    /// Specifies how many times each frame should repeat consecutively before moving to the next.
    /// A value of `1` means the frame plays once. Must have the same length as `frames`. Defaults to `1`.
    #[serde(default)]
    pub frame_repetitions: Vec<usize>,
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
    /// Initializes associated vectors (`enabled_frames`, `scripts`, `frame_names`, `frame_repetitions`)
    /// to default values matching the number of frames provided.
    /// Runtime state fields are initialized to indicate no playback has started.
    pub fn new(frames: Vec<f64>) -> Self {
        let n_frames = frames.len();
        let scripts = (0..n_frames)
            .map(|i| {
                let mut script = script::Script::default();
                script.index = i;
                script.line_index = usize::MAX;
                Arc::new(script)
            })
            .collect();
        Line {
            frames,
            index: usize::MAX,
            enabled_frames: vec![true; n_frames],
            vars: VariableStore::new(),
            scripts,
            frame_names: vec![None; n_frames],
            frame_repetitions: vec![1; n_frames],
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
        }
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

        if self.frame_names.len() != n_frames {
            self.frame_names.resize(n_frames, None);
        }
        if self.enabled_frames.len() != n_frames {
            self.enabled_frames.resize(n_frames, true);
        }
        if self.frame_repetitions.len() != n_frames {
            self.frame_repetitions.resize(n_frames, 1);
        }
        while self.scripts.len() < n_frames {
            let mut script = script::Script::default();
            script.index = self.scripts.len();
            script.line_index = self.index;
            self.scripts.push(Arc::new(script));
            self.enabled_frames.push(true);
            self.frame_repetitions.push(1);
        }
        if self.scripts.len() > n_frames {
            self.scripts.drain(n_frames..);
            if self.enabled_frames.len() > n_frames {
                self.enabled_frames.drain(n_frames..);
            }
            if self.frame_names.len() > n_frames {
                self.frame_names.drain(n_frames..);
            }
            if self.frame_repetitions.len() > n_frames {
                self.frame_repetitions.drain(n_frames..);
            }
        }
        for (i, script_arc) in self.scripts.iter_mut().enumerate() {
            if script_arc.index != i {
                let new_script = Arc::make_mut(script_arc);
                new_script.index = i;
                new_script.line_index = self.index;
            }
        }

        // Ensure frame_repetitions contains valid values (at least 1)
        for reps in self.frame_repetitions.iter_mut() {
            if *reps == 0 {
                *reps = 1;
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
        self.start_date + clock.beats_to_micros(self.beats_len())
    }

    /// Returns the total number of frames in this line.
    #[inline]
    pub fn n_frames(&self) -> usize {
        self.frames.len()
    }

    /// Calculates the total duration of the line in beats by summing the durations of *all* frames.
    ///
    /// Note: This does *not* consider `enabled_frames`, `frame_repetitions`, `start_frame`, `end_frame`,
    /// or `custom_length`. Use `effective_beats_len` for the duration considering the playback range.
    ///
    /// Uses high-precision rational arithmetic to eliminate cumulative floating-point rounding errors.
    #[inline]
    pub fn beats_len(&self) -> f64 {
        use crate::util::decimal_operations::precise_sum;
        precise_sum(self.frames.iter().copied())
    }

    /// Returns an iterator over the durations (`f64`) of all frames in the line.
    #[inline]
    pub fn frames_iter(&self) -> impl Iterator<Item = &f64> {
        self.frames.iter()
    }

    /// Returns a mutable iterator over the durations (`f64`) of all frames in the line.
    #[inline]
    pub fn frames_iter_mut(&mut self) -> impl Iterator<Item = &mut f64> {
        self.frames.iter_mut()
    }

    /// Returns an iterator over the scripts of all frames in the line.
    #[inline]
    pub fn scripts_iter(&self) -> impl Iterator<Item = &Script> {
        self.scripts.iter().map(|s| &**s)
    }

    /// Returns a mutable iterator over the arc of scripts of all frames in the line.
    #[inline]
    pub fn scripts_iter_mut(&mut self) -> impl Iterator<Item = &mut Arc<Script>> {
        self.scripts.iter_mut()
    }

    /// Returns the duration in beats of the frame at the specified index.
    ///
    /// Handles wrapping: if `index` is out of bounds, it wraps around using the modulo operator.
    /// Returns `f64::INFINITY` if the line has no frames.
    pub fn frame_len(&self, index: usize) -> f64 {
        if self.frames.is_empty() {
            return f64::INFINITY;
        }
        let index = index % self.frames.len();
        self.frames[index]
    }

    /// Returns a slice containing the durations of all frames.
    #[inline]
    pub fn frames(&self) -> &[f64] {
        &self.frames
    }

    /// Replaces the entire set of frame durations with `new_frames`.
    ///
    /// This also adjusts the lengths of `enabled_frames`, `scripts`, `frame_names`, and `frame_repetitions`
    /// to match the new number of frames, potentially adding defaults or truncating.
    /// Finally, it calls `make_consistent` to ensure internal state integrity.
    pub fn set_frames(&mut self, new_frames: Vec<f64>) {
        let new_n_frames = new_frames.len();

        self.frames = new_frames;

        if self.frame_names.len() != new_n_frames {
            self.frame_names.resize(new_n_frames, None);
        }
        if self.enabled_frames.len() != new_n_frames {
            self.enabled_frames.resize(new_n_frames, true);
        }
        if self.frame_repetitions.len() != new_n_frames {
            self.frame_repetitions.resize(new_n_frames, 1);
        }

        while self.scripts.len() < new_n_frames {
            let script = script::Script::default();
            self.scripts.push(Arc::new(script));
        }
        if self.scripts.len() > new_n_frames {
            self.scripts.drain(new_n_frames..);
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
    pub fn insert_frame(&mut self, position: usize, value: f64) {
        if position > self.frames.len() {
            // Allow inserting at the end (position == len)
            log_eprintln!("[!] Frame::insert_frame: Invalid position {}", position);
            return;
        }

        // Insert into frames
        self.frames.insert(position, value);

        // Insert default enabled state
        self.enabled_frames.insert(position, true);

        // Insert default script
        let default_script = script::Script::default();
        // Index will be fixed by make_consistent
        self.scripts.insert(position, Arc::new(default_script));

        // Insert default name (None)
        self.frame_names.insert(position, None);

        // Insert default repetitions (1)
        self.frame_repetitions.insert(position, 1);

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
            "[LINE DEBUG] remove_frame({}): BEFORE - frames={}, enabled={}, scripts={}",
            position,
            self.frames.len(),
            self.enabled_frames.len(),
            self.scripts.len()
        );
        self.frames.remove(position);
        self.enabled_frames.remove(position);
        self.scripts.remove(position);
        self.frame_names.remove(position);
        self.frame_repetitions.remove(position);
        log_println!(
            "[LINE DEBUG] remove_frame({}): AFTER - frames={}, enabled={}, scripts={}",
            position,
            self.frames.len(),
            self.enabled_frames.len(),
            self.scripts.len()
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
        let index = index % self.frames.len();
        self.frames[index] = value
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
        let index = index % self.frames.len();
        script.index = index;
        script.line_index = self.index;
        self.scripts[index] = Arc::new(script);
    }

    /// Enables the frame at the specified `frame` index for playback.
    ///
    /// Handles wrapping: if `frame` index is out of bounds, it wraps around using the modulo operator.
    /// Does nothing if the line has no frames.
    pub fn enable_frame(&mut self, frame: usize) {
        if self.frames.is_empty() {
            return;
        }
        let index = frame % self.frames.len();
        self.enabled_frames[index] = true;
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
        let index = frame % self.frames.len();
        self.enabled_frames[index] = false;
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
        let n_frames = self.frames.len();
        for &frame_index in frames {
            let index = frame_index % n_frames;
            if index < self.enabled_frames.len() {
                self.enabled_frames[index] = true;
            }
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
        let n_frames = self.frames.len();
        for &frame_index in frames {
            let index = frame_index % n_frames;
            if index < self.enabled_frames.len() {
                self.enabled_frames[index] = false;
            }
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
        let index = index % self.frames.len();
        self.enabled_frames[index]
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
    pub fn get_effective_frames(&self) -> &[f64] {
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
        precise_sum(self.get_effective_frames().iter().copied())
    }

    /// Sets the optional name for the frame at the specified `frame_index`.
    ///
    /// If `frame_index` is out of bounds, an error is printed to stderr and no change is made.
    pub fn set_frame_name(&mut self, frame_index: usize, name: Option<String>) {
        if frame_index < self.frame_names.len() {
            self.frame_names[frame_index] = name;
        } else {
            log_eprintln!("[!] Line::set_frame_name: Invalid index {}", frame_index);
        }
    }
}
