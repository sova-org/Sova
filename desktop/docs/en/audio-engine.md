# Audio Engine

Doux is Sova's built-in synthesizer. It runs inside the server, produces audio
directly, and needs no external software or hardware. If the server has audio
enabled (the default), Doux is available as soon as you start.

## What Doux can do

Oscillators (sine, saw, square, triangle, noise), sample playback, filters
(lowpass, highpass, bandpass, ladder variants), reverb, delay, distortion,
chorus, phaser, FM synthesis, compression, and live recording into reusable
samples. The full parameter list is in the Cagire **Language Reference**.

## Audio panel

Open the Audio panel to configure the engine:

- Output device -- which audio interface to use.
- Sample paths -- directories where Doux loads samples from.
- Voices -- simultaneous synthesis voices.

The panel shows whether the engine is running.

## Scope, Spectrum, VU Meter

Three visualization panels monitor the audio output:

- Scope shows the waveform. Detachable as a separate window.
- Spectrum shows frequency content. Also detachable.
- VU Meter shows signal level.

They update in real time from the server. Useful during sound design and as
visuals during performance.

## Using Doux from Cagire

Cagire is the primary language for Doux. A sample:

```forth
"kick" snd .
```

A filtered sawtooth with reverb:

```forth
"saw" snd c4 note 0.5 gain 800 lpf 0.3 verb .
```

FM synthesis with envelope:

```forth
"sine" snd c4 note 200 fm 2 fmh 0.01 att 0.3 dec .
```

Live recording, then playback with effects:

```forth
"loop" rec              ;; start recording
```

```forth
"loop" rec              ;; stop, sample is registered
loop snd 0.5 speed 800 lpf 0.4 verb .
```

Sidechain compression between orbits:

```forth
0 orbit "kick" snd .                 ;; kick on orbit 0
1 orbit "saw" snd c3 note 0.8 comp 0 corbit .  ;; duck synth from orbit 0
```

## Using Doux from other languages

Bob, Boinx, and BaLi can send note events to the Doux device slot. Doux
responds to MIDI-style note on/off with its default voice:

```
DEV 2
>> [note: 60 vel: 100]
WAIT 1
>> [note: 64 vel: 80]
```

For full synthesis control (filters, effects, FM), use Cagire.

## Setup

Doux is enabled by default. When you start the built-in server from the desktop
app, the audio engine starts automatically and occupies a device slot (check the
Devices panel to see which one).

1. Open the Audio panel, select your output device.
2. Confirm the engine is running.
3. Route events to the Doux slot.

In a multiplayer session, all players share the same engine -- any client can
trigger sound. If you only use external MIDI hardware, you can ignore Doux
entirely. It consumes no resources when idle.
