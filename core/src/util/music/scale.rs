use serde::{Deserialize, Serialize};

use crate::util::music::tuning::{NOTE_C, note_freq};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub tonic: f64,
    pub deviation: f64,
    pub octave_factor: f64,
    pub intervals: Vec<usize>
}

impl Scale {

    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    pub fn divisions(&self) -> usize {
        self.intervals.iter().sum()
    }

    pub fn increment(&self) -> f64 {
        f64::powf(self.octave_factor, 1.0 / self.divisions() as f64) + self.deviation
    }

    pub fn note(&self, index: i64) -> f64 {
        let len = self.len() as i64;
        let octave_delta = (index / len) as f64;
        let mut index = index % len;
        let mut delta : i64 = 0;
        while index > 0 {
            delta += self.intervals[index as usize] as i64;
            index -= 1;
        }
        while index < 0 {
            delta -= self.intervals[(len - index) as usize] as i64;
            index += 1;
        }
        let divs = self.divisions() as f64;
        self.tonic
            + octave_delta * (self.octave_factor + divs * self.deviation)
            + (delta as f64) * self.increment()
    }

    pub fn major(note: i16, octave: i16) -> Scale {
        Scale {
            tonic: note_freq(note, octave),
            deviation: 0.0,
            octave_factor: 2.0,
            intervals: vec![2,2,1,2,2,2,1],
        }
    }
    
    pub fn minor(note: i16, octave: i16) -> Scale {
        Scale {
            tonic: note_freq(note, octave),
            deviation: 0.0,
            octave_factor: 2.0,
            intervals: vec![2,1,2,2,1,2,2],
        }
    }

    fn chromatic(note: i16, octave: i16) -> Self {
        Scale { 
            tonic: note_freq(note, octave), 
            deviation: 0.0, 
            octave_factor: 2.0, 
            intervals: vec![1;12] 
        }
    }

}

impl Default for Scale {
    fn default() -> Self {
        Self::chromatic(NOTE_C, 4)
    }
}
