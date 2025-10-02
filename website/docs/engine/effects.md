
Each source can be processed by a chain of effects before being sent to the output. By default, the engine provides a set of basic effects. There are two types of effects distinguished by their position in the chain and by the amount of voices they process:
- **Local effects**: applied to each voice individually, before mixing. One instance per voice.
- **Global effects**: applied to the mixed output of all voices in a track. One instance per track.

Just like sources, new effects can be added as needed, provided they adhere to the defined interface.

## Local Effects

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

