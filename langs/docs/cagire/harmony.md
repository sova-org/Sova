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
| `pwr` | 0 7 | 60 67 |

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
| `augmaj7` | 0 4 8 11 | 60 64 68 71 |
| `7sus4` | 0 5 7 10 | 60 65 67 70 |

**Sixth chords:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `maj6` | 0 4 7 9 | 60 64 67 69 |
| `min6` | 0 3 7 9 | 60 63 67 69 |
| `maj69` | 0 4 7 9 14 | 60 64 67 69 74 |
| `min69` | 0 3 7 9 14 | 60 63 67 69 74 |

**Extended chords:**

| Word | Intervals | Example (C4) |
|------|-----------|-------------|
| `dom9` | 0 4 7 10 14 | 60 64 67 70 74 |
| `maj9` | 0 4 7 11 14 | 60 64 67 71 74 |
| `min9` | 0 3 7 10 14 | 60 63 67 70 74 |
| `dom11` | 0 4 7 10 14 17 | 60 64 67 70 74 77 |
| `min11` | 0 3 7 10 14 17 | 60 63 67 70 74 77 |
| `dom13` | 0 4 7 10 14 21 | 60 64 67 70 74 81 |
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

Chord tones are varargs — they eat the entire stack. So a chord word should come right after the root note:

```forth
c4 maj note sine snd .    ;; plays all 3 notes as one chord
```

## Voicings

Four words reshape chord voicings without changing the harmony.

`inv` moves the bottom note up an octave (inversion):

```forth
c4 maj inv note sine snd .     ;; E4 G4 C5 — first inversion
c4 maj inv inv note sine snd . ;; G4 C5 E5 — second inversion
```

`dinv` moves the top note down an octave:

```forth
c4 maj dinv note sine snd .    ;; G3 C4 E4
```

`drop2` and `drop3` are jazz voicing techniques for four-note chords. `drop2` takes the second-from-top note and drops it an octave:

```forth
c4 maj7 drop2 note saw snd .   ;; G3 C4 E4 B4
```

`drop3` drops the third-from-top:

```forth
c4 maj7 drop3 note saw snd .   ;; E3 C4 G4 B4
```

These create wider, more open voicings common in jazz guitar and piano.

## Transposition

`tp` shifts every integer on the stack by N semitones:

```forth
c4 maj 3 tp note sine snd .    ;; C major transposed up 3 = Eb major
c4 min7 -2 tp note saw snd .   ;; down 2 semitones = Bb minor 7
```

Unlike `oct` (which shifts a single note by octaves), `tp` shifts everything on the stack at once.

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
0 1 2 3 4 5 6 7 8 cycle minor note sine snd .
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

## Diatonic Harmony

`triad` and `seventh` build chords from scale degrees. Instead of specifying a chord type, you get whatever chord the scale produces at that degree:

```forth
0 major triad note sine snd .     ;; C E G — major triad (degree 0)
1 major triad note sine snd .     ;; D F A — minor triad (degree 1)
4 major triad note sine snd .     ;; G B D — major triad (degree 4)
```

`seventh` adds a fourth note:

```forth
0 major seventh note saw snd .    ;; C E G B — Cmaj7
4 major seventh note saw snd .    ;; G B D F — G7 (dominant)
```

The scale determines the chord quality automatically. Use `key!` to change the tonal center (default is C4):

```forth
g3 key! 0 major triad note sine snd .    ;; G major triad rooted at G3
a3 key! 0 minor seventh note saw snd .   ;; Am7 rooted at A3
```

A I-vi-IV-V chord progression using `pcycle`:

```forth
( 0 major seventh ) ( 5 major seventh )
( 3 major seventh ) ( 4 major seventh ) 4 pcycle
note saw snd .
```

Combine with voicings for smoother voice leading:

```forth
( 0 major seventh ) ( 5 major seventh inv )
( 3 major seventh ) ( 4 major seventh drop2 ) 4 pcycle
note saw snd .
```

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
( c3 maj7 ) ( f3 maj7 ) ( g3 dom7 ) ( c3 maj7 ) 4 pcycle
note sine snd .
```

Arpeggiate a chord across time divisions using `at`:

```forth
0 0.25 0.5 0.75 ( c4 e4 g4 b4 4 cycle note 0.5 decay sine snd . ) at
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
