# Control LFOs

Control rate oscillators that return a single number you can plug into any parameter. Frequency is in cycles per beat, so `1 ctlsine` runs one full cycle per beat, `0.25 ctlsine` runs one cycle every four beats.

Each shape comes in two flavors:

- `ctlxxx` returns values in `[0, 1]`.
- `ctlbxxx` returns values in `[-1, 1]`.

Pair them with `range` to scale into the range you actually want.

## Sine

Smooth oscillation between 0 and 1.

```
0.5 ctlsine 200 800 range lpf sine snd .
```

## Triangle, saw, square

Same idea, different shapes.

```
0.25 ctltriangle 100 1000 range lpf
0.25 ctlsaw 0 1 range gain
0.5 ctlsquare 0.1 0.9 range pan
```

## Ramps

A ramp climbs from 0 to 1 over each cycle. The shape is set by a curve exponent:

- `ctllinramp` linear, curve 1
- `ctlexpramp` exponential, curve 3
- `ctllogramp` logarithmic, curve 0.3
- `ctlramp` lets you pick the curve yourself

```
0.5 2.0 ctlramp 0 1 range gain
```

## Perlin

Smooth, slowly drifting random walk.

```
0.25 ctlperlin 100 800 range lpf
```

## Noise and sample-and-hold

`ctlnoise` and `ctlsh` both produce a fresh random value every `1/freq` beats and hold it. They are independent streams so you can layer them without correlation.

```
4 ctlnoise 200 2000 range lpf
2 ctlsh 0 1 range gain
```

## Pairing with range

`range` does the scaling: `(value lo hi -- scaled)`.

```
0.5 ctlbsine -50 50 range note
```
