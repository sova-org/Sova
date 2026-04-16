# Chords

A chord in Cagire is a *quality* (a set of intervals like `[0 4 7]` for a major triad) looked up by name when you call `chord`. You write the name as a plain word and `chord` resolves it at emit time:

```forth
c4 note
maj7 chord
sine snd 
.5 decay
.3 verb
2 fm 0.99 fmh
.
```

This emits `[60 64 67 71]` polyphonically. Every chord tone goes out on its own voice. `maj7` is just a string value, which means qualities are first-class data: you can put them in lists, choose between them, and stash them in variables.

## How chord names resolve

Cagire tries to interpret each token in order: number, defined word, variable, note name, interval, and finally, *unknown words become strings*. Chord names take advantage of that last step. They aren't dictionary words, they fall through to strings, and `chord` looks the string up in the chord table at runtime. You can also write the name explicitly as a string literal whenever you want to be unambiguous:

```forth
;; same as `maj7 chord` above
c4 note
"maj7" chord
sine snd
.
```

The string form is only **required** when the bare token would mean something else:

- **Names colliding with the interval words** `M7 m7 M9 m9 M6 m6 M13 m13` (those words add semitones to a note). Use `"m7"`, `"M9"`, etc. when you mean the chord.
- **Names starting with `+`** (`+7`, `+M7`, `+maj7`, `+`). The number parser eats the `+` and turns the token into an integer. Use `"+7"`, `"+M7"`, `"+"`.
- **Names containing `/`** (`6/9`, `m6/9`). These look like ratio literals. Use `"6/9"`, `"m6/9"`.

Everything else can be written bare, including names with `#` (`7#11`, `maj7#11`), names with `b` (`7b9`, `dom7b9b5`), and even Unicode aliases (`Δ7`, `ø7`).

## Numeric Shortcuts

A handful of qualities can also be passed as bare integers, no quotes and no name needed:

| Number | Quality |
|--------|---------|
| `5` | `pwr` |
| `6` | `maj6` |
| `7` | `dom7` |
| `9` | `dom9` |
| `11` | `dom11` |
| `13` | `dom13` |

```forth
c4 note 7 chord sine snd .       ;; same as dom7 chord
c3 note 5 chord saw  snd .       ;; power chord
```

## Alias Conventions

Most qualities accept several names so you can reach for whichever notation feels natural:

- **`m` ↔ `min`** for every minor chord (e.g. `min7` = `"m7"`, `min9` = `"m9"`)
- **`M` ↔ `maj`** for every major-7-family chord (e.g. `maj7` = `"M7"`, `maj9` = `"M9"`)
- **`o`, `o7`** for diminished triad / seventh
- **`+`, `+7`, `+M7`** for the augmented family (always quoted)
- **`h7`, `hdim7`, `halfdim7`, `ø7`** for half-diminished (a.k.a. `m7b5`)
- **`Δ7`** for major 7
- **`#` and `s`** are interchangeable in altered names (`7#9` = `7s9`, `maj7#11` = `maj7s11`)
- **Numeric** shortcuts above (no quotes) for the most common cases

## Built-in Qualities

Every alias listed below is interchangeable with its canonical name. Aliases that need quotes are shown quoted; the rest can be written bare.

**Triads:**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `maj` | `M`, `major` | 0 4 7 | 60 64 67 |
| `m` | `min`, `minor` | 0 3 7 | 60 63 67 |
| `dim` | `o` | 0 3 6 | 60 63 66 |
| `aug` | `"+"` | 0 4 8 | 60 64 68 |
| `sus2` |  | 0 2 7 | 60 62 67 |
| `sus4` | `sus` | 0 5 7 | 60 65 67 |
| `pwr` | `5`, `power` | 0 7 | 60 67 |

**Sixth chords:**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `maj6` | `6`, `"M6"` | 0 4 7 9 | 60 64 67 69 |
| `min6` | `"m6"` | 0 3 7 9 | 60 63 67 69 |
| `maj69` | `M69`, `"6/9"` | 0 4 7 9 14 | 60 64 67 69 74 |
| `min69` | `m69`, `"m6/9"` | 0 3 7 9 14 | 60 63 67 69 74 |

**Seventh chords:**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `maj7` | `"M7"`, `Δ7` | 0 4 7 11 | 60 64 67 71 |
| `min7` | `"m7"` | 0 3 7 10 | 60 63 67 70 |
| `dom7` | `7` | 0 4 7 10 | 60 64 67 70 |
| `dim7` | `o7` | 0 3 6 9 | 60 63 66 69 |
| `m7b5` | `min7b5`, `h7`, `hdim7`, `halfdim7`, `ø7` | 0 3 6 10 | 60 63 66 70 |
| `minmaj7` | `mM7`, `mmaj7` | 0 3 7 11 | 60 63 67 71 |
| `aug7` | `"+7"` | 0 4 8 10 | 60 64 68 70 |
| `augmaj7` | `"+M7"`, `"+maj7"` | 0 4 8 11 | 60 64 68 71 |
| `7sus4` | `sus47` | 0 5 7 10 | 60 65 67 70 |
| `7sus2` | `sus27` | 0 2 7 10 | 60 62 67 70 |

**Ninth chords:**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `dom9` | `9` | 0 4 7 10 14 | 60 64 67 70 74 |
| `maj9` | `"M9"` | 0 4 7 11 14 | 60 64 67 71 74 |
| `min9` | `"m9"` | 0 3 7 10 14 | 60 63 67 70 74 |
| `9sus4` | `sus49` | 0 5 7 10 14 | 60 65 67 70 74 |

**Eleventh chords:**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `dom11` | `11` | 0 4 7 10 14 17 | 60 64 67 70 74 77 |
| `maj11` | `M11` | 0 4 7 11 14 17 | 60 64 67 71 74 77 |
| `min11` | `m11` | 0 3 7 10 14 17 | 60 63 67 70 74 77 |

**Thirteenth chords:**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `dom13` | `13` | 0 4 7 10 14 21 | 60 64 67 70 74 81 |
| `maj13` | `"M13"` | 0 4 7 11 14 21 | 60 64 67 71 74 81 |
| `min13` | `"m13"` | 0 3 7 10 14 21 | 60 63 67 70 74 81 |

**Add chords:**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `add9` | `add2` | 0 4 7 14 | 60 64 67 74 |
| `add11` | `add4` | 0 4 7 17 | 60 64 67 77 |
| `madd9` | `madd2` | 0 3 7 14 | 60 63 67 74 |

**Altered dominants:**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `dom7b9` | `7b9` | 0 4 7 10 13 | 60 64 67 70 73 |
| `dom7s9` | `7s9`, `7#9` | 0 4 7 10 15 | 60 64 67 70 75 |
| `dom7b5` | `7b5` | 0 4 6 10 | 60 64 66 70 |
| `dom7s5` | `7s5`, `7#5` | 0 4 8 10 | 60 64 68 70 |
| `dom7s11` | `7s11`, `7#11` | 0 4 7 10 18 | 60 64 67 70 78 |
| `dom7b9b5` | `7b9b5` | 0 4 6 10 13 | 60 64 66 70 73 |
| `dom7s9b5` | `7s9b5`, `7#9b5` | 0 4 6 10 15 | 60 64 66 70 75 |
| `dom7b9s5` | `7b9s5`, `7b9#5` | 0 4 8 10 13 | 60 64 68 70 73 |
| `dom7s9s5` | `7s9s5`, `7#9#5` | 0 4 8 10 15 | 60 64 68 70 75 |
| `alt` | `dom7alt`, `7alt` | 0 4 10 13 15 | 60 64 70 73 75 |

**Major sharp 11 (Lydian):**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `maj7s11` | `M7s11`, `maj7#11`, `M7#11` | 0 4 7 11 18 | 60 64 67 71 78 |
| `maj9s11` | `M9s11`, `maj9#11`, `M9#11` | 0 4 7 11 14 18 | 60 64 67 71 74 78 |

**Minor-major extensions:**

| Canonical | Aliases | Intervals | Example (C4) |
|-----------|---------|-----------|--------------|
| `minmaj9` | `mM9`, `mmaj9` | 0 3 7 11 14 | 60 63 67 71 74 |
| `minmaj11` | `mM11`, `mmaj11` | 0 3 7 11 14 17 | 60 63 67 71 74 77 |
| `minmaj13` | `mM13`, `mmaj13` | 0 3 7 11 14 21 | 60 63 67 71 74 81 |

## Anchored Voicings

By default, a chord is voiced from the root upward in close position. Add `anchor` to ask Cagire for the inversion *and* register that places one of the chord tones closest to a target pitch. The anchor parameter takes a MIDI note:

```forth
;; First example with anchor on g4
c4 note
maj7 chord
g4 anchor
sine snd .
```

```forth
;; Another one now with anchor on d5
c4 note
maj7 chord
d5 anchor
sine snd .
```

This searches every inversion and every plausible octave shift, scoring each candidate voicing by how close its closest chord tone is to the anchor (and breaking ties by total displacement and upward bias). The best-scoring voicing is what gets emitted. Use `anchor` to keep a chord progression smooth: voice each chord around the same anchor pitch and Cagire will pick inversions that minimize voice leading.

## Selecting Single Tones with `cn`

When `cn` (chord note) is set, the chord doesn't emit polyphonically. Instead it emits *one* note picked from the realized voicing by index, low to high:

```forth
c4 note
min7 chord
[ 0 3 0 3 ] cycle cn
sine snd .
```

Indexes wrap by octave: a value above the voicing length climbs upward, and negative values descend. So with a 4-note chord and `cn = 4`, you get the root one octave up; `cn = -1` gives the highest tone an octave down. This makes `cn` perfect for arpeggiation across multiple octaves without ever falling off the chord:

```forth
c4 note
min7 chord
[ 0 1 2 3 4 5 6 7 ] cycle cn
sine snd .
```

`cn` works with or without `anchor`. Without an anchor, the indexes count through the canonical close-position voicing; with an anchor, they count through whichever inversion `anchor` selected.

## Storing and Choosing Qualities

Because chord names are plain string values, they can be stored, cycled, or chosen at runtime:

```forth
[ maj7 min7 dom7 maj7 ] cycle chord
c4 note sine snd .
```

```forth
( maj7 ) ( min9 ) ( m7b5 ) 3 choose
chord
c4 note sine snd .
```

You can also stash a quality in a variable for reuse across frames:

```forth
maj9 !my_chord
c4 note @my_chord chord sine snd .
```

## Putting It Together

A I-IV-V-I progression cycling per line iteration:

```forth
( c3 note maj7 ) ( f3 note maj7 )
( g3 note 7 ) ( c3 note maj7 ) 4 pcycle chord
sine snd
.5 decay .3 chorus . 
```

Arpeggiating a min7 chord across the frame using `at`:

```forth
0 0.25 0.5 0.75 (
  c4 note min7 chord
  [ 0 1 2 3 ] cycle cn
  0.5 decay sine snd .
) at
```

A jazz vamp with smooth voice leading via `anchor`:

```forth
( c4 note maj9 chord )
( a3 note min9 chord )
( d4 note min7 chord )
( g3 note 7   chord )
4 pcycle
g4 anchor
sine snd
.2 chorus
1 decay
.
```
