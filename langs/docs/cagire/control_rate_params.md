A static parameter is the same number on every frame. A control rate parameter changes value each time the frame fires, because Cagire recomputes the whole stack from scratch on every run. This gives you movement at the pattern level: variation from one hit to the next, rotating cutoffs, random jitter between hits. The value is still a single number by the time Doux hears it. What varies is which number.

This article is an umbrella. The words it talks about are documented in full detail in the [Cycling](#), [Randomness](#), [Periodic Execution](#), and [Probability](#) articles. The purpose here is to frame those words as modulation tools.

For the bigger picture of how sounds reach Doux, see the [Audio Engine](#) and [Static Parameters](#) articles.

## Two Rules

Control rate modulation in Cagire follows two simple rules:

1. **Every frame reruns the script.** Any word that returns a different value on each run produces control rate modulation. `rand` rolls a fresh number. `cycle` advances its counter. `choose` picks again. By the time `.` fires, the value on the stack is the one Doux will see for this hit.
2. **The modulation happens in Cagire, not in Doux.** Doux receives a plain number for this one hit. No smoothing happens between hits. If you want smooth motion *between* frame boundaries, that is the job of [Audio Rate Modulation](#).

## Cycling as Modulation

`cycle` walks a list of values deterministically, one per frame trigger. Point it at any parameter:

```forth
saw snd c2 note 0.01 0.3 ad
400 800 1600 3200 4 cycle lpf .
```

The cutoff steps through 400, 800, 1600, 3200 on successive frames and wraps around. The same trick works on any parameter that takes a number:

```forth
sine snd c3 note
0.1 0.3 0.5 0.7 4 cycle gain .      ;; volume steps
```

`pcycle` counts by line iterations instead of frame triggers, and `bounce` ping pongs at the ends instead of wrapping. See the [Cycling](#) article for the full set.

## Randomness as Modulation

`rand`, `exprand`, `logrand`, `choose`, and `wchoose` roll fresh values per frame. Applied to a parameter, they give you sampled jitter:

```forth
sine snd 60 72 rand note 0.3 0.8 rand gain .
```

The note and the gain are different on every hit. `exprand` and `logrand` let you bias the distribution when a uniform random would sit in the wrong register of the parameter range:

```forth
saw snd c3 note 200 8000 exprand lpf .   ;; mostly low cutoffs
```

`choose` works when the set of values is discrete rather than a range:

```forth
sine tri saw pulse 4 choose snd c4 note .
```

See the [Randomness](#) article.

## Periodic and Probabilistic Gating

The periodic words (`every`, `except`, `bjork`) and the probability words (`chance`, `always`, `rarely`, and friends) do not directly change a parameter value. They decide whether a whole block of code fires at all. Wrap a parameter change in a quotation and you can gate it on or off:

```forth
saw snd c3 note 0.01 0.3 ad
( 2000 lpf ) 4 every                ;; bright filter every 4 iterations
( 500 lpf ) 4 except
.
```

On iteration 0, 4, 8, ... the cutoff is 2000. Every other iteration it is 500. The `every` / `except` pair covers both cases so the register always has a cutoff set.

Probability works the same way:

```forth
saw snd c3 note
( 0.6 verb ) 0.3 chance             ;; 30% of frames get reverb
.
```

See the [Periodic Execution](#) and [Probability](#) articles.

## Modulation Inside One Frame with at

`at` subdivides a frame into smaller time slots and reruns a quotation at each slot. Inside the quotation, control rate words roll fresh values on every subdivision, so you get four random values in a single frame instead of one:

```forth
0 0.25 0.5 0.75 (
  hat snd 0.3 0.8 rand gain .
) at
```

Four hats per frame, each with its own random gain. This is the only way to drive a parameter faster than the frame rate without reaching for audio rate modulation. See the [Timing](#) article for the full `at` vocabulary.

## The Boundary with Audio Rate

Control rate gives you one number per frame (or per `at` subdivision). Between those moments the parameter holds. That is fine for stepped music: each hit gets its own value and nothing moves during the hit.

When you want the parameter to move *during* the hit, as in a filter opening smoothly while the note sustains, control rate does not help. That is the job of [Audio Rate Modulation](#).

## Combining

Control rate words layer freely. A single patch can draw from all four families at once:

```forth
saw snd
[ c2 g2 eb2 f2 ] cycle note       ;; cycle through a bass line
400 800 1600 3200 4 cycle lpf     ;; step the cutoff
0.4 0.8 rand gain                 ;; jitter the gain
0.01 0.2 ad
( 0.4 verb ) 4 every              ;; reverb every 4 hits
.
```

Four independent modulation sources, each doing its own thing, all flowing into a single Doux event per frame.
