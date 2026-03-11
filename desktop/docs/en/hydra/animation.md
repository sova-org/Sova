Four variables are available for animation: `time`, `beat`, `tempo`, and
`phase`. They produce dynamic GLSL expressions — the shader evaluates them
every frame, so visuals animate continuously.

## time

Elapsed seconds since the engine started. Equivalent to Hydra's `time`.

```
osc(60.0, sin(time)).out()
```

## beat

Current beat position from the musical clock (Ableton Link). Increases
continuously — beat 4.5 means halfway through the fifth beat.

```
osc(60.0, 0.1).rotate(0.0, beat * 0.01).out()
```

## phase

Position within the current measure, from 0 to the quantum length (usually 4).
Resets at each downbeat.

```
noise(10.0, 0.1).colorama(phase * 0.1).out()
```

## tempo

Current tempo in BPM. Useful for scaling animation speeds relative to the
musical tempo.

## Arithmetic

These variables support `+`, `-`, `*`, `/` with numbers and with each other.
Standard math functions work too: `sin`, `cos`, `abs`, `fract`.

```
osc(60.0 + sin(time) * 20.0, 0.1).out()
```

```
osc(40.0, 0.1).rotate(sin(beat * 0.5), 0.0).out()
```

Intermediate expressions can be stored in variables:

```
let spd = sin(time * 0.5) * 0.1
osc(60.0, spd).rotate(0.0, spd).out()
```

## Patterns

Arrays cycle through values over time, like in Hydra. Pass an array anywhere a
number is expected:

```
osc([60, 80, 100], 0.1).out()
```

By default, values step once per second. `.fast(n)` multiplies the rate:

```
osc([60, 80, 100].fast(4), 0.1).out()
```

`.smooth()` interpolates between values instead of stepping:

```
osc([60, 80, 100].fast(2).smooth(), 0.1).out()
```

`.offset(n)` shifts the starting phase:

```
osc([60, 80, 100].fast(2).offset(0.5), 0.1).out()
```

Patterns work in any argument position:

```
osc(60.0, 0.1).rotate([0, 1, 2, 3].fast(0.5).smooth()).out()
```
