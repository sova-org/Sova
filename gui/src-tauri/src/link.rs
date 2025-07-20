use rusty_link::{AblLink, SessionState};
use std::sync::Mutex;

pub struct LinkClock {
    link: AblLink,
    session_state: Mutex<SessionState>,
    quantum: Mutex<f64>,
}

impl LinkClock {
    pub fn new() -> Self {
        let link = AblLink::new(120.0);
        link.enable(true);
        let session_state = Mutex::new(SessionState::new());
        
        Self {
            link,
            session_state,
            quantum: Mutex::new(4.0),
        }
    }

    pub fn get_phase(&self) -> f64 {
        let mut session_state = self.session_state.lock().unwrap();
        self.link.capture_app_session_state(&mut session_state);
        
        let quantum = *self.quantum.lock().unwrap();
        let beat = session_state.beat_at_time(self.link.clock_micros(), quantum);
        beat % quantum
    }

    pub fn get_tempo(&self) -> f64 {
        let session_state = self.session_state.lock().unwrap();
        session_state.tempo()
    }

    pub fn set_tempo(&self, tempo: f64) {
        let mut session_state = self.session_state.lock().unwrap();
        let timestamp = self.link.clock_micros();
        session_state.set_tempo(tempo, timestamp);
        self.link.commit_app_session_state(&session_state);
    }

    pub fn set_quantum(&self, quantum: f64) {
        *self.quantum.lock().unwrap() = quantum;
    }

    pub fn get_quantum(&self) -> f64 {
        *self.quantum.lock().unwrap()
    }

    pub fn num_peers(&self) -> usize {
        self.link.num_peers() as usize
    }
}

unsafe impl Send for LinkClock {}
unsafe impl Sync for LinkClock {}