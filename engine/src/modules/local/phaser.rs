use crate::modules::{AudioModule, Frame, LocalEffect, ParameterDescriptor};

const PARAM_RATE: &str = "phaser_rate";
const PARAM_DEPTH: &str = "phaser_depth";
const PARAM_FEEDBACK: &str = "phaser_feedback";
const PARAM_BASSCUT: &str = "phaser_basscut";
const PARAM_PHASE: &str = "phaser_phase";
const PARAM_COLOR: &str = "phaser_color";

const DEFAULT_RATE: f32 = 0.2;
const DEFAULT_DEPTH: f32 = 75.0;
const DEFAULT_FEEDBACK: f32 = 75.0;
const DEFAULT_BASSCUT: f32 = 500.0;
const DEFAULT_PHASE: f32 = 0.0;
const DEFAULT_COLOR: f32 = 1.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_RATE,
        aliases: &["phrt"],
        min_value: 0.01,
        max_value: 5.0,
        default_value: DEFAULT_RATE,
        unit: "Hz",
        description: "LFO rate",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_DEPTH,
        aliases: &["phde"],
        min_value: 0.0,
        max_value: 99.0,
        default_value: DEFAULT_DEPTH,
        unit: "%",
        description: "Effect depth",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_FEEDBACK,
        aliases: &["phfb"],
        min_value: 0.0,
        max_value: 99.0,
        default_value: DEFAULT_FEEDBACK,
        unit: "%",
        description: "Feedback amount",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_BASSCUT,
        aliases: &["phbc"],
        min_value: 10.0,
        max_value: 5000.0,
        default_value: DEFAULT_BASSCUT,
        unit: "Hz",
        description: "Feedback bass cut",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_PHASE,
        aliases: &["phph"],
        min_value: -180.0,
        max_value: 180.0,
        default_value: DEFAULT_PHASE,
        unit: "deg",
        description: "Stereo phase",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_COLOR,
        aliases: &["phco"],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_COLOR,
        unit: "",
        description: "Color mode",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "stone_phaser";
    declare author "Jean Pierre Cimalando";
    declare version "1.2.2";
    declare license "CC0-1.0 or BSL-1.0";

    // Référence :
    //     Kiiski, R., Esqueda, F., & Välimäki, V. (2016).
    //     Time-variant gray-box modeling of a phaser pedal.
    //     In 19th International Conference on Digital Audio Effects (DAFx-16).

    import("stdfaust.lib");

    /////////////
    // Control //
    /////////////

    color = hslider("[0] phaser_color", 1, 0, 1, 1);
    lf = hslider("[1] phaser_rate [unit:Hz] [scale:log]", 0.2, 0.01, 5., 0.01) : tsmooth;
    fb = hslider("[2] phaser_feedback [unit:%] [integer]", 75, 0, 99, 1) : *(0.01) : tsmooth;
    fbHf = hslider("[3] phaser_basscut [unit:Hz] [scale:log]", 500., 10., 5000., 1.) : tsmooth;
    ph = hslider("[4] phaser_phase [unit:deg] [integer]", 0., -180., +180., 1.) : /(360.) : +(1.) : tsmooth;

    //////////////////////////
    // All-pass filter unit //
    //////////////////////////

    allpass1(f) = fi.iir((a,1.),(a)) with {
      a = -1.+2.*ma.PI*f/ma.SR;
    };

    //////////////////////
    // High-pass filter //
    //////////////////////

    highpass1(f) = fi.iir((0.5*(1.+p), -0.5*(1.+p)), (-p)) with {
      p = exp(-2.*ma.PI*f/ma.SR);
    };

    //////////////////////
    // Low-pass filter //
    //////////////////////

    lowpass1(f) = fi.iir((1.-p), (-p)) with {
      p = exp(-2.*ma.PI*f/ma.SR);
    };

    ////////////////////////////////////////////
    // Smooth filter with fixed time constant //
    ////////////////////////////////////////////

    tsmooth = si.smooth(ba.tau2pole(t)) with { t = 100e-3; };

    //////////
    // LFOs //
    //////////

    lfoTriangle(pos, y1, y2) = val*(y2-y1)+y1 with {
      val = 1.-abs(2.*pos-1.);
    };

    lfoRectifiedSine(pos, y1, y2) = val*(y2-y1)+y1 with {
      val = rsin(pos);
    };

    lfoAnalogTriangle(roundness, pos, y1, y2) = val*(y2-y1)+y1 with {
      val = sineTri(roundness, pos);
    };

    lfoExponentialTriangle(roundness, slopeUp, slopeDown, pos, y1, y2) = val*(y2-y1)+y1 with {
      val = expTri(roundness, slopeUp, slopeDown, pos);
    };

    ////////////
    // Phaser //
    ////////////

    mono_phaser(x, lfo_pos) = (x <: highpass1(33.0) : (+:a1:a2:a3:a4)~feedback)
    with {

      colorFb = ba.if(color, fb, 0.1*fb) : tsmooth;
      feedback = highpass1(fbHf) : *(colorFb);

      lfoLoF = ba.if(color, ba.hz2midikey(80.), ba.hz2midikey(300.)) : tsmooth;
      lfoHiF = ba.if(color, ba.hz2midikey(2200.), ba.hz2midikey(6000.)) : tsmooth;

      modFreq = ba.midikey2hz(lfoAnalogTriangle(0.95, lfo_pos, lfoLoF, lfoHiF));
      //modFreq = ba.midikey2hz(lfoExponentialTriangle(128., 0.6, 0.9, lfo_pos, lfoLoF, lfoHiF));

      a1 = allpass1(modFreq);
      a2 = allpass1(modFreq);
      a3 = allpass1(modFreq);
      a4 = allpass1(modFreq);
    };

    stereo_phaser(x1, x2, lfo_pos) =
      left_phaser, right_phaser
    with {
      left_phaser = mono_phaser(x1, lfo_pos);
      right_phaser = mono_phaser(x2, lfo_pos2);
      lfo_pos2 = wrap(lfo_pos + ph);
      wrap(p) = p-float(int(p));
    };

    /////////////
    // Utility //
    /////////////

    lerp(tab, pos, size) = (tab(i1), tab(i2)) : si.interpolate(mu) with {
      fracIndex = max(0, min(size-1, pos*size));
      i1 = int(fracIndex);
      i2 = (i1+1)%size;
      mu = fracIndex-float(i1);
    };

    rsin(pos) = lerp(tab, pos, ts) with {
      ts = 128;
      tab(i) = rdtable(ts, abs(os.sinwaveform(ts)), int(i) % ts);
    };

    sineTriWaveform(roundness, tablesize) = 1.-sin(2.*ba.if(x<0.5, x, 1.-x)*asin(a))/a with {
      a = max(0., min(1., roundness * 0.5 + 0.5));
      x = wrap(float(ba.time)/float(tablesize));
      wrap(p) = p-float(int(p));
    };

    sineTri(roundness, pos) = lerp(tab, pos, ts) with {
      ts = 128;
      tab(i) = rdtable(ts, sineTriWaveform(roundness, ts), int(i) % ts);
    };

    /*
      # Gnuplot code of the sineTri function
      sineTri(r, x)=sineTri_(r, wrap(x+0.5))
      sineTri_(r, x)=1.-sin(((x<0.5)?x:(1.-x))*2.*asin(r))/r
      wrap(x)=x-floor(x)
      set xrange [0:1]
      plot(sineTri(0.99, x))
    */

    expTriWaveform(roundness, slopeUp, slopeDown, tablesize) = ba.if(x<0.5, expUp, expDown) with {
      normExp(a, b, x) = (1.-pow(a, -b*x))/(1.-pow(a, -b));
      expUp = 1.-normExp(roundness, slopeUp, (-x+0.5)*2);
      expDown = 1.-normExp(roundness, slopeDown, (x-0.5)*2);
      x = wrap(float(ba.time)/float(tablesize));
      wrap(p) = p-float(int(p));
    };

    expTri(roundness, slopeUp, slopeDown, pos) = lerp(tab, pos, ts) with {
      ts = 128;
      tab(i) = rdtable(ts, expTriWaveform(roundness, slopeUp, slopeDown, ts), int(i) % ts);
    };

    /*
      # Gnuplot code of the expTri function
      roundness=128
      slopeUp = 0.6
      slopeDown = 0.9
      normExp(a,b,x)=(1.-a**-(b*x))/(1.-a**-b)
      set xrange [0:1]
      plot (x<0.5) ? (1.-normExp(roundness, slopeUp, (-x+0.5)*2)) : (1.-normExp(roundness, slopeDown, (x-0.5)*2))
    */

    //////////
    // Main //
    //////////

    process_mono(x) = mono_phaser(x, os.lf_sawpos(lf));
    process_stereo(x1, x2) = stereo_phaser(x1, x2, os.lf_sawpos(lf));
    process = process_stereo;
);

pub struct Phaser {
    rate: f32,
    depth: f32,
    feedback: f32,
    basscut: f32,
    phase: f32,
    color: f32,
    faust_processor: stone_phaser::StonePhaser,
    sample_rate: f32,
    is_active: bool,
    left_input: [f32; 1024],
    right_input: [f32; 1024],
    left_output: [f32; 1024],
    right_output: [f32; 1024],
}

impl Default for Phaser {
    fn default() -> Self {
        Self::new()
    }
}

impl Phaser {
    pub fn new() -> Self {
        let mut faust_processor = stone_phaser::StonePhaser::new();
        faust_processor.init(44100);

        Self {
            rate: DEFAULT_RATE,
            depth: DEFAULT_DEPTH,
            feedback: DEFAULT_FEEDBACK,
            basscut: DEFAULT_BASSCUT,
            phase: DEFAULT_PHASE,
            color: DEFAULT_COLOR,
            faust_processor,
            sample_rate: 44100.0,
            is_active: true,
            left_input: [0.0; 1024],
            right_input: [0.0; 1024],
            left_output: [0.0; 1024],
            right_output: [0.0; 1024],
        }
    }

    fn update_faust_params(&mut self) {
        self.faust_processor
            .set_param(faust_types::ParamIndex(0), self.color);
        self.faust_processor
            .set_param(faust_types::ParamIndex(1), self.rate);
        self.faust_processor
            .set_param(faust_types::ParamIndex(2), self.feedback);
        self.faust_processor
            .set_param(faust_types::ParamIndex(3), self.basscut);
        self.faust_processor
            .set_param(faust_types::ParamIndex(4), self.phase);
    }
}

impl AudioModule for Phaser {
    fn get_name(&self) -> &'static str {
        "phaser"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_RATE => {
                self.rate = value.clamp(0.01, 5.0);
                self.update_faust_params();
                true
            }
            PARAM_DEPTH => {
                self.depth = value.clamp(0.0, 99.0);
                // Note: depth parameter is conceptually similar to feedback in this phaser design
                self.feedback = value.clamp(0.0, 99.0);
                self.update_faust_params();
                true
            }
            PARAM_FEEDBACK => {
                self.feedback = value.clamp(0.0, 99.0);
                self.update_faust_params();
                true
            }
            PARAM_BASSCUT => {
                self.basscut = value.clamp(10.0, 5000.0);
                self.update_faust_params();
                true
            }
            PARAM_PHASE => {
                self.phase = value.clamp(-180.0, 180.0);
                self.update_faust_params();
                true
            }
            PARAM_COLOR => {
                self.color = value.clamp(0.0, 1.0);
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

impl LocalEffect for Phaser {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.faust_processor.init(sample_rate as i32);
            self.update_faust_params();
        }

        for chunk in buffer.chunks_mut(256) {
            let chunk_size = chunk.len();

            for (i, frame) in chunk.iter().enumerate() {
                self.left_input[i] = frame.left;
                self.right_input[i] = frame.right;
                self.left_output[i] = 0.0;
                self.right_output[i] = 0.0;
            }

            let inputs = [
                &self.left_input[..chunk_size],
                &self.right_input[..chunk_size],
            ];
            let mut outputs = [
                &mut self.left_output[..chunk_size],
                &mut self.right_output[..chunk_size],
            ];

            self.faust_processor
                .compute(chunk_size, &inputs, &mut outputs);

            for (i, frame) in chunk.iter_mut().enumerate() {
                frame.left = self.left_output[i];
                frame.right = self.right_output[i];
            }
        }
    }
}

pub fn create_phaser() -> Box<dyn LocalEffect> {
    Box::new(Phaser::new())
}