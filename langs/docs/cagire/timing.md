# Timing

Every frame has a duration. By default, sounds emit at the very start of that duration. `at` changes *when* within the frame sounds fire, giving you sub-frame rhythmic control without adding more frames.

## The Basics

`at` pops a quotation from the stack, then drains remaining stack values as timing offsets. It loops the quotation once per offset. Each value is a fraction of the frame duration: 0 = start, 0.5 = halfway, 1.0 = next frame boundary.

```forth
;; kick at start and midpoint
0 0.5 ( kick snd . ) at
;; four hats, evenly spaced
0 0.25 0.5 0.75 ( hat snd . ) at 
```

Each iteration gets its own independent state. 
Nondeterministic ops (rand, choose, coin) roll fresh values per delta. 
This is also how you get control rate modulation faster than the frame rate: see [Modulations](#).

```forth
;; different random sample each hit
0 0.5 ( kick snd 1 4 rand n . ) at
```

If you want to run side-effects per delta without emitting sound, just leave `.` out of the quotation:

```forth
0 0.5 ( !x ) at   ;; set variable at two time points, no emit
```

## Nesting at

`at` is composable: you can put another `at` inside its quotation.
The inner `at` subdivides the outer's slot, so timings compose multiplicatively rather than overwriting each other.

```forth
;; 4 hats at 0, 0.25, 0.5, 0.75
0 0.5 (
  0 0.5 ( hat snd . ) at 
) at
```

The outer `at` splits the frame into two halves; the inner `at` then splits each half into two halves again.
Three levels of binary nesting give eight evenly-spaced hits, and so on.
The same rule applies to pattern-mode `at`: each pattern hit owns a slice whose width is its gate, and a nested `at` subdivides that exact slice.
You can also mix the two modes freely:

```forth
0 0.5 ( "x.x." ( hat snd . ) at ) at  ;; pattern inside float
"x.x." ( 0 0.5 ( hat snd . ) at ) at  ;; float inside pattern
```

Random and cycling state still re-rolls per leaf iteration: each subdivision gets its own fresh draw, no matter how deeply you nest.

State you set *before* an `at` survives the whole loop and is visible to every subdivision. This means common setup can live outside the quotation:

```forth
"sine" sound 0 0.5 ( 60 note . ) at   ;; sine on both hits
```

And state set inside one iteration of an outer `at` survives any inner `at` it runs, so the outer body's trailing emits still see what the body set up earlier.

## Polyphony Inside at

Cycling lists inside `at` quotations work as usual. Each delta iteration expands polyphonically:

```forth
;; chord at 0, chord at 0.5
0 0.5 (
  c4 e4 g4 note sine snd . 
) at
```

## Generating Deltas

You rarely type deltas by hand. Use generators:

Evenly spaced via `.,`:

```forth
;; 0 0.25 0.5 0.75 1.0
0 1 0.25 ., (
  hat snd .
) at
```

Euclidean distribution via `euclid`:

```forth
;; 3 hats at euclidean positions
3 8 euclid (
  hat snd . )
at
```

Random timing via `gen`:

```forth
;; 4 hats at random positions
( 0.0 1.0 rand ) 4 gen
( hat snd . ) at
```

Geometric spacing via `geom..`:

```forth
;; exponentially spaced
0.0 2.0 4 geom.. (
  hat snd .
) at
```

## Gating at

Wrap the whole expression in quotations for conditional timing:

```forth
;; 16th-note hats every other bar
( 0 0.25 0.5 0.75 ( hat snd . ) at ) 2 every
```

```forth
;; 50% chance of double-hit
( 0 0.5 ( kick snd . ) at ) 0.5 chance
```

When the quotation doesn't execute, no deltas are set: you get no emit from that expression.

## Fitting Samples to Beats

`loop` stretches or compresses a sample to fit a given number of beats:

```forth
"break" snd 4 loop .    ;; fit the sample into 4 beats
"break" snd 2 loop .    ;; same sample, half the time
```

This adjusts the sample's playback speed so it aligns with the frame duration and tempo.
