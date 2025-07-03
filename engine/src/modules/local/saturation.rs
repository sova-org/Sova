use crate::dsp::math::fast_soft_clip;
use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_DRIVE: &str = "drive";

const DEFAULT_DRIVE: f32 = 0.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_DRIVE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DRIVE,
        unit: "",
        description: "Saturation drive amount",
        modulable: true,
    },
];

pub struct Saturation {
    drive: f32,
    is_active: bool,
}

impl Default for Saturation {
    fn default() -> Self {
        Self::new()
    }
}

impl Saturation {
    pub fn new() -> Self {
        Self {
            drive: DEFAULT_DRIVE,
            is_active: true,
        }
    }
}

impl AudioModule for Saturation {
    fn get_name(&self) -> &'static str {
        "saturation"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_DRIVE => {
                self.drive = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl LocalEffect for Saturation {
    fn process(&mut self, buffer: &mut [Frame], _sample_rate: f32) {
        // Scale 0.0-1.0 to 1.0-20.0
        let actual_drive = 1.0 + self.drive * 19.0;
        
        for frame in buffer.iter_mut() {
            frame.left = fast_soft_clip(frame.left, actual_drive);
            frame.right = fast_soft_clip(frame.right, actual_drive);
        }
    }
}

impl ModuleMetadata for Saturation {
    fn get_static_name() -> &'static str {
        "saturation"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_saturation() -> Box<dyn LocalEffect> {
    Box::new(Saturation::new())
}
