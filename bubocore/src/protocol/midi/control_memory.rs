#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiInMemory {
    data: [[i8; 128]; 16]
}

/// Memory for Control Change messages
impl MidiInMemory {

    pub fn new() -> Self {
        MidiInMemory {
            data: [[0; 128]; 16]
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
