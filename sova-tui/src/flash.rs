use std::time::Duration;

use tokio::time::Instant;

pub struct Flash {
    started: Option<Instant>,
    pub duration: Duration
}

impl Flash {

    pub fn new(duration: Duration) -> Self {
        Self { started: None, duration }
    }

    pub fn trigger(&mut self) {
        self.started = Some(Instant::now());
    }

    pub fn is_active(&self) -> bool {
        match &self.started {
            Some(instant) => {
                instant.elapsed() <= self.duration
            }
            None => false
        }
    }

}