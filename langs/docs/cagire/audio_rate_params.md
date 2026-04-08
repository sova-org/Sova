Audio rate modulation happens inside Doux, continuously, between frame triggers. Instead of handing Doux a single number and letting it hold, you hand Doux a *description of a movement*: "sweep from 200 to 3000 over two seconds", "oscillate between -1 and 1 every half second", "fire an envelope from 50 to 8000 across attack, decay, sustain, release". Doux updates the parameter at audio rate while the sound plays.

This is how you get filter sweeps that actually sweep instead of stepping, tremolo that is smooth, and envelopes that shape any parameter you want, not just amplitude.

For the mental model of Doux and the sound register, start with the [Audio Engine](#) article. For setting plain static values, see [Static Parameters](#). For changing values from one frame to the next, see [Control Rate Modulation](#).

## Three Ways to Drive a Filter

The clearest way to see what audio rate modulation does is to compare it with the other two options on the same parameter.

Static: a single number that holds for the whole note.

```forth
saw snd c3 note 1200 lpf .
```

Control rate: a different number on each frame trigger, driven by `cycle`.

```forth
saw snd c3 note 400 800 1600 3200 4 cycle lpf .
```

Audio rate: a smooth sweep between 200 and 3000 Hz that takes two seconds to cycle, running continuously as the note plays.

```forth
saw snd c3 note 200 3000 2 lfo lpf .
```

In the third example, `lfo` sits between the values and the parameter name. It pops three values off the stack (min, max, period) and pushes back a modulation description. The `lpf` word then consumes that description the same way it would consume a plain number. Doux sees "this parameter moves" and runs the movement.

Stack effect for the oscillator family: `(min max period -- mod)`. Stack effect for the transition family: `(start end dur -- mod)`. Every audio rate word follows this shape: pop a few numbers, push one modulation value, let the parameter word consume it.

## The LFO Family

Four oscillator shapes. All share `(min max period -- mod)` and all run forever while the sound plays.

| Word | Shape |
|------|-------|
| `lfo`  | Sine |
| `tlfo` | Triangle |
| `wlfo` | Sawtooth |
| `qlfo` | Square |

`period` is in seconds. `min` and `max` set the travel range.

Sine LFO sweeping a filter cutoff. The decay is long enough that you hear two full cycles of the sweep as the sound fades:

```forth
saw snd c3 note 0.01 4 ad 200 3000 2 lfo lpf .
```

Triangle LFO wobbling the pan between hard left and hard right:

```forth
sine snd c4 note 0.01 4 ad -1 1 0.5 tlfo pan .
```

Square LFO making the same pan hard and discontinuous, a ping pong effect:

```forth
tri snd c3 note 0.01 4 ad -1 1 0.25 qlfo pan .
```

Sawtooth LFO climbing the filter cutoff in repeated upward ramps:

```forth
saw snd c2 note 0.01 4 ad 300 4000 1 wlfo lpf .
```

`lfo` is the default choice; its sine is gentle and musical. `tlfo` is almost the same but with pointier peaks. `wlfo` and `qlfo` are discontinuous, so they sound glitchy or stepped depending on what you hit with them.

## Slides and Slews

Slides and slews are single movements rather than loops. Use them when you want a filter to open once as the note sustains, or pitch to drift into place from below.

**Slide** goes from a start value to an end value over a duration:

```forth
saw snd c3 note 0.01 0.5 ad 200 4000 0.4 slide lpf .
```

Stack effect: `(start end dur -- mod)`. In this example the cutoff opens from 200 to 4000 Hz across 0.4 seconds, then holds at 4000.

**Slew** slides from wherever the parameter currently is to a target value. Stack effect: `(target dur -- mod)`. It only makes sense on voices that persist across events so that "current value" means something; on a one shot note with no prior state, reach for `slide` instead.

Both come in six curve shapes. The suffix in the word name picks the shape:

| Suffix | Curve |
|--------|-------|
| (none)   | Linear |
| `exp`    | Exponential |
| `s`      | Smooth S curve |
| `i`      | Slow start, fast finish (swell) |
| `o`      | Fast attack, slow settle (pluck) |
| `p`      | Eight discrete steps (stair) |

So `expslide`, `sslide`, `islide`, `oslide`, `pslide` are all valid. Same for the slew family: `expslew`, `sslew`, `islew`, `oslew`, `pslew`.

```forth
saw snd
c2 note
0.01 0.4 ad
100 3000 0.3 expslide lpf
.
```

Exponential slide on a filter gives you a much more natural "opening up" feel than linear.

## Jitter and Drunk Walks

Three words generate random motion at audio rate rather than stepped motion at control rate.

- `jit` is random hold: pick a new random value every `period` seconds, snap to it, hold.
- `sjit` is smoothed random: the same walk but interpolated between hold points.
- `drunk` is a drunk walk: each new value is close to the previous one, not independent.

All share `(min max period -- mod)`.

```forth
saw snd c3 note 0.01 0.3 ad 300 3000 0.1 jit lpf .
```

Random cutoff steps every 100 ms give a glitchy filtering effect.

```forth
tri snd c4 note 0.5 sus -0.5 0.5 0.3 sjit pan .
```

Smoothly wandering stereo position across the life of a pad.

```forth
saw snd
c3 note
0.01 0.4 ad
100 15000 0.1 drunk llpf
0.4 llpq
.
```

`drunk` on a ladder filter feels like analog instability: always moving, never jumping very far.

## Envelope Modulation

The amplitude envelope is handled by the `attack` / `decay` / `sustain` / `release` words (see [Static Parameters](#)). The envelopes in this section are different: they modulate any parameter you want, not amplitude. Pluck a filter cutoff with an ADSR shape, sweep pitch with an envelope, drive FM depth with a percussive ramp.

All of them take `min max` at the bottom of the stack and the envelope stages on top.

- `ead`: percussive attack and decay. Stack effect: `(min max a d -- mod)`.
- `eadr`: attack, decay, and release tail. Stack effect: `(min max a d r -- mod)`.
- `eadsr`: full ADSR. Stack effect: `(min max a d s r -- mod)`.
- `env`: DAHDSR (delay, attack, hold, decay, sustain, release) with the same `(min max a d s r -- mod)` shape exposed.

A percussive filter pluck using `ead`:

```forth
0 0.5 (
  pulse snd
  [ c2 c3 ] cycle note
  200 8000 4000 rand
  0.1 .1 .3 rand ead lpf
  0.2 1.0 rand lpq
  1 decay
  . ) at
```

A full ADSR modulating a ladder filter:

```forth
0 0.5 (
  pulse snd
  [ c2 c3 ] cycle note
  50 200 rand 8000    ;; min, max
  0.1                 ;; attack
  0 ;; decay
  0.1 0.5 rand        ;; sustain
  2.8                 ;; release
  eadsr llpf
  0.2 0.7 rand llpq
  1 decay
  . ) at
```

`lpg` is a special case: it is not an envelope you attach to a parameter, it is a shortcut that pairs an amplitude envelope with matching filter movement, imitating the low pass gate modules found on Buchla style synths. Stack effect: `(min max depth --)`.

```forth
saw snd c3 note 0.01 0.1 ad 200 8000 1 lpg .
```

## Mixing with Control Rate

The numbers you feed into an audio rate word are plain values on the stack, so you can compute them with any control rate word you like. The modulation descriptor is built fresh on every frame, which means the LFO itself can be reseeded with new bounds or a new period every time:

```forth
saw snd c3 note 0.01 4 ad
100 500 rand 2000 8000 rand 1 lfo lpf .
```

Each frame builds a fresh LFO with different low and high bounds and hands it to Doux. Doux runs that LFO until the sound ends, then the next frame replaces it with a new one. The same trick works for the period:

```forth
saw snd c3 note 0.01 4 ad
200 3000 0.5 2 rand lfo lpf .
```

The filter still sweeps smoothly inside the hit, but from one hit to the next the sweep speed jumps around.

## Where to Go Next

The word reference has an example for every audio rate word, and the individual entries for `lfo`, `slide`, `slew`, `jit`, `ead`, `eadsr`, `env`, and `lpg` list their exact stack effects. This article covers the shape of the family. Once the shape is in your hands, the rest is picking which curve, which speed, which destination parameter.

For modulation that fires once per frame rather than continuously, go back to [Control Rate Modulation](#). For the full catalogue of destination parameters, the [Audio Engine](#) article is the tour.
