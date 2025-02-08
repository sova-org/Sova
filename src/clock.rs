use rusty_link::{
    AblLink,
    SessionState
};
use serde::{Deserialize, Serialize};

pub type SyncTime = u64;

/// Time duration: either absolute
/// or relative to musical tempo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeSpan {
    Micros(SyncTime),
    Beats(f64),
    Steps(f64)
}

impl TimeSpan {

    pub fn as_micros(&self, clock : &Clock) -> SyncTime {
        match self {
            TimeSpan::Micros(m) => *m,
            TimeSpan::Beats(b) => clock.beats_to_micros(*b),
            TimeSpan::Steps(_) => todo!(),
        }
    }

}

/// Ableton Link Server and Clock
pub struct Clock {
    pub link: AblLink,
    pub session_state: SessionState,
    pub quantum : f64
}

impl Clock {

    pub fn new(tempo: f64, quantum: f64) -> Self {
        return Clock {
            link: AblLink::new(tempo),
            session_state: SessionState::new(),
            quantum
        }
    }

    /// Capturer l'état de l'horloge
    pub fn capture_app_state(&mut self) {
        self.link.capture_app_session_state(&mut self.session_state);
    }

    /// Pousser un nouvel état
    pub fn commit_app_state(&self) {
        self.link.commit_app_session_state(&self.session_state);
    }

    /// Pousser la synchronisation
    pub fn set_start_stop_sync(&self) {
        let state = self.link.is_start_stop_sync_enabled();
        self.link.enable_start_stop_sync(!state);
        self.commit_app_state();
    }

    pub fn set_tempo(&mut self, tempo: f64) {
        let tempo = if tempo < 20.0 { 20.0 } else { tempo };
        let timestamp = self.link.clock_micros();
        self.session_state.set_tempo(tempo, timestamp);
        self.commit_app_state();
    }

    pub fn micros(&self) -> SyncTime {
        self.link.clock_micros() as SyncTime
    }

    pub fn date_at_relative_beats(&self, beats : f64) -> SyncTime {
        let beat = self.session_state.beat_at_time(self.link.clock_micros(), self.quantum) + beats;
        self.session_state.time_at_beat(beat, self.quantum) as SyncTime
    }

    pub fn beats_to_micros(&self, beats : f64) -> SyncTime {
        let tempo = self.session_state.tempo();
        let duration_s = beats * (60.0f64 / tempo);
        (duration_s.round() as SyncTime) * 1_000_000
    }

}
