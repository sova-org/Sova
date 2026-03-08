# The Scene

A scene is your live session. It holds everything playing right now: the
parallel tracks, the code in each slot, the timing. When you perform with Sova,
you are editing a scene in real time.

## Structure

A scene has **lines** and **frames**. Lines are columns on the grid. They run in
parallel, each producing its own stream of events. Inside a line, **frames** run
in sequence. When a frame's duration elapses, the next one starts.

One line might run a kick drum pattern. Another plays a bass. A third sends OSC
to a visual synth. They all tick forward independently, at their own pace.

## Frames

Each frame holds a script (your code) and a few properties:

- Duration (beats): how long the frame lasts. Default is 1 beat. Fractional
  values work: 0.25 for a sixteenth note feel, 4 for a whole bar at 4/4.
- Repetitions: how many times the script runs within that duration. A frame with
  duration 4 and repetitions 4 runs its script once per beat. Repetitions of 8
  would fire the script on every eighth note instead.
- Enabled: toggle a frame on or off. Disabled frames get skipped. Good for
  muting a section mid-performance without losing the code.
- Name: optional label shown on the grid cell. Use it.
- Script: the code itself, plus which language it uses (Bob, Boinx, Cagire,
  or BaLi).

The total time a frame occupies is `duration * repetitions`. A frame with
duration 0.5 and 8 repetitions takes 4 beats.

## Lines

Lines have their own controls:

- Looping: when on, the line restarts from the top after its last frame. When
  off, it plays once and stops.
- Trailing: when on, events from previous frames keep ringing while the next
  frame starts. When off, they cut.
- Speed factor: multiplier on the line's tempo. Set 2.0 to play double time,
  0.5 for half. One line at normal speed, another at half speed -- polymetric
  structures come naturally.
- Start frame / End frame: restrict playback to a range within the line. During
  a performance, narrow the range to loop a specific section while you edit
  what comes next.

## Execution modes

The execution mode controls how lines sync when the scene starts or restarts.
Change it from the transport bar.

**Free** is the default. Lines start immediately and loop at their own pace.
Each line is independent. Good for jamming, layering patterns that drift against
each other, building textures.

**AtQuantum** makes lines wait for the next quantum boundary (bar line) before
starting. Everything snaps to the global phrase. Use this for tight
arrangements where parts need to land on the downbeat.

**LongestLine** waits for the longest running line to finish its full cycle
before any line restarts. All lines reset together. This turns the scene into a
synchronized loop grid -- useful when you want all your parts to cycle as a
single unit.

## Saving and loading

Save and load scenes through the scene menu. A scene file captures everything:
lines, frames, scripts, variables, and configuration. When you connect to a
server, you receive its current scene automatically.

See **The Grid** for navigation and editing shortcuts.
