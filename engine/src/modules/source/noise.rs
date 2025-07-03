use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use std::any::Any;

/// Amplitude calibration constant for noise oscillator
/// This value normalizes the output to approximately 0 dBFS
const AMP_CALIBRATION: f32 = 0.5;

faust_macro::dsp!(
    declare name "noise_oscillator";
    declare version "1.0";
    
    import("stdfaust.lib");
    
    process = no.noise;
);

/// Simple white noise generator
pub struct NoiseOscillator {
    dsp: Box<noise_oscillator::NoiseOscillator>,
    sample_rate: f32,
    output: [f32; 1024],
}

impl NoiseOscillator {
    /// Creates a new noise oscillator
    pub fn new() -> Self {
        let mut dsp = Box::new(noise_oscillator::NoiseOscillator::new());
        dsp.init(48000);
        Self {
            dsp,
            sample_rate: 48000.0,
            output: [0.0; 1024],
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
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.dsp.init(sample_rate as i32);
        }
        
        for chunk in buffer.chunks_mut(256) {
            let chunk_size = chunk.len();
            
            for i in 0..chunk_size {
                self.output[i] = 0.0;
            }
            
            let inputs: [&[f32]; 0] = [];
            let mut outputs = [&mut self.output[..chunk_size]];
            
            self.dsp.compute(chunk_size, &inputs, &mut outputs);
            
            for (i, frame) in chunk.iter_mut().enumerate() {
                let sample = self.output[i] * AMP_CALIBRATION;
                frame.left = sample;
                frame.right = sample;
            }
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