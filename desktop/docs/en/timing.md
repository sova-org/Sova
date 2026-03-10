# Timing

Sova measures time in beats. One beat at 120 BPM lasts 500 ms. At 60 BPM, a
full second. Frame durations, waits, and note lengths are all expressed in
beats. Change the tempo and your patterns follow — no recalculation needed.

## Tempo and sync

The clock runs on Ableton Link. Every Link-enabled app on the network shares
the same tempo and beat position. Change the BPM in Sova and Ableton Live sees
it. Change it in Live and Sova follows. If nothing else is on the network, Sova
runs its own clock. No configuration required. The two-thread execution model
ensures precise timing: the scheduler prepares events 30 ms ahead, while the
real-time thread dispatches them to devices with microsecond accuracy.

Link also shares start/stop state. Press play in Sova and other Link peers can
start with you.

## Bars and phrases

The **quantum** sets how many beats make a bar. Default is 4 — a standard 4/4
bar. The **phase** indicates your position inside that bar: beat 0, 1, 2, or 3.

This is important for launching lines. In **AtQuantum** mode, lines wait for
the downbeat (phase 0) before starting. You edit code mid-bar and the change
lands on the next downbeat. In **Free** mode, lines start immediately — suited
to polyrhythmic independence.

## The transport bar

At the top of the screen: play/stop, BPM, quantum, and the current beat
position. Click the BPM to type a new value. Minimum 20 BPM.

## Spacing events in code

Without explicit timing, all events in a script fire simultaneously — at beat
zero of the frame. You space them apart with waits.

In Cagire, `at` sets timing offsets as fractions of the frame duration:

```forth
0 0.5 at kick snd .       ;; kick at start and halfway through
0 0.25 0.5 0.75 at hat snd .  ;; four hats, evenly spaced
```

In Bob, `WAIT` advances time by beats:

```
>> [note: 60 vel: 100]
WAIT 0.5
>> [note: 64 vel: 80]
WAIT 0.5
>> [note: 67 vel: 100]
```

## Frames, duration, and repetitions

Each frame has a duration in beats. A 2-beat frame gives your script 2 beats
to fill with events.

Repetitions subdivide that duration. A 4-beat frame with 4 repetitions runs
the script 4 times, once per beat. This produces rhythmic loops without
explicit loop code:

```
-- Bob: a kick every beat for 4 beats (frame duration=4, reps=4)
>> [note: 36 vel: 100]
```

```forth
;; Cagire: same principle
36 note 100 vel .
```

One line of code, four kicks. The sequencer handles the repetition.

## Line speed

The speed factor on a line multiplies tempo relative to the global BPM. At 2.0,
double time. At 0.5, half time. Combine with different quantum values across
lines for polymetric structures.

## Execution modes

Three modes control how lines start after edits or scene changes:

- **Free** — lines start immediately. Independent timing.
- **AtQuantum** — lines wait for the next bar downbeat. Everything stays
  phrase-aligned.
- **LongestLine** — waits for the longest currently playing line to finish its
  cycle before restarting.

Choose **AtQuantum** for tight arrangements. Choose **Free** when you want
patterns to drift and overlap.
