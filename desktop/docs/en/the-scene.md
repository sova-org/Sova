# The Scene

The scene represents your performance session. It holds everything playing right
now: parallel tracks, code in each slot, timing. When you perform with Sova, you
edit a scene in real time.

Sova can be thought of as a step sequencer where each step's behavior is defined
by code. Unlike conventional step sequencers, step duration is not fixed — a
step can last a fraction of a beat or a full measure. Scripts can be modified and
reprogrammed in real time, enabling dynamic and spontaneous performances.

## Structure

A scene contains **lines** and **frames**. Lines are columns on the grid. They
run in parallel, each producing its own stream of events. Inside a line,
**frames** play in sequence. When a frame's duration elapses, the next one
starts.

One line runs a kick pattern. Another plays a bass. A third sends OSC to a
visual synth. Each advances at its own pace, independently.

## Frames

Each frame holds a script (your code) and a few properties:

- Duration (beats): how long the frame lasts. Default is 1 beat. Fractional
  values work: 0.25 for a sixteenth-note subdivision, 4 for a full bar at 4/4.
- Repetitions: how many times the script runs within that duration. A frame with
  duration 4 and 4 repetitions runs its script once per beat. With 8
  repetitions, the script fires on every eighth note.
- Enabled: toggles the frame on or off. Disabled frames are skipped. Useful for
  muting a section mid-performance without losing the code.
- Name: optional label shown on the grid cell.
- Script: the code, along with the language it uses (Bob, Boinx, Cagire,
  or BaLi).

The total time a frame occupies is `duration × repetitions`. A frame with
duration 0.5 and 8 repetitions takes 4 beats.

## Lines

Lines have their own controls:

- Loop: when enabled, the line restarts from the top after its last frame.
  Otherwise, it plays once and stops.
- Trailing: when enabled, events from previous frames keep ringing while the
  next frame starts. Otherwise, they are cut.
- Speed: multiplier on the line's tempo. 2.0 for double time, 0.5 for half.
  One line at normal speed, another at half — polymetric structures arise
  naturally.
- Start frame / End frame: restricts playback to a range within the line.
  During a performance, narrow the range to loop a specific section while you
  edit what comes next.

## Execution modes

The execution mode controls how lines synchronize when the scene starts or
restarts. Change it from the transport bar.

**Free** is the default. Lines start immediately and loop at their own pace.
Each line is independent. Suited to jamming, layering patterns that drift
against each other, building textures.

**AtQuantum** makes lines wait for the next quantum boundary (bar line) before
starting. Everything snaps to the global phrase. For tight arrangements where
parts need to land on the downbeat.

**LongestLine** waits for the longest running line to finish its full cycle
before anything restarts. All lines reset together. The scene becomes a grid of
synchronized loops — useful when all parts should cycle as a single unit.

## Saving and loading

Save and load scenes through the scene menu. The file captures everything:
lines, frames, scripts, variables, and configuration. When you connect to a
server, you receive its current scene automatically.

See **The Grid** for navigation and editing shortcuts.
