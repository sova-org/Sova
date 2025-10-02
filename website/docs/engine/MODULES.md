# Sova Engine Module Reference

Complete reference documentation for all audio modules in the Sova Engine.

## Table of Contents

- [Source Modules](#source-modules)
- [Local Effects](#local-effects)
- [Global Effects](#global-effects)
- [Engine Parameters](#engine-parameters)

---

## Source Modules

Audio generators and oscillators for creating sound.

### sine

**Pure sine wave oscillator with sub-oscillator**

Efficient wavetable-based sine wave generator with an optional sub-oscillator one octave below.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Oscillator frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | - | 0.0 - 1.0 | 0.0 | Sub-oscillator mix (one octave down) |

---

### saw

**Sawtooth oscillator with sub-oscillator**

PolyBLEP-based sawtooth wave with anti-aliasing and optional sub-oscillator.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Oscillator frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | - | 0.0 - 1.0 | 0.0 | Sub-oscillator mix (one octave down) |

---

### square

**Square wave oscillator with variable duty cycle**

Square wave generator with adjustable duty cycle and sub-oscillator.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Oscillator frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | - | 0.0 - 1.0 | 0.0 | Sub-oscillator mix (one octave down) |
| `z2` | - | 0.01 - 0.99 | 0.5 | Duty cycle (0.5 = 50% square wave) |

---

### triangle

**Triangle wave oscillator with sub-oscillator**

Smooth triangle wave generator with integrated sawtooth approach.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Oscillator frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | - | 0.0 - 1.0 | 0.0 | Sub-oscillator mix (one octave down) |

---

### dsine

**Detuned stereo sine oscillator**

True stereo sine oscillator with detuned oscillator pair and wobble modulation for rich, chorused tones.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Oscillator frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | `detune` | 0.0 - 10.0 | 1.0 | Detune amount between left/right oscillators |
| `z2` | `wobble` | 0.0 - 1.0 | 0.3 | LFO-based detune wobble amount |

---

### dsaw

**Detuned stereo sawtooth oscillator**

True stereo sawtooth oscillator with PolyBLEP anti-aliasing, detuned pair, and wobble modulation.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Oscillator frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | `detune` | 0.0 - 10.0 | 1.0 | Detune amount between left/right oscillators |
| `z2` | `wobble` | 0.0 - 1.0 | 0.3 | LFO-based detune wobble amount |

---

### dsquare

**Detuned stereo square oscillator**

True stereo square oscillator with 50% duty cycle, detuned pair, and wobble modulation.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Oscillator frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | `detune` | 0.0 - 10.0 | 1.0 | Detune amount between left/right oscillators |
| `z2` | `wobble` | 0.0 - 1.0 | 0.3 | LFO-based detune wobble amount |

---

### dtriangle

**Detuned stereo triangle oscillator**

True stereo triangle oscillator with smooth edges, detuned pair, and wobble modulation for warm tones.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Oscillator frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | `detune` | 0.0 - 10.0 | 1.0 | Detune amount between left/right oscillators |
| `z2` | `wobble` | 0.0 - 1.0 | 0.3 | LFO-based detune wobble amount |

---

### sinefm

**2-operator FM synthesis oscillator**

Classic FM synthesis with carrier and modulator oscillators for complex harmonic timbres.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Carrier frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | `index` | 0.0 - 10.0 | 1.0 | FM modulation index (brightness) |
| `z2` | `ratio` | 0.25 - 8.0 | 1.0 | Modulator frequency ratio (harmonic content) |

---

### noise

**White noise generator**

High-quality white noise using deterministic LCG algorithm.

**No parameters**

---

### wave

**Wavetable oscillator**

Wavetable synthesis with smooth interpolation between waveforms.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `frequency` | `freq` | 20.0 - 20000.0 Hz | 440.0 | Oscillator frequency in Hz |
| `note` | - | 0.0 - 127.0 | 69.0 | MIDI note number (overrides frequency) |
| `z1` | - | 0.0 - 99.0 | 0.0 | Wavetable interpolation index |

---

### sample

**Stereo sampler with pitch control**

Versatile sampler supporting variable playback speed, looping, start/end points, and MIDI note-based pitch control.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `sample_name` | `sn`, `folder`, `fd` | - | - | Sample folder name (e.g., 'kick', 'bass') |
| `sample_number` | `sp`, `nb` | 0.0 - 9999.0 | 0.0 | Sample index (supports float for crossfading) |
| `speed` | - | -999.0 - 999.0 | 1.0 | Playback speed (1.0=normal, -1.0=reverse, 2.0=double) |
| `begin` | - | 0.0 - 1.0 | 0.0 | Sample start position (0.0=start, 1.0=end) |
| `end` | - | 0.0 - 1.0 | 1.0 | Sample end position |
| `loop` | - | 0.0 - 1.0 | 0.0 | Loop enable (1.0=loop, 0.0=one-shot) |
| `note` | `n`, `midi` | 0.0 - 127.0 | 60.0 | MIDI note for pitch (overrides speed) |
| `root_note` | `root` | 0.0 - 127.0 | 60.0 | Original pitch reference for note transposition |

---

## Local Effects

Per-voice effects applied before track mixing.

### lowpass

**Moog ladder-style low-pass filter**

Warm low-pass filter capable of self-oscillation at high resonance.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `lpf` | - | 20.0 - 20000.0 Hz | 1000.0 | Cutoff frequency |
| `lpq` | - | 0.0 - 1.0 | 0.0 | Resonance (internally scaled 0.0-4.0 for self-oscillation) |

---

### highpass

**High-pass filter with resonance**

Biquad-based high-pass filter for removing low frequencies.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `hpf` | - | 20.0 - 20000.0 Hz | 1000.0 | Cutoff frequency |
| `hpq` | - | 0.0 - 1.0 | 0.0 | Resonance (internally scaled 0.707-15.0 Q) |

---

### bandpass

**Band-pass filter**

Isolates a frequency band with adjustable center frequency and bandwidth.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `bpf` | - | 20.0 - 20000.0 Hz | 1000.0 | Center frequency |
| `bpq` | - | 0.0 - 1.0 | 0.0 | Bandwidth/resonance (higher=narrower, scaled 0.707-30.0 Q) |

---

### notch

**Notch filter**

Removes a narrow frequency band for precise frequency elimination.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `npf` | - | 20.0 - 20000.0 Hz | 1000.0 | Center frequency |
| `npq` | - | 0.0 - 1.0 | 0.0 | Notch width (higher=narrower/deeper, scaled 0.707-30.0 Q) |

---

### bitcrusher

**Digital degradation effect**

Reduces bit depth and sample rate for lo-fi digital distortion.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `bits` | - | 2.0 - 32.0 | 16.0 | Bit depth reduction |
| `rate` | - | 0.0 - 1.0 | 1.0 | Sample rate reduction factor (1.0=full rate) |

---

### saturation

**Soft-clipping saturation**

Warm analog-style distortion using fast soft-clipping algorithm.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `drive` | - | 0.0 - 1.0 | 0.0 | Saturation drive amount (internally scaled 1.0-20.0) |

---

### tremolo

**Amplitude modulation effect**

Rhythmic volume variations using LFO modulation.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `tdepth` | - | 0.0 - 1.0 | 0.5 | Modulation depth |
| `trate` | - | 0.1 - 20.0 Hz | 5.0 | Modulation rate |

---

### ringmod

**Ring modulation effect**

Multiplies signal with sine carrier for metallic/inharmonic timbres.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `ringfreq` | `rfreq` | 0.01 - 1000.0 Hz | 5.0 | Carrier frequency |
| `ringdepth` | `rdepth` | 0.0 - 1.0 | 1.0 | Effect depth/mix |

---

### flanger

**Classic flanger effect**

Sweeping comb filter effect using modulated delay line.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `fdepth` | - | 0.0 - 0.01 s | 0.003 | Delay modulation depth |
| `frate` | - | 0.01 - 10.0 Hz | 0.5 | LFO rate |
| `ffeedback` | - | 0.0 - 1.0 | 0.5 | Feedback amount (internally scaled to 0.4 max) |
| `fmix` | - | 0.0 - 1.0 | 0.5 | Dry/wet mix |

---

### phaser

**Multi-stage phaser effect**

Cascaded all-pass filters with LFO modulation for sweeping notches.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `pdepth` | - | 0.0 - 1.0 | 0.5 | Modulation depth |
| `prate` | - | 0.01 - 10.0 Hz | 0.5 | LFO rate |
| `pfeedback` | - | 0.0 - 1.0 | 0.7 | Feedback amount (internally scaled to 0.7 max) |
| `pstages` | - | 2.0 - 8.0 | 4.0 | Number of all-pass stages (not modulable) |
| `pmix` | - | 0.0 - 1.0 | 0.5 | Dry/wet mix |

---

## Global Effects

Track-level effects applied to mixed voice output.

### echo

**Echo/Delay with feedback**

Classic delay effect with feedback and integrated tone control.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `echodur` | - | 0.01 - 2.0 s | 0.25 | Delay time |
| `echofb` | - | 0.0 - 0.98 | 0.3 | Feedback amount |
| `echolpf` | - | 100.0 - 15000.0 Hz | 4000.0 | Tone control (low-pass cutoff) |

Features: DC blocking, soft limiting, automatic silence detection

---

### reverb

**Freeverb reverb algorithm**

Classic algorithmic reverb using parallel comb and series all-pass filters.

| Parameter | Aliases | Range | Default | Description |
|-----------|---------|-------|---------|-------------|
| `size` | - | 0.0 - 1.0 | 0.5 | Room size/decay time |
| `width` | - | 0.0 - 1.0 | 0.5 | Stereo width |
| `freeze` | - | 0.0 - 1.0 | 0.0 | Freeze mode (1.0=infinite reverb) |

---

## Engine Parameters

Core parameters available on all voices.

| Parameter | Aliases | Range | Default | Modulable | Description |
|-----------|---------|-------|---------|-----------|-------------|
| `amp` | `amplitude` | 0.0 - 1.0 | 0.8 | ✓ | Voice amplitude |
| `pan` | - | -1.0 - 1.0 | 0.0 | ✓ | Stereo position (-1=left, 0=center, 1=right) |
| `attack` | `atk`, `a` | 0.0 - 10.0 s | 0.01 | ✓ | ADSR envelope attack time |
| `decay` | `dec`, `d` | 0.0 - 10.0 s | 0.1 | ✓ | ADSR envelope decay time |
| `sustain` | `sus` | 0.0 - 1.0 | 0.7 | ✓ | ADSR envelope sustain level |
| `release` | `rel`, `r` | 0.0 - 10.0 s | 0.1 | ✓ | ADSR envelope release time |
| `dur` | `duration` | 0.0 - ∞ s | 1.0 | ✓ | Voice duration (0.0=infinite until release) |
| `attack_curve` | `atk_curve`, `ac` | 0.0 - 1.0 | 0.5 | ✓ | Attack curve shape (0=linear, 1=exponential) |
| `decay_curve` | `dec_curve`, `dc` | 0.0 - 1.0 | 0.5 | ✓ | Decay curve shape (0=linear, 1=exponential) |
| `release_curve` | `rel_curve`, `rc` | 0.0 - 1.0 | 0.5 | ✓ | Release curve shape (0=linear, 1=exponential) |
| `track` | `t`, `trk` | 0.0 - 15.0 | 0.0 | ✗ | Audio track assignment for routing |

---

## Module Summary

- **12 Source Modules**: sine, saw, square, triangle, dsine, dsaw, dsquare, dtriangle, sinefm, noise, wave, sample
- **10 Local Effects**: lowpass, highpass, bandpass, notch, bitcrusher, saturation, tremolo, ringmod, flanger, phaser
- **2 Global Effects**: echo, reverb
- **11 Engine Parameters**: amp, pan, attack, decay, sustain, release, dur, attack_curve, decay_curve, release_curve, track

---

## Naming Conventions

- **Detuned oscillators (d-prefix)**: Use `z1` for detune amount and `z2` for wobble modulation
- **Basic oscillators**: Use `z1` for sub-oscillator mix (one octave down)
- **Filter parameters**: Named by type (lpf, hpf, bpf, npf) with corresponding Q (lpq, hpq, etc.)
- **Effect parameters**: Prefixed by effect type (f=flanger, p=phaser, t=tremolo, etc.)
- **Note parameters**: MIDI note numbers override frequency parameters when specified

## Modulation Syntax

Parameters marked as modulable support real-time modulation using the syntax: `param:lfo:depth:rate`

Example: `lpf:1000:500:2` modulates lowpass cutoff from 500Hz to 1500Hz at 2Hz rate
