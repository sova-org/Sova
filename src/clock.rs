use rusty_link::{
    AblLink,
    SessionState
};

pub type SyncTime = u64;

/// Time duration: either absolute
/// or relative to musical tempo
pub enum TimeSpan {
    Micros(SyncTime),
    Beats(u64)
}

/// Ableton Link Server and Clock
pub struct Clock {
    pub link: AblLink,
    pub session_state: SessionState,
}

impl Clock {

    pub fn new(tempo: f64, quantum: f64) -> Self {
        return Clock {
            link: AblLink::new(tempo),
            session_state: SessionState::new()
        }
    }

    /// Capturer l'état de l'horloge
    pub fn capture_app_state(&mut self) {
        self.link.capture_app_session_state(&mut self.session_state);
    }

    /// Pousser un nouvel état
    pub fn commit_app_state(&mut self) {
        self.link.commit_app_session_state(&self.session_state);
    }

    /// Pousser la synchronisation
    pub fn set_start_stop_sync(&mut self) {
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

}
