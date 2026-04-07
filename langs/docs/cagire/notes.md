# Notes

In Cagire, a note is just a number. Note names are syntactic sugar that compile to MIDI integers, so anything you can do with `60` you can do with `c4`. The trick is that the language gives you a few small operators (note literals and interval words) that build chords, voicings, and melodies on the stack from those integers.

## Note Literals

Write a note name followed by an octave number. It pushes the matching MIDI value onto the stack:

```forth
c4        ;; 60 (middle C)
a4        ;; 69 (concert A)
e3        ;; 52
```

Sharps use `s` or `#`. Flats use `b`:

```forth
fs4       ;; 66 (F sharp 4)
f#4       ;; 66 (same thing)
bb3       ;; 58 (B flat 3)
eb4       ;; 63
```

The formula is `(octave + 1) * 12 + base + modifier`, where the base steps are `c=0, d=2, e=4, f=5, g=7, a=9, b=11`. Octave range is `-1` to `9`, matching the MIDI specification.

Because a note literal is indistinguishable from an integer literal once parsed, it works everywhere an integer works:

```forth
c4 note sine snd .            ;; play middle C as a sine
a4 note 0.5 gain modal snd .  ;; concert A, quieter
```

### Solfège

If you prefer French / Italian solfège, those names work too:

```forth
do4       ;; 60
re4       ;; 62
mi4       ;; 64
fa4       ;; 65
sol4      ;; 67
la4       ;; 69
si4       ;; 71
ti4       ;; 71  (English variant)
ut4       ;; 60  (archaic French for C)
```

Modifiers are the same as the letter form: `do#4`, `mib4`, etc. Solfège names are interchangeable with letter names. Pick whichever reads better in context.

## The `note` Word

A bare note literal sits on the stack as an integer. To make a sound use it as the *root note* of an emit, hand it to `note`:

```forth
60 note sine snd .    ;; equivalent to: c4 note sine snd .
```

`note` is a parameter word: it consumes the top of the stack and stores it on the command register under the key `note`. When `.` (emit) fires, `note` is read as the MIDI pitch, either as a single note or, if `chord` is also active, as the root of the chord. See the [Chords](#) article for the chord interaction.

## Intervals

An interval word duplicates the top of the stack and adds semitones. This is the canonical way to build chords and voicings without typing out raw MIDI numbers:

```forth
c4 M3 P5       ;; stack: 60 64 67  (C major triad)
c4 m3 P5       ;; stack: 60 63 67  (C minor triad)
a3 P5          ;; stack: 57 64     (A plus a fifth above)
```

`M3` is exactly `dup 4 +`. So `c4 M3 P5` desugars to `60 dup 4 + dup 7 +`, which leaves the three triad notes on the stack in ascending order. To turn a stack of MIDI numbers into a polyphonic emit, see the [Chords](#) article. The short version is "use as many `note` calls as you want or use `chord`."

Simple intervals (within one octave):

| Interval | Semitones | Name |
|----------|-----------|------|
| `P1` / `unison` | 0 | Perfect unison |
| `m2` | 1 | Minor 2nd |
| `M2` | 2 | Major 2nd |
| `m3` | 3 | Minor 3rd |
| `M3` | 4 | Major 3rd |
| `P4` | 5 | Perfect 4th |
| `aug4` / `dim5` / `tritone` | 6 | Tritone |
| `P5` | 7 | Perfect 5th |
| `m6` | 8 | Minor 6th |
| `M6` | 9 | Major 6th |
| `m7` | 10 | Minor 7th |
| `M7` | 11 | Major 7th |
| `P8` | 12 | Octave |

Compound intervals (beyond one octave):

| Interval | Semitones |
|----------|-----------|
| `m9` | 13 |
| `M9` | 14 |
| `m10` | 15 |
| `M10` | 16 |
| `P11` | 17 |
| `aug11` | 18 |
| `P12` | 19 |
| `m13` | 20 |
| `M13` | 21 |
| `m14` | 22 |
| `M14` | 23 |
| `P15` | 24 |

## Octave Shifting

To shift a note by whole octaves, just add or subtract a multiple of 12:

```forth
c4 12 +       ;; 72 (C5)
c4 12 -       ;; 48 (C3)
c4 24 +       ;; 84 (C6)
```

Since notes are plain integers, every arithmetic word in [Cagire vs Classic Forth](#) works on them. There is no special "octave" operator.

## Frequency Conversion

`mtof` converts a MIDI note to a frequency in Hz. `ftom` does the reverse:

```forth
69 mtof       ;; 440.0  (A4)
60 mtof       ;; 261.63 (C4)
440 ftom      ;; 69.0
```

Both accept floats, so fractional MIDI numbers work and you get accurate microtonal frequencies. Use `mtof` when a synth parameter expects Hz instead of MIDI:

```forth
c4 mtof freq sine snd .
```

This is also the bridge between [Scales](#) (which return Hz from `deg`) and oscillators that take a `freq` parameter directly.

## Putting It Together

Stacked intervals for a custom voicing:

```forth
c3 P5 P8 M10    ;; C3, G3, C4, E4
note sine snd .
```

A bass line walking scale-degree intervals around a tonic:

```forth
c3 12 - 0 2 4 5 7 5 4 2 8 cycle + note
sine snd .
```

A simple two-octave arpeggio:

```forth
c4 M3 P5 P8 M10 P12 P15
note modal snd .
```
