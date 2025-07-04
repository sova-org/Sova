use crate::memory::MemoryPool;
use crate::modules::GlobalEffect;
use crate::registry::ModuleRegistry;
use std::collections::HashMap;
use std::sync::Arc;

pub struct GlobalEffectPool {
    effects: HashMap<String, Vec<Box<dyn GlobalEffect>>>,
}

impl GlobalEffectPool {
    pub fn new(
        registry: &ModuleRegistry,
        _memory_pool: Arc<MemoryPool>,
        max_tracks: usize,
    ) -> Self {
        let mut effects = HashMap::new();

        for effect_name in registry.get_available_global_effects() {
            let mut track_effects = Vec::with_capacity(max_tracks);

            for _ in 0..max_tracks {
                if let Some(effect) = registry.create_global_effect(effect_name) {
                    track_effects.push(effect);
                }
            }

            if !track_effects.is_empty() {
                effects.insert(effect_name.to_string(), track_effects);
            }
        }

        Self { effects }
    }

    pub fn get_effect_mut(
        &mut self,
        effect_name: &str,
        track_id: usize,
    ) -> Option<&mut Box<dyn GlobalEffect>> {
        self.effects
            .get_mut(effect_name)
            .and_then(|track_effects| track_effects.get_mut(track_id))
    }

    pub fn get_available_effects(&self) -> Vec<&str> {
        self.effects.keys().map(|s| s.as_str()).collect()
    }
}
