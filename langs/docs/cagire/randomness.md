# Randomness

Music needs surprise. A line that plays identically every time gets boring fast. Cagire has a rich set of words for injecting randomness and controlled variation into your sequences.

## Random Numbers

`coin` pushes 0 or 1 with equal probability:

```forth
coin note sine s .    ;; either 0 or 1 as the note
```

`rand` takes a range and returns a random value. If both bounds are integers, the result is an integer. If either is a float, you get a float:

```forth
60 72 rand note sine s .       ;; random MIDI note from 60 to 72
0.3 0.9 rand gain sine s .     ;; random gain between 0.3 and 0.9
```

`exprand` and `logrand` give you weighted distributions. `exprand` is biased toward the low end, `logrand` toward the high end:

```forth
200.0 8000.0 exprand freq sine s .   ;; mostly low frequencies
200.0 8000.0 logrand freq sine s .   ;; mostly high frequencies
```

These are useful for parameters where perception is logarithmic, like frequency and duration.

## Conditional Execution

The probability words take a quotation and execute it with some chance. `chance` takes a float from 0.0 to 1.0, `prob` takes a percentage from 0 to 100:

```forth
{ hat s . } 0.25 chance    ;; 25% chance
{ hat s . } 75 prob        ;; 75% chance
```

Named probability words save you from remembering numbers:

| Word | Probability |
|------|------------|
| `always` | 100% |
| `almostAlways` | 90% |
| `often` | 75% |
| `sometimes` | 50% |
| `rarely` | 25% |
| `almostNever` | 10% |
| `never` | 0% |

```forth
{ hat s . } often          ;; 75%
{ snare s . } sometimes    ;; 50%
{ clap s . } rarely        ;; 25%
```

`always` and `never` are useful when you want to temporarily mute or unmute a voice without deleting code. Change `sometimes` to `never` to silence it, `always` to bring it back.

Use `?` and `!?` with `coin` for quick coin-flip decisions:

```forth
{ hat s . } coin ?     ;; execute if coin is 1
{ rim s . } coin !?    ;; execute if coin is 0
```

## Selection

`choose` picks randomly from n items on the stack:

```forth
kick snare hat 3 choose s .    ;; random drum hit
60 64 67 72 4 choose note sine s .    ;; random note from a set
```

When a chosen item is a quotation, it gets executed:

```forth
{ 0.1 decay } { 0.5 decay } { 0.9 decay } 3 choose
sine s .
```

`wchoose` lets you assign weights to each option. Push value/weight pairs:

```forth
kick 0.5  snare 0.3  hat 0.2  3 wchoose s .
```

Kick plays 50% of the time, snare 30%, hat 20%. Weights don't need to sum to 1 — they're normalized automatically.

`shuffle` randomizes the order of n items on the stack:

```forth
60 64 67 72 4 shuffle    ;; stack now has the same 4 values in random order
```

Combined with `note`, this gives you a random permutation of a chord every time the frame runs.

## Cycling

Cycling steps through values deterministically. No randomness — pure rotation.

`cycle` selects based on how many times this frame has triggered (its `runs` count):

```forth
60 64 67 3 cycle note sine s .    ;; 60, 64, 67, 60, 64, 67, ...
```

`pcycle` selects based on the line iteration count (`iter`):

```forth
kick snare 2 pcycle s .    ;; kick on even iterations, snare on odd
```

The difference matters when lines have different lengths. `cycle` counts per frame, `pcycle` counts per line.

Quotations work here too:

```forth
{ c4 note } { e4 note } { g4 note } 3 cycle
sine s .
```

`bounce` ping-pongs instead of wrapping around:

```forth
60 64 67 72 4 bounce note sine s .    ;; 60, 64, 67, 72, 67, 64, 60, 64, ...
```

## Periodic Execution

`every` runs a quotation once every n line iterations:

```forth
{ crash s . } 4 every    ;; crash cymbal every 4th iteration
```

`bjork` and `pbjork` use Bjorklund's algorithm to distribute k hits across n positions as evenly as possible. Classic Euclidean rhythms:

```forth
{ hat s . } 3 8 bjork     ;; tresillo: x..x..x. (by frame triggers)
{ hat s . } 5 8 pbjork    ;; cinquillo: x.xx.xx. (by line iterations)
```

`bjork` counts by frame triggers (how many times this particular frame has triggered). `pbjork` counts by line iterations. Some classic patterns:

| k | n | Name |
|---|---|------|
| 3 | 8 | tresillo |
| 5 | 8 | cinquillo |
| 5 | 16 | bossa nova |
| 7 | 16 | samba |

## Seeding

By default, every run produces different random values. Use `seed` to make randomness reproducible:

```forth
42 seed
60 72 rand note sine s .    ;; always the same "random" note
```

The seed is set at the start of the script. Same seed, same sequence. Useful when you want a specific random pattern to repeat.

## Combining Words

The real power comes from mixing techniques. A hi-hat pattern with ghost notes:

```forth
hat s
{ 0.3 0.6 rand gain } { 0.8 gain } 2 cycle
.
```

Full volume on even triggers, random quiet on odd triggers.

A bass line that changes every 4 bars:

```forth
{ c2 note } { e2 note } { g2 note } { a2 note } 4 pcycle
{ 0.5 decay } often
sine s .
```

Layered percussion with different densities:

```forth
{ kick s . } always
{ snare s . } 2 every
{ hat s . } 5 8 bjork
{ rim s . } rarely
```

A melodic frame with weighted note selection and random timbre:

```forth
c4 0.4  e4 0.3  g4 0.2  b4 0.1  4 wchoose note
0.3 0.7 rand decay
1.0 4.0 exprand harmonics
modal s .
```

The root note plays most often. Higher chord tones are rarer. Decay and harmonics vary continuously.
