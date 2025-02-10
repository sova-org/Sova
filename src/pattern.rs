use std::sync::Arc;

use script::Script;

pub mod script;

#[derive(Debug, Clone)]
pub struct Track {
    pub steps : Vec<f64>,  // Each step is defined by its length in beats
    pub scripts : Vec<Arc<Script>>,
    pub speed_factor : f64
}

#[derive(Debug, Default)]
pub struct Pattern {
    pub tracks : Vec<Track>,
    pub track_index : usize
}

impl Pattern {

    pub fn current_track(&self) -> Option<&Track> {
        self.tracks.get(self.track_index)
    }

    pub fn current_track_mut(&mut self) -> Option<&mut Track> {
        self.tracks.get_mut(self.track_index)
    }

}
