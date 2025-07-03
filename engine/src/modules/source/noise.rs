use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use crate::dsp::oscillators::NoiseGenerator;
use std::any::Any;

/// Simple white noise generator
/// 
/// Pure Rust implementation using high-quality LCG algorithm.
/// Features zero-allocation real-time processing and deterministic output.
pub struct NoiseOscillator {
    /// Core noise generator
    noise: NoiseGenerator,
    
    /// State tracking
    initialized: bool,
}

impl NoiseOscillator {
    /// Creates a new noise oscillator
    pub fn new() -> Self {
        Self {
            noise: NoiseGenerator::new(),
            initialized: false,
        }
    }

    /// Initialize noise generator
    fn initialize(&mut self) {
        if !self.initialized {
            self.initialized = true;
            // Seed with a deterministic but varied seed based on current time
            // This ensures different instances get different noise patterns
            let seed = std::ptr::addr_of!(self) as usize as u32;
            self.noise.seed(seed);
        }
    }
}

impl AudioModule for NoiseOscillator {
    fn get_name(&self) -> &'static str {
        "noise"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, _name: &str, _value: f32) -> bool {
        false
    }

    fn is_active(&self) -> bool {
        true
    }
}

impl Source for NoiseOscillator {
    fn generate(&mut self, buffer: &mut [Frame], _sample_rate: f32) {
        // Initialize noise generator
        self.initialize();
        
        // Generate noise samples
        for frame in buffer.iter_mut() {
            let sample = self.noise.next_sample();
            frame.left = sample;
            frame.right = sample;
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

static PARAMETER_DESCRIPTORS: [ParameterDescriptor; 0] = [];

impl ModuleMetadata for NoiseOscillator {
    fn get_static_name() -> &'static str {
        "noise"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }
}

/// Creates a new noise oscillator instance
pub fn create_noise_oscillator() -> Box<dyn Source> {
    Box::new(NoiseOscillator::new())
}
