use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MidiInMemory {
    //data: [[i8; 128]; 16]
    data: Vec<Vec<i8>>
}

/// Memory for Control Change messages
impl MidiInMemory {

    pub fn new() -> Self {
        let data = std::iter::repeat(
            std::iter::repeat(0).take(128).collect::<Vec<_>>()
        ).take(16).collect::<Vec<_>>();
        MidiInMemory {
            data
        }
    }

    /// Getter for a MIDI Controller CC value
    pub fn get(&self, channel: i8, control: i8) -> i8 {
        self.data[channel as usize][control as usize]
    }

    /// Setter for a MIDI Controller CC value
    pub fn set(&mut self, channel: i8, control: i8, value: i8) {
        self.data[channel as usize][control as usize] = value;
    }
}
