use crate::{clock::NEVER, lang::{evaluation_context::PartialContext, event::ConcreteEvent, interpreter::InterpreterDirectory}, scene::{Frame, script::Script}, util::decimal_operations::precise_division};

use serde::{Deserialize, Serialize};

use crate::{
    clock::{Clock, SyncTime},
    lang::variable::VariableStore,
    log_eprintln,
};

/// Default speed factor for lines if not specified.
/// Returns `1.0`. Used for serde default.
pub fn default_speed_factor() -> f64 {
    1.0f64
}

/// Represents a sequence of timed frames within a scene, each with associated scripts and properties.
///
/// A `Line` defines a linear progression of events, where each event (frame) has a duration
/// specified in musical beats. Lines can have their playback speed adjusted, contain variables,
/// and support looping, repetition of individual frames, and enabling/disabling specific frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    /// Frames of the line
    pub frames: Vec<Frame>,
    /// A multiplier applied to the duration of beats. `1.0` is normal speed, `< 1.0` is slower, `> 1.0` is faster.
    #[serde(default = "default_speed_factor")]
    pub speed_factor: f64,
    /// A store for variables specific to this line's execution context.
    #[serde(default, skip_serializing_if="VariableStore::is_empty")]
    pub vars: VariableStore,
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
    #[serde(skip)]
    pub last_trigger: SyncTime,
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
            ..Default::default()
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

        for frame in self.frames.iter_mut() {
            frame.make_consistent();
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

    pub fn reset(&mut self) {
        self.current_iteration = 0;
        self.current_frame = 0;
        self.current_repetition = 0;
        self.frames_passed = 0;
        self.frames_executed = 0;
        self.last_trigger = NEVER;
        self.start_date = 0;
    }

    pub fn configure(&mut self, other: &Line) {
        self.speed_factor = other.speed_factor;
        self.start_frame = other.start_frame;
        self.end_frame = other.end_frame;
        self.custom_length = other.custom_length;
    }

    /// Returns light version without frames
    pub fn configuration(&self) -> Line {
        let mut res = Line::default();
        res.configure(self);
        res
    }

    /// Calculates the expected absolute end time of the line's current playback cycle.
    ///
    /// This is based on the `start_date` and the total duration of *all* frames in beats
    /// (as returned by `beats_len`), converted to microseconds using the provided `clock`.
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
        for frame in self.frames[start..=end].iter() {
            len += frame.effective_duration();
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
    pub fn frame(&self, index: usize) -> Option<&Frame> {
        if index >= self.n_frames() {
            return None;
        }
        Some(&self.frames[index % self.n_frames()])
    }

    pub fn get_current_frame(&self) -> Option<&Frame> {
        self.frame(self.current_frame)
    }

    pub fn get_current_frame_mut(&mut self) -> Option<&mut Frame> {
        if self.current_frame >= self.n_frames() { 
            return None;
        }
        Some(self.frame_mut(self.current_frame))
    }

    /// Returns the frame at given index. Handles overflow by rotating back to vector beginning.
    pub fn frame_mut(&mut self, index: usize) -> &mut Frame {
        if index >= self.n_frames() {
            self.frames.resize(index + 1, Frame::default());
            self.make_consistent();
        }
        &mut self.frames[index]
    }

    pub fn set_frame(&mut self, index: usize, frame: Frame) {
        if index >= self.n_frames() {
            self.frames.resize(index + 1, Frame::default());
        }
        self.frames[index].change(frame);
        self.make_consistent();
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

    #[inline]
    pub fn structure(&self) -> Vec<f64> {
        self.frames.iter().map(|f| f.duration).collect()
    }

    /// Returns an iterator over the scripts of all frames in the line.
    #[inline]
    pub fn scripts_iter(&self) -> impl Iterator<Item = &Script> {
        self.frames.iter().map(|f| f.script())
    }

    /// Returns a slice containing the durations of all frames.
    #[inline]
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    #[inline]
    pub fn frames_mut(&mut self) -> &mut [Frame] {
        &mut self.frames
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
        self.frames.remove(position);
        // Ensure consistency (updates indices, bounds, etc.)
        self.make_consistent();
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
            return &self.frames;
        }
        let start = self.get_effective_start_frame();
        let end = self.get_effective_end_frame();
        &self.frames[start..=end]
    }

    /// Returns a slice representing the frame durations within the effective playback range.
    /// Uses the indices determined by `get_effective_start_frame` and `get_effective_end_frame`.
    /// Returns an empty slice if the line has no frames.
    pub fn get_effective_frames_mut(&mut self) -> &mut [Frame] {
        if self.n_frames() == 0 {
            return &mut self.frames;
        }
        let start = self.get_effective_start_frame();
        let end = self.get_effective_end_frame();
        &mut self.frames[start..=end]
    }

    /// Calculates the total beat length of the frames within the effective playback range.
    /// Sums the durations of the frames returned by `get_effective_frames`.
    /// Does *not* account for `frame_repetitions` or `speed_factor`.
    /// Uses high-precision rational arithmetic to eliminate cumulative floating-point rounding errors.
    pub fn effective_beats_len(&self) -> f64 {
        use crate::util::decimal_operations::precise_sum;
        precise_sum(self.get_effective_frames().iter().map(|f| f.duration))
    }

    pub fn kill_executions(&mut self) {
        self.frames.iter_mut().for_each(Frame::kill_executions);
    }

    pub fn update_executions<'a>(&'a mut self, date: SyncTime, mut partial: PartialContext<'a>) 
        -> (Vec<ConcreteEvent>, Option<SyncTime>)
    {
        partial.line_vars = Some(&mut self.vars);
        let mut events = Vec::new();
        let mut next_wait = Some(NEVER);
        for (index, frame) in self.frames.iter_mut().enumerate() {
            let mut partial_child = partial.child();
            partial_child.frame_index = Some(index);
            let (mut new_events, wait) = frame.update_executions(date, partial_child);
            events.append(&mut new_events);
            if let Some(wait) = wait {
                next_wait
                    .as_mut()
                    .map(|value| *value = std::cmp::min(*value, wait));
            }
        }
        (events, next_wait)
    }

    pub fn remaining_before_next_update(&self, date: SyncTime) -> SyncTime {
        self.frames
            .iter()
            .map(|frame| frame.remaining_before_next_update(date))
            .min()
            .unwrap_or(NEVER)
    }

    pub fn before_next_frame(&self, clock: &Clock, date: SyncTime) -> SyncTime {
        let frame = self.get_current_frame();
        if frame.is_none() || self.last_trigger == NEVER {
            return if self.is_empty() {
                NEVER
            } else {
                0
            };
        }
        let frame = frame.unwrap();
        let relative_date = date.saturating_sub(self.last_trigger);
        let frame_len = clock.beats_to_micros(
            precise_division(frame.duration, self.speed_factor)
        );
        frame_len.saturating_sub(relative_date)
    }

    pub fn step(&mut self, clock: &Clock, mut date: SyncTime, interpreters: &InterpreterDirectory) -> bool {
        if self.before_next_frame(clock, date) > 0 {
            return false;
        }
        if let Some(frame) = self.get_current_frame() {
            if self.last_trigger != NEVER {
                // Precise date correction if the exact time has been stepped over
                let frame_len = clock.beats_to_micros(
                    precise_division(frame.duration, self.speed_factor)
                );
                date = self.last_trigger + frame_len;

                if self.current_repetition < (frame.repetitions - 1) {
                    self.current_repetition += 1;
                } else {
                    self.current_frame += 1;
                    self.current_repetition = 0;
                    self.frames_passed += 1;
                    if self.current_frame > self.get_effective_end_frame() {
                        self.current_frame = self.get_effective_start_frame();
                        self.current_iteration += 1;
                    }
                }
            }
        } else {
            self.current_frame = self.get_effective_start_frame();
        }
        let frame = self.get_current_frame_mut().unwrap();
        frame.trigger(date, interpreters);
        self.frames_executed += 1;
        self.last_trigger = date;
        true
    }

    pub fn go_to_frame(&mut self, frame: usize, repetition: usize) {
        self.current_frame = frame;
        self.current_repetition = repetition;
        self.last_trigger = NEVER;
    }

    pub fn go_to_date(&mut self, clock: &Clock, date: SyncTime) {
        if self.is_empty() {
            return;
        }
        let len = self.length();
        let len = clock.beats_to_micros(len);
        let mut date = date % len;
        let mut frame_id = self.get_effective_start_frame();
        let mut repetition = 0;
        while frame_id <= self.get_effective_end_frame() {
            let frame = self.frame(frame_id).unwrap();
            let dur = clock.beats_to_micros(frame.duration);
            date = date.saturating_sub(dur);
            if date == 0 {
                self.go_to_frame(frame_id, repetition);
                return;
            }
            if repetition < (frame.repetitions - 1) {
                repetition += 1;
            } else {
                repetition = 0;
                frame_id += 1;
            }
        }
    }

    pub fn go_to_beat(&mut self, clock: &Clock, beat: f64) {
        self.go_to_date(clock, clock.beats_to_micros(beat));
    }

    pub fn position(&self) -> (usize, usize) {
        (self.current_frame, self.current_repetition)
    }

    pub fn calculate_frame_index(
        &self,
        clock: &Clock,
        date: SyncTime,
    ) -> (usize, usize, usize, SyncTime, SyncTime) {
        // TODO: FAIRE MIEUX
        let effective_loop_length_beats = self.length();

        if effective_loop_length_beats <= 0.0 {
            return (usize::MAX, usize::MAX, 0, SyncTime::MAX, SyncTime::MAX);
        }

        let current_absolute_beat = clock.beat_at_date(date);
        if current_absolute_beat < 0.0 {
            return (usize::MAX, usize::MAX, 0, SyncTime::MAX, SyncTime::MAX);
        }

        use crate::util::decimal_operations::precise_modulo;

        let beat_in_effective_loop =
            precise_modulo(current_absolute_beat, effective_loop_length_beats);
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
            let frame_len = self.frames[absolute_frame_index].duration;
            let single_rep_len_beats =
                precise_division(frame_len, speed_factor);
            let total_repetitions = self.frames[absolute_frame_index].repetitions;

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

impl Default for Line {
    fn default() -> Self {
        Line {
            frames: vec![Frame::default()],
            speed_factor: default_speed_factor(),
            vars: Default::default(),
            start_frame: Default::default(),
            end_frame: Default::default(),
            custom_length: Default::default(),
            current_frame: Default::default(),
            current_iteration: Default::default(),
            current_repetition: Default::default(),
            frames_executed: Default::default(),
            frames_passed: Default::default(),
            start_date: 0,
            last_trigger: NEVER,
        }
    }
}