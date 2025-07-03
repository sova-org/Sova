use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use crate::dsp::oscillators::SquareOscillator as DSPSquare;
use std::any::Any;

/// Simple square oscillator with frequency control and sub-oscillator
/// 
/// Pure Rust implementation using efficient square wave generation.
/// Features zero-allocation real-time processing and automatic engine parameter detection.
/// Includes intelligent sub-oscillator mixing controlled by z1 parameter.
pub struct SquareOscillator {
    /// Core square oscillator
    osc: DSPSquare,
    
    /// Sub-oscillator (one octave down)
    sub_osc: DSPSquare,
    
    /// Current sample rate (detected from engine)
    sample_rate: f32,
    
    /// Parameters
    frequency: f32,
    note: Option<f32>,
    z1: f32,
    z2: f32,
    
    /// State tracking
    initialized: bool,
    params_dirty: bool,
}

impl SquareOscillator {
    /// Creates a new square oscillator
    pub fn new() -> Self {
        Self {
            osc: DSPSquare::new(),
            sub_osc: DSPSquare::new(),
            sample_rate: 0.0,
            frequency: 440.0,
            note: None,
            z1: 0.0,
            z2: 0.5,
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
            self.osc.set_duty_cycle(self.z2);
            
            // Only set up sub-oscillator if z1 > 0.0
            if self.z1 > 0.0 {
                self.sub_osc.set_frequency(frequency * 0.5, self.sample_rate);
                self.sub_osc.set_duty_cycle(self.z2);
            }
            
            self.params_dirty = false;
        }
    }
}

impl AudioModule for SquareOscillator {
    fn get_name(&self) -> &'static str {
        "square"
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
            "z2" => {
                let new_z2 = value.clamp(0.01, 0.99);
                if self.z2 != new_z2 {
                    self.z2 = new_z2;
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

impl Source for SquareOscillator {
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

static PARAMETER_DESCRIPTORS: [ParameterDescriptor; 4] = [
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
    ParameterDescriptor {
        name: "z2",
        aliases: &[],
        min_value: 0.01,
        max_value: 0.99,
        default_value: 0.5,
        unit: "",
        description: "Duty cycle (0.5 = square wave)",
        modulable: true,
    },
];

impl ModuleMetadata for SquareOscillator {
    fn get_static_name() -> &'static str {
        "square"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }
}

/// Creates a new square oscillator instance
pub fn create_square_oscillator() -> Box<dyn Source> {
    Box::new(SquareOscillator::new())
}