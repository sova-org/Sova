Periodic execution words run a quotation only on certain iterations of a line, leaving it dormant the rest of the time. They are how you build patterns like "kick on every beat, snare on every fourth, crash once a bar, hi-hats following a Euclidean rhythm" without writing any conditional logic by hand.

All the words in this article are deterministic. They read the line iteration counter (`iter`) and decide whether to fire. Same iteration, same decision. They don't care about wall-clock time, just about how many times the line has been visited.

## every

`every` runs a quotation once every `n` line iterations:

```forth
( crash snd . ) 4 every    ;; crash on iterations 0, 4, 8, 12, ...
```

Stack effect: `(quot n --)`. The first hit is at iteration 0, then every `n` after that. If you want the first hit later, use `every+` (below).

`every` is the natural way to layer accents on top of a steady stream:

```forth
( hat snd . )                      ;; hat on every iteration (no every needed)
( open_hat snd . ) 4 every         ;; open hat replaces hat every 4
```

## except

`except` is the inverse. It runs a quotation on *every* iteration except multiples of `n`:

```forth
( hat snd . ) 4 except    ;; hat on iterations 1, 2, 3, 5, 6, 7, 9, 10, 11, ...
```

Stack effect: `(quot n --)`. Useful for "do this thing all the time except when something more important happens." The classic application is muting a steady part on the downbeats so an accent stands out:

```forth
( hat snd . ) 4 except
( crash snd . ) 4 every
```

## every+ / except+

Both `every` and `except` have a `+` variant that takes a phase offset, shifting the firing schedule by the offset value:

```forth
( snare snd . ) 4 2 every+     ;; fires at iter 2, 6, 10, 14...
( snare snd . ) 4 2 except+    ;; skips at iter 2, 6, 10, 14...
```

Stack effect: `(quot n offset --)`. Without the offset, `every` fires at `0, n, 2n, ...`. With offset `k`, it fires at `k, n+k, 2n+k, ...`. The offset is taken modulo `n`, so values larger than `n` simply wrap.

The point of the `+` variants is to interleave patterns that share the same period but land on different beats. A backbeat snare on top of a steady kick:

```forth
( kick snd . ) 4 every          ;; kick on 0, 4, 8, ...
( snare snd . ) 4 2 every+      ;; snare on 2, 6, 10, ...
```

Or two voices of a hi-hat pattern that interlock:

```forth
( hat snd 0.6 gain . ) 2 every       ;; hat on even iters
( hat snd 0.3 gain . ) 2 1 every+    ;; ghost hat on odd iters
```

## bjork / pbjork

`bjork` distributes `k` hits across `n` positions as evenly as possible using Bjorklund's algorithm. The result is a Euclidean rhythm: the same kind of pattern used by tresillo, cinquillo, and many traditional grooves around the world.

```forth
( hat snd . ) 3 8 bjork     ;; tresillo: x..x..x.
( hat snd . ) 5 8 bjork     ;; cinquillo: x.xx.xx.
```

Stack effect: `(quot k n --)`. `bjork` fires when the *frame trigger* counter (`runs`) lands on one of the active steps, so two `bjork` lines with the same `(k, n)` will stay locked together as long as they trigger at the same rate.

`pbjork` is the line-iteration variant: it consults `iter` instead of `runs`, the same way `pcycle` relates to `cycle`. Use it when you want a Euclidean pattern that walks at the line's iteration rate rather than the frame rate:

```forth
( hat snd . ) 5 8 pbjork    ;; cinquillo by line iteration
```

Some classic Euclidean patterns:

| k | n | Name |
|---|---|------|
| 3 | 8 | tresillo |
| 5 | 8 | cinquillo |
| 5 | 16 | bossa nova |
| 7 | 16 | samba |
| 3 | 7 | Ruchenitza |
| 7 | 12 | West African |

## Sectional gating

The following words also read `iter`, but they gate a line over a *section* of its life rather than on a repeating schedule. They are how you write intros, drops, and "this voice plays from iteration 16 onward" without reaching for counters.

### first

`first` fires only on the first `n` iterations, then stays silent:

```forth
( roll snd 0.4 gain . ) 16 first    ;; roll for 16 iterations, then gone
```

Stack effect: `(quot n --)`.

### after

`after` is the complement. Silent for `n` iterations, then fires on every one after that:

```forth
( kick snd . ) 16 after    ;; silent for 16, then steady kick
```

Stack effect: `(quot n --)`. Pair `first` with `after` using the same `n` to hand a voice off at a bar boundary:

```forth
( roll snd . ) 16 first
( kick snd . ) 16 after
```

`after 0` is a valid no-op gate: it fires on every iteration. That lets you flip the count from `0` to `16` as a programmable "silence then drop" knob without restructuring the line.

### once

`once` fires only on iteration 0. Sugar for the common "do this exactly when the line starts" case:

```forth
( crash snd . ) once    ;; one hit on the downbeat, then done
```

Stack effect: `(quot --)`. No count needed.

## Layering Periodic Words

Periodic words compose freely. The most common pattern is to layer several voices, each with its own firing rule:

```forth
( kick snd . ) always
( snare snd . ) 4 2 every+
( hat snd . )  always
( open_hat snd . ) 8 every
( crash snd . ) 32 every
```

(`always` lives in the [Probability](#) article. It just unconditionally executes the quotation, useful for muting voices by swapping `always` for `never` without restructuring the code.)

A Euclidean drum kit with phase-offset accents:

```forth
( kick snd . ) 3 8 bjork
( snare snd . ) 4 2 every+
( hat snd . ) 5 8 bjork
( ride snd . ) 7 16 bjork
```

For *cycling through values* on each iteration (rather than gating execution), see the [Cycling](#) article. For sub-frame timing within a single iteration, see [Timing](#). The `at` word splits a single frame into evenly-spaced sub-events and pairs naturally with these periodic words. For the wider framing of periodic gating as parameter modulation, see [Control Rate Modulation](#).
