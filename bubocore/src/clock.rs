use std::sync::Arc;

use rusty_link::{AblLink, SessionState};
use serde::{Deserialize, Serialize};

pub type SyncTime = u64;

/// Time duration: either absolute
/// or relative to musical tempo
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeSpan {
    Micros(SyncTime),
    Beats(f64),
    Frames(f64),
}

impl TimeSpan {
    pub fn as_micros(&self, clock: &Clock, frame_len : f64) -> SyncTime {
        match self {
            TimeSpan::Micros(m) => *m,
            TimeSpan::Beats(b) => clock.beats_to_micros(*b),
            TimeSpan::Frames(s) => clock.beats_to_micros((*s) * frame_len),
        }
    }

    pub fn detached_micros(&self, clock: &Clock) -> SyncTime {
        self.as_micros(clock, 1.0)
    }

    pub fn add(self, other: TimeSpan, clock: &Clock, frame_len : f64) -> TimeSpan {

        let in_micros = self.as_micros(clock, frame_len) + other.as_micros(clock, frame_len);

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len),
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => TimeSpan::Beats(clock.micros_to_beats(in_micros)),
            _ => TimeSpan::Micros(in_micros),
        }
    }

    pub fn div(self, other: TimeSpan, clock: &Clock, frame_len : f64) -> TimeSpan {

        let other_micros = other.as_micros(clock, frame_len);
        let in_micros = if other_micros != 0 {
            self.as_micros(clock, frame_len) / other_micros
        } else {
            0
        };

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len),
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => TimeSpan::Beats(clock.micros_to_beats(in_micros)),
            _ => TimeSpan::Micros(in_micros),
        }
    }

    pub fn rem(self, other: TimeSpan, clock: &Clock, frame_len : f64) -> TimeSpan {

        let other_micros = other.as_micros(clock, frame_len);
        let in_micros = if other_micros != 0 {
            self.as_micros(clock, frame_len) % other_micros
        } else {
            self.as_micros(clock, frame_len)
        };

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len),
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => TimeSpan::Beats(clock.micros_to_beats(in_micros)),
            _ => TimeSpan::Micros(in_micros),
        }
    }

    pub fn mul(self, other: TimeSpan, clock: &Clock, frame_len : f64) -> TimeSpan {

        let in_micros = self.as_micros(clock, frame_len) * other.as_micros(clock, frame_len);

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len),
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => TimeSpan::Beats(clock.micros_to_beats(in_micros)),
            _ => TimeSpan::Micros(in_micros),
        }
    }

    pub fn sub(self, other: TimeSpan, clock: &Clock, frame_len : f64) -> TimeSpan {

        let in_micros = self.as_micros(clock, frame_len) - other.as_micros(clock, frame_len);

        match (self, other) {
            (TimeSpan::Frames(_), _) | (_, TimeSpan::Frames(_)) => TimeSpan::Frames(clock.micros_to_beats(in_micros) / frame_len),
            (TimeSpan::Beats(_), _) | (_, TimeSpan::Beats(_)) => TimeSpan::Beats(clock.micros_to_beats(in_micros)),
            _ => TimeSpan::Micros(in_micros),
        }
    }
}

/// Ableton Link Server and Clock
pub struct ClockServer {
    pub link: AblLink,
    pub quantum: f64,
}

impl ClockServer {
    pub fn new(tempo: f64, quantum: f64) -> Self {
        let link = AblLink::new(tempo);
        link.enable_start_stop_sync(true);
        ClockServer {
            link,
            quantum,
        }
    }
}

pub struct Clock {
    pub server: Arc<ClockServer>,
    pub session_state: SessionState,
}

impl Clock {
    /// Capturer l'état de l'horloge
    pub fn capture_app_state(&mut self) {
        self.server
            .link
            .capture_app_session_state(&mut self.session_state);
    }

    /// Pousser un nouvel état
    pub fn commit_app_state(&self) {
        self.server
            .link
            .commit_app_session_state(&self.session_state);
    }

    /// Pousser la synchronisation
    pub fn set_start_stop_sync(&self) {
        let state = self.server.link.is_start_stop_sync_enabled();
        self.server.link.enable_start_stop_sync(!state);
        self.commit_app_state();
    }

    pub fn set_tempo(&mut self, tempo: f64) {
        let tempo = if tempo < 20.0 { 20.0 } else { tempo };
        let timestamp = self.server.link.clock_micros();
        self.session_state.set_tempo(tempo, timestamp);
        self.commit_app_state();
    }

    pub fn micros(&self) -> SyncTime {
        self.server.link.clock_micros() as SyncTime
    }

    pub fn tempo(&self) -> f64 {
        self.session_state.tempo()
    }

    pub fn quantum(&self) -> f64 {
        self.server.quantum
    }

    pub fn beat(&self) -> f64 {
        let date = self.server.link.clock_micros();
        self.session_state.beat_at_time(date, self.quantum())
    }

    pub fn date_at_beat(&self, beat: f64) -> SyncTime {
        self.session_state.time_at_beat(beat, self.server.quantum) as SyncTime
    }

    pub fn date_at_relative_beats(&self, beats: f64) -> SyncTime {
        let beat = self
            .session_state
            .beat_at_time(self.server.link.clock_micros(), self.server.quantum)
            + beats;
        self.session_state.time_at_beat(beat, self.server.quantum) as SyncTime
    }

    pub fn beat_at_date(&self, date: SyncTime) -> f64 {
        self.session_state
            .beat_at_time(date as i64, self.server.quantum)
    }

    pub fn beat_at_relative_date(&self, date: SyncTime) -> f64 {
        let rel_date = self.server.link.clock_micros() + date as i64;
        self.session_state
            .beat_at_time(rel_date, self.server.quantum)
    }

    pub fn beats_to_micros(&self, beats: f64) -> SyncTime {
        let tempo = self.session_state.tempo();
        let duration_s = beats * (60.0f64 / tempo);
        (duration_s * 1_000_000.0).round() as SyncTime
    }

    pub fn micros_to_beats(&self, micros: SyncTime) -> f64 {
        let tempo = self.session_state.tempo();
        let beat_duration = (60.0f64 / tempo) * 1_000_000.0;
        (micros as f64) / beat_duration
    }

}

impl From<Arc<ClockServer>> for Clock {
    fn from(server: Arc<ClockServer>) -> Self {
        let mut c = Clock {
            server,
            session_state: SessionState::new(),
        };
        c.capture_app_state();
        c
    }
}

impl From<&Arc<ClockServer>> for Clock {
    fn from(server: &Arc<ClockServer>) -> Self {
        Arc::clone(server).into()
    }
}
