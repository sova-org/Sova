use std::{collections::HashMap, sync::Arc, usize};

use script::Script;
use serde::{Deserialize, Serialize};

use crate::{clock::{Clock, SyncTime}, lang::variable::VariableStore};

pub mod script;

fn default_speed_factor() -> f64 {
    return 1.0f64;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sequence {
    pub steps : Vec<f64>,  // Each step is defined by its length in beats
    pub enabled_steps : Vec<bool>,
    pub scripts : Vec<Arc<Script>>,
    #[serde(default = "default_speed_factor")]
    pub speed_factor : f64,
    #[serde(default)]
    pub vars : VariableStore,
    #[serde(default)]
    pub index : usize,
    /// Optional start step index (inclusive) for playback loop. Defaults to 0.
    #[serde(default)]
    pub start_step: Option<usize>,
    /// Optional end step index (inclusive) for playback loop. Defaults to last step.
    #[serde(default)]
    pub end_step: Option<usize>,
    #[serde(skip)]
    pub current_step : usize,
    #[serde(skip)]
    pub first_iteration_index : usize,
    #[serde(skip)]
    pub current_iteration : usize,
    #[serde(skip)]
    pub steps_executed : usize,
    #[serde(skip)]
    pub steps_passed : usize,
    #[serde(skip)]
    pub start_date : SyncTime
}

impl Sequence {

    pub fn new(steps : Vec<f64>) -> Self {
        let n_steps = steps.len();
        let scripts = (0..n_steps).map(|i| {
            let mut script = Script::default();
            script.index = i;
            Arc::new(script)
        }).collect();
        Sequence {
            steps,
            index: usize::MAX,
            enabled_steps: vec![true ; n_steps],
            vars: HashMap::new(),
            scripts,
            speed_factor: 1.0f64,
            current_step: 0,
            steps_executed: 0,
            steps_passed : 0,
            start_date : SyncTime::MAX,
            first_iteration_index: usize::MAX,
            current_iteration: usize::MAX,
            start_step: None,
            end_step: None,
        }
    }

    pub fn make_consistent(&mut self) {
        let n_steps = self.n_steps();

        if self.enabled_steps.len() != n_steps {
            self.enabled_steps.resize(n_steps, true);
        }
        while self.scripts.len() < n_steps {
            let mut script = Script::default();
            script.index = self.scripts.len();
            self.scripts.push(Arc::new(script));
            self.enabled_steps.push(true);
        }
        if self.scripts.len() > n_steps {
            self.scripts.drain(n_steps..);
            if self.enabled_steps.len() > n_steps {
                 self.enabled_steps.drain(n_steps..);
            }
        }
        for (i, script) in self.scripts.iter_mut().enumerate() {
            if script.index != i {
                let mut new_script = Script::clone(&script);
                new_script.index = i;
                *script = Arc::new(new_script);
            }
        }

        if let Some(start) = self.start_step {
            if start >= n_steps {
                self.start_step = None;
            }
        }
        if let Some(end) = self.end_step {
             if end >= n_steps {
                 self.end_step = if n_steps > 0 { Some(n_steps - 1) } else { None };
             }
        }
        if let (Some(start), Some(end)) = (self.start_step, self.end_step) {
            if start > end {
                self.start_step = None;
                self.end_step = None;
            }
        }
    }

    pub fn expected_end_date(&self, clock : &Clock) -> SyncTime {
        self.start_date + clock.beats_to_micros(self.beats_len())
    }

    #[inline]
    pub fn n_steps(&self) -> usize {
        self.steps.len()
    }

    #[inline]
    pub fn beats_len(&self) -> f64 {
        self.steps.iter().sum()
    }

    #[inline]
    pub fn steps_iter(&self) -> impl Iterator<Item = &f64> {
        self.steps.iter()
    }

    pub fn step_len(&self, index : usize) -> f64 {
        if self.steps.is_empty() {
            return f64::INFINITY;
        }
        let index = index % self.steps.len();
        self.steps[index]
    }

    #[inline]
    pub fn steps(&self) -> &[f64] {
        &self.steps
    }

    pub fn set_steps(&mut self, new_steps : Vec<f64>) {
        while self.scripts.len() < new_steps.len() {
            let mut script = Script::default();
            script.index = self.scripts.len();
            self.scripts.push(Arc::new(script));
            self.enabled_steps.push(true);
        }
        if self.steps.len() > new_steps.len() {
            self.scripts.drain(new_steps.len()..);
            self.enabled_steps.drain(new_steps.len()..);
        }
        self.steps = new_steps;
    }

    pub fn change_step(&mut self, index : usize, value : f64) {
        if self.steps.is_empty() {
            return;
        }
        let index = index % self.steps.len();
        self.steps[index] = value
    }

    pub fn set_script(&mut self, index : usize, mut script : Script) {
        if self.steps.is_empty() {
            return;
        }
        let index = index % self.steps.len();
        script.index = index;
        self.scripts[index] = Arc::new(script);
    }

    pub fn enable_step(&mut self, step : usize) {
        if self.steps.is_empty() {
            return;
        }
        let index = step % self.steps.len();
        self.enabled_steps[index] = true;
    }

    pub fn disable_step(&mut self, step : usize) {
        if self.steps.is_empty() {
            return;
        }
        let index = step % self.steps.len();
        self.enabled_steps[index] = false;
    }

    pub fn is_step_enabled(&self, index : usize) -> bool {
        if self.steps.is_empty() {
            return false;
        }
        let index = index % self.steps.len();
        self.enabled_steps[index]
    }

    /// Gets the effective start step index for playback (defaults to 0).
    pub fn get_effective_start_step(&self) -> usize {
        self.start_step.unwrap_or(0)
    }

    /// Gets the effective end step index (inclusive) for playback (defaults to n_steps - 1).
    pub fn get_effective_end_step(&self) -> usize {
        let n_steps = self.n_steps();
        self.end_step.unwrap_or(n_steps.saturating_sub(1))
    }

    /// Returns the number of steps in the effective playback range.
    pub fn get_effective_num_steps(&self) -> usize {
         if self.n_steps() == 0 {
             return 0;
         }
         let start = self.get_effective_start_step();
         let end = self.get_effective_end_step();
         end.saturating_sub(start) + 1
    }

    /// Returns a slice representing the steps within the effective playback range.
    pub fn get_effective_steps(&self) -> &[f64] {
        if self.n_steps() == 0 {
            return &[];
        }
        let start = self.get_effective_start_step();
        let end = self.get_effective_end_step();
        &self.steps[start..=end]
    }

    /// Calculates the total beat length of the steps within the effective playback range.
    pub fn effective_beats_len(&self) -> f64 {
         self.get_effective_steps().iter().sum()
    }

}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub sequences : Vec<Sequence>,
}

impl Pattern {

    pub fn new(mut sequences : Vec<Sequence>) -> Self {
        for (i,s) in sequences.iter_mut().enumerate() {
            s.index = i;
        }
        Pattern { sequences }
    }

    pub fn make_consistent(&mut self) {
        for (i,s) in self.sequences.iter_mut().enumerate() {
            s.index = i;
            s.make_consistent();
        }
    }

    #[inline]
    pub fn n_sequences(&self) -> usize {
        self.sequences.len()
    }

    pub fn sequences_iter(&self) -> impl Iterator<Item = &Sequence> {
        self.sequences.iter()
    }

    pub fn sequences_iter_mut(&mut self) -> impl Iterator<Item = &mut Sequence> {
        self.sequences.iter_mut()
    }

    pub fn sequences(&self) -> &[Sequence] {
        &self.sequences
    }

    pub fn mut_sequences(&mut self) -> &mut [Sequence] {
        &mut self.sequences
    }

    pub fn add_sequence(&mut self, mut seq : Sequence) {
        seq.index = self.n_sequences();
        seq.make_consistent();
        self.sequences.push(seq);
    }

    pub fn set_sequence(&mut self, index : usize, mut seq : Sequence) {
        if self.sequences.is_empty() {
            eprintln!("Warning: Attempted to set sequence with index {} in an empty Pattern. Ignoring.", index);
            return;
        }
        let index = index % self.sequences.len();
        seq.index = index;
        seq.make_consistent();
        self.sequences[index] = seq;
    }

    pub fn remove_sequence(&mut self, index : usize) {
        if self.sequences.is_empty() {
            eprintln!("Warning: Attempted to remove sequence with index {} from an empty Pattern. Ignoring.", index);
            return;
        }
        let index = index % self.sequences.len();
        self.sequences.remove(index);
        for (i,seq) in self.sequences[index..].iter_mut().enumerate() {
            seq.index = index + i;
        }
    }

    pub fn sequence(&self, index : usize) -> &Sequence {
        if self.sequences.is_empty() {
            panic!("Attempted to get sequence with index {} from an empty Pattern", index);
        }
        let index = index % self.sequences.len();
        &self.sequences[index]
    }

    pub fn mut_sequence(&mut self, index : usize) -> &mut Sequence {
        if self.sequences.is_empty() {
            panic!("Attempted to get mutable sequence with index {} from an empty Pattern", index);
        }
        let index = index % self.sequences.len();
        &mut self.sequences[index]
    }

    pub fn get_step_positions(&self) -> Vec<usize> {
        self.sequences_iter().map(|s| s.current_step).collect()
    }

}
