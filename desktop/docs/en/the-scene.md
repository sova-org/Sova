# The Scene

The scene is your performance session. It holds every track, script, and timing
configuration. You edit it in real time.

Sova is a step sequencer where each step is defined by code. Step duration is
not fixed — a step can last a sixteenth note or a full measure (see [Timing](timing)).
Scripts can be modified during playback.

## Structure

A scene contains **lines** and **frames**. Lines are columns on the grid. They
run in parallel, each producing its own stream of events. Inside a line, frames
play in sequence. When a frame's duration elapses, the next one starts.

## Frames

Each frame holds a script and a set of properties:

- **Duration** — how long the frame lasts, in beats (see [Timing](timing)). Default is
  1 beat. Fractional values work: 0.25 for a sixteenth note, 4 for a full bar
  at 4/4.
- **Repetitions** — how many times the script runs within that duration. A frame
  with duration 4 and 4 repetitions runs its script once per beat. With 8
  repetitions, the script fires every eighth note. The total time the frame
  occupies remains 4 beats either way.
- **Enabled** — toggles the frame on or off. Disabled frames are skipped during
  playback. The code is preserved.
- **Name** — optional label shown in the frame header.
- **Script** — the code, along with the language it uses (Bob, Boinx, Cagire,
  or BaLi).

Frame properties (duration, repetitions, name, language, enabled) are edited
directly in the frame header. Each frame also contains an inline code editor.

## Modal interaction

The scene view uses two modes:

**Navigation mode** (default) — arrow keys and vim keys move between frames and
lines. Single-key shortcuts operate on frames. No typing goes to any editor.

**Edit mode** — entered by pressing Enter or i on a frame. The code editor
receives focus. All typing goes to the editor. Press Escape to return to
Navigation mode.

Clicking inside a code editor also enters Edit mode. Pressing Escape always
returns to Navigation mode.

### Navigation mode shortcuts

| Shortcut | Action |
|----------|--------|
| Arrow keys | Move between frames and lines |
| h / j / k / l | Move (Vim-style) |
| Shift+Up/Down | Extend selection vertically |
| Enter / i | Enter Edit mode |
| Escape | Clear cursor and selection |
| Cmd+D | Duplicate frame after |
| Cmd+Shift+D | Duplicate frame before |
| Shift+I | Insert empty frame after |
| Cmd+Shift+I | Insert empty frame before |
| Delete | Delete selected frame(s) |
| Shift+J | Move frame(s) down |
| Shift+K | Move frame(s) up |
| e | Toggle enabled |
| . | Toggle looping on line |
| , | Toggle trailing on line |
| Alt+H | Move line left |
| Alt+L | Move line right |
| Cmd+C | Copy |
| Cmd+X | Cut |
| Cmd+V | Paste after cursor |
| Cmd+A | Select all frames in current line |
| Cmd+Delete | Remove entire line |

### Edit mode shortcuts

| Shortcut | Action |
|----------|--------|
| Escape | Exit to Navigation mode |
| Cmd+Enter | Evaluate script |
| Cmd+L | Open language selector |
| Cmd+F | Search in editor |

## Lines

Lines have their own controls, visible in the line header:

- **Loop** — the line restarts from the top after its last frame. Otherwise, it
  plays once and stops.
- **Trailing** — events from previous frames keep ringing while the next frame
  starts. Otherwise, they are cut.
- **Speed** — multiplier on the line's tempo. 2.0 for double time, 0.5 for half.
  One line at normal speed, another at half — polymetric structures emerge.
- **Start frame / End frame** — restricts playback to a range within the line.
  Narrow the range to loop a section while you build the next one.

## Execution modes

The execution mode controls how lines synchronize when the scene starts or
restarts. Change it from the transport bar.

**Free** — lines start immediately and loop at their own pace. Each line is
independent.

**AtQuantum** — lines wait for the next quantum boundary (bar line) before
starting. Parts land on the downbeat.

**LongestLine** — all lines wait for the longest one to finish its cycle before
restarting. The scene loops as a single unit.

## Saving and loading

Save and load scenes through the scene menu. The file captures lines, frames,
scripts, [Variables](variables), and configuration. Connecting to a server loads its
current scene automatically.

## Workflow tips

- **Name your frames** — use the name field in the frame header. Unnamed frames
  are hard to tell apart during a performance.
- **Duplicate before modifying** — Cmd+D copies the frame. The original stays
  intact as a fallback.
- **Reorder on the fly** — Shift+J/K moves frames mid-performance without
  stopping playback.
- **Disable instead of deleting** — press e to toggle a frame off. The code
  stays visible but is skipped.
- **Isolate with ranges** — set start/end frame on a line to loop a section
  while you build the next one.
- **One role per line** — drums, bass, melody, effects. Easier to mute, isolate,
  or rearrange.
