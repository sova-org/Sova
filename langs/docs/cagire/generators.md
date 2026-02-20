# Generators & Sequences

Sequences of values drive music: arpeggios, parameter sweeps, rhythmic patterns. Cagire has dedicated words for building sequences on the stack, transforming them, and collapsing them to single values.

## Ranges

`..` pushes an integer range onto the stack. Both endpoints are inclusive. If start exceeds end, it counts down:

```forth
1 5 ..          ;; 1 2 3 4 5
5 1 ..          ;; 5 4 3 2 1
0 7 ..          ;; 0 1 2 3 4 5 6 7
```

`.,` adds a step parameter. Works with floats:

```forth
0 1 0.25 .,     ;; 0 0.25 0.5 0.75 1
0 10 2 .,       ;; 0 2 4 6 8 10
1 0 0.5 .,      ;; 1 0.5 0 (descending)
```

`geom..` builds a geometric sequence. Takes start, ratio, and count:

```forth
1 2 4 geom..    ;; 1 2 4 8
100 0.5 4 geom..  ;; 100 50 25 12.5
```

Build a harmonic series:

```forth
110 2 5 geom.. 5 rev freq
```

That gives you 110, 220, 440, 880, 1760 (reversed), ready to feed into `freq`.

## Computed Sequences

`gen` executes a quotation n times and collects all results. The quotation must push exactly one value per call:

```forth
{ 1 6 rand } 4 gen       ;; 4 random values between 1 and 6
{ coin } 8 gen            ;; 8 random 0s and 1s
```

Contrast with `times`, which executes for side effects and does not collect. `times` sets `@i` to the current index:

```forth
4 { @i } times            ;; 0 1 2 3 (pushes @i each iteration)
4 { @i 60 + note sine s . } times   ;; plays 4 notes, collects nothing
```

The distinction: `gen` is for building data. `times` is for doing things.

## Euclidean Patterns

`euclid` distributes k hits evenly across n positions and pushes the hit indices:

```forth
3 8 euclid      ;; 0 3 5
4 8 euclid      ;; 0 2 4 6
5 8 euclid      ;; 0 1 3 5 6
```

`euclidrot` adds a rotation parameter that shifts the pattern:

```forth
3 8 0 euclidrot   ;; 0 3 5 (no rotation)
3 8 1 euclidrot   ;; 1 4 6
3 8 2 euclidrot   ;; 1 4 6
```

These give you raw indices as data on the stack. This is different from `bjork` and `pbjork` (covered in the Randomness article), which execute a quotation on matching triggers. `euclid` gives you numbers to work with; `bjork` triggers actions.

## Transforming Sequences

Four words reshape values already on the stack. All take n (the count of items to operate on) from the top:

`rev` reverses order:

```forth
1 2 3 4 4 rev     ;; 4 3 2 1
c4 e4 g4 3 rev    ;; g4 e4 c4 (descending arpeggio)
```

`shuffle` randomizes order:

```forth
c4 e4 g4 b4 4 shuffle   ;; random permutation each time
```

`sort` and `rsort` for ascending and descending:

```forth
3 1 4 1 5 5 sort    ;; 1 1 3 4 5
3 1 4 1 5 5 rsort   ;; 5 4 3 1 1
```

## Reducing Sequences

`sum` and `prod` collapse n values into one:

```forth
1 2 3 4 4 sum      ;; 10
1 2 3 4 4 prod     ;; 24
```

Useful for computing averages or accumulating values:

```forth
{ 1 6 rand } 4 gen 4 sum   ;; sum of 4 dice rolls
```

## Replication

`dupn` duplicates a value n times:

```forth
440 3 dupn        ;; 440 440 440
c4 4 dupn         ;; c4 c4 c4 c4
```

Build a drone chord — same note, different octaves:

```forth
c3 note 0.5 gain sine s .
c3 note 12 + 0.5 gain sine s .
c3 note 24 + 0.3 gain sine s .
```

Or replicate a value for batch processing:

```forth
0.5 4 dupn 4 sum   ;; 2.0
```

The generator words produce raw material. The transform words shape it. Together they let you express complex musical ideas in a few words.
