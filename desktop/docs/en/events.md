# Events

Your code produces events: MIDI messages, OSC messages, or audio commands sent
to devices.

## MIDI notes

A note event fires a Note On, then a Note Off after the duration elapses. You
never send Note Offs yourself.

Bob:

```
>> [note: 60 vel: 100 dur: 0.5]
```

Cagire:

```forth
60 note 100 vel 0.5 dur .
```

Parameters: pitch (0–127), velocity (0–127), duration (beats), channel (1–16),
device (1–16). Defaults: velocity 100, duration 0.5, channel 1, device 1.

## Control Change

CC messages control knobs, faders, and parameters on external synths.

Bob:

```
>> [cc: 74 val: 100]
```

Cagire:

```forth
74 ccnum 100 ccout .
```

## Pitch bend

Range: -1.0 (full down) to 1.0 (full up), center 0.0.

```forth
0.5 bend .
```

## Program Change

```
>> [pc: 12]
```

```forth
12 program .
```

## OSC messages

OSC sends messages over UDP to SuperCollider, Max/MSP, Pure Data, or any
OSC-capable application.

Bob:

```
>> [addr: "/synth" freq: 440 amp: 0.5]
```

`addr` sets the OSC address. Every other key becomes an argument. Route to an
OSC device slot with `dev`.

## Device and channel routing

Every event carries a device slot and a MIDI channel.

Bob:

```
DEV 1
>> [note: 60 chan: 0]
DEV 2
>> [note: 48 chan: 2]
```

Cagire:

```forth
1 dev 60 note .
2 dev 48 note 3 chan .
```

Device selects the output slot (1–16). Channel selects the MIDI channel.
Slot 0 is the log console — use it to inspect events before routing to a real
output. You can switch device and channel mid-script.

## Chords and sequences

Without waits, events fire simultaneously — chords:

```
>> [note: 60] >> [note: 64] >> [note: 67]
```

Add waits for a sequence:

```
>> [note: 60] WAIT 0.5 >> [note: 64] WAIT 0.5 >> [note: 67]
```

In Cagire, `at` with `arp` places one note per time slot:

```forth
0 0.33 0.66 at
c4 e4 g4 arp note sine snd .
```

See **Timing** for details on `at` and `arp`.

## Reading MIDI input

Cagire reads incoming CC values from hardware controllers:

```forth
74 1 ccval 127 / 200 2740 range lpf
```

Reads CC 74 on channel 1, normalizes to 0.0–1.0, scales to 200–2740, and
applies the result as a lowpass cutoff. See each language's reference for the
full input API.
