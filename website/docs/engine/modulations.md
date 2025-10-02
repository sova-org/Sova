# Modulation Reference

Most parameters can be modulated using a special string parsing system. This allows the generation of dynamic and evolving sounds, introducing oscillations, ramps, and sequences for controlling invidual values in a message to the engine. For instance, a filter can receive an envelope to create a sweeping effect, or a pitch parameter can be modulated with an LFO for vibrato. More complex modulations allow for rhythmic patterns and sample & hold effects.

## Modulation Types

### Static

The most basic type of modulation is a static value, which simply passes through the number as-is.

```
440.0
```

### Oscillator (LFO)

Each parameter can be modulated with a low-frequency oscillator (LFO) to create periodic changes. There is a special syntax to define the oscillator's characteristics: `osc:base:depth:rate:shape:duration`. Here are the components:
- `base` - center value
- `depth` - deviation amount (±depth)
- `rate` - frequency in Hz
- `shape` - waveform: `sine`, `triangle`, `saw`, `square`, `noise`
- `duration` - total time (0 = infinite)

There are a couple of waveforms to choose from:
- `sine` - smooth, natural curves
- `triangle` - linear rise/fall
- `saw` - linear ramp with reset
- `square` - binary on/off
- `noise` - random values

```
osc:440:50:2:sine:4.0    # Vibrato: 390-490Hz at 2Hz for 4 seconds
osc:440:5:5:sine:0       # 5Hz vibrato, ±5Hz depth
osc:0.5:0.5:4:sine:0     # 4Hz amplitude modulation (tremolo)
osc:0:1:4:square:0       # 4Hz on/off switching (rhythmic gating)
osc:440:20:0.5:noise:0   # Subtle random pitch drift
```

### Envelope

You can generate one-shot transitions from a start to an end value, similar to traditional two-stage envelopes. The shape of the transition can be controlled with different curves: `env:start:end:curve:duration`. Here are the components:
- `start` - initial value
- `end` - final value
- `curve` - interpolation: `linear`, `exp`, `log`, `quad`, `cubic`
- `duration` - envelope time

```
env:0:1:exp:2.0          # Exponential fade-in over 2 seconds
env:100:10000:exp:3      # Exponential filter opening (filter sweep)
```

### Ramp
Smooth transition with curve control.

**Format:** `ramp:start:end:duration:curve`

- `start` - starting value
- `end` - ending value
- `duration` - ramp time
- `curve` - `linear`, `exp`, `log`, `quad`, `cubic`

**Example:**
```
ramp:20:20000:3:log      # Filter sweep with logarithmic curve
```

### Sequence

The sequence is a special type of modulation that cycles through a list of values at a specified rate for a certain duration. It is reminescent of sample & hold generators on traditional synthesizers but with clearly defined steps: `seq:val1:val2:val3:...:rate:duration`
- `val1, val2, ...` - values to cycle (max 8)
- `rate` - steps per second
- `duration` - total time (0 = infinite)

**Example:**
```
seq:440:550:660:880:2:8.0    # Arpeggio at 2 steps/second for 8 seconds
seq:262:330:392:4:16         # C-E-G progression at 4Hz (chord progression)
```

## Curve Types

There are four curve types available for envelopes and ramps:
- `linear` - constant rate of change
- `exp` / `quad` - accelerating (slow start → fast finish)
- `log` - decelerating (fast start → slow finish)
- `cubic` - extreme acceleration
