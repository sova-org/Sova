A *scale* picks an ordered list of degree indices from a tuning. Each index points at a step in the tuning's cents table. `0` is the root, `1` is the first step, and so on. Once you have a scale, `deg` resolves a degree against a root note and returns a frequency in Hz, ready to feed into a synth's `freq` parameter. See the [Tunings](tunings.md) article for how to build the tuning a scale sits on top of. Every example below uses `12 edo` unless noted, but the same words work over any tuning.

## Quick start

If you don't know anything about scales yet and just want notes coming out of a synth, this one-liner is the shortest path:

```forth
c4 minor 0 7 rand deg freq
sine snd .
```

Read left to right: "root is middle C, scale is minor, pick a random degree between 0 and 7, turn it into a frequency, feed it to a sine, play it." Swap `minor` for any other name from the built-in scales list further down (`dorian`, `blues`, `pentatonic`, `harmonicminor`, ...) to change the mood. Swap `c4` to move everything up or down. Everything else in this article builds on that pattern.

## Building a scale

```forth
;; This is the major scale built from scratch
[ 0 2 4 5 7 9 11 ] 12 edo scale
```

Stack effect: `([i1 i2 ...] tuning -- scale)`. The example above builds a major scale from 12-EDO: it picks tuning steps `0, 2, 4, 5, 7, 9, 11`, which correspond to `C, D, E, F, G, A, B`. Each index in the list must be a valid step in the tuning (0 through `tuning_size - 1`) and indices must be unique. Order matters: it determines the degree numbering, so `[ 0 2 4 5 7 9 11 ]` and `[ 0 4 7 11 5 9 2 ]` are technically the same set of notes but different scales for `deg` purposes.

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

### `note` vs `deg`

This is the most common point of confusion, so it's worth calling out. `note` (from the [Notes](notes.md) article) and `deg` look like they do similar jobs but they sit in different slots in the emit pipeline:

- `note` is a setter. You hand it a MIDI number and it stores it on the emit. Nothing lands back on the stack. `c4 note sine snd .` plays middle C.
- `deg` is a computation. It takes `(root scale degree -- hz)` and returns a frequency on the stack. It never touches the emit by itself. `c4 minor 0 deg` just leaves `261.63` sitting there until you do something with it.

So they don't replace each other. `deg` is partnered with `freq`, not with `note`:

```forth
c4 note sine snd .              ;; MIDI path, whole semitones
c4 minor 0 deg freq sine snd .  ;; scale path, frequency in Hz
```

Why the split? MIDI notes are whole numbers from 0 to 127. Tunings can be microtonal: 19-EDO, Bohlen-Pierce, Pythagorean, anything in between. If `deg` stored a MIDI integer, every fractional-cent pitch would get rounded to the nearest semitone and custom tunings would be pointless. Returning Hz keeps the precision alive all the way to the oscillator.

Rule of thumb: if you have a specific pitch in mind, reach for `note`. If you're thinking in scale degrees ("walk up the scale", "pick the third", "grab a random one"), reach for `deg` + `freq`.

### Wrapping behavior

`deg` wraps degrees over the scale length and adds full periods when you cross a boundary, so degrees outside `0..len` always resolve to *something musical* rather than failing:

- `degree % len` selects which scale step.
- `degree / len` (Euclidean division) decides how many full periods to shift.

So for a 7-degree scale, degree `7` is the same step as degree `0` but one period higher; degree `-1` is the same step as degree `6` but one period lower. This matches what you'd expect from a real instrument and works for any tuning, including non-octave ones. `deg` uses the tuning's period, so 19-EDO wraps every 19 degrees, Bohlen-Pierce wraps every 13, and so on.

## Sequencing Through a Scale

Combined with `cycle` or `index`, `deg` becomes a melody generator:

```forth
c4 minor [ 0 1 2 3 4 5 6 7 ] cycle deg freq
sine snd .
```

This walks the C minor scale from degree 0 through 7 and back, picking the next degree on every frame trigger. Add `at` for sub-frame motion:

```forth
0 0.25 0.5 0.75 (
  c4 major
  [ 0 2 4 7 ] cycle
  deg freq sine snd .
) at
```

## Putting It Together

A simple bass line walking degrees of C minor:

```forth
c2 minor [ 0 2 4 5 7 5 4 2 ] cycle deg freq
0.8 gain pulse snd 500 1000.0 rand lpf .
```

A custom 19-EDO scale with seven degrees:

```forth
[ 0 3 6 8 11 14 17 ] 19 edo scale
c4 swap 0 deg freq sine snd .
```
