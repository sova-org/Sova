use std::sync::Arc;

use rusty_link::{AblLink, SessionState};
use serde::{Deserialize, Serialize};

/// Type alias for time measured in microseconds.
pub type SyncTime = u64;
pub const NEVER : SyncTime = SyncTime::MAX;

/// Represents a duration that can be measured in microseconds, beats, or frames.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeSpan {
    /// Duration in microseconds.
    Micros(SyncTime),
    /// Duration in musical beats, relative to the current tempo.
    Beats(f64),
    /// Duration in frames, relative to the current tempo and a given frame length in beats.
    Frames(f64),
}

impl TimeSpan {
    /// Converts the `TimeSpan` into microseconds based on the provided `Clock` context and frame length.
    ///
    /// # Arguments
    ///
    /// * `clock` - The `Clock` instance providing tempo context.
    /// * `frame_len` - The length of a frame in beats, used for `Frames` conversion.
    pub fn as_micros(&self, clock: &Clock, frame_len: f64) -> SyncTime {
        match self {
            TimeSpan::Micros(m) => *m,
            TimeSpan::Beats(b) => clock.beats_to_micros(*b),
            TimeSpan::Frames(s) => clock.beats_to_micros((*s) * frame_len),
        }
    }

    pub fn as_beats(&self, clock: &Clock, frame_len: f64) -> f64 {
        match self {
            TimeSpan::Micros(m) => clock.micros_to_beats(*m),
            TimeSpan::Beats(b) => *b,
            TimeSpan::Frames(s) => *s * frame_len,
        }
    }

    /// Converts the `TimeSpan` into microseconds based on the provided `Clock` context, assuming a frame length of 1.0 beat.
    /// This is useful when frame length context is not applicable or available.
    ///
    /// # Arguments
    ///
    /// * `clock` - The `Clock` instance providing tempo context.
    pub fn detached_micros(&self, clock: &Clock) -> SyncTime {
        self.as_micros(clock, 1.0)
    }

    /// Adds two `TimeSpan` values, converting them to a common unit based on the most specific type.
    ///
    /// The result type prioritizes `Frames`, then `Beats`, then `Micros`.
    ///
    /// # Arguments
    ///
    /// * `other` - The `TimeSpan` to add.
    /// * `clock` - The `Clock` instance providing tempo context.
    /// * `frame_len` - The length of a frame in beats.
    pub fn add(self, other: TimeSpan, clock: &Clock, frame_len: f64) -> TimeSpan {
        let in_micros = self.as_micros(clock, frame_len) + other.as_micros(clock, frame_len);

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => {
                TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len)
            }
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => {
                TimeSpan::Beats(clock.micros_to_beats(in_micros))
            }
            _ => TimeSpan::Micros(in_micros),
        }
    }

    /// Divides one `TimeSpan` by another, converting them to a common unit based on the most specific type.
    /// Returns zero if the divisor is zero.
    ///
    /// The result type prioritizes `Frames`, then `Beats`, then `Micros`.
    ///
    /// # Arguments
    ///
    /// * `other` - The `TimeSpan` divisor.
    /// * `clock` - The `Clock` instance providing tempo context.
    /// * `frame_len` - The length of a frame in beats.
    pub fn div(self, other: TimeSpan, clock: &Clock, frame_len: f64) -> TimeSpan {
        let other_micros = other.as_micros(clock, frame_len);
        let in_micros = if other_micros != 0 {
            self.as_micros(clock, frame_len) / other_micros
        } else {
            0 // Division by zero results in 0
        };

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => {
                TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len)
            }
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => {
                TimeSpan::Beats(clock.micros_to_beats(in_micros))
            }
            _ => TimeSpan::Micros(in_micros),
        }
    }

    /// Calculates the remainder of dividing one `TimeSpan` by another.
    /// Returns the original `TimeSpan` if the divisor is zero.
    ///
    /// The result type prioritizes `Frames`, then `Beats`, then `Micros`.
    ///
    /// # Arguments
    ///
    /// * `other` - The `TimeSpan` divisor.
    /// * `clock` - The `Clock` instance providing tempo context.
    /// * `frame_len` - The length of a frame in beats.
    pub fn rem(self, other: TimeSpan, clock: &Clock, frame_len: f64) -> TimeSpan {
        let other_micros = other.as_micros(clock, frame_len);
        let in_micros = if other_micros != 0 {
            self.as_micros(clock, frame_len) % other_micros
        } else {
            self.as_micros(clock, frame_len) // Remainder by zero returns original value
        };

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => {
                TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len)
            }
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => {
                TimeSpan::Beats(clock.micros_to_beats(in_micros))
            }
            _ => TimeSpan::Micros(in_micros),
        }
    }

    /// Multiplies two `TimeSpan` values, converting them to a common unit based on the most specific type.
    ///
    /// The result type prioritizes `Frames`, then `Beats`, then `Micros`.
    ///
    /// # Arguments
    ///
    /// * `other` - The `TimeSpan` to multiply by.
    /// * `clock` - The `Clock` instance providing tempo context.
    /// * `frame_len` - The length of a frame in beats.
    pub fn mul(self, other: TimeSpan, clock: &Clock, frame_len: f64) -> TimeSpan {
        let in_micros = self.as_micros(clock, frame_len) * other.as_micros(clock, frame_len);

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => {
                TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len)
            }
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => {
                TimeSpan::Beats(clock.micros_to_beats(in_micros))
            }
            _ => TimeSpan::Micros(in_micros),
        }
    }

    /// Subtracts one `TimeSpan` from another, converting them to a common unit based on the most specific type.
    ///
    /// The result type prioritizes `Frames`, then `Beats`, then `Micros`.
    ///
    /// # Arguments
    ///
    /// * `other` - The `TimeSpan` to subtract.
    /// * `clock` - The `Clock` instance providing tempo context.
    /// * `frame_len` - The length of a frame in beats.
    pub fn sub(self, other: TimeSpan, clock: &Clock, frame_len: f64) -> TimeSpan {
        let in_micros = self.as_micros(clock, frame_len) - other.as_micros(clock, frame_len);

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => {
                TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len)
            }
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => {
                TimeSpan::Beats(clock.micros_to_beats(in_micros))
            }
            _ => TimeSpan::Micros(in_micros),
        }
    }
}

/// Manages the Ableton Link instance and global clock properties.
///
/// This struct holds the core `AblLink` object and the musical quantum (beats per bar).
/// It is typically shared using an `Arc` to allow multiple `Clock` instances to reference it.
pub struct ClockServer {
    /// The underlying Ableton Link instance.
    pub link: AblLink,
    /// The musical quantum, defining the number of beats per bar or phrase.
    pub quantum: f64,
}

impl ClockServer {
    /// Creates a new `ClockServer` with a specified initial tempo and quantum.
    /// Enables start/stop synchronization by default.
    ///
    /// # Arguments
    ///
    /// * `tempo` - The initial tempo in beats per minute (BPM).
    /// * `quantum` - The musical quantum (e.g., 4.0 for 4/4 time).
    pub fn new(tempo: f64, quantum: f64) -> Self {
        let link = AblLink::new(tempo);
        link.enable_start_stop_sync(true);
        ClockServer { link, quantum }
    }

}

/// Represents a snapshot of the Ableton Link session state.
///
/// This struct holds a reference to the shared `ClockServer` and the current
/// `SessionState` captured from the Link instance. It provides methods for
/// interacting with the Link timeline based on this captured state.
pub struct Clock {
    /// A shared reference to the `ClockServer` containing the Link instance.
    pub server: Arc<ClockServer>,
    /// The captured session state from Ableton Link.
    pub session_state: SessionState,
    /// A micro-seconds drift
    pub drift: SyncTime
}

impl Clock {
    /// Captures the current application session state from the Ableton Link instance.
    /// This updates the `session_state` field with the latest timing information.
    pub fn capture_app_state(&mut self) {
        self.server
            .link
            .capture_app_session_state(&mut self.session_state);
    }

    /// Commits the current application session state back to the Ableton Link instance.
    /// This is necessary after modifying tempo or other properties in `session_state`.
    pub fn commit_app_state(&self) {
        self.server
            .link
            .commit_app_session_state(&self.session_state);
    }

    /// Toggles the start/stop synchronization feature in Ableton Link.
    /// Commits the state change immediately.
    pub fn set_start_stop_sync(&self) {
        let state = self.server.link.is_start_stop_sync_enabled();
        self.server.link.enable_start_stop_sync(!state);
        self.commit_app_state();
    }

    /// Sets a new tempo for the Ableton Link session.
    ///
    /// The tempo is clamped to a minimum of 20.0 BPM. The change is associated
    /// with the current Link time and committed immediately.
    ///
    /// # Arguments
    ///
    /// * `tempo` - The desired tempo in beats per minute (BPM).
    pub fn set_tempo(&mut self, tempo: f64) {
        let tempo = if tempo < 20.0 { 20.0 } else { tempo };
        let timestamp = self.server.link.clock_micros();
        self.session_state.set_tempo(tempo, timestamp);
        self.commit_app_state();
    }

    /// Returns the current Ableton Link clock time in microseconds.
    pub fn micros(&self) -> SyncTime {
        (self.server.link.clock_micros() as SyncTime) + self.drift
    }

    /// Returns the tempo (BPM) from the captured session state.
    pub fn tempo(&self) -> f64 {
        self.session_state.tempo()
    }

    /// Returns the musical quantum (beats per bar/phrase) from the server configuration.
    pub fn quantum(&self) -> f64 {
        self.server.quantum
    }

    /// Returns the current beat position on the timeline based on the current Link time and quantum.
    pub fn beat(&self) -> f64 {
        let date = self.server.link.clock_micros() + self.drift as i64;
        self.session_state.beat_at_time(date, self.quantum())
    }

    /// Calculates the absolute Link time (microseconds) corresponding to a specific beat position.
    ///
    /// # Arguments
    ///
    /// * `beat` - The target beat position on the timeline.
    pub fn date_at_beat(&self, beat: f64) -> SyncTime {
        self.session_state.time_at_beat(beat, self.server.quantum) as SyncTime
    }

    /// Calculates the absolute Link time (microseconds) corresponding to a beat position relative to the current time.
    ///
    /// # Arguments
    ///
    /// * `beats` - The number of beats relative to the current beat position.
    pub fn date_at_relative_beats(&self, beats: f64) -> SyncTime {
        let current_micros = self.server.link.clock_micros() + self.drift as i64;
        let current_beat = self
            .session_state
            .beat_at_time(current_micros, self.server.quantum);
        let target_beat = current_beat + beats;
        self.session_state
            .time_at_beat(target_beat, self.server.quantum) as SyncTime
    }

    /// Calculates the beat position corresponding to a specific absolute Link time (microseconds).
    ///
    /// # Arguments
    ///
    /// * `date` - The target absolute time in microseconds.
    pub fn beat_at_date(&self, date: SyncTime) -> f64 {
        self.session_state
            .beat_at_time(date as i64, self.server.quantum)
    }

    /// Calculates the beat position corresponding to a Link time relative to the current time.
    ///
    /// # Arguments
    ///
    /// * `date` - The time offset in microseconds relative to the current Link time.
    pub fn beat_at_relative_date(&self, date: SyncTime) -> f64 {
        let rel_date = self.server.link.clock_micros() 
            + date as i64 
            + self.drift as i64;
        self.session_state
            .beat_at_time(rel_date, self.server.quantum)
    }

    /// Converts a duration in beats to microseconds based on the current tempo.
    ///
    /// # Arguments
    ///
    /// * `beats` - The duration in beats.
    pub fn beats_to_micros(&self, beats: f64) -> SyncTime {
        let tempo = self.session_state.tempo();
        if tempo == 0.0 {
            return 0;
        }
        ((beats / tempo) * 60_000_000.0).round() as SyncTime
    }

    /// Converts a duration in microseconds to beats based on the current tempo.
    ///
    /// # Arguments
    ///
    /// * `micros` - The duration in microseconds.
    pub fn micros_to_beats(&self, micros: SyncTime) -> f64 {
        let tempo = self.session_state.tempo();
        if tempo == 0.0 {
            return 0.0;
        }
        (tempo * (micros as f64)) / 60_000_000.0
    }

    pub fn next_phase_reset_date(&self) -> SyncTime {
        let date = self.server.link.clock_micros();
        let quantum = self.quantum();
        let phase = self.session_state.phase_at_time(date, quantum);
        let remaining = quantum - phase;
        (date as SyncTime) + self.beats_to_micros(remaining)
    }

    pub fn next_phase_reset_beat(&self) -> f64 {
        self.beat() + self.quantum() - (self.beat() % self.quantum())
    }

    pub fn with_drift(mut self, drift: SyncTime) -> Clock {
        self.drift = drift;
        self
    }

}

/// Creates a `Clock` instance from a shared `ClockServer`.
/// Captures the initial application state upon creation.
impl From<Arc<ClockServer>> for Clock {
    fn from(server: Arc<ClockServer>) -> Self {
        let mut c = Clock {
            server,
            session_state: SessionState::new(),
            drift: 0
        };
        c.capture_app_state();
        c
    }
}

/// Creates a `Clock` instance from a reference to a shared `ClockServer`.
/// Clones the `Arc` and captures the initial application state upon creation.
impl From<&Arc<ClockServer>> for Clock {
    fn from(server: &Arc<ClockServer>) -> Self {
        Arc::clone(server).into()
    }
}
