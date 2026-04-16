Every parameter in the sound register can be set in one of three ways. A static value is a single number that holds for the whole note. A control rate value changes from one frame to the next, because Cagire recomputes the stack on every run. An audio rate value tells Doux to move the parameter continuously while the sound plays.

If you have not read the [Audio Engine](#) article, start there for the broader picture of what a parameter *is*.

## Static Parameters

The simplest way to shape a sound is to hand Doux a number and leave it alone. Set the cutoff to 1200, the gain to 0.7, the attack to 0.01, and let the sound play. Nothing moves over time.

### The Stacking Pattern

Every parameter word pops one value off the stack and attaches it to the current sound:

```forth
saw snd c4 note 0.7 gain 1200 lpf 0.3 verb .
```

Read right to left if the direction is not yet second nature: `0.3 verb` means "take 0.3 off the stack and attach it as the `verb` parameter." Same for `1200 lpf`, `0.7 gain`, and so on. Stack effect: `(v --)`.

The order does not matter. These three lines all produce the same sound:

```forth
saw snd c4 note 0.7 gain 1200 lpf .
```

```forth
saw snd 1200 lpf c4 note 0.7 gain .
```

```forth
saw snd 0.7 gain c4 note 1200 lpf .
```

What matters is that every value you mean as a parameter is on the stack at the moment its parameter word runs. If the stack is empty when a parameter word fires, you get a runtime error.

### Several Values at Once

A few words take several stack values at once. The two common ones are `ad` and `adsr`.

`ad` is a percussive attack and decay shorthand:

```forth
saw snd c3 note 0.01 0.3 ad .
```

`adsr` is the full attack, decay, sustain, release envelope:

```forth
saw snd c3 note 0.01 0.2 0.6 0.4 adsr .
```

Stack effects: `(a d --)` for `ad` and `(a d s r --)` for `adsr`. They are sugar over the individual `attack`, `decay`, `sustain`, `release` words. Writing `0.01 0.3 ad` is identical to writing `0.01 attack 0.3 decay`.

### Overriding

If you set the same parameter twice before `.`, the last write wins:

```forth
saw snd c4 note 0.3 gain 0.8 gain .   ;; plays at 0.8
```

This sounds like a footgun but it is useful. A common idiom is to establish defaults inside a word you define yourself, then override one of them at the call site. Start by defining the voice:

```forth
: bass  saw snd 0.01 0.3 ad 800 lpf 0.7 gain ;
```

Now it plays with its default 800 Hz cutoff:

```forth
c2 note bass .
```

And this overrides the cutoff to 2000 Hz without touching the rest:

```forth
c2 note bass 2000 lpf .
```

See the [Creating Words](#) article for more on defining your own voices.

### Defaults

Any parameter you do not set falls back to a Doux default. For most parameters that means "off" or "neutral": no reverb if you do not set `verb`, no delay if you do not set `delay`, center pan, full gain. Exact default values can change between Doux releases, so treat the word reference as the source of truth and do not rely on implicit values when it really matters.

There is one exception worth naming: when you do not specify `gate`, Cagire fills it in with the current frame duration so the sound lasts exactly one step. Set `gate` yourself to override.

### Chords and Lists as Values

A parameter word does not have to pop a single number. `note` is variadic: push several notes in a row and it eats them all, emitting one voice per note:

```forth
c4 e4 g4 note saw snd 0.4 verb .
```

A named chord works the same way:

```forth
c4 note maj7 chord saw snd 0.4 verb .
```

All voices share the same static parameters. Pitch is the only thing that differs. See the [Chords](#) article for the full chord vocabulary.

### Resetting and Clearing

`reset` removes a single parameter from the register after it was set, reverting it to the Doux default:

```forth
saw snd c3 note 0.5 gain reset gain .   ;; gain is back to default
```

`clear` throws the whole register away without emitting:

```forth
kick snd 0.5 gain      ;; filling the register
clear                  ;; register is emptied
hat snd .              ;; only the hat plays
```

`clear` is the escape hatch when a conditional decides the sound should not play at all. Set up the register unconditionally, then `clear` when you want to cancel.

## Control Rate Modulation

A control rate parameter changes value each time the frame fires, because Cagire recomputes the whole stack from scratch on every run. This gives you movement at the pattern level: variation from one hit to the next, rotating cutoffs, random jitter between hits. The value is still a single number by the time Doux hears it. What varies is which number.

The words below are documented in full detail in the [Cycling](#), [Randomness](#), [Periodic Execution](#), and [Probability](#) articles. The purpose here is to frame them as modulation tools.

### Two Rules

1. **Every frame reruns the script.** Any word that returns a different value on each run produces control rate modulation. `rand` rolls a fresh number. `cycle` advances its counter. `choose` picks again. By the time `.` fires, the value on the stack is the one Doux will see for this hit.
2. **The modulation happens in Cagire, not in Doux.** Doux receives a plain number for this one hit. No smoothing happens between hits. If you want smooth motion between frame boundaries, that is the job of audio rate modulation (below).

### Cycling as Modulation

`cycle` walks a list of values deterministically, one per frame trigger. Point it at any parameter:

```forth
saw snd c2 note 0.01 0.3 ad
400 800 1600 3200 4 cycle lpf .
```

The cutoff steps through 400, 800, 1600, 3200 on successive frames and wraps around. The same trick works on any parameter that takes a number:

```forth
sine snd c3 note
0.1 0.3 0.5 0.7 4 cycle gain .      ;; volume steps
```

`pcycle` counts by line iterations instead of frame triggers, and `bounce` ping pongs at the ends instead of wrapping. See the [Cycling](#) article for the full set.

### Randomness as Modulation

`rand`, `exprand`, `logrand`, `choose`, and `wchoose` roll fresh values per frame. Applied to a parameter, they give you sampled jitter:

```forth
sine snd 60 72 rand note 0.3 0.8 rand gain .
```

The note and the gain are different on every hit. `exprand` and `logrand` let you bias the distribution when a uniform random would sit in the wrong register of the parameter range:

```forth
saw snd c3 note 200 8000 exprand lpf .   ;; mostly low cutoffs
```

`choose` works when the set of values is discrete rather than a range:

```forth
sine tri saw pulse 4 choose snd c4 note .
```

See the [Randomness](#) article.

### Periodic and Probabilistic Gating

The periodic words (`every`, `except`, `bjork`) and the probability words (`chance`, `always`, `rarely`, and friends) do not directly change a parameter value. They decide whether a whole block of code fires at all. Wrap a parameter change in a quotation and you can gate it on or off:

```forth
saw snd c3 note 0.01 0.3 ad
( 2000 lpf ) 4 every                ;; bright filter every 4 iterations
( 500 lpf ) 4 except
.
```

On iteration 0, 4, 8, ... the cutoff is 2000. Every other iteration it is 500. The `every` / `except` pair covers both cases so the register always has a cutoff set.

Probability works the same way:

```forth
saw snd c3 note
( 0.6 verb ) 0.3 chance             ;; 30% of frames get reverb
.
```

See the [Periodic Execution](#) and [Probability](#) articles.

### Modulation Inside One Frame with at

`at` subdivides a frame into smaller time slots and reruns a quotation at each slot. Inside the quotation, control rate words roll fresh values on every subdivision, so you get four random values in a single frame instead of one:

```forth
0 0.25 0.5 0.75 (
  hat snd 0.3 0.8 rand gain .
) at
```

Four hats per frame, each with its own random gain. This is the only way to drive a parameter faster than the frame rate without reaching for audio rate modulation. See the [Timing](#) article for the full `at` vocabulary.

### Combining

Control rate words layer freely. A single patch can draw from all four families at once:

```forth
saw snd
[ c2 g2 eb2 f2 ] cycle note       ;; cycle through a bass line
400 800 1600 3200 4 cycle lpf     ;; step the cutoff
0.4 0.8 rand gain                 ;; jitter the gain
0.01 0.2 ad
( 0.4 verb ) 4 every              ;; reverb every 4 hits
.
```

Four independent modulation sources, each doing its own thing, all flowing into a single Doux event per frame.

## Audio Rate Modulation

Audio rate modulation happens inside Doux, continuously, between frame triggers. Instead of handing Doux a single number and letting it hold, you hand Doux a *description of a movement*: "sweep from 200 to 3000 over two seconds", "oscillate between -1 and 1 every half second", "fire an envelope from 50 to 8000 across attack, decay, sustain, release". Doux updates the parameter at audio rate while the sound plays.

This is how you get filter sweeps that actually sweep instead of stepping, tremolo that is smooth, and envelopes that shape any parameter you want, not just amplitude.

### Three Ways to Drive a Filter

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

### The LFO Family

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

### Slides and Slews

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

### Jitter and Drunk Walks

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

### Envelope Modulation

The amplitude envelope is handled by the `attack` / `decay` / `sustain` / `release` words (see Static Parameters above). The envelopes in this section are different: they modulate any parameter you want, not amplitude. Pluck a filter cutoff with an ADSR shape, sweep pitch with an envelope, drive FM depth with a percussive ramp.

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

### Mixing Control Rate and Audio Rate

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
