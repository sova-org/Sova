# Timing

Every frame has a duration. By default, sounds emit at the very start of that duration. `at` changes *when* within the frame sounds fire — giving you sub-frame rhythmic control without adding more frames.

## The Basics

`at` pops a quotation from the stack, then drains remaining stack values as timing offsets. It loops the quotation once per offset. Each value is a fraction of the frame duration: 0 = start, 0.5 = halfway, 1.0 = next frame boundary.

```forth
0 0.5 ( kick snd . ) at           ;; kick at start and midpoint
0 0.25 0.5 0.75 ( hat snd . ) at  ;; four hats, evenly spaced
```

Each iteration gets its own independent state — nondeterministic ops (rand, choose, coin) roll fresh values per delta:

```forth
0 0.5 ( kick snd 1 4 rand n . ) at   ;; different random sample each hit
```

If you want to run side-effects per delta without emitting sound, just leave `.` out of the quotation:

```forth
0 0.5 ( !x ) at   ;; set variable at two time points, no emit
```

## Polyphony Inside at

CycleLists inside `at` quotations work as usual — each delta iteration expands polyphonically:

```forth
0 0.5 ( [c4 e4 g4] note sine snd . ) at   ;; chord at 0, chord at 0.5
```

## Generating Deltas

You rarely type deltas by hand. Use generators:

Evenly spaced via `.,`:

```forth
0 1 0.25 ., ( hat snd . ) at        ;; 0 0.25 0.5 0.75 1.0
```

Euclidean distribution via `euclid`:

```forth
3 8 euclid ( hat snd . ) at         ;; 3 hats at euclidean positions
```

Random timing via `gen`:

```forth
( 0.0 1.0 rand ) 4 gen ( hat snd . ) at   ;; 4 hats at random positions
```

Geometric spacing via `geom..`:

```forth
0.0 2.0 4 geom.. ( hat snd . ) at  ;; exponentially spaced
```

## Gating at

Wrap the whole expression in quotations for conditional timing:

```forth
( 0 0.25 0.5 0.75 ( hat snd . ) at ) 2 every    ;; 16th-note hats every other bar

( 0 0.5 ( kick snd . ) at ) 0.5 chance           ;; 50% chance of double-hit
```

When the quotation doesn't execute, no deltas are set — you get no emit from that expression.

## Fitting Samples to Beats

`loop` stretches or compresses a sample to fit a given number of beats:

```forth
"break" snd 4 loop .    ;; fit the sample into 4 beats
"break" snd 2 loop .    ;; same sample, half the time
```

This adjusts the sample's playback speed so it aligns with the frame duration and tempo.
