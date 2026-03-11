# Audio Engine

Doux is Sova's built-in synthesizer. It runs inside the server and produces
audio directly, with no external software or hardware needed. Doux lets you
start making sound immediately — an accessible entry point for beginners and a
full-featured tool for advanced sound design. If the server has audio enabled
(the default), Doux is available as soon as you start.

## What Doux can do

Oscillators (sine, saw, square, triangle, noise), sample playback, filters
(lowpass, highpass, bandpass, ladder variants), reverb, delay, distortion,
chorus, phaser, FM synthesis, compression, and live recording into reusable
samples. Cagire has the deepest integration with Doux — see the Cagire language
tab for the full parameter list.

## Audio panel

Open the Audio panel to configure the engine:

- Output device — which audio interface to use.
- Sample paths — directories where Doux loads samples from.
- Voices — simultaneous synthesis voices.

The panel shows whether the engine is running.

## Scope, spectrum, VU meter

Three visualization panels monitor the audio output:

- The scope shows the waveform. Detachable as a separate window.
- The spectrum shows frequency content. Also detachable.
- The VU meter shows signal level.

They update in real time from the server. Useful for sound design and as a
visual element during performance.

## Using Doux

Cagire is the primary language for Doux, offering direct control over every
synthesis parameter: oscillator type, pitch, gain, filter cutoff, envelope
shape, FM depth, effects, orbits, and sidechain compression. See the Cagire
language tab for the full synthesis API.

Bob, Boinx, and BaLi can send note events to the Doux device slot. Doux
responds to MIDI-style Note On/Off with its default voice. For full synthesis
control (filters, effects, FM), use Cagire.

## Recording

Doux can record its own output into samples that you immediately play back and
manipulate with effects. Four Cagire words handle recording:

- `rec` — toggle recording audio output to a named sample (`"loop1" rec`)
- `overdub` (alias `dub`) — toggle overdub recording (`"loop1" overdub`)
- `orec` — toggle recording a single orbit (`"drums" 0 orec`)
- `odub` — toggle overdub recording a single orbit (`"drums" 0 odub`)

The captured audio becomes a named sample available to all scripts in the
session.

## Setup

Doux is enabled by default. When you start the built-in server from the desktop
app, the audio engine starts automatically and occupies slot 1. If another
device was already on slot 1, it gets bumped to slot 2.

1. Open the Audio panel and select your output device.
2. Confirm the engine is running.
3. Route events to the Doux slot.

In a multiplayer session, all musicians share the same engine — any client can
trigger sound. If you only use external MIDI hardware, you can ignore Doux. It
consumes no resources when idle.
