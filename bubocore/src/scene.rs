use std::{sync::Arc, usize};

use serde::{Deserialize, Serialize};

use crate::{clock::{Clock, SyncTime}, lang::variable::VariableStore};

pub mod script;

fn default_speed_factor() -> f64 {
    return 1.0f64;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Line {
    pub frames : Vec<f64>,  // Each frame is defined by its length in beats
    pub enabled_frames : Vec<bool>,
    pub scripts : Vec<Arc<script::Script>>,
    #[serde(default = "default_speed_factor")]
    pub speed_factor : f64,
    #[serde(default)]
    pub vars : VariableStore,
    #[serde(default)]
    pub index : usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_frame: Option<usize>,
    /// Optional custom loop length in beats for this line, overriding scene length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_length: Option<f64>,
    #[serde(skip)]
    pub current_frame : usize,
    #[serde(skip)]
    pub first_iteration_index : usize,
    #[serde(skip)]
    pub current_iteration : usize,
    #[serde(skip)]
    pub frames_executed : usize,
    #[serde(skip)]
    pub frames_passed : usize,
    #[serde(skip)]
    pub start_date : SyncTime
}

impl Line {

    pub fn new(frames : Vec<f64>) -> Self {
        let n_frames = frames.len();
        let scripts = (0..n_frames).map(|i| {
            let mut script = script::Script::default();
            script.index = i;
            Arc::new(script)
        }).collect();
        Line {
            frames,
            index: usize::MAX,
            enabled_frames: vec![true ; n_frames],
            vars: VariableStore::new(),
            scripts,
            speed_factor: 1.0f64,
            current_frame: 0,
            frames_executed: 0,
            frames_passed : 0,
            start_date : SyncTime::MAX,
            first_iteration_index: usize::MAX,
            current_iteration: usize::MAX,
            start_frame: None,
            end_frame: None,
            custom_length: None,
        }
    }

    pub fn make_consistent(&mut self) {
        let n_frames = self.n_frames();

        if self.enabled_frames.len() != n_frames {
            self.enabled_frames.resize(n_frames, true);
        }
        while self.scripts.len() < n_frames {
            let mut script = script::Script::default();
            script.index = self.scripts.len();
            self.scripts.push(Arc::new(script));
            self.enabled_frames.push(true);
        }
        if self.scripts.len() > n_frames {
            self.scripts.drain(n_frames..);
            if self.enabled_frames.len() > n_frames {
                 self.enabled_frames.drain(n_frames..);
            }
        }
        for (i, script_arc) in self.scripts.iter_mut().enumerate() {
            println!("[LINE DEBUG] make_consistent loop: i={}, script_arc.index={}", i, script_arc.index);
            if script_arc.index != i {
                let mut new_script = script::Script::clone(&script_arc);
                new_script.index = i;
                *script_arc = Arc::new(new_script);
            }
        }

        if let Some(start) = self.start_frame {
            if start >= n_frames {
                self.start_frame = None;
            }
        }
        if let Some(end) = self.end_frame {
             if end >= n_frames {
                 self.end_frame = if n_frames > 0 { Some(n_frames - 1) } else { None };
             }
        }
        if let (Some(start), Some(end)) = (self.start_frame, self.end_frame) {
            if start > end {
                self.start_frame = None;
                self.end_frame = None;
            }
        }
    }

    pub fn expected_end_date(&self, clock : &Clock) -> SyncTime {
        self.start_date + clock.beats_to_micros(self.beats_len())
    }

    #[inline]
    pub fn n_frames(&self) -> usize {
        self.frames.len()
    }

    #[inline]
    pub fn beats_len(&self) -> f64 {
        self.frames.iter().sum()
    }

    #[inline]
    pub fn frames_iter(&self) -> impl Iterator<Item = &f64> {
        self.frames.iter()
    }

    pub fn frame_len(&self, index : usize) -> f64 {
        if self.frames.is_empty() {
            return f64::INFINITY;
        }
        let index = index % self.frames.len();
        self.frames[index]
    }

    #[inline]
    pub fn frames(&self) -> &[f64] {
        &self.frames
    }

    pub fn set_frames(&mut self, new_frames : Vec<f64>) {
        let new_n_frames = new_frames.len();

        self.frames = new_frames;

        // Resize enabled_frames, adding 'true' if needed
        self.enabled_frames.resize(new_n_frames, true);

        // Resize scripts, adding default scripts if needed
        while self.scripts.len() < new_n_frames {
            let script = script::Script::default();
            // Index will be fixed by make_consistent later
            self.scripts.push(Arc::new(script));
        }
        // Truncate scripts if new_frames is shorter
        if self.scripts.len() > new_n_frames {
            self.scripts.drain(new_n_frames..);
        }

        // Now ensure everything is aligned and indices/bounds are correct
        self.make_consistent();
    }

    /// Inserts a new frame with the given value at the specified position.
    /// Adjusts `enabled_frames` and `scripts` accordingly.
    pub fn insert_frame(&mut self, position: usize, value: f64) {
        if position > self.frames.len() { // Allow inserting at the end (position == len)
            eprintln!("[!] Frame::insert_frame: Invalid position {}", position);
            return;
        }

        // Insert into frames
        self.frames.insert(position, value);

        // Insert default enabled state
        self.enabled_frames.insert(position, true);

        // Insert default script
        let default_script = script::Script::default();
        // Index will be fixed by make_consistent
        self.scripts.insert(position, Arc::new(default_script));

        // Ensure consistency (updates indices, bounds, etc.)
        self.make_consistent();
    }

    /// Removes the frame at the specified position.
    /// Adjusts `enabled_frames` and `scripts` accordingly.
    pub fn remove_frame(&mut self, position: usize) {
        if position >= self.frames.len() {
            eprintln!("[!] Frame::remove_frame: Invalid position {}", position);
            return;
        }

        // Remove from vectors
        println!("[LINE DEBUG] remove_frame({}): BEFORE - frames={}, enabled={}, scripts={}", position, self.frames.len(), self.enabled_frames.len(), self.scripts.len());
        self.frames.remove(position);
        self.enabled_frames.remove(position);
        self.scripts.remove(position);
        println!("[LINE DEBUG] remove_frame({}): AFTER - frames={}, enabled={}, scripts={}", position, self.frames.len(), self.enabled_frames.len(), self.scripts.len());

        // Ensure consistency (updates indices, bounds, etc.)
        self.make_consistent();
    }

    pub fn change_frame(&mut self, index : usize, value : f64) {
        if self.frames.is_empty() {
            return;
        }
        let index = index % self.frames.len();
        self.frames[index] = value
    }

    pub fn set_script(&mut self, index : usize, mut script : script::Script) {
        if self.frames.is_empty() {
            return;
        }
        let index = index % self.frames.len();
        script.index = index;
        self.scripts[index] = Arc::new(script);
    }

    pub fn enable_frame(&mut self, frame : usize) {
        if self.frames.is_empty() {
            return;
        }
        let index = frame % self.frames.len();
        self.enabled_frames[index] = true;
    }

    pub fn disable_frame(&mut self, frame : usize) {
        if self.frames.is_empty() {
            return;
        }
        let index = frame % self.frames.len();
        self.enabled_frames[index] = false;
    }

    pub fn enable_frames(&mut self, frames: &[usize]) {
         if self.frames.is_empty() {
             return;
         }
         let n_frames = self.frames.len();
         for &frame_index in frames {
             let index = frame_index % n_frames;
             if index < self.enabled_frames.len() {
                 self.enabled_frames[index] = true;
             }
         }
     }

    pub fn disable_frames(&mut self, frames: &[usize]) {
         if self.frames.is_empty() {
             return;
         }
         let n_frames = self.frames.len();
         for &frame_index in frames {
             let index = frame_index % n_frames;
             if index < self.enabled_frames.len() {
                 self.enabled_frames[index] = false;
             }
         }
     }

    pub fn is_frame_enabled(&self, index : usize) -> bool {
        if self.frames.is_empty() {
            return false;
        }
        let index = index % self.frames.len();
        self.enabled_frames[index]
    }

    /// Gets the effective start frame index for playback (defaults to 0).
    pub fn get_effective_start_frame(&self) -> usize {
        self.start_frame.unwrap_or(0)
    }

    /// Gets the effective end frame index (inclusive) for playback (defaults to n_frames - 1).
    pub fn get_effective_end_frame(&self) -> usize {
        let n_frames = self.n_frames();
        self.end_frame.unwrap_or(n_frames.saturating_sub(1))
    }

    /// Returns the number of frames in the effective playback range.
    pub fn get_effective_num_frames(&self) -> usize {
         if self.n_frames() == 0 {
             return 0;
         }
         let start = self.get_effective_start_frame();
         let end = self.get_effective_end_frame();
         end.saturating_sub(start) + 1
    }

    /// Returns a slice representing the frames within the effective playback range.
    pub fn get_effective_frames(&self) -> &[f64] {
        if self.n_frames() == 0 {
            return &[];
        }
        let start = self.get_effective_start_frame();
        let end = self.get_effective_end_frame();
        &self.frames[start..=end]
    }

    /// Calculates the total beat length of the frames within the effective playback range.
    pub fn effective_beats_len(&self) -> f64 {
         self.get_effective_frames().iter().sum()
    }

}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub length : usize,
    pub lines : Vec<Line>,
}

impl Scene {

    pub fn new(mut lines : Vec<Line>) -> Self {
        for (i,s) in lines.iter_mut().enumerate() {
            s.index = i;
        }
        Scene { lines, length: 4 }
    }

    pub fn make_consistent(&mut self) {
        for (i,s) in self.lines.iter_mut().enumerate() {
            s.index = i;
            s.make_consistent();
        }
    }

    pub fn set_length(&mut self, length : usize) {
        self.length = length;
    }

    pub fn length(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn n_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn lines_iter(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter()
    }

    pub fn lines_iter_mut(&mut self) -> impl Iterator<Item = &mut Line> {
        self.lines.iter_mut()
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn mut_lines(&mut self) -> &mut [Line] {
        &mut self.lines
    }

    pub fn add_line(&mut self, mut line : Line) {
        line.index = self.n_lines();
        line.make_consistent();
        self.lines.push(line);
    }

    pub fn set_line(&mut self, index : usize, mut line : Line) {
        if self.lines.is_empty() {
            eprintln!("Warning: Attempted to set line with index {} in an empty Scene. Ignoring.", index);
            return;
        }
        let index = index % self.lines.len();
        line.index = index;
        line.make_consistent();
        self.lines[index] = line;
    }

    pub fn remove_line(&mut self, index : usize) {
        if self.lines.is_empty() {
            eprintln!("Warning: Attempted to remove line with index {} from an empty Scene. Ignoring.", index);
            return;
        }
        let index = index % self.lines.len();
        self.lines.remove(index);
        for (i, line) in self.lines[index..].iter_mut().enumerate() {
            line.index = index + i;
        }
    }

    pub fn line(&self, index : usize) -> &Line {
        if self.lines.is_empty() {
            panic!("Attempted to get Line with index {} from an empty Scene", index);
        }
        let index = index % self.lines.len();
        &self.lines[index]
    }

    pub fn mut_line(&mut self, index : usize) -> &mut Line {
        if self.lines.is_empty() {
            panic!("Attempted to get mutable Line with index {} from an empty Scene", index);
        }
        let index = index % self.lines.len();
        &mut self.lines[index]
    }

    pub fn get_frames_positions(&self) -> Vec<usize> {
        self.lines_iter().map(|s| s.current_frame).collect()
    }

}