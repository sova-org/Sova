use crate::memory::MemoryPool;
use crate::modules::{AudioModule, Frame, GlobalEffect};
use crate::modules::global::echo::EchoEffect;
use crate::modules::global::reverb::ZitaReverb;
use crate::registry::ModuleRegistry;
use std::collections::HashMap;
use std::sync::Arc;

pub enum PooledEffect {
    Echo(Box<EchoEffect>),
    Reverb(Box<ZitaReverb>),
}

impl PooledEffect {
    pub fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        match self {
            PooledEffect::Echo(effect) => effect.process(buffer, sample_rate),
            PooledEffect::Reverb(effect) => effect.process(buffer, sample_rate),
        }
    }

    pub fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match self {
            PooledEffect::Echo(effect) => effect.set_parameter(param, value),
            PooledEffect::Reverb(effect) => effect.set_parameter(param, value),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            PooledEffect::Echo(effect) => effect.is_active(),
            PooledEffect::Reverb(effect) => effect.is_active(),
        }
    }
}

pub struct GlobalEffectPool {
    effects: HashMap<String, Vec<PooledEffect>>,
}

impl GlobalEffectPool {
    pub fn new(registry: &ModuleRegistry, _memory_pool: Arc<MemoryPool>, max_tracks: usize) -> Self {
        let mut effects = HashMap::new();

        for effect_name in registry.get_available_global_effects() {
            let mut track_effects = Vec::with_capacity(max_tracks);

            for _ in 0..max_tracks {
                match effect_name {
                    "echo" => {
                        track_effects.push(PooledEffect::Echo(Box::new(EchoEffect::new())));
                    },
                    "reverb" => {
                        track_effects.push(PooledEffect::Reverb(Box::new(ZitaReverb::new())));
                    },
                    _ => {} // Unknown effects are ignored
                }
            }

            effects.insert(effect_name.to_string(), track_effects);
        }

        Self { effects }
    }

    pub fn get_effect_mut(&mut self, effect_name: &str, track_id: usize) -> Option<&mut PooledEffect> {
        self.effects.get_mut(effect_name)
            .and_then(|track_effects| track_effects.get_mut(track_id))
    }

    pub fn get_available_effects(&self) -> Vec<&str> {
        self.effects.keys().map(|s| s.as_str()).collect()
    }
}