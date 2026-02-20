# Timing

Every frame has a duration. By default, sounds emit at the very start of that duration. `at` changes *when* within the frame sounds fire — giving you sub-frame rhythmic control without adding more frames.

## The Basics

`at` drains the entire stack and stores the values as timing offsets. Each value is a fraction of the frame duration: 0 = start, 0.5 = halfway, 1.0 = next frame boundary.

```forth
0.5 at kick s .         ;; kick at the midpoint
```

Push multiple values before calling `at` to get multiple emits from a single `.`:

```forth
0 0.5 at kick s .       ;; two kicks: one at start, one at midpoint
0 0.25 0.5 0.75 at hat s .   ;; four hats, evenly spaced
```

The deltas persist across multiple `.` calls until `clear` or a new `at`:

```forth
0 0.5 at
kick s .                ;; 2 kicks
hat s .                 ;; 2 hats (same timing)
clear
snare s .               ;; 1 snare (deltas cleared)
```

## Cross-product: at Without arp

Without `arp`, deltas multiply with polyphonic voices. If you have 3 notes and 2 deltas, you get 6 emits — every note at every delta:

```forth
0 0.5 at
c4 e4 g4 note sine s .   ;; 6 emits: 3 notes x 2 deltas
```

This is a chord played twice per frame.

## 1:1 Pairing: at With arp

`arp` changes the behavior. Instead of cross-product, deltas and arp values pair up 1:1. Each delta gets one note from the arpeggio:

```forth
0 0.33 0.66 at
c4 e4 g4 arp note sine s .   ;; c4 at 0, e4 at 0.33, g4 at 0.66
```

If the lists differ in length, the shorter one wraps around:

```forth
0 0.25 0.5 0.75 at
c4 e4 arp note sine s .       ;; c4, e4, c4, e4 at 4 time points
```

This is THE key distinction. Without `arp`: every note at every time. With `arp`: one note per time slot.

## Generating Deltas

You rarely type deltas by hand. Use generators:

Evenly spaced via `.,`:

```forth
0 1 0.25 ., at hat s .        ;; 0 0.25 0.5 0.75 1.0
```

Euclidean distribution via `euclid`:

```forth
3 8 euclid at hat s .         ;; 3 hats at positions 0, 3, 5
```

Random timing via `gen`:

```forth
{ 0.0 1.0 rand } 4 gen at hat s .   ;; 4 hats at random positions
```

Geometric spacing via `geom..`:

```forth
0.0 2.0 4 geom.. at hat s .  ;; exponentially spaced
```

## Gating at

Wrap `at` expressions in quotations for conditional timing:

```forth
{ 0 0.25 0.5 0.75 at } 2 every    ;; 16th-note hats every other bar
hat s .

{ 0 0.5 at } 0.5 chance           ;; 50% chance of double-hit
kick s .
```

When the quotation doesn't execute, no deltas are set — you get the default single emit at beat start.
