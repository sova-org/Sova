use std::{collections::HashMap, sync::Arc, usize};

use script::Script;

use crate::{clock::{Clock, SyncTime, TimeSpan}, lang::variable::VariableStore};

pub mod script;

#[derive(Debug)]
pub struct Sequence {
    steps : Vec<f64>,  // Each step is defined by its length in beats
    pub index : usize,
    pub enabled_steps : Vec<bool>,
    pub vars : VariableStore,
    pub scripts : Vec<Arc<Script>>,
    pub speed_factor : f64,
    pub current_step : usize,
    pub first_iteration_index : usize,
    pub current_iteration : usize,
    pub steps_executed : usize,
    pub steps_passed : usize,
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
        }
    }

    pub fn expected_end_date(&self, clock : &Clock) -> SyncTime {
        self.start_date + self.beats_len().as_micros(clock)
    }

    #[inline]
    pub fn n_steps(&self) -> usize {
        self.steps.len()
    }

    #[inline]
    pub fn beats_len(&self) -> TimeSpan {
        TimeSpan::Beats(self.steps.iter().sum())
    }

    #[inline]
    pub fn steps_iter(&self) -> impl Iterator<Item = &f64> {
        self.steps.iter()
    }

    #[inline]
    pub fn step_len(&self, index : usize) -> f64 {
        self.steps[index]
    }

    #[inline]
    pub fn steps(&self) -> &Vec<f64> {
        &self.steps
    }

    pub fn set_script(&mut self, index : usize, mut script : Script) {
        if self.steps.is_empty() {
            return;
        }
        let index = index % self.steps.len();
        script.index = index;
        self.scripts[index] = Arc::new(script);
    }

    pub fn toggle_step(&mut self, step : usize) {
        if self.steps.is_empty() {
            return;
        }
        let index = step % self.steps.len();
        self.enabled_steps[index] = !self.enabled_steps[index]
    }

    pub fn is_step_enabled(&self, index : usize) -> bool {
        let index = index % self.steps.len();
        self.enabled_steps[index]
    }

}

#[derive(Debug, Default)]
pub struct Pattern {
    pub sequences : Vec<Sequence>,
}

impl Pattern {

    pub fn sequence(&mut self, index : usize) -> &Sequence {
        let index = index % self.sequences.len();
        &self.sequences[index]
    }

    pub fn mut_sequence(&mut self, index : usize) -> &mut Sequence {
        let index = index % self.sequences.len();
        &mut self.sequences[index]
    }

}
