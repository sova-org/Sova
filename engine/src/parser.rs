use crate::memory::MemoryPool;
use crate::modulation::Modulation;
use crate::registry::{ENGINE_PARAM_DESCRIPTORS, ModuleRegistry};
use crate::types::{EngineMessage, VoiceId};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

static mut ROUND_ROBIN_VOICE: VoiceId = 0;

pub fn parse_command(
    command: &str,
    max_voices: usize,
    registry: &ModuleRegistry,
) -> Option<EngineMessage> {
    parse_command_with_pool(
        command,
        max_voices,
        registry,
        &Arc::new(MemoryPool::new(64 * 1024)),
    )
}

pub fn parse_command_with_pool(
    command: &str,
    max_voices: usize,
    registry: &ModuleRegistry,
    pool: &Arc<MemoryPool>,
) -> Option<EngineMessage> {
    let mut parts = [""; 64];
    let mut part_count = 0;

    for part in command.split_whitespace() {
        if part_count < 64 {
            parts[part_count] = part;
            part_count += 1;
        } else {
            break;
        }
    }

    if part_count < 2 || parts[0] != "/play" {
        return None;
    }

    let voice_id = if parts[1] == "s" {
        unsafe {
            let id = ROUND_ROBIN_VOICE;
            ROUND_ROBIN_VOICE = (ROUND_ROBIN_VOICE + 1) % (max_voices as VoiceId);
            id
        }
    } else {
        parts[1].parse().ok()?
    };

    let mut track_id = 1;
    let mut param_start = 2;

    if part_count > 2 && parts[2] != "s" && !parts[2].chars().all(|c| c.is_alphabetic()) {
        if let Ok(tid) = parts[2].parse() {
            track_id = tid;
            param_start = 3;
        }
    }

    let mut parameters: HashMap<String, Box<dyn Any + Send>> = HashMap::with_capacity(16);
    let mut source_name = None;

    let mut i = param_start;
    while i + 1 < part_count {
        if parts[i] == "s" {
            let available_sources = registry.get_available_sources();
            let resolved_source = if available_sources.contains(&parts[i + 1]) {
                parts[i + 1].to_string()
            } else if available_sources.iter().any(|s| s.contains(parts[i + 1])) {
                available_sources
                    .iter()
                    .find(|s| s.contains(parts[i + 1]))
                    .unwrap()
                    .to_string()
            } else {
                parts[i + 1].to_string()
            };
            source_name = Some(resolved_source);
        }
        i += 2;
    }

    i = param_start;
    while i + 1 < part_count {
        if parts[i] != "s" {
            let normalized_key = normalize_parameter_name(parts[i], source_name.as_ref(), registry);

            if is_valid_parameter(normalized_key, source_name.as_ref(), registry) {
                let param_value = parse_parameter_value_smart(normalized_key, parts[i + 1], pool);
                parameters.insert(normalized_key.to_string(), param_value);
            }
        }
        i += 2;
    }

    let source_name = source_name?;

    add_missing_defaults(&mut parameters);

    if registry.validate_timestamp(&parameters).is_err() {
        return None;
    }

    parameters.remove("due");

    Some(EngineMessage::Play {
        voice_id,
        track_id,
        source_name,
        parameters,
    })
}

fn normalize_parameter_name(
    param: &str,
    source_name: Option<&String>,
    registry: &ModuleRegistry,
) -> &'static str {
    for desc in &ENGINE_PARAM_DESCRIPTORS {
        if desc.name == param {
            return desc.name;
        }
        for alias in desc.aliases {
            if *alias == param {
                return desc.name;
            }
        }
    }

    if let Some(source) = source_name {
        if registry.sources.contains_key(source) {
            let module = registry.sources.get(source).unwrap()();
            for desc in module.get_parameter_descriptors() {
                if desc.name == param {
                    return desc.name;
                }
                for alias in desc.aliases {
                    if *alias == param {
                        return desc.name;
                    }
                }
            }
        }
    }

    Box::leak(param.to_string().into_boxed_str())
}

fn is_valid_parameter(
    param_name: &str,
    source_name: Option<&String>,
    registry: &ModuleRegistry,
) -> bool {
    for desc in &ENGINE_PARAM_DESCRIPTORS {
        if desc.name == param_name {
            return true;
        }
        for alias in desc.aliases {
            if *alias == param_name {
                return true;
            }
        }
    }

    if let Some(source) = source_name {
        if registry.sources.contains_key(source) {
            let module = registry.sources.get(source).unwrap()();
            for desc in module.get_parameter_descriptors() {
                if desc.name == param_name {
                    return true;
                }
                for alias in desc.aliases {
                    if *alias == param_name {
                        return true;
                    }
                }
            }
        }
    }

    // Check global effect parameters
    for effect_name in registry.get_available_global_effects() {
        if let Some(factory) = registry.global_effects.get(effect_name) {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.name == param_name {
                    return true;
                }
                for alias in desc.aliases {
                    if *alias == param_name {
                        return true;
                    }
                }
            }
        }
    }

    // Check local effect parameters
    for effect_name in registry.get_available_local_effects() {
        if let Some(factory) = registry.local_effects.get(effect_name) {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.name == param_name {
                    return true;
                }
                for alias in desc.aliases {
                    if *alias == param_name {
                        return true;
                    }
                }
            }
        }
    }

    // Check generic wet parameters for global effects
    if registry.is_global_effect_wet_parameter(param_name).is_some() {
        return true;
    }

    // Debug: Print what we're checking
    if param_name.ends_with("_wet") {
        let effect_name = &param_name[..param_name.len() - 4];
        println!("DEBUG: Checking wet param '{}' for effect '{}'", param_name, effect_name);
        println!("DEBUG: Available global effects: {:?}", registry.get_available_global_effects());
        println!("DEBUG: Contains effect? {}", registry.global_effects.contains_key(effect_name));
    }

    false
}

fn add_missing_defaults(parameters: &mut HashMap<String, Box<dyn Any + Send>>) {
    for desc in &ENGINE_PARAM_DESCRIPTORS {
        if !parameters.contains_key(desc.name) {
            parameters.insert(
                desc.name.to_string(),
                Box::new(desc.default_value) as Box<dyn Any + Send>,
            );
        }
    }
}

fn parse_parameter_value_smart(
    key: &str,
    value: &str,
    pool: &Arc<MemoryPool>,
) -> Box<dyn Any + Send> {
    if key == "sample_name" || key == "folder" || key == "fd" {
        Box::new(value.to_string())
    } else if value.contains(':') {
        Box::new(Modulation::parse_with_pool(value, pool))
    } else {
        Box::new(value.parse::<f32>().unwrap_or(0.0))
    }
}
