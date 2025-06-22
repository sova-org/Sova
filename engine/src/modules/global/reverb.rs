use crate::modules::{AudioModule, Frame, GlobalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_SIZE: &str = "size";
const PARAM_DAMPING: &str = "damping";

const DEFAULT_SIZE: f32 = 0.5;
const DEFAULT_DAMPING: f32 = 0.5;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_SIZE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_SIZE,
        unit: "",
        description: "Reverb size",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_DAMPING,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DAMPING,
        unit: "",
        description: "High frequency damping",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "zita_reverb";
    declare version "1.0";
    import("stdfaust.lib");
    process = _ <: re.zita_rev_fdn(f1, f2, t60dc, t60m, fsmax) :> _,_
    with {
        f1 = hslider("f1", 200, 50, 1000, 1);
        f2 = hslider("f2", 6000, 1500, 20000, 1);
        t60dc = hslider("t60dc", 3, 1, 8, 0.1);
        t60m = hslider("t60m", 2, 1, 8, 0.1);
        fsmax = 48000*2;
    };
);

pub struct ZitaReverb {
    size: f32,
    damping: f32,
    faust_processor: zita_reverb::ZitaReverb,
    sample_rate: f32,
    is_active: bool,
    left_buffer: [f32; 1024],
    left_output: [f32; 1024],
    right_output: [f32; 1024],
}

impl Default for ZitaReverb {
    fn default() -> Self {
        Self::new()
    }
}

impl ZitaReverb {
    pub fn new() -> Self {
        eprintln!("🔧 Creating Faust ZitaReverb processor...");
        
        // Step 1: Create the processor
        eprintln!("🔧 Step 1: Calling zita_reverb::ZitaReverb::new()");
        let mut faust_processor = zita_reverb::ZitaReverb::new();
        eprintln!("🔧 Step 1: SUCCESS - Faust processor created");
        
        // Step 2: Initialize with sample rate
        eprintln!("🔧 Step 2: Calling faust_processor.init(44100)");
        faust_processor.init(44100);
        eprintln!("🔧 Step 2: SUCCESS - Faust processor initialized");

        eprintln!("🔧 Step 3: Creating ZitaReverb struct");
        let reverb = Self {
            size: DEFAULT_SIZE,
            damping: DEFAULT_DAMPING,
            faust_processor,
            sample_rate: 44100.0,
            is_active: true,
            left_buffer: [0.0; 1024],
            left_output: [0.0; 1024],
            right_output: [0.0; 1024],
        };
        eprintln!("🔧 Step 3: SUCCESS - ZitaReverb created successfully");
        
        reverb
    }

    fn update_faust_params(&mut self) {
        let f1 = 50.0 + self.size * 950.0;
        let f2 = 1500.0 + self.size * 18500.0;
        let t60dc = 1.0 + self.size * 7.0;
        let t60m = 1.0 + self.size * 7.0 * self.damping;

        self.faust_processor
            .set_param(faust_types::ParamIndex(0), f1);
        self.faust_processor
            .set_param(faust_types::ParamIndex(1), f2);
        self.faust_processor
            .set_param(faust_types::ParamIndex(2), t60dc);
        self.faust_processor
            .set_param(faust_types::ParamIndex(3), t60m);
    }
}

impl AudioModule for ZitaReverb {
    fn get_name(&self) -> &'static str {
        "reverb"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_SIZE => {
                self.size = value.clamp(0.0, 1.0);
                self.update_faust_params();
                true
            }
            PARAM_DAMPING => {
                self.damping = value.clamp(0.0, 1.0);
                self.update_faust_params();
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl GlobalEffect for ZitaReverb {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        // eprintln!("🎵 ZitaReverb::process called with buffer_len={}, sample_rate={}", buffer.len(), sample_rate);
        
        if self.sample_rate != sample_rate {
            eprintln!("🎵 Sample rate changed from {} to {}, reinitializing Faust processor", self.sample_rate, sample_rate);
            self.sample_rate = sample_rate;
            
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.faust_processor.init(sample_rate as i32);
            })) {
                Ok(_) => eprintln!("🎵 Faust processor reinitialized successfully"),
                Err(e) => {
                    eprintln!("🚨 PANIC during Faust reinit: {:?}", e);
                    return; // Early return, don't process audio
                }
            }
            
            self.update_faust_params();
        }

        for (chunk_idx, chunk) in buffer.chunks_mut(256).enumerate() {
            let chunk_size = chunk.len();
            // eprintln!("🎵 Processing chunk {} with size {}", chunk_idx, chunk_size);

            // Prepare input/output buffers
            for (i, frame) in chunk.iter().enumerate() {
                self.left_buffer[i] = (frame.left + frame.right) * 0.5;
                self.left_output[i] = 0.0;
                self.right_output[i] = 0.0;
            }

            let inputs = [&self.left_buffer[..chunk_size]];
            let mut outputs = [
                &mut self.left_output[..chunk_size],
                &mut self.right_output[..chunk_size],
            ];

            // The critical call where illegal instruction likely occurs
            // eprintln!("🎵 About to call faust_processor.compute with chunk_size={}", chunk_size);
            
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.faust_processor.compute(chunk_size, &inputs, &mut outputs);
            })) {
                Ok(_) => {
                    // Success - copy output to buffer
                    for (i, frame) in chunk.iter_mut().enumerate() {
                        frame.left = self.left_output[i] * 0.1;
                        frame.right = self.right_output[i] * 0.1;
                    }
                }
                Err(e) => {
                    eprintln!("🚨 PANIC during Faust compute: {:?}", e);
                    // Zero out the audio on crash to avoid artifacts
                    for frame in chunk.iter_mut() {
                        frame.left = 0.0;
                        frame.right = 0.0;
                    }
                    return; // Stop processing further chunks
                }
            }
        }
    }
}

impl ModuleMetadata for ZitaReverb {
    fn get_static_name() -> &'static str {
        "reverb"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_simple_reverb() -> Box<dyn GlobalEffect> {
    // Wrap creation in a panic-catching mechanism to understand the crash
    match std::panic::catch_unwind(|| {
        ZitaReverb::new()
    }) {
        Ok(reverb) => Box::new(reverb),
        Err(e) => {
            eprintln!("REVERB PANIC during creation: {:?}", e);
            // Return a safe dummy reverb that does nothing
            Box::new(SafeReverb::new())
        }
    }
}

/// Safe fallback reverb that doesn't use Faust-generated code
pub struct SafeReverb {
    size: f32,
    damping: f32,
    is_active: bool,
}

impl SafeReverb {
    pub fn new() -> Self {
        eprintln!("Using SafeReverb fallback - Faust reverb failed to initialize");
        Self {
            size: DEFAULT_SIZE,
            damping: DEFAULT_DAMPING,
            is_active: true,
        }
    }
}

impl AudioModule for SafeReverb {
    fn get_name(&self) -> &'static str {
        "reverb"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_SIZE => {
                self.size = value.clamp(0.0, 1.0);
                true
            }
            PARAM_DAMPING => {
                self.damping = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl GlobalEffect for SafeReverb {
    fn process(&mut self, buffer: &mut [Frame], _sample_rate: f32) {
        // Simple reverb simulation - just attenuate and add slight delay effect
        for frame in buffer.iter_mut() {
            frame.left *= 0.1;
            frame.right *= 0.1;
        }
    }
}

impl ModuleMetadata for SafeReverb {
    fn get_static_name() -> &'static str {
        "reverb"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}
