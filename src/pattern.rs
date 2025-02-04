use crate::lang::{variable::VariableStore, Program};

#[derive(Debug, Default)]
pub struct Script {
    pub content : String,
    pub compiled : Option<Program>,
    pub persistents : VariableStore,
}

impl Script {
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

#[derive(Debug)]
pub struct Track {
    pub steps : Vec<Script>,
    pub speed_factor : f64
}

#[derive(Debug, Default)]
pub struct Pattern {
    pub tracks : Vec<Track>
}
