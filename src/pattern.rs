use script::Script;

pub mod script;

#[derive(Debug)]
pub struct Track {
    pub steps : Vec<Script>,
    pub speed_factor : f64
}

#[derive(Debug, Default)]
pub struct Pattern {
    pub tracks : Vec<Track>
}
