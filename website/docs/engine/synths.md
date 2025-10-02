Each synthesis voice starts with a generator. A generator can be anything from a simple waveform oscillator to a complex sampler. The generated signal is then processed by a series of effects before being sent to the output. By default, the engine provides a set of basic generators and effects. New modules can be added as needed, provided they adhere to the defined interface.

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
