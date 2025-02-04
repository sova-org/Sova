use crate::lang::Program;

pub struct Script {
    pub content : String,
    pub compiled : Option<Program>
}

impl Script {
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

pub struct Track {
    pub steps : Vec<Script>,
    pub speed_factor : f64
}

pub struct Pattern {
    pub tracks : Vec<Track>
}
