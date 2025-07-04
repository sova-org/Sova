//! OSC command parser for engine control
//!
//! This module handles parsing of incoming OSC commands into engine messages.
//! The parser is optimized for real-time performance with minimal allocations
//! and efficient parameter validation.
//!
//! # Command Format
//!
//! Commands follow the pattern:
//! ```text
//! /play <param_key> <param_value> [more params...]
//! ```
//!
//! All parameters use key-value pairs for complete consistency.
//!
//! ## Parameters
//! Key-value pairs controlling audio generation:
//! - **id**: Voice ID (0-127, "s" for auto-assignment, defaults to auto)
//! - **s**: Source module (sine, saw, sample, etc.)
//! - **track**: Track assignment (1-10, defaults to 1)
//! - **f**: Fundamental frequency in Hz
//! - **a**: Amplitude (0.0-1.0)
//! - **d**: Duration in seconds
//! - **pan**: Stereo positioning (-1.0 to 1.0)
//!
//! ## Voice ID Aliases
//! - **id**: Full parameter name
//! - **voice**: Alternative name
//! - **v**: Short alias
//!
//! ## Parameter Values
//! - **Static**: `440.0` (constant value)
//! - **Modulated**: `osc:440:50:2:sine:4` (oscillating)
//! - **String**: Sample names, folder paths
//!
//! # Examples
//!
//! ```text
//! /play id 0 s sine f 440 a 0.8 d 2.0     # explicit voice 0
//! /play voice 5 track 2 s sample sample_name kick.wav a 1.0
//! /play v s track 3 s saw f osc:220:100:3:triangle:8 a env:0:1:exp:1.5
//! /play s sine f 440 a 0.8                # auto-assign voice, track defaults to 1
//! /play id s t 4 s sine f 880             # explicit auto-assignment with track alias
//! /play voice 10 s triangle f 220         # explicit voice 10
//! ```
//!
//! # Performance Characteristics
//!
//! - **Zero-allocation parsing**: Fixed-size arrays, no dynamic allocation
//! - **Parameter validation**: Real-time type checking and normalization
//! - **Alias resolution**: Short parameter names expand to full descriptors
//! - **Smart type inference**: Automatic detection of modulation vs static values
//! - **Round-robin voice allocation**: Automatic voice management for polyphony

use crate::memory::MemoryPool;
use crate::modulation::Modulation;
use crate::registry::{ENGINE_PARAM_DESCRIPTORS, ModuleRegistry};
use crate::types::{EngineMessage, TrackId, VoiceId};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Global counter for round-robin voice allocation
///
/// Used when voice_id is "s" to automatically assign voices in sequence.
/// Wraps around at max_voices to ensure polyphony limits are respected.
static mut ROUND_ROBIN_VOICE: u32 = 0;

/// Parses OSC command string into engine message
///
/// Convenience wrapper that creates a temporary memory pool.
/// For high-frequency parsing, use `parse_command_with_pool` instead.
///
/// # Arguments
/// * `command` - Raw OSC command string
/// * `max_voices` - Maximum number of available voices
/// * `registry` - Module registry for parameter validation
///
/// # Returns
/// `Some(EngineMessage)` if parsing succeeds, `None` otherwise
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

/// Parses OSC command string using provided memory pool
///
/// Main parsing entry point optimized for real-time performance.
/// Uses fixed-size arrays and minimal branching for consistent timing.
///
/// # Arguments
/// * `command` - Raw OSC command string
/// * `max_voices` - Maximum number of available voices
/// * `registry` - Module registry for parameter validation
/// * `pool` - Pre-allocated memory pool for modulation parsing
///
/// # Returns
/// `Some(EngineMessage::Play)` with parsed parameters, or `None` if invalid
///
/// # Performance Notes
/// - Fixed 64-parameter limit to avoid dynamic allocation
/// - Early return on invalid commands to minimize processing
/// - Parameter validation happens after parsing for efficiency
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

    if part_count < 1 || parts[0] != "/play" {
        return None;
    }

    let param_start = 1;

    let mut parameters: HashMap<String, Box<dyn Any + Send>> = HashMap::with_capacity(16);
    let mut source_name = None;
    let mut voice_param: Option<&str> = None;

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
        } else if parts[i] == "id" || parts[i] == "voice" || parts[i] == "v" {
            voice_param = Some(parts[i + 1]);
        }
        i += 2;
    }

    i = param_start;
    while i + 1 < part_count {
        if parts[i] != "s" && parts[i] != "id" && parts[i] != "voice" && parts[i] != "v" {
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

    // Accept all messages unconditionally - no timestamp validation
    // Remove "due" parameter if present (it's for scheduling, not voice parameters)
    parameters.remove("due");

    let voice_id = if let Some(voice_param) = voice_param {
        if voice_param == "s" {
            // Auto-assign using round-robin - get current value then increment
            let current_id = unsafe { ROUND_ROBIN_VOICE };
            unsafe { ROUND_ROBIN_VOICE = (current_id + 1) % (max_voices as u32) };
            current_id
        } else {
            // Parse explicit voice ID with validation
            if let Ok(parsed_id) = voice_param.parse::<VoiceId>() {
                if (parsed_id as usize) < max_voices {
                    parsed_id
                } else {
                    // Voice ID out of range, fallback to auto-assign
                    let current_id = unsafe { ROUND_ROBIN_VOICE };
                    unsafe { ROUND_ROBIN_VOICE = (current_id + 1) % (max_voices as u32) };
                    current_id
                }
            } else {
                // Parse failed, fallback to auto-assign
                let current_id = unsafe { ROUND_ROBIN_VOICE };
                unsafe { ROUND_ROBIN_VOICE = (current_id + 1) % (max_voices as u32) };
                current_id
            }
        }
    } else {
        // No voice ID specified - auto-assign
        let current_id = unsafe { ROUND_ROBIN_VOICE };
        unsafe { ROUND_ROBIN_VOICE = (current_id + 1) % (max_voices as u32) };
        current_id
    };

    let track_id = parameters
        .get("track")
        .and_then(|t| t.downcast_ref::<f32>())
        .map(|&f| f as TrackId)
        .unwrap_or(1);

    Some(EngineMessage::Play {
        voice_id,
        track_id,
        source_name,
        parameters,
    })
}

/// Normalizes parameter name to canonical form
///
/// Resolves parameter aliases to their full names for consistent
/// internal representation. Checks engine parameters first, then
/// source-specific parameters.
///
/// # Arguments
/// * `param` - Raw parameter name from command
/// * `source_name` - Active source module name
/// * `registry` - Module registry for parameter lookup
///
/// # Returns
/// Canonical parameter name as static string reference
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
        if let Some(entry) = registry.sources.get(source) {
            let (_, descriptors) = (entry.metadata)();
            for desc in descriptors {
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

/// Validates parameter name against available descriptors
///
/// Checks if parameter exists in engine globals, source modules,
/// or effect modules. Also handles special cases like wet parameters
/// for global effects.
///
/// # Arguments
/// * `param_name` - Normalized parameter name to validate
/// * `source_name` - Active source module name
/// * `registry` - Module registry for parameter lookup
///
/// # Returns
/// `true` if parameter is valid and supported, `false` otherwise
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
        if let Some(entry) = registry.sources.get(source) {
            let (_, descriptors) = (entry.metadata)();
            for desc in descriptors {
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
        if let Some(entry) = registry.global_effects.get(effect_name) {
            let (_, descriptors) = (entry.metadata)();
            for desc in descriptors {
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
        if let Some(entry) = registry.local_effects.get(effect_name) {
            let (_, descriptors) = (entry.metadata)();
            for desc in descriptors {
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

    // Check generic send parameters for global effects
    if registry
        .is_global_effect_send_parameter(param_name)
        .is_some()
    {
        return true;
    }

    false
}

/// Adds default values for missing engine parameters
///
/// Ensures all required engine parameters have values by inserting
/// defaults from parameter descriptors. This prevents runtime errors
/// from missing required parameters.
///
/// # Arguments
/// * `parameters` - Mutable parameter map to populate with defaults
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

/// Intelligently parses parameter value based on content and context
///
/// Uses heuristics to determine value type:
/// - String parameters: sample_name, folder, fd
/// - Modulation: Contains ':' separator
/// - Numeric: Everything else, parsed as f32
///
/// # Arguments
/// * `key` - Parameter name for context-sensitive parsing
/// * `value` - Raw string value to parse
/// * `pool` - Memory pool for modulation object allocation
///
/// # Returns
/// Boxed parameter value ready for engine consumption
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
