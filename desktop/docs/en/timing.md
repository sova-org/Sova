Sova measures time in beats. Frame lengths, event offsets, note values: all
expressed in beats. Change the tempo and every pattern scales proportionally.
This article covers what beats are, how they are shaped into durations, and how
the clock keeps everything tight.

## The beat

### Beats, not seconds

One beat at 120 BPM lasts 500 ms. At 60 BPM, one full second. At 180 BPM,
333 ms. The conversion is direct: multiply beats by 60,000 / BPM to get
milliseconds. Sova never stores durations in seconds or milliseconds at the
user level. Beats are the unit. This means a pattern written at any tempo
plays correctly at any other tempo. Only the speed changes.

### Tempo and Ableton Link

The clock runs on Ableton Link. Every Link-enabled application on the network
shares the same tempo and beat position. Change BPM in Sova and Ableton Live
sees it. Change it in Live and Sova follows. With no other peers on the
network, Sova runs its own clock. Link also shares transport state: play and
stop propagate across peers. Tempo range: 20 to 300 BPM.

### Quantum and phase

The quantum defines how many beats form one cycle. Default is 4 (standard 4/4).
Range: 1 to 16. The phase is the current position within that cycle: at quantum
4, phase cycles through 0, 1, 2, 3. Phase 0 is the downbeat. Quantum matters for synchronization. In `AtQuantum` mode, lines wait for the next downbeat before starting. Deferred actions default to the next phase reset. See [The Scene](the-scene) for execution modes.

## Duration

### Frame duration

Each frame has a duration in beats. This is the time window given to its script.
Default: 1 beat. Fractional values work. Set 0.5 for an eighth note, 0.25 for a
sixteenth, 4 for a full bar at quantum 4. Without explicit timing within the script, all events fire at beat zero of that window. Each language provides its own mechanism for distributing events across the frame's duration. See the language tabs for syntax.

### Repetitions

A frame repeats its script a given number of times before the line advances to
the next frame. Each repetition gets the same duration window. A frame with
duration 1 and 4 repetitions runs its script four times, one beat each,
occupying 4 beats total on the timeline. Default: 1 (no repetition). The total time a frame occupies equals duration times repetitions. See [The Scene](the-scene) for all frame properties.

### Line speed

Each line carries a speed factor. Default: 1.0. At 2.0, all frame durations on that line double. At 0.5, they halve. The actual time per frame execution is the frame's duration multiplied by the line's speed factor. Different speed factors across lines produce polymetric structures. One line at 1.0, another at 1.5, a third at 0.75. Each cycles through its frames at its own rate against the shared tempo. See [The Scene](the-scene) for line properties.

### The transport bar

At the top of the screen you can see buttons to control the palying state, to hush and panic (stop actions), to edit the current BPM and quantum. Click BPM or quantum to input a new value that will take effect immediately. The phase bar shows progress through the current cycle. The execution mode selector determines how lines synchronize at launch.

### Deferred timing

Not all actions take effect immediately. Tempo changes, transport commands, and scene modifications can be scheduled to land at the next beat, the next downbeat (phase 0), or a specific modulo. The default is the next bar boundary. This prevents mid-beat transitions and keeps changes musically coherent.

### Under the hood

The scheduler prepares events 30 ms ahead of real time. A dedicated real-time thread dispatches them to devices: MIDI messages are sent 2 ms early to compensate for hardware interface latency, OSC and audio engine messages 20 ms early. The user does not interact with this mechanism directly, but it explains why timing remains tight on a general-purpose operating system and why events scheduled close together still land in the correct order.
