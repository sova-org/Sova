use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use crate::dsp::oscillators::TriangleOscillator as DSPTriangle;
use std::any::Any;

/// Simple triangle oscillator with frequency control and sub-oscillator
/// 
/// Pure Rust implementation using efficient integrated sawtooth approach.
/// Features zero-allocation real-time processing and automatic engine parameter detection.
/// Includes intelligent sub-oscillator mixing controlled by z1 parameter.
pub struct TriangleOscillator {
    /// Core triangle oscillator
    osc: DSPTriangle,
    
    /// Sub-oscillator (one octave down)
    sub_osc: DSPTriangle,
    
    /// Current sample rate (detected from engine)
    sample_rate: f32,
    
    /// Parameters
    frequency: f32,
    note: Option<f32>,
    z1: f32,
    
    /// State tracking
    initialized: bool,
    params_dirty: bool,
}

impl TriangleOscillator {
    /// Creates a new triangle oscillator
    pub fn new() -> Self {
        Self {
            osc: DSPTriangle::new(),
            sub_osc: DSPTriangle::new(),
            sample_rate: 0.0,
            frequency: 440.0,
            note: None,
            z1: 0.0,
            initialized: false,
            params_dirty: true,
        }
    }

    /// Initialize oscillator with engine parameters
    fn initialize(&mut self, sample_rate: f32) {
        if !self.initialized || self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.initialized = true;
            self.params_dirty = true;
        }
    }

    /// Update oscillator frequency when parameters change
    fn update_params(&mut self) {
        if self.params_dirty {
            let frequency = if let Some(note) = self.note {
                crate::dsp::math::midi_to_freq(note)
            } else {
                self.frequency
            };
            
            self.osc.set_frequency(frequency, self.sample_rate);
            
            // Only set up sub-oscillator if z1 > 0.0
            if self.z1 > 0.0 {
                self.sub_osc.set_frequency(frequency * 0.5, self.sample_rate);
            }
            
            self.params_dirty = false;
        }
    }
}

impl AudioModule for TriangleOscillator {
    fn get_name(&self) -> &'static str {
        "triangle"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        match name {
            "frequency" | "freq" => {
                let new_freq = value.clamp(20.0, 20000.0);
                if self.frequency != new_freq {
                    self.frequency = new_freq;
                    self.note = None;
                    self.params_dirty = true;
                }
                true
            }
            "note" => {
                let new_note = value.clamp(0.0, 127.0);
                if self.note != Some(new_note) {
                    self.note = Some(new_note);
                    self.params_dirty = true;
                }
                true
            }
            "z1" => {
                let new_z1 = value.clamp(0.0, 1.0);
                if self.z1 != new_z1 {
                    self.z1 = new_z1;
                    self.params_dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        true
    }
}

impl Source for TriangleOscillator {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        // Auto-detect and initialize with engine parameters
        self.initialize(sample_rate);
        
        // Update parameters if needed
        self.update_params();
        
        // Generate audio samples
        for frame in buffer.iter_mut() {
            let main_sample = self.osc.next_sample();
            
            let output = if self.z1 > 0.0 {
                let sub_sample = self.sub_osc.next_sample();
                main_sample * (1.0 - self.z1) + sub_sample * self.z1
            } else {
                main_sample
            };
            
            frame.left = output;
            frame.right = output;
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

static PARAMETER_DESCRIPTORS: [ParameterDescriptor; 3] = [
    ParameterDescriptor {
        name: "frequency",
        aliases: &["freq"],
        min_value: 20.0,
        max_value: 20000.0,
        default_value: 440.0,
        unit: "Hz",
        description: "Oscillator frequency in Hz",
        modulable: true,
    },
    ParameterDescriptor {
        name: "note",
        aliases: &[],
        min_value: 0.0,
        max_value: 127.0,
        default_value: 69.0,
        unit: "",
        description: "MIDI note number (overrides frequency)",
        modulable: true,
    },
    ParameterDescriptor {
        name: "z1",
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: 0.0,
        unit: "",
        description: "Sub-oscillator mix (one octave down)",
        modulable: true,
    },
];

impl ModuleMetadata for TriangleOscillator {
    fn get_static_name() -> &'static str {
        "triangle"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }
}

/// Creates a new triangle oscillator instance
pub fn create_triangle_oscillator() -> Box<dyn Source> {
    Box::new(TriangleOscillator::new())
}
