use std::sync::Mutex;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

mod shape;
pub use shape::*;

mod modifier;
pub use modifier::*;

mod state;
pub use state::*;

use crate::{clock::{SyncTime, TimeSpan}, vm::{EvaluationContext, variable::VariableValue}};

fn default_generator_rng() -> Mutex<ChaCha20Rng> {
    Mutex::new(ChaCha20Rng::from_os_rng())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueGenerator {
    pub shape: GeneratorShape,
    pub modifiers: Vec<GeneratorModifier>,
    pub span: TimeSpan,
    pub started: SyncTime,
    pub state_id: usize,
    #[serde(skip, default="default_generator_rng")]
    pub rng: Mutex<ChaCha20Rng>,
    seed: Option<u64>
}

impl Default for ValueGenerator {
    fn default() -> Self {
        Self { 
            shape: Default::default(), 
            modifiers: Default::default(), 
            span: Default::default(), 
            started: Default::default(), 
            state_id: Default::default(), 
            rng: default_generator_rng(), 
            seed: Default::default() 
        }
    }
}

impl Clone for ValueGenerator {
    fn clone(&self) -> Self {
        Self { 
            shape: self.shape.clone(), 
            modifiers: self.modifiers.clone(), 
            span: self.span.clone(), 
            started: self.started.clone(), 
            state_id: self.state_id.clone(), 
            rng: Mutex::new(ChaCha20Rng::clone(&self.rng.lock().unwrap())), 
            seed: self.seed.clone() 
        }
    }
}

impl PartialEq for ValueGenerator {
    fn eq(&self, other: &Self) -> bool {
        self.shape == other.shape 
        && self.modifiers == other.modifiers 
        && self.span == other.span 
        && self.started == other.started 
        && self.state_id == other.state_id 
        && self.seed == other.seed
    }
}

impl ValueGenerator {
    pub fn of_shape(shape: GeneratorShape) -> Self {
        ValueGenerator {
            shape, ..Default::default()
        }
    }

    pub fn start(&mut self, date: SyncTime) {
        self.started = date;
        if let Some(seed) = self.seed {
            self.rng = Mutex::new(ChaCha20Rng::seed_from_u64(seed));
        }
    }

    pub fn get_current(&self, ctx: &EvaluationContext) -> VariableValue {
        self.get(ctx, ctx.logic_date)
    }

    pub fn seed(&mut self, seed: u64) {
        self.seed = Some(seed);
    }

    pub fn get(&self, ctx: &EvaluationContext, date: SyncTime) -> VariableValue {
        let mut rng = self.rng.lock().unwrap();
        let span = self.span.as_beats(ctx.clock, ctx.frame_len);
        if span == 0.0 {
            return VariableValue::default();
        }
        let elapsed = ctx.clock.micros_to_beats(date.saturating_sub(self.started));
        let mut phase = elapsed / span;
        for modif in self.modifiers.iter() {
            phase = modif.get_phase(ctx, &mut rng, phase, span);
        }
        if phase >= 0.0 && phase <= 1.0 {
            self.shape.get_value(ctx, &mut rng, phase)
        } else {
            Default::default()
        }
    }

}