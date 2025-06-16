
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopePhase {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvelopeParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub attack_curve: f32,
    pub decay_curve: f32,
    pub release_curve: f32,
}

impl Default for EnvelopeParams {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
            attack_curve: 0.3,
            decay_curve: 0.3,
            release_curve: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvelopeState {
    pub phase: EnvelopePhase,
    pub current_level: f32,
    pub phase_time: f32,
    pub gate_open: bool,
    pub gate_time: f32,
    pub duration: f32,
    pub release_start_level: f32,
}

impl Default for EnvelopeState {
    fn default() -> Self {
        Self {
            phase: EnvelopePhase::Idle,
            current_level: 0.0,
            phase_time: 0.0,
            gate_open: false,
            gate_time: 0.0,
            duration: 1.0,
            release_start_level: 0.0,
        }
    }
}

impl EnvelopeState {
    #[inline]
    pub fn trigger(&mut self) {
        if self.phase != EnvelopePhase::Idle {
            self.retrigger();
        } else {
            self.phase = EnvelopePhase::Attack;
            self.phase_time = 0.0;
            self.gate_open = true;
            self.gate_time = 0.0;
            self.current_level = 0.0;
            self.release_start_level = 0.0;
        }
    }

    #[inline]
    pub fn retrigger(&mut self) {
        self.phase = EnvelopePhase::Attack;
        self.phase_time = 0.0;
        self.gate_open = true;
        self.gate_time = 0.0;
        self.current_level = 0.0;
        self.release_start_level = 0.0;
    }

    #[inline]
    pub fn release(&mut self) {
        if self.gate_open && !matches!(self.phase, EnvelopePhase::Release) {
            self.gate_open = false;
            self.release_start_level = self.current_level;
            self.phase = EnvelopePhase::Release;
            self.phase_time = 0.0;
        }
    }

    #[inline]
    pub fn set_idle(&mut self) {
        self.phase = EnvelopePhase::Idle;
        self.current_level = 0.0;
        self.phase_time = 0.0;
        self.gate_open = false;
        self.gate_time = 0.0;
        self.duration = 1.0;
        self.release_start_level = 0.0;
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        matches!(self.phase, EnvelopePhase::Idle) || 
        (matches!(self.phase, EnvelopePhase::Release) && self.current_level <= 0.001)
    }

    #[inline]
    pub fn scale_to_duration(&mut self, dur: f32) {
        self.duration = dur.max(0.001);
    }
}

pub struct Envelope;

impl Envelope {
    #[inline]
    fn curve_transform(t: f32, curve: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let curve = curve.clamp(0.001, 0.999);
        
        if curve < 0.5 {
            let factor = curve * 2.0;
            t * (1.0 + factor * (1.0 - t))
        } else {
            let factor = (curve - 0.5) * 2.0;
            let inv = 1.0 - t;
            1.0 - inv * (1.0 + factor * t)
        }
    }

    #[inline]
    fn flush_denormals(x: f32) -> f32 {
        const DENORMAL_THRESHOLD: f32 = 1e-15;
        if x.abs() < DENORMAL_THRESHOLD { 0.0 } else { x }
    }

    #[inline]
    fn update_envelope_state(params: &EnvelopeParams, state: &mut EnvelopeState, dt: f32) {
        if state.gate_open {
            state.gate_time += dt;
            if state.gate_time >= state.duration {
                state.release();
            }
        }

        state.phase_time += dt;

        match state.phase {
            EnvelopePhase::Idle => {
                state.current_level = 0.0;
            }
            EnvelopePhase::Attack => {
                if params.attack <= 0.001 {
                    state.current_level = 1.0;
                    state.phase = EnvelopePhase::Decay;
                    state.phase_time = 0.0;
                } else {
                    let progress = (state.phase_time / params.attack).clamp(0.0, 1.0);
                    if progress >= 1.0 {
                        state.current_level = 1.0;
                        state.phase = EnvelopePhase::Decay;
                        state.phase_time = 0.0;
                    } else {
                        let curve_val = Self::curve_transform(progress, params.attack_curve);
                        state.current_level = curve_val;
                    }
                }
            }
            EnvelopePhase::Decay => {
                if params.decay <= 0.001 {
                    state.current_level = params.sustain;
                    state.phase = EnvelopePhase::Sustain;
                    state.phase_time = 0.0;
                } else {
                    let progress = (state.phase_time / params.decay).clamp(0.0, 1.0);
                    if progress >= 1.0 {
                        state.current_level = params.sustain;
                        state.phase = EnvelopePhase::Sustain;
                        state.phase_time = 0.0;
                    } else {
                        let curve_val = Self::curve_transform(progress, params.decay_curve);
                        state.current_level = 1.0 - curve_val * (1.0 - params.sustain);
                    }
                }
            }
            EnvelopePhase::Sustain => {
                state.current_level = params.sustain;
            }
            EnvelopePhase::Release => {
                if params.release <= 0.001 {
                    state.current_level = 0.0;
                    state.phase = EnvelopePhase::Idle;
                    state.phase_time = 0.0;
                } else {
                    let progress = (state.phase_time / params.release).clamp(0.0, 1.0);
                    if progress >= 1.0 {
                        state.current_level = 0.0;
                        state.phase = EnvelopePhase::Idle;
                        state.phase_time = 0.0;
                    } else {
                        let curve_val = Self::curve_transform(progress, params.release_curve);
                        state.current_level = state.release_start_level * (1.0 - curve_val);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn process_block(params: &EnvelopeParams, state: &mut EnvelopeState, buffer: &mut [f32], sample_rate: f32) {
        let dt = 1.0 / sample_rate;
        
        for sample in buffer.iter_mut() {
            Self::update_envelope_state(params, state, dt);
            *sample = Self::flush_denormals(state.current_level.clamp(0.0, 1.0));
        }
    }

    #[inline]
    pub fn get_amplitude(params: &EnvelopeParams, state: &mut EnvelopeState, dt: f32) -> f32 {
        if matches!(state.phase, EnvelopePhase::Idle) {
            return 0.0;
        }
        
        Self::update_envelope_state(params, state, dt);
        Self::flush_denormals(state.current_level.clamp(0.0, 1.0))
    }
}