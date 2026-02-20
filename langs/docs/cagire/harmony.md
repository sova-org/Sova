# Notes & Harmony

Cagire speaks music theory. Notes, intervals, chords, and scales are all first-class words that compile to stack operations on MIDI values.

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
c4 note sine s .            ;; play middle C as a sine
a4 note 0.5 gain modal s .  ;; concert A, quieter
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

Chord words take a root note and push all the chord tones. They eat the root and replace it with the full voicing:

```forth
c4 maj        ;; stack: 60 64 67
c4 min7       ;; stack: 60 63 67 70
c4 dom9       ;; stack: 60 64 67 70 74
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

**Seventh chords:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `maj7` | 0 4 7 11 | 60 64 67 71 |
| `min7` | 0 3 7 10 | 60 63 67 70 |
| `dom7` | 0 4 7 10 | 60 64 67 70 |
| `dim7` | 0 3 6 9 | 60 63 66 69 |
| `m7b5` | 0 3 6 10 | 60 63 66 70 |
| `minmaj7` | 0 3 7 11 | 60 63 67 71 |
| `aug7` | 0 4 8 10 | 60 64 68 70 |

**Sixth chords:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `maj6` | 0 4 7 9 | 60 64 67 69 |
| `min6` | 0 3 7 9 | 60 63 67 69 |

**Extended chords:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `dom9` | 0 4 7 10 14 | 60 64 67 70 74 |
| `maj9` | 0 4 7 11 14 | 60 64 67 71 74 |
| `min9` | 0 3 7 10 14 | 60 63 67 70 74 |
| `dom11` | 0 4 7 10 14 17 | 60 64 67 70 74 77 |
| `min11` | 0 3 7 10 14 17 | 60 63 67 70 74 77 |
| `dom13` | 0 4 7 10 14 21 | 60 64 67 70 74 81 |

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

Chord tones are varargs — they eat the entire stack. So a chord word should come right after the root note:

```forth
c4 maj note sine s .    ;; plays all 3 notes as one chord
```

## Scales

Scale words convert a degree index into a MIDI note. The base note is C4 (MIDI 60). Degrees wrap around with octave transposition:

```forth
0 major       ;; 60 (C4 — degree 0)
4 major       ;; 67 (G4 — degree 4)
7 major       ;; 72 (C5 — degree 7, wraps to next octave)
-1 major      ;; 59 (B3 — negative degrees go down)
```

Use scales with `cycle` or `rand` to walk through pitches:

```forth
0 1 2 3 4 5 6 7 8 cycle minor note sine s .
```

**Standard modes:**

| Word | Pattern (semitones) |
|------|-------------------|
| `major` | 0 2 4 5 7 9 11 |
| `minor` | 0 2 3 5 7 8 10 |
| `dorian` | 0 2 3 5 7 9 10 |
| `phrygian` | 0 1 3 5 7 8 10 |
| `lydian` | 0 2 4 6 7 9 11 |
| `mixolydian` | 0 2 4 5 7 9 10 |
| `aeolian` | 0 2 3 5 7 8 10 |
| `locrian` | 0 1 3 5 6 8 10 |

**Pentatonic and blues:**

| Word | Pattern |
|------|---------|
| `pentatonic` | 0 2 4 7 9 |
| `minpent` | 0 3 5 7 10 |
| `blues` | 0 3 5 6 7 10 |

**Chromatic and whole tone:**

| Word | Pattern |
|------|---------|
| `chromatic` | 0 1 2 3 4 5 6 7 8 9 10 11 |
| `wholetone` | 0 2 4 6 8 10 |

**Harmonic and melodic minor:**

| Word | Pattern |
|------|---------|
| `harmonicminor` | 0 2 3 5 7 8 11 |
| `melodicminor` | 0 2 3 5 7 9 11 |

**Jazz / Bebop:**

| Word | Pattern |
|------|---------|
| `bebop` | 0 2 4 5 7 9 10 11 |
| `bebopmaj` | 0 2 4 5 7 8 9 11 |
| `bebopmin` | 0 2 3 5 7 8 9 10 |
| `altered` | 0 1 3 4 6 8 10 |
| `lyddom` | 0 2 4 6 7 9 10 |

**Symmetric:**

| Word | Pattern |
|------|---------|
| `halfwhole` | 0 1 3 4 6 7 9 10 |
| `wholehalf` | 0 2 3 5 6 8 9 11 |
| `augmented` | 0 3 4 7 8 11 |
| `tritone` | 0 1 4 6 7 10 |
| `prometheus` | 0 2 4 6 9 10 |

**Modal variants (from melodic minor):**

| Word | Pattern |
|------|---------|
| `dorianb2` | 0 1 3 5 7 9 10 |
| `lydianaug` | 0 2 4 6 8 9 11 |
| `mixb6` | 0 2 4 5 7 8 10 |
| `locrian2` | 0 2 3 5 6 8 10 |

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
c4 mtof freq sine s .
```

## Putting It Together

A chord progression cycling every line iteration:

```forth
{ c3 maj7 } { f3 maj7 } { g3 dom7 } { c3 maj7 } 4 pcycle
note sine s .
```

Arpeggiate a chord across the frame's time divisions:

```forth
c4 min7 arp note 0.5 decay sine s .
```

Random notes from a scale:

```forth
0 7 rand minor note sine s .
```

A bass line walking scale degrees:

```forth
0 2 4 5 7 5 4 2 8 cycle minor note
-2 oct 0.8 gain sine s .
```

Chord voicings with random inversion:

```forth
e3 min9
{ } { 1 oct } 2 choose
note modal s .
```

Stacked intervals for custom voicings:

```forth
c3 P5 P8 M10    ;; C3, G3, C4, E4
note sine s .
```
