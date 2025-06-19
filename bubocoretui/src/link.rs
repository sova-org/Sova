use rusty_link::{AblLink, SessionState};

pub struct Link {
    pub link: AblLink,
    pub session_state: SessionState,
    pub quantum: f64,
}

impl Default for Link {
    fn default() -> Self {
        Self::new()
    }
}

impl Link {
    pub fn new() -> Self {
        let link = AblLink::new(120.0);
        let session_state = SessionState::new();
        let quantum = 4.0;
        Self {
            link,
            session_state,
            quantum,
        }
    }

    pub fn capture_app_state(&mut self) {
        self.link.capture_app_session_state(&mut self.session_state);
    }

    pub fn commit_app_state(&self) {
        self.link.commit_app_session_state(&self.session_state);
    }

    pub fn get_phase(&mut self) -> f64 {
        self.capture_app_state();
        let beat = self
            .session_state
            .beat_at_time(self.link.clock_micros(), self.quantum);
        beat % self.quantum
    }
}
