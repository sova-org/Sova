# Scales

A *scale* picks an ordered list of degree indices from a tuning. Each index points at a step in the tuning's cents table. `0` is the root, `1` is the first step, and so on. Once you have a scale, `deg` resolves a degree against a root note and returns a frequency in Hz, ready to feed into a synth's `freq` parameter.

See the Tunings article for how to build the tuning a scale sits on top of. Every example below uses `12 edo` unless noted, but the same words work over any tuning.

## Building a scale

```forth
[ 0 2 4 5 7 9 11 ] 12 edo scale
```

Stack effect: `([i1 i2 ...] tuning -- scale)`. The example above builds a major scale from 12-EDO: it picks tuning steps `0, 2, 4, 5, 7, 9, 11`, which correspond to `C, D, E, F, G, A, B`.

Each index in the list must be a valid step in the tuning (0 through `tuning_size - 1`) and indices must be unique. Order matters: it determines the degree numbering, so `[ 0 2 4 5 7 9 11 ]` and `[ 0 4 7 11 5 9 2 ]` are technically the same set of notes but different scales for `deg` purposes.

## Built-in scales

Most common 12-EDO scales already exist as named words. They are ordinary scale values, not magic. You can store them in variables, choose between them, or rotate them with `mode` like any other scale:

```
major minor dorian phrygian lydian mixolydian aeolian locrian
pentatonic minpent blues chromatic wholetone harmonicminor
melodicminor bebop bebopmaj bebopmin altered lyddom halfwhole
wholehalf augmented tritone prometheus dorianb2 lydianaug
mixb6 locrian2
```

Each one pushes a scale value built against `12 edo`:

```forth
major         ;; pushes the major scale
harmonicminor ;; pushes the harmonic minor scale
```

Use them anywhere a scale is expected:

```forth
c4 minor 0 deg     ;; first degree of C minor, ~261.63 Hz
```

## Modes

`mode` rotates a scale's degree ordering while keeping the same tuning. The shift is the number of degrees to rotate:

```forth
1 major mode      ;; major rotated by one, dorian
2 major mode      ;; rotated by two, phrygian
```

Stack effect: `(n scale -- scale)`. Negative shifts work too (they wrap with rem_euclid). The result is just another scale value, so you can chain `mode` with `deg`, store the result in a variable, etc.

`mode` lets you treat any scale as the parent of its modal family. Useful for non-diatonic scales where the rotated forms don't have standard names.

## Resolving Degrees

`deg` is the bridge from "scale degree" to "playable frequency". It takes a root note (MIDI), a scale, and a degree number, and returns a frequency in Hz:

```forth
c4 major 0 deg      ;; 261.63... (the root)
c4 major 7 deg      ;; 523.25... (the octave above the root)
c4 major -1 deg     ;; 246.94... (one degree below the root)
```

Stack effect: `(root scale degree -- hz)`. The result is a frequency, not a MIDI note. Feed it directly into `freq`:

```forth
c4 minor 0 deg freq sine snd .
```

### Wrapping behavior

`deg` wraps degrees over the scale length and adds full periods when you cross a boundary, so degrees outside `0..len` always resolve to *something musical* rather than failing:

- `degree % len` selects which scale step.
- `degree / len` (Euclidean division) decides how many full periods to shift.

So for a 7-degree scale, degree `7` is the same step as degree `0` but one period higher; degree `-1` is the same step as degree `6` but one period lower. This matches what you'd expect from a real instrument and works for any tuning, including non-octave ones. `deg` uses the tuning's period, so 19-EDO wraps every 19 degrees, Bohlen-Pierce wraps every 13, and so on.

## Sequencing Through a Scale

Combined with `cycle` or `index`, `deg` becomes a melody generator:

```forth
c4 minor 0 1 2 3 4 5 6 7 8 cycle deg freq sine snd .
```

This walks the C minor scale from degree 0 through 7 and back, picking the next degree on every frame trigger. Add `at` for sub-frame motion:

```forth
0 0.25 0.5 0.75 (
  c4 major
  [ 0 2 4 7 ] cycle
  deg freq sine snd .
) at
```

Random degrees from a scale:

```forth
0 7 rand minor c4 swap deg freq sine snd .
```

## Putting It Together

A simple bass line walking degrees of C minor:

```forth
c2 minor [ 0 2 4 5 7 5 4 2 ] cycle deg freq
0.8 gain sine snd .
```

A custom 19-EDO scale with seven degrees:

```forth
[ 0 3 6 8 11 14 17 ] 19 edo scale          ;; major-ish in 19-EDO
c4 swap 0 deg freq sine snd .
```

Modal interchange. Switch the parent rotation per line:

```forth
( 0 ) ( 1 ) ( 2 ) ( 5 ) 4 pcycle major mode
c4 swap 0 1 2 3 cycle deg freq
sine snd .
```

A Bohlen-Pierce scale (13 equal divisions of 1902 cents) used for melody:

```forth
[ 146.3 292.6 438.9 585.2 731.5 877.8 1024.1 1170.4 1316.7
  1463.0 1609.3 1755.6 ] 1902 tuning
[ 0 2 4 6 8 10 ] swap scale       ;; 6-degree subset
c4 swap 0 1 2 3 4 5 6 cycle deg freq
sine snd .
```
