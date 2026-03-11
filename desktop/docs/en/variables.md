# Variables

Variables store values that persist between events or between frames. Use them
to coordinate scripts, accumulate state, and build evolving patterns.

## Scopes

Four scopes. The scope determines who sees the variable and how long it lives.

**Instance** — local scratch space. Resets every time the script runs. Use it
for intermediate calculations that don't need to survive beyond a single
execution.

**Frame** — survives repetitions within the same frame. Resets when the line
advances to the next frame. Well suited to counters that accumulate across
repetitions: each run reads the previous value, modifies it, and stores it back.

**Line** — shared across all frames in a line. One frame writes a value, another
reads it. Useful for passing context along a sequence: set a root note in frame
A, transpose from it in frames B and C.

**Global** — visible to every script in the session, across all lines and
frames. Use sparingly. Best reserved for session-wide state like a key center or
a shared counter that multiple lines need to read.

## Storing and fetching

Each language has its own syntax for reading and writing variables. The
underlying mechanism is the same: store a value under a name, fetch it later.
Unknown variables return zero. See the language tabs for syntax details.

Scope is part of the variable name. A prefix indicates whether you're addressing
a frame, line, or global variable. Without a prefix, the variable is instance-
scoped.

## Accumulators

The most common pattern: fetch a value, modify it, store it back. This turns a
variable into a counter, a phase accumulator, or any evolving quantity. Combined
with frame-scoped variables and repetitions, a single line of logic can generate
an entire sequence that shifts on every repetition.

## Naming sounds

Store a sound name in a line-scoped variable and reference it from multiple
frames. Change the value in one place and every frame in the line picks up the
new sound. This avoids duplicating sound names across frames and makes live
adjustments faster.

## Environment values

Read-only values injected by the runtime. Scripts can read them but not write
them.

The VM provides six environment functions:

- `GetTempo` — current session tempo as integer
- `RandomInt` — unrestricted random integer
- `RandomFloat` — random float in [0, 1)
- `RandomUInt(n)` — random integer in [0, n)
- `RandomDecInBounds(min, max)` — random decimal in [min, max]
- `FrameLen(line, frame)` — duration of a specific frame in beats

These are the only values available at the VM level.

Languages extend this set with their own context values. Bob adds tempo, a
random 0-127 value, loop index, and element. BaLi adds loop index and tempo.
Cagire adds step position, beat, pattern index, slot, run count, iteration,
step duration, fill, and phase. See the language tabs for the exact names and
how to access them.

## Visibility timing

Within a frame, you read back what you just wrote. Changes become visible to
other frames only after the current frame finishes executing. If two frames run
in the same scheduling pass, each sees the other's previous values, not the
current ones. This prevents ordering surprises: the result doesn't depend on
which frame the scheduler happens to run first.
