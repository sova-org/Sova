//!
//! Module registry and parameter management for the real-time audio engine.
//!
//! This module provides centralized management of audio modules (sources, effects),
//! parameter definitions, and message timestamp validation. It serves as the factory
//! and configuration center for all audio processing components.
//!
//! # Architecture
//!
//! The registry system consists of:
//! - **Engine Parameters**: Core audio parameters shared across all voices
//! - **Module Registry**: Factory for creating audio processing modules
//! - **Timestamp Validation**: Message scheduling validation for real-time safety
//!
//! # Performance Characteristics
//!
//! - Module creation uses function pointers for zero-overhead factory pattern
//! - Parameter lookups use compile-time constants for maximum performance
//! - HashMap-based module storage optimized for initialization-time access
//! - No heap allocations during real-time audio processing
//!
//! # Thread Safety
//!
//! The registry is designed for safe concurrent access:
//! - Module factories are immutable function pointers
//! - Parameter descriptors are compile-time constants
//! - Registry modification should occur during initialization only

use crate::constants::{
    AMP_MAX, AMP_MIN, ATTACK_MAX, ATTACK_MIN, CURVE_MAX, CURVE_MIN, DECAY_MAX, DECAY_MIN,
    DEFAULT_AMP, DEFAULT_ATTACK, DEFAULT_ATTACK_CURVE, DEFAULT_DECAY, DEFAULT_DECAY_CURVE,
    DEFAULT_DURATION, DEFAULT_PAN, DEFAULT_RELEASE, DEFAULT_RELEASE_CURVE, DEFAULT_SUSTAIN,
    DEFAULT_TRACK, DURATION_MAX, DURATION_MIN, ENGINE_PARAM_COUNT, PAN_MAX, PAN_MIN, RELEASE_MAX,
    RELEASE_MIN, SUSTAIN_MAX, SUSTAIN_MIN, TRACK_MAX, TRACK_MIN,
};
use crate::modulation::Modulation;
use crate::modules::{GlobalEffect, LocalEffect, ParameterDescriptor, Source};
use std::any::Any;
use std::collections::HashMap;

/// Core engine parameter definitions for all voices.
///
/// This array defines the fundamental parameters that every voice in the engine
/// supports. These parameters are indexed using the `ENGINE_PARAM_*` constants
/// for maximum performance during real-time parameter access.
///
/// # Parameter Definitions
///
/// - **amp**: Voice amplitude (0.0-1.0)
/// - **pan**: Stereo positioning (-1.0=left, 0.0=center, 1.0=right)  
/// - **attack**: ADSR envelope attack time in seconds
/// - **decay**: ADSR envelope decay time in seconds
/// - **sustain**: ADSR envelope sustain level (0.0-1.0)
/// - **release**: ADSR envelope release time in seconds
/// - **dur**: Voice duration in seconds (0.0=infinite until release)
///
/// # Performance Notes
///
/// All parameters support real-time modulation and are accessed via compile-time
/// constants for zero-overhead parameter lookups.
pub const ENGINE_PARAM_DESCRIPTORS: [ParameterDescriptor; ENGINE_PARAM_COUNT] = [
    ParameterDescriptor {
        name: "amp",
        aliases: &["amplitude"],
        min_value: AMP_MIN,
        max_value: AMP_MAX,
        default_value: DEFAULT_AMP,
        unit: "",
        description: "",
        modulable: true,
    },
    ParameterDescriptor {
        name: "pan",
        aliases: &[],
        min_value: PAN_MIN,
        max_value: PAN_MAX,
        default_value: DEFAULT_PAN,
        unit: "",
        description: "",
        modulable: true,
    },
    ParameterDescriptor {
        name: "attack",
        aliases: &["atk", "a"],
        min_value: ATTACK_MIN,
        max_value: ATTACK_MAX,
        default_value: DEFAULT_ATTACK,
        unit: "",
        description: "",
        modulable: true,
    },
    ParameterDescriptor {
        name: "decay",
        aliases: &["dec", "d"],
        min_value: DECAY_MIN,
        max_value: DECAY_MAX,
        default_value: DEFAULT_DECAY,
        unit: "",
        description: "",
        modulable: true,
    },
    ParameterDescriptor {
        name: "sustain",
        aliases: &["sus"],
        min_value: SUSTAIN_MIN,
        max_value: SUSTAIN_MAX,
        default_value: DEFAULT_SUSTAIN,
        unit: "",
        description: "",
        modulable: true,
    },
    ParameterDescriptor {
        name: "release",
        aliases: &["rel", "r"],
        min_value: RELEASE_MIN,
        max_value: RELEASE_MAX,
        default_value: DEFAULT_RELEASE,
        unit: "",
        description: "",
        modulable: true,
    },
    ParameterDescriptor {
        name: "dur",
        aliases: &["duration"],
        min_value: DURATION_MIN,
        max_value: DURATION_MAX,
        default_value: DEFAULT_DURATION,
        unit: "",
        description: "",
        modulable: true,
    },
    ParameterDescriptor {
        name: "attack_curve",
        aliases: &["atk_curve", "ac"],
        min_value: CURVE_MIN,
        max_value: CURVE_MAX,
        default_value: DEFAULT_ATTACK_CURVE,
        unit: "",
        description: "Attack curve shape (0.0=linear, 1.0=exponential)",
        modulable: true,
    },
    ParameterDescriptor {
        name: "decay_curve",
        aliases: &["dec_curve", "dc"],
        min_value: CURVE_MIN,
        max_value: CURVE_MAX,
        default_value: DEFAULT_DECAY_CURVE,
        unit: "",
        description: "Decay curve shape (0.0=linear, 1.0=exponential)",
        modulable: true,
    },
    ParameterDescriptor {
        name: "release_curve",
        aliases: &["rel_curve", "rc"],
        min_value: CURVE_MIN,
        max_value: CURVE_MAX,
        default_value: DEFAULT_RELEASE_CURVE,
        unit: "",
        description: "Release curve shape (0.0=linear, 1.0=exponential)",
        modulable: true,
    },
    ParameterDescriptor {
        name: "track",
        aliases: &["t", "trk"],
        min_value: TRACK_MIN,
        max_value: TRACK_MAX,
        default_value: DEFAULT_TRACK,
        unit: "",
        description: "Audio track assignment for routing and effects",
        modulable: false,
    },
];

/// Checks if a parameter name corresponds to an engine parameter.
///
/// This function performs a linear search through engine parameter descriptors
/// to match parameter names and aliases. Used for routing parameters to the
/// correct processing path during voice configuration.
///
/// # Arguments
///
/// * `param_name` - Parameter name to check
///
/// # Returns
///
/// `true` if the parameter is handled by the engine, `false` if it should
/// be routed to module-specific parameter handling.
///
/// # Performance Notes
///
/// Linear search is acceptable here as the engine parameter count is small
/// and this function is typically called during voice setup, not real-time processing.
pub fn is_engine_parameter(param_name: &str) -> bool {
    ENGINE_PARAM_DESCRIPTORS
        .iter()
        .any(|desc| desc.matches_name(param_name))
}

/// Gets the array index for an engine parameter by name.
///
/// Returns the index into `ENGINE_PARAM_DESCRIPTORS` for the given parameter
/// name or alias. Used for direct array access during parameter updates.
///
/// # Arguments
///
/// * `param_name` - Parameter name or alias to look up
///
/// # Returns
///
/// `Some(index)` if the parameter exists, `None` otherwise.
///
/// # Example
///
/// ```rust
/// let index = get_engine_parameter_index("amp").unwrap();
/// assert_eq!(index, ENGINE_PARAM_AMP);
///
/// let index = get_engine_parameter_index("a").unwrap(); // alias for attack
/// assert_eq!(index, ENGINE_PARAM_ATTACK);
/// ```
pub fn get_engine_parameter_index(param_name: &str) -> Option<usize> {
    ENGINE_PARAM_DESCRIPTORS
        .iter()
        .position(|desc| desc.matches_name(param_name))
}

/// Central registry for audio processing modules and configuration.
///
/// The `ModuleRegistry` serves as the factory and configuration center for all
/// audio processing components in the engine. It manages three types of modules:
/// - **Sources**: Audio generators (oscillators, samplers, etc.)
/// - **Local Effects**: Per-voice processing (filters, distortion, etc.)  
/// - **Global Effects**: Track-level processing (reverb, delay, etc.)
///
/// # Architecture
///
/// The registry uses a factory pattern with function pointers for zero-overhead
/// module creation. Each module type is stored in a separate HashMap with both
/// full names and short aliases for flexible access.
///
/// # Performance Characteristics
///
/// - Module registration occurs during initialization only
/// - Factory functions are stored as function pointers (zero runtime cost)
/// - Module creation uses trait objects for runtime polymorphism
/// - No heap allocations during module lookup operations
///
/// # Thread Safety
///
/// The registry is designed for concurrent read access:
/// - Module factories are immutable after registration
/// - Multiple threads can safely create modules simultaneously
/// - Modification should occur during initialization phase only
///
/// # Usage
///
/// ```rust
/// let mut registry = ModuleRegistry::new();
/// registry.register_default_modules();
///
/// // Create audio modules
/// let sine_osc = registry.create_source("sine_oscillator").unwrap();
/// let reverb = registry.create_global_effect("simple_reverb").unwrap();
///
/// // Query available modules
/// let sources = registry.get_available_sources();
/// ```
#[derive(Clone)]
pub struct ModuleRegistry {
    /// Factory functions for audio source modules
    pub sources: HashMap<String, fn() -> Box<dyn Source>>,
    /// Factory functions for local effect modules  
    pub local_effects: HashMap<String, fn() -> Box<dyn LocalEffect>>,
    /// Factory functions for global effect modules
    pub global_effects: HashMap<String, fn() -> Box<dyn GlobalEffect>>,
}

impl Default for ModuleRegistry {
    /// Creates a new registry with default modules loaded.
    ///
    /// This convenience constructor initializes the registry and loads
    /// all default audio modules for immediate use.
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRegistry {
    /// Creates a new empty module registry.
    ///
    /// The registry is initialized with empty module collections.
    /// Call `register_default_modules()` or register individual modules to populate the registry.
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            local_effects: HashMap::new(),
            global_effects: HashMap::new(),
        }
    }

    /// Registers an audio source module with the registry.
    ///
    /// Stores the factory function under both the provided name and the
    /// module's internal name (if different) for flexible access patterns.
    ///
    /// # Arguments
    ///
    /// * `name` - Primary name for the module
    /// * `factory` - Function that creates new module instances
    ///
    /// # Performance Notes
    ///
    /// Registration involves creating a temporary module instance to extract
    /// the internal name. This should only be done during initialization.
    pub fn register_source(&mut self, name: &str, factory: fn() -> Box<dyn Source>) {
        self.sources.insert(name.to_string(), factory);
        let module = factory();
        let short_name = module.get_name();
        if short_name != name {
            self.sources.insert(short_name.to_string(), factory);
        }
    }

    /// Registers a local effect module with the registry.
    ///
    /// Local effects are applied per-voice and process audio before
    /// it reaches the track's global effects chain.
    ///
    /// # Arguments
    ///
    /// * `name` - Primary name for the module
    /// * `factory` - Function that creates new module instances
    pub fn register_local_effect(&mut self, name: &str, factory: fn() -> Box<dyn LocalEffect>) {
        self.local_effects.insert(name.to_string(), factory);
        let module = factory();
        let short_name = module.get_name();
        if short_name != name {
            self.local_effects.insert(short_name.to_string(), factory);
        }
    }

    /// Registers a global effect module with the registry.
    ///
    /// Global effects are applied at the track level and process the
    /// mixed output of all voices assigned to that track.
    ///
    /// # Arguments
    ///
    /// * `name` - Primary name for the module
    /// * `factory` - Function that creates new module instances
    pub fn register_global_effect(&mut self, name: &str, factory: fn() -> Box<dyn GlobalEffect>) {
        self.global_effects.insert(name.to_string(), factory);
        let module = factory();
        let short_name = module.get_name();
        if short_name != name {
            self.global_effects.insert(short_name.to_string(), factory);
        }
    }

    /// Gets the parameter names for a specific module.
    ///
    /// Creates a temporary instance of the specified module and extracts
    /// the names of all available parameters. Used for dynamic parameter
    /// discovery and validation.
    ///
    /// # Arguments
    ///
    /// * `module_name` - Name of the module to query
    ///
    /// # Returns
    ///
    /// `Some(Vec<&str>)` containing parameter names if the module exists,
    /// `None` if the module is not registered.
    ///
    /// # Performance Notes
    ///
    /// This method creates a temporary module instance and should be used
    /// during initialization or configuration, not in real-time processing.
    pub fn get_module_parameters(&self, module_name: &str) -> Option<Vec<&'static str>> {
        if let Some(factory) = self.sources.get(module_name) {
            let module = factory();
            return Some(
                module
                    .get_parameter_descriptors()
                    .iter()
                    .map(|d| d.name)
                    .collect(),
            );
        }
        if let Some(factory) = self.local_effects.get(module_name) {
            let module = factory();
            return Some(
                module
                    .get_parameter_descriptors()
                    .iter()
                    .map(|d| d.name)
                    .collect(),
            );
        }
        if let Some(factory) = self.global_effects.get(module_name) {
            let module = factory();
            return Some(
                module
                    .get_parameter_descriptors()
                    .iter()
                    .map(|d| d.name)
                    .collect(),
            );
        }
        None
    }

    /// Checks if a parameter name is a generic wet parameter for a global effect.
    ///
    /// Generic wet parameters follow the pattern "{effect_name}_wet" and are
    /// automatically available for all global effects to control dry/wet mixing.
    ///
    /// # Arguments
    ///
    /// * `param_name` - Parameter name to check
    ///
    /// # Returns
    ///
    /// `Some(effect_name)` if this is a wet parameter, `None` otherwise.
    pub fn is_global_effect_wet_parameter<'a>(&self, param_name: &'a str) -> Option<&'a str> {
        if let Some(effect_name) = param_name.strip_suffix("_wet") {
            if self.global_effects.contains_key(effect_name) {
                return Some(effect_name);
            }
        }
        None
    }

    /// Registers all default audio modules with the registry.
    ///
    /// Loads the standard set of audio processing modules included with
    /// the engine:
    /// - **sine_oscillator**: Basic sine wave audio source
    /// - **sample**: Stereo audio sample playback
    /// - **lowpass_filter**: Low-pass filter effect  
    /// - **simple_reverb**: Basic reverb effect
    ///
    /// This method should be called during engine initialization to
    /// populate the registry with commonly used modules.
    ///
    /// # Performance Notes
    ///
    /// Registration involves importing and storing function pointers.
    /// This should only be called during initialization.
    pub fn register_default_modules(&mut self) {
        use crate::modules::global::echo::create_echo_effect;
        use crate::modules::global::reverb::create_simple_reverb;
        use crate::modules::local::bitcrusher::create_bitcrusher;
        use crate::modules::local::flanger::create_flanger;
        // TEMPORARILY DISABLED: Causes illegal hardware instruction crash
        // use crate::modules::local::mooglpf::create_mooglpf_filter;
        use crate::modules::local::phaser::create_phaser;
        use crate::modules::local::ringmod::create_ring_modulator;
        use crate::modules::local::svf_filter::create_svf_filter;
        use crate::modules::source::fm::create_fm_oscillator;
        use crate::modules::source::sample::create_stereo_sampler;
        use crate::modules::source::saw::create_saw_oscillator;
        use crate::modules::source::sine::create_sine_oscillator;
        use crate::modules::source::square::create_square_oscillator;
        use crate::modules::source::triangle::create_triangle_oscillator;

        self.register_source("fm_oscillator", create_fm_oscillator);
        self.register_source("sine_oscillator", create_sine_oscillator);
        self.register_source("sample", create_stereo_sampler);
        self.register_source("saw_oscillator", create_saw_oscillator);
        self.register_source("square_oscillator", create_square_oscillator);
        self.register_source("triangle_oscillator", create_triangle_oscillator);
        self.register_local_effect("bitcrusher", create_bitcrusher);
        self.register_local_effect("flanger", create_flanger);
        // TEMPORARILY DISABLED: Causes illegal hardware instruction crash
        // self.register_local_effect("mooglpf_filter", create_mooglpf_filter);
        self.register_local_effect("phaser", create_phaser);
        self.register_local_effect("ring_modulator", create_ring_modulator);
        self.register_local_effect("svf_filter", create_svf_filter);
        self.register_global_effect("echo", create_echo_effect);
        // TEMPORARILY DISABLED: Causes illegal hardware instruction crash
        // self.register_global_effect("reverb", create_simple_reverb);
    }

    /// Returns a list of all registered audio source module names.
    ///
    /// Provides a snapshot of available source modules for discovery
    /// and validation purposes. The list includes both primary names
    /// and aliases.
    ///
    /// # Returns
    ///
    /// Vector of string slices containing all registered source names.
    pub fn get_available_sources(&self) -> Vec<&str> {
        self.sources.keys().map(|s| s.as_str()).collect()
    }

    /// Returns a list of all registered local effect module names.
    ///
    /// Provides a snapshot of available local effect modules for discovery
    /// and validation purposes. The list includes both primary names
    /// and aliases.
    ///
    /// # Returns
    ///
    /// Vector of string slices containing all registered local effect names.
    pub fn get_available_local_effects(&self) -> Vec<&str> {
        self.local_effects.keys().map(|s| s.as_str()).collect()
    }

    /// Returns a list of all registered global effect module names.
    ///
    /// Provides a snapshot of available global effect modules for discovery
    /// and validation purposes. The list includes both primary names
    /// and aliases.
    ///
    /// # Returns
    ///
    /// Vector of string slices containing all registered global effect names.
    pub fn get_available_global_effects(&self) -> Vec<&str> {
        self.global_effects.keys().map(|s| s.as_str()).collect()
    }

    /// Creates a new audio source module instance.
    ///
    /// Uses the registered factory function to create a new instance of
    /// the specified source module. Each call returns a fresh instance
    /// with default parameter values.
    ///
    /// # Arguments
    ///
    /// * `name` - Name or alias of the source module to create
    ///
    /// # Returns
    ///
    /// `Some(Box<dyn Source>)` if the module exists, `None` otherwise.
    ///
    /// # Performance Notes
    ///
    /// Module creation involves heap allocation and should be done during
    /// voice setup, not in real-time audio processing loops.
    pub fn create_source(&self, name: &str) -> Option<Box<dyn Source>> {
        self.sources.get(name).map(|factory| factory())
    }

    /// Creates a new local effect module instance.
    ///
    /// Uses the registered factory function to create a new instance of
    /// the specified local effect module. Each call returns a fresh instance
    /// with default parameter values.
    ///
    /// # Arguments
    ///
    /// * `name` - Name or alias of the local effect module to create
    ///
    /// # Returns
    ///
    /// `Some(Box<dyn LocalEffect>)` if the module exists, `None` otherwise.
    ///
    /// # Performance Notes
    ///
    /// Module creation involves heap allocation and should be done during
    /// voice setup, not in real-time audio processing loops.
    pub fn create_local_effect(&self, name: &str) -> Option<Box<dyn LocalEffect>> {
        self.local_effects.get(name).map(|factory| factory())
    }

    /// Creates a new global effect module instance.
    ///
    /// Uses the registered factory function to create a new instance of
    /// the specified global effect module. Each call returns a fresh instance
    /// with default parameter values.
    ///
    /// # Arguments
    ///
    /// * `name` - Name or alias of the global effect module to create
    ///
    /// # Returns
    ///
    /// `Some(Box<dyn GlobalEffect>)` if the module exists, `None` otherwise.
    ///
    /// # Performance Notes
    ///
    /// Module creation involves heap allocation and should be done during
    /// track setup, not in real-time audio processing loops.
    pub fn create_global_effect(&self, name: &str) -> Option<Box<dyn GlobalEffect>> {
        self.global_effects.get(name).map(|factory| factory())
    }

    /// Normalizes a parameter name to its canonical form using aliases.
    ///
    /// This method resolves parameter aliases to their canonical names for
    /// consistent parameter handling across different input methods (OSC, BaLi, etc.).
    /// It checks engine parameters first, then source-specific parameters, then
    /// local and global effects.
    ///
    /// # Arguments
    ///
    /// * `param` - Parameter name or alias to normalize
    /// * `source_name` - Optional source name for source-specific parameter lookup
    ///
    /// # Returns
    ///
    /// The canonical parameter name, or the original name if no match is found.
    pub fn normalize_parameter_name(
        &self,
        param: &str,
        source_name: Option<&String>,
    ) -> &'static str {
        // Check engine parameters first
        for desc in &ENGINE_PARAM_DESCRIPTORS {
            if desc.matches_name(param) {
                return desc.name;
            }
        }

        // Check source-specific parameters
        if let Some(source) = source_name {
            if self.sources.contains_key(source) {
                let module = self.sources.get(source).unwrap()();
                for desc in module.get_parameter_descriptors() {
                    if desc.matches_name(param) {
                        return desc.name;
                    }
                }
            }
        }

        // Check local effects
        for factory in self.local_effects.values() {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.matches_name(param) {
                    return desc.name;
                }
            }
        }

        // Check global effects
        for factory in self.global_effects.values() {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.matches_name(param) {
                    return desc.name;
                }
            }
        }

        // If no match found, return the original parameter name
        Box::leak(param.to_string().into_boxed_str())
    }

    /// Checks if a parameter name is valid for the given source and context.
    ///
    /// Validates that a parameter name (after normalization) is supported by
    /// the engine, the specified source, or any registered effects.
    ///
    /// # Arguments
    ///
    /// * `param_name` - Parameter name to validate
    /// * `source_name` - Optional source name for source-specific validation
    ///
    /// # Returns
    ///
    /// `true` if the parameter is valid, `false` otherwise.
    pub fn is_valid_parameter(&self, param_name: &str, source_name: Option<&String>) -> bool {
        // Check engine parameters
        for desc in &ENGINE_PARAM_DESCRIPTORS {
            if desc.matches_name(param_name) {
                return true;
            }
        }

        // Check source-specific parameters
        if let Some(source) = source_name {
            if self.sources.contains_key(source) {
                let module = self.sources.get(source).unwrap()();
                for desc in module.get_parameter_descriptors() {
                    if desc.matches_name(param_name) {
                        return true;
                    }
                }
            }
        }

        // Check local effects
        for factory in self.local_effects.values() {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.matches_name(param_name) {
                    return true;
                }
            }
        }

        // Check global effects
        for factory in self.global_effects.values() {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.matches_name(param_name) {
                    return true;
                }
            }
        }

        // Check generic wet parameters for global effects
        if self.is_global_effect_wet_parameter(param_name).is_some() {
            return true;
        }

        false
    }

    /// Parses a parameter value string into an appropriate boxed type.
    ///
    /// Handles modulation syntax (containing ':') and numeric values.
    /// String values are passed through as-is for source names and other
    /// string parameters.
    ///
    /// # Arguments
    ///
    /// * `value` - String value to parse
    ///
    /// # Returns
    ///
    /// Boxed value ready for engine parameter processing.
    pub fn parse_parameter_value(&self, value: &str) -> Box<dyn Any + Send> {
        if value.contains(':') {
            Box::new(Modulation::parse(value))
        } else if let Ok(float_val) = value.parse::<f32>() {
            Box::new(float_val)
        } else {
            Box::new(value.to_string())
        }
    }

    /// Normalizes a complete set of parameters using alias resolution.
    ///
    /// This method provides unified parameter processing for all entry points
    /// to the engine. It resolves aliases, validates parameters, and ensures
    /// consistent handling whether parameters come from OSC, BaLi, or other sources.
    ///
    /// # Arguments
    ///
    /// * `raw_parameters` - HashMap of parameter names to values
    /// * `source_name` - Optional source name for source-specific parameter lookup
    ///
    /// # Returns
    ///
    /// HashMap with normalized parameter names and validated parameters only.
    pub fn normalize_parameters(
        &self,
        raw_parameters: HashMap<String, Box<dyn Any + Send>>,
        source_name: Option<&String>,
    ) -> HashMap<String, Box<dyn Any + Send>> {
        let mut normalized_parameters = HashMap::with_capacity(raw_parameters.len());

        for (key, value) in raw_parameters {
            if key == "s" {
                // Source parameter passes through unchanged
                normalized_parameters.insert(key, value);
            } else {
                let normalized_key = self.normalize_parameter_name(&key, source_name);
                if self.is_valid_parameter(normalized_key, source_name) {
                    normalized_parameters.insert(normalized_key.to_string(), value);
                }
            }
        }

        normalized_parameters
    }
}
