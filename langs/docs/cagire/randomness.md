Music needs surprise. A line that plays identically every time gets boring fast. This article covers Cagire's words for drawing non-deterministic *values*: random numbers, weighted selection, shuffling, and seeding. For running a quotation with some chance, see the [Probability](#) article. For deterministic stepping through a list of values, see [Cycling](#). For deterministic firing on certain iterations, see [Periodic Execution](#).

## Random Numbers

`coin` pushes 0 or 1 with equal probability:

```forth
coin note sine snd .    ;; either 0 or 1 as the note
```

`rand` takes a range and returns a random value. If both bounds are integers, the result is an integer. If either is a float, you get a float:

```forth
60 72 rand note sine snd .       ;; random MIDI note from 60 to 72
0.3 0.9 rand gain sine snd .     ;; random gain between 0.3 and 0.9
```

`exprand` and `logrand` give you weighted distributions over a range. `exprand` is biased toward the low end, `logrand` toward the high end:

```forth
200.0 8000.0 exprand freq sine snd .   ;; mostly low frequencies
200.0 8000.0 logrand freq sine snd .   ;; mostly high frequencies
```

These are useful for parameters where perception is logarithmic (frequency, duration, gain), because a uniform `rand` over the same range will spend most of its values in the upper octave.

Each fresh draw of any of these words is independent. Inside an `at` quotation they re-roll on every subdivision (see [Timing](#) for the rule).

## Random Selection

`choose` picks one item randomly from `n` items on the stack:

```forth
kick snare hat 3 choose snd .          ;; random drum hit
60 64 67 72 4 choose note sine snd .   ;; random note from a set
```

Stack effect: `(v1..vn n -- selected)`. As with `cycle`, when the chosen item is a quotation it gets executed instead of being left on the stack:

```forth
( 0.1 decay ) ( 0.5 decay ) ( 0.9 decay ) 3 choose
sine snd .
```

This is the standard idiom for "pick one of several alternative phrases."

`wchoose` lets you assign weights to each option. Push value/weight pairs and the count of *pairs*:

```forth
kick 0.5  snare 0.3  hat 0.2  3 wchoose snd .
```

Kick plays 50% of the time, snare 30%, hat 20%. Weights don't have to sum to 1, they're normalized automatically. Stack effect: `(v1 w1 v2 w2 ... n -- selected)`.

`shuffle` randomizes the order of `n` items on the stack in place:

```forth
60 64 67 72 4 shuffle    ;; same 4 values, random order
```

Combined with `note`, this gives you a random permutation of a chord every time the frame runs. Useful for arpeggiations that should never settle into a recognizable pattern.

## Seeding

By default every run produces fresh random values, so a `rand` call gives a different number each time the frame triggers. Use `seed` to make randomness reproducible:

```forth
42 seed
60 72 rand note sine snd .    ;; always the same "random" note
```

The seed is set at the start of the script. Same seed, same sequence. Useful when you've stumbled into a random pattern you like and want to lock it in, or when you're debugging a section that depends on a specific draw.

## Combining Words

Most interesting things happen when you mix randomness with cycling, periodic firing, and probability gates. A hi-hat with ghost notes:

```forth
hat snd
( 0.3 0.6 rand gain ) ( 0.8 gain ) 2 cycle
.
```

Full volume on even triggers, random quiet on odd triggers. `cycle` does the alternation, `rand` provides the variation within one of the slots.

Layered percussion with different densities:

```forth
( kick snd . ) always
( snare snd . ) 2 every
( hat snd . ) 5 8 bjork
( rim snd . ) rarely
```

Each line has its own firing rule: deterministic for kick/snare/hat, random for rim. See the [Periodic Execution](#) article for `every` and `bjork`.

A melodic frame with weighted note selection and random timbre:

```forth
c4 0.4  e4 0.3  g4 0.2  b4 0.1  4 wchoose note
0.3 0.7 rand decay
1.0 4.0 exprand harmonics
modal snd .
```

The root plays most often. Higher chord tones are rarer. Decay and harmonics vary continuously, with `exprand` keeping `harmonics` mostly low.

For the wider framing of random values as a way to modulate sound parameters, see [Control Rate Modulation](#). For random motion that happens continuously at audio rate instead of once per frame, see [Audio Rate Modulation](#).
