use std::cmp;

use crate::{
    clock::NEVER, scene::{Frame, script::Script}, util::decimal_operations::precise_division, vm::{PartialContext, event::ConcreteEvent, interpreter::InterpreterDirectory}
};

use serde::{Deserialize, Serialize};

use crate::{
    clock::{Clock, SyncTime},
    vm::variable::VariableStore,
    log_eprintln,
};

/// Default speed factor for lines if not specified.
/// Returns `1.0`. Used for serde default.
pub fn default_speed_factor() -> f64 {
    1.0f64
}

#[derive(Debug, Clone)]
pub struct LineState {
    pub current_frame: usize,
    /// The current repetition count for the currently active frame (0-based). Resets when moving to a new frame.
    pub current_repetition: usize,
    pub last_trigger: SyncTime,
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
    #[serde(default, skip_serializing_if = "VariableStore::is_empty")]
    pub vars: VariableStore,
    /// If set, playback starts at this frame index (inclusive). Overrides the default start at index 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<usize>,
    /// If set, playback ends at this frame index (inclusive). Overrides the default end at the last frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_frame: Option<usize>,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub trailing: bool,

    // --- Runtime State (Not Serialized) ---
    /// The current loop iteration number for the line.
    #[serde(skip)]
    pub current_iteration: usize,
    #[serde(skip)]
    /// Total number of frames that have been *executed* (started) during playback, including repetitions.
    pub frames_executed: usize,
    /// Total number *unique* frames that have been *executed* (started) during playback.
    pub frames_passed: usize,
    #[serde(skip)]
    states: Vec<LineState>,
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
        self.frames_passed = 0;
        self.frames_executed = 0;
        self.vars.clear();
        self.states.clear();
    }

    pub fn configure(&mut self, other: &Line) {
        self.speed_factor = other.speed_factor;
        self.start_frame = other.start_frame;
        self.end_frame = other.end_frame;
        self.looping = other.looping;
        self.trailing = other.trailing;
    }

    /// Returns light version without frames
    pub fn configuration(&self) -> Line {
        let mut res = Line::default();
        res.configure(self);
        res
    }

    /// Returns the effective length in beats (counting only effective frames, and their repetitions)
    pub fn length(&self) -> f64 {
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

    /// Returns the frame at given index.
    pub fn frame(&self, index: usize) -> Option<&Frame> {
        self.frames.get(index)
    }

    pub fn get_current_frame(&self, state: &LineState) -> Option<&Frame> {
        self.frame(state.current_frame)
    }

    pub fn get_current_frame_mut(&mut self, state: &LineState) -> Option<&mut Frame> {
        if state.current_frame >= self.n_frames() {
            return None;
        }
        Some(self.frame_mut(state.current_frame))
    }

    /// Returns the frame at given index. Handles overflow by resizing frames to comply.
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
        for state in self.states.iter_mut() {
            if state.current_frame >= position {
                state.current_frame += 1;
            }
        }
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
        for state in self.states.iter_mut() {
            if state.current_frame > position {
                state.current_frame = state.current_frame.saturating_sub(1);
            }
        }
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

    pub fn kill_executions(&mut self) {
        self.frames.iter_mut().for_each(Frame::kill_executions);
    }

    pub fn update_executions<'a>(
        &'a mut self,
        mut partial: PartialContext<'a>,
    ) -> (Vec<ConcreteEvent>, SyncTime) {
        partial.line_vars = Some(&mut self.vars);
        let mut events = Vec::new();
        let mut next_wait = NEVER;
        for (index, frame) in self.frames.iter_mut().enumerate() {
            let mut partial_child = partial.child();
            partial_child.frame_index = Some(index);
            let (mut new_events, wait) = frame.update_executions(partial_child);
            events.append(&mut new_events);
            next_wait = std::cmp::min(next_wait, wait);
        }
        (events, next_wait)
    }

    pub fn before_next_update(&self, date: SyncTime) -> SyncTime {
        self.frames
            .iter()
            .map(|frame| frame.before_next_update(date))
            .min()
            .unwrap_or(NEVER)
    }

    fn before_next_state_trigger(frame: &Frame, state: &LineState, clock: &Clock, date: SyncTime, speed_factor: f64) -> SyncTime {
        if state.last_trigger == NEVER {
            return 0;
        }
        let relative_date = date.saturating_sub(state.last_trigger);
        let frame_len = clock.beats_to_micros(precise_division(frame.duration, speed_factor));
        frame_len.saturating_sub(relative_date)
    }

    pub fn before_next_trigger(&self, clock: &Clock, date: SyncTime) -> SyncTime {
        let mut next = NEVER;
        for state in self.states.iter() {
            let Some(frame) = self.get_current_frame(state) else {
                continue;
            };
            next = cmp::min(next, Self::before_next_state_trigger(frame, state, clock, date, self.speed_factor));
        }
        next
    }

    pub fn start(&mut self) {
        if !self.trailing {
            self.states.clear();
        }
        self.states.push(LineState { 
            current_frame: self.get_effective_start_frame(), 
            current_repetition: 0, 
            last_trigger: NEVER 
        });
        self.current_iteration += 1;
    }

    pub fn start_at(&mut self, frame_id: usize) {
        self.start();
        self.states.last_mut().unwrap().current_frame = frame_id;
    }

    pub fn step(
        &mut self,
        clock: &Clock,
        mut date: SyncTime,
        interpreters: &InterpreterDirectory,
    ) -> bool {
        let mut stepped = false;
        let start_frame = self.get_effective_start_frame();
        let end_frame = self.get_effective_end_frame();
        let frames = &mut self.frames;
        let n_states = self.states.len();
        for state in self.states.iter_mut() {
            let Some(frame) = frames.get(state.current_frame) else {
                continue;
            };
            if Self::before_next_state_trigger(frame, state, clock, date, self.speed_factor) > 0 {
                continue;
            }
            stepped = true;
            if state.last_trigger != NEVER {
                // Precise date correction if the exact time has been stepped over
                let frame_len = clock.beats_to_micros(frame.duration / self.speed_factor);
                date = state.last_trigger + frame_len;

                if state.current_repetition < (frame.repetitions - 1) {
                    state.current_repetition += 1;
                } else {
                    state.current_frame += 1;
                    state.current_repetition = 0;
                    self.frames_passed += 1;
                    if state.current_frame > end_frame {
                        if self.looping && n_states == 1 {
                            state.current_frame = start_frame;
                        } else {
                            state.current_frame = usize::MAX;
                            continue;
                        }
                    }
                }
            }
            let frame = frames.get_mut(state.current_frame).unwrap();
            frame.trigger(date, interpreters);
            self.frames_executed += 1;
            state.last_trigger = date;
        }
        let n_frames = self.n_frames();
        self.states.retain(|state| state.current_frame < n_frames);
        stepped
    }

    pub fn go_to_frame(&mut self, frame: usize, repetition: usize) {
        self.states.push(LineState { 
            current_frame: frame, 
            current_repetition: repetition, 
            last_trigger: NEVER 
        });
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

    pub fn position(&self) -> Vec<(usize, usize)> {
        self.states.iter().map(|s| (s.current_frame, s.current_repetition)).collect()
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
            current_iteration: Default::default(),
            states: Default::default(),
            frames_executed: Default::default(),
            frames_passed: Default::default(),
            looping: false,
            trailing: false
        }
    }
}
