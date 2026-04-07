# Notes & Harmony

Cagire speaks music theory. Notes and intervals still work as numeric pitch values, while chords, tunings, and scales are now first-class runtime values.

## MIDI Notes

Write a note name followed by an octave number. It compiles to a MIDI integer:

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

Octave range is -1 to 9. The formula is `(octave + 1) * 12 + base + modifier`, where C=0, D=2, E=4, F=5, G=7, A=9, B=11.

Note literals push a single integer onto the stack, just like writing `60` directly. They work everywhere an integer works:

```forth
c4 note sine snd .            ;; play middle C as a sine
a4 note 0.5 gain modal snd .  ;; concert A, quieter
```

## Intervals

An interval duplicates the top of the stack and adds semitones. This lets you build chords by stacking:

```forth
c4 M3 P5       ;; stack: 60 64 67 (C major triad)
c4 m3 P5       ;; stack: 60 63 67 (C minor triad)
a3 P5           ;; stack: 57 64 (A plus a fifth)
```

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

## Chords

Chord qualities are values. Push a quality like `min7` or `maj9`, then call `chord` to activate it for note playback:

```forth
c4 note
min7 chord
sine snd .
```

When `note` and `chord` are both present, `note` is the root note. If `cn` is absent, `.` emits the full voicing as polyphonic MIDI notes.

Add `anchor` to ask Cagire for the inversion/register whose chord tone lands closest to that pitch:

```forth
c4 note
maj7 chord
g4 anchor
sine snd .
```

With `anchor`, `cn` indexes the realized voiced notes from low to high. Indexes still wrap by octave: values above the voicing length climb upward, and negative values descend:

```forth
c4 note
min7 chord
[ 0 3 0 3 ] cycle cn
sine snd .
```

```forth
c4 note
min7 chord
[ 0 1 2 3 4 ] cycle cn
sine snd .
```

Here `4` resolves to the root one octave higher.

Numeric aliases also work through `chord`:

```forth
c4 note
6 chord      ;; maj6
sine snd .
```

**Triads:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `maj` | 0 4 7 | 60 64 67 |
| `m` | 0 3 7 | 60 63 67 |
| `dim` | 0 3 6 | 60 63 66 |
| `aug` | 0 4 8 | 60 64 68 |
| `sus2` | 0 2 7 | 60 62 67 |
| `sus4` | 0 5 7 | 60 65 67 |
| `pwr` | 0 7 | 60 67 |

**Seventh chords:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `maj7` | 0 4 7 11 | 60 64 67 71 |
| `min7` | 0 3 7 10 | 60 63 67 70 |
| `dom7` / `7` | 0 4 7 10 | 60 64 67 70 |
| `dim7` | 0 3 6 9 | 60 63 66 69 |
| `m7b5` | 0 3 6 10 | 60 63 66 70 |
| `minmaj7` | 0 3 7 11 | 60 63 67 71 |
| `aug7` | 0 4 8 10 | 60 64 68 70 |
| `augmaj7` | 0 4 8 11 | 60 64 68 71 |
| `7sus4` | 0 5 7 10 | 60 65 67 70 |

**Sixth chords:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `maj6` / `6` | 0 4 7 9 | 60 64 67 69 |
| `min6` | 0 3 7 9 | 60 63 67 69 |
| `maj69` | 0 4 7 9 14 | 60 64 67 69 74 |
| `min69` | 0 3 7 9 14 | 60 63 67 69 74 |

**Extended chords:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `dom9` / `9` | 0 4 7 10 14 | 60 64 67 70 74 |
| `maj9` | 0 4 7 11 14 | 60 64 67 71 74 |
| `min9` | 0 3 7 10 14 | 60 63 67 70 74 |
| `dom11` / `11` | 0 4 7 10 14 17 | 60 64 67 70 74 77 |
| `min11` | 0 3 7 10 14 17 | 60 63 67 70 74 77 |
| `dom13` / `13` | 0 4 7 10 14 21 | 60 64 67 70 74 81 |
| `9sus4` | 0 5 7 10 14 | 60 65 67 70 74 |
| `maj11` | 0 4 7 11 14 17 | 60 64 67 71 74 77 |
| `maj13` | 0 4 7 11 14 21 | 60 64 67 71 74 81 |
| `min13` | 0 3 7 10 14 21 | 60 63 67 70 74 81 |

**Add chords:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `add9` | 0 4 7 14 | 60 64 67 74 |
| `add11` | 0 4 7 17 | 60 64 67 77 |
| `madd9` | 0 3 7 14 | 60 63 67 74 |

**Altered dominants:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `dom7b9` | 0 4 7 10 13 | 60 64 67 70 73 |
| `dom7s9` | 0 4 7 10 15 | 60 64 67 70 75 |
| `dom7b5` | 0 4 6 10 | 60 64 66 70 |
| `dom7s5` | 0 4 8 10 | 60 64 68 70 |
| `dom7s11` | 0 4 7 10 18 | 60 64 67 70 78 |

## Tunings & Scales

Scales are now built in two layers:

- A `tuning` is a set of cents positions inside a repeating period.
- A `scale` is an ordered selection of tuning steps.
- `deg` resolves a scale degree against a root note and returns a frequency in Hz.

Equal divisions are easy:

```forth
12 edo              ;; 12-step tuning over 2/1
19 edo              ;; 19-EDO tuning
```

Custom tunings use cents within one period:

```forth
[ 90.225 204.090 294.135 408.000 498.045 588.090 702.000 ] 1200 tuning
```

Scales select steps from a tuning:

```forth
[ 0 2 4 5 7 9 11 ] 12 edo scale
```

Built-in names like `major`, `minor`, `dorian`, and `pentatonic` push reusable scale values. They are ordinary scale objects, not special degree operators.

Resolve degrees with `deg`:

```forth
c4 major 0 deg      ;; 261.63...
c4 major 7 deg      ;; 523.25...
c4 major -1 deg     ;; 246.94...
```

Use `mode` to rotate the degree ordering while keeping the same tuning:

```forth
1 major mode
```

A simple melodic line with the new flow:

```forth
c4 minor 0 1 2 3 4 5 6 7 cycle deg freq sine snd .
```

Built-in scale names:

`major`, `minor`, `dorian`, `phrygian`, `lydian`, `mixolydian`, `aeolian`, `locrian`, `pentatonic`, `minpent`, `blues`, `chromatic`, `wholetone`, `harmonicminor`, `melodicminor`, `bebop`, `bebopmaj`, `bebopmin`, `altered`, `lyddom`, `halfwhole`, `wholehalf`, `augmented`, `tritone`, `prometheus`, `dorianb2`, `lydianaug`, `mixb6`, `locrian2`

## Octave Shifting

`oct` transposes a note by octaves:

```forth
c4 1 oct      ;; 72 (C5)
c4 -1 oct     ;; 48 (C3)
c4 2 oct      ;; 84 (C6)
```

Stack effect: `(note shift -- transposed)`. The shift is multiplied by 12 and added to the note.

## Frequency Conversion

`mtof` converts a MIDI note to frequency in Hz. `ftom` does the reverse:

```forth
69 mtof       ;; 440.0 (A4)
60 mtof       ;; 261.63 (C4)
440 ftom      ;; 69.0
```

Useful when a synth parameter expects Hz rather than MIDI:

```forth
c4 mtof freq sine snd .
```

## Putting It Together

A chord progression cycling every line iteration:

```forth
( c3 note maj7 chord . ) ( f3 note maj7 chord . )
( g3 note 7 chord . ) ( c3 note maj7 chord . ) 4 pcycle
```

Arpeggiate a chord across time divisions using `at`:

```forth
0 0.25 0.5 0.75 ( c4 note maj7 chord [ 0 1 2 3 ] cycle cn 0.5 decay sine snd . ) at
```

Random notes from a scale:

```forth
0 7 rand minor note sine snd .
```

A bass line walking scale degrees:

```forth
0 2 4 5 7 5 4 2 8 cycle minor note
-2 oct 0.8 gain sine snd .
```

Chord voicings with random inversion:

```forth
e3 min9
( ) ( 1 oct ) 2 choose
note modal snd .
```

Stacked intervals for custom voicings:

```forth
c3 P5 P8 M10    ;; C3, G3, C4, E4
note sine snd .
```
