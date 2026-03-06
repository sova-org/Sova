# Audio Engine

Sova includes a built-in audio engine called **Doux**. It provides synthesis
and audio processing directly inside the server, so you can produce sound
without any external software or hardware.

## What is Doux

Doux is a real-time audio engine that runs alongside the Sova server. It
occupies a device slot (typically slot 2) and responds to events just like a
MIDI output would — but instead of sending MIDI to an external synth, it
generates audio internally.

Doux is especially tightly integrated with the **Cagire** language, which has
dedicated words for audio synthesis, sample playback, and signal processing.
Other languages can send events to the Doux device slot for basic triggering.

## Audio panel

Open the Audio panel to configure the engine:

- **Output device** — select which audio interface to use for playback.
- **Sample paths** — directories where Doux looks for audio sample files.
- **Voices** — the number of simultaneous synthesis voices available.

The Audio panel also shows the engine's status (running or stopped).

## Visualization panels

Several panels let you monitor the audio output in real time:

- **Scope** — waveform display. Shows the audio signal as it plays. Can be
  detached as a separate window.
- **Spectrum** — frequency spectrum analyzer. Shows the frequency content of
  the audio. Can also be detached.
- **VU Meter** — level meter showing signal amplitude.
- **Scope Bar** — a compact waveform display that fits in a toolbar.

These panels receive data from the server and update in real time. They're
useful both for monitoring and as a visual element during performance.

## Using Doux from code

The primary way to use Doux is through **Cagire**, the stack-based language.
Cagire provides words for:

- Oscillators (sine, saw, square, triangle, noise)
- Sample playback
- Filters and effects
- Amplitude envelopes
- Signal routing

See the **Cagire** tab in the documentation for the full reference of audio
synthesis words.

From other languages (Bob, Boinx, BaLi), you can send note events to the Doux
device slot. Doux responds to MIDI-style note on/off messages with its default
voice, giving you basic synthesis without writing Cagire code.

## Setup

Doux is enabled by default when the server is compiled with the `audio` feature
(which it is in standard builds). When you start the built-in server from the
desktop app, the audio engine is available automatically.

To use Doux:

1. Open the Audio panel and select your output device.
2. Check that the Doux engine is running.
3. The engine is assigned to a device slot (check the Devices panel).
4. Route your events to that slot and play.

## Tips

- Doux runs on the server side. In a multiplayer session, all players share the
  same audio engine — events from any client can trigger sound.
- Use the Scope and Spectrum panels during sound design to see what your
  synthesis code is actually producing.
- If you don't need built-in audio (e.g. you're routing MIDI to external
  hardware), you can ignore Doux entirely. It doesn't consume resources if no
  events are routed to it.
