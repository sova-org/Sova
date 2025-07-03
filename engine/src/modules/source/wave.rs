use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use crate::dsp::wavetables::WavetableOscillator;
use std::any::Any;

pub struct WaveOscillator {
    osc: WavetableOscillator,
    sample_rate: f32,
    frequency: f32,
    note: Option<f32>,
    z1: f32,
    initialized: bool,
    params_dirty: bool,
}

impl WaveOscillator {
    pub fn new() -> Self {
        Self {
            osc: WavetableOscillator::new(),
            sample_rate: 0.0,
            frequency: 440.0,
            note: None,
            z1: 0.0,
            initialized: false,
            params_dirty: true,
        }
    }

    fn initialize(&mut self, sample_rate: f32) {
        if !self.initialized || self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.initialized = true;
            self.params_dirty = true;
        }
    }

    fn update_params(&mut self) {
        if self.params_dirty {
            let frequency = if let Some(note) = self.note {
                crate::dsp::math::midi_to_freq(note)
            } else {
                self.frequency
            };
            
            self.osc.set_frequency(frequency, self.sample_rate);
            self.osc.set_wavetable_index(self.z1);
            self.params_dirty = false;
        }
    }
}

impl AudioModule for WaveOscillator {
    fn get_name(&self) -> &'static str {
        "wave"
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
                let new_z1 = value.clamp(0.0, (self.osc.get_num_wavetables() - 1) as f32);
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

impl Source for WaveOscillator {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        self.initialize(sample_rate);
        self.update_params();
        
        for frame in buffer.iter_mut() {
            let sample = self.osc.next_sample();
            frame.left = sample;
            frame.right = sample;
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
        max_value: 99.0,
        default_value: 0.0,
        unit: "",
        description: "Wavetable interpolation index",
        modulable: true,
    },
];

impl ModuleMetadata for WaveOscillator {
    fn get_static_name() -> &'static str {
        "wave"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }
}

pub fn create_wave_oscillator() -> Box<dyn Source> {
    Box::new(WaveOscillator::new())
}